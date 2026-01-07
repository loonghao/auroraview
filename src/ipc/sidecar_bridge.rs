//! Sidecar Bridge - IPC Server integration for MCP Sidecar mode.
//!
//! This module provides the bridge between the main WebView process and
//! the MCP Sidecar process. It runs an IPC Server that:
//!
//! 1. Accepts connections from the Sidecar
//! 2. Routes tool call requests through the MessageQueue
//! 3. Returns results back to the Sidecar
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Main Process (WebView)                       │
//! │  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────┐  │
//! │  │  SidecarBridge  │───>│  IPC Server     │───>│ MessageQueue│  │
//! │  │  (manages)      │    │  (bg thread)    │    │ (main thrd) │  │
//! │  └─────────────────┘    └────────┬────────┘    └─────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//!                                    │ IPC
//!                                    ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Sidecar Process                              │
//! │  ┌─────────────────┐    ┌─────────────────┐                     │
//! │  │  MCP Server     │───>│  IPC Client     │                     │
//! │  │  (HTTP)         │    │                 │                     │
//! │  └─────────────────┘    └─────────────────┘                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use auroraview_mcp_server::{generate_auth_token, generate_channel_name, IpcServer, ToolDefinition};

/// Tool handler function type.
pub type ToolHandler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

/// Configuration for the Sidecar Bridge.
#[derive(Debug, Clone)]
pub struct SidecarBridgeConfig {
    /// Custom channel name (auto-generated if None).
    pub channel_name: Option<String>,
    /// Custom auth token (auto-generated if None).
    pub auth_token: Option<String>,
}

impl Default for SidecarBridgeConfig {
    fn default() -> Self {
        Self {
            channel_name: None,
            auth_token: None,
        }
    }
}

/// Bridge between WebView main process and MCP Sidecar.
///
/// Manages the IPC Server lifecycle and tool registration.
pub struct SidecarBridge {
    /// IPC channel name
    channel_name: String,
    /// Authentication token
    auth_token: String,
    /// Running flag (shared with server thread)
    running: Arc<AtomicBool>,
    /// Server thread handle
    server_thread: Option<JoinHandle<()>>,
    /// Registered tool handlers (shared with server)
    tool_handlers: Arc<RwLock<HashMap<String, ToolHandler>>>,
    /// Tool definitions (shared with server)
    tool_definitions: Arc<RwLock<Vec<ToolDefinition>>>,
}

impl SidecarBridge {
    /// Create a new Sidecar Bridge.
    pub fn new(config: SidecarBridgeConfig) -> Self {
        let pid = std::process::id();
        let channel_name = config
            .channel_name
            .unwrap_or_else(|| generate_channel_name(pid));
        let auth_token = config.auth_token.unwrap_or_else(generate_auth_token);

        Self {
            channel_name,
            auth_token,
            running: Arc::new(AtomicBool::new(false)),
            server_thread: None,
            tool_handlers: Arc::new(RwLock::new(HashMap::new())),
            tool_definitions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the IPC channel name.
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    /// Get the authentication token.
    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    /// Check if the bridge is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Register a tool handler.
    pub fn register_tool<F>(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let name = name.into();
        let definition = ToolDefinition {
            name: name.clone(),
            description: Some(description.into()),
            input_schema: None,
            output_schema: None,
        };

        // Store tool definition and handler
        self.tool_definitions.write().push(definition);
        self.tool_handlers.write().insert(name.clone(), Box::new(handler));
        tracing::debug!("[SidecarBridge] Registered tool: {}", name);
    }

    /// Start the IPC Server in a background thread.
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Bridge already running".to_string());
        }

        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let channel_name = self.channel_name.clone();
        let auth_token = self.auth_token.clone();
        let tool_handlers = Arc::clone(&self.tool_handlers);
        let tool_definitions = Arc::clone(&self.tool_definitions);

        let handle = thread::spawn(move || {
            tracing::info!("[SidecarBridge] IPC Server starting on: {}", channel_name);

            // Create IPC Server in the thread with shared running flag
            // This allows external shutdown via the running flag
            let server = IpcServer::with_running_flag(&channel_name, &auth_token, running.clone());

            // Register all tools with the server
            let definitions = tool_definitions.read().clone();
            for def in definitions {
                let name = def.name.clone();
                // Create a handler that forwards to the shared handlers
                let handlers_clone = Arc::clone(&tool_handlers);
                let name_clone = name.clone();
                server.register_tool(def, move |args| {
                    let handlers = handlers_clone.read();
                    if let Some(h) = handlers.get(&name_clone) {
                        h(args)
                    } else {
                        Err(format!("Tool '{}' not found", name_clone))
                    }
                });
            }

            if let Err(e) = server.start() {
                tracing::error!("[SidecarBridge] IPC Server error: {}", e);
            }

            running.store(false, Ordering::SeqCst);
            tracing::info!("[SidecarBridge] IPC Server stopped");
        });

        self.server_thread = Some(handle);
        Ok(())
    }

    /// Stop the IPC Server.
    ///
    /// This method:
    /// 1. Sets the running flag to false
    /// 2. Creates a dummy connection to unblock the accept() call
    /// 3. Waits for the server thread to finish
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        tracing::info!("[SidecarBridge] Stopping IPC Server...");
        self.running.store(false, Ordering::SeqCst);

        // Create a dummy connection to unblock the accept() call
        // This is necessary because accept() is a blocking call
        let channel_name = self.channel_name.clone();
        std::thread::spawn(move || {
            use ipckit::local_socket::LocalSocketStream;
            // Try to connect - this will unblock the accept loop
            let _ = LocalSocketStream::connect(&channel_name);
            tracing::debug!("[SidecarBridge] Dummy connection sent to unblock accept");
        });

        // Wait for the server thread to finish with timeout
        if let Some(handle) = self.server_thread.take() {
            // Give the thread some time to finish
            match handle.join() {
                Ok(_) => tracing::info!("[SidecarBridge] IPC Server thread joined"),
                Err(_) => tracing::warn!("[SidecarBridge] IPC Server thread join failed"),
            }
        }
    }

    /// Get environment variables for launching the Sidecar.
    ///
    /// These should be passed to the Sidecar process.
    pub fn get_sidecar_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert(
            "AURORAVIEW_IPC_CHANNEL".to_string(),
            self.channel_name.clone(),
        );
        env.insert("AURORAVIEW_IPC_TOKEN".to_string(), self.auth_token.clone());
        env.insert(
            "AURORAVIEW_PARENT_PID".to_string(),
            std::process::id().to_string(),
        );
        env
    }
}

impl Drop for SidecarBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

