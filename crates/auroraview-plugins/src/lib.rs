//! AuroraView Plugin System
//!
//! This crate provides a plugin architecture for extending AuroraView with
//! native desktop capabilities. Inspired by Tauri's plugin system.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    JavaScript API                            │
//! │  window.auroraview.fs.readFile()                            │
//! │  window.auroraview.clipboard.write()                        │
//! │  window.auroraview.shell.open()                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Plugin Command Router                           │
//! │  invoke("plugin:fs|read_file", { path, ... })               │
//! ├────────────┬────────────┬────────────┬──────────────────────┤
//! │ fs_plugin  │ clipboard  │ shell      │ dialog               │
//! ├────────────┴────────────┴────────────┴──────────────────────┤
//! │               auroraview-plugins                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Available Plugins
//!
//! - **fs**: File system operations (read, write, list, etc.)
//! - **clipboard**: System clipboard access (read/write text, images)
//! - **shell**: Execute commands, open URLs/files
//! - **dialog**: Native file/folder dialogs
//!
//! ## Command Format
//!
//! Plugin commands use the format: `plugin:<plugin_name>|<command_name>`
//!
//! Example: `plugin:fs|read_file`

pub mod clipboard;
pub mod dialog;
pub mod fs;
pub mod process;
pub mod scope;
pub mod shell;
mod types;

pub use scope::{PathScope, ScopeConfig, ScopeError};
pub use types::{PluginCommand, PluginError, PluginErrorCode, PluginResult};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Event callback type for plugins to emit events
pub type PluginEventCallback = Arc<dyn Fn(&str, Value) + Send + Sync>;

/// Plugin command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// Plugin name (e.g., "fs", "clipboard")
    pub plugin: String,
    /// Command name (e.g., "read_file", "write")
    pub command: String,
    /// Command arguments as JSON
    pub args: Value,
    /// Optional request ID for async response
    pub id: Option<String>,
}

impl PluginRequest {
    /// Parse a command string in format "plugin:<plugin>|<command>"
    pub fn from_invoke(invoke_cmd: &str, args: Value) -> Option<Self> {
        if !invoke_cmd.starts_with("plugin:") {
            return None;
        }

        let rest = &invoke_cmd[7..]; // Skip "plugin:"
        let parts: Vec<&str> = rest.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }

        Some(Self {
            plugin: parts[0].to_string(),
            command: parts[1].to_string(),
            args,
            id: None,
        })
    }

    /// Create a new plugin request
    pub fn new(plugin: impl Into<String>, command: impl Into<String>, args: Value) -> Self {
        Self {
            plugin: plugin.into(),
            command: command.into(),
            args,
            id: None,
        }
    }

    /// Set the request ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Plugin command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// Success flag
    pub success: bool,
    /// Response data (if success)
    pub data: Option<Value>,
    /// Error message (if failure)
    pub error: Option<String>,
    /// Error code (if failure)
    pub code: Option<String>,
    /// Request ID (echoed from request)
    pub id: Option<String>,
}

impl PluginResponse {
    /// Create a success response
    pub fn ok(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            code: None,
            id: None,
        }
    }

    /// Create an error response
    pub fn err(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            code: Some(code.into()),
            id: None,
        }
    }

    /// Set the request ID
    pub fn with_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
    }
}

/// Trait for plugin implementations
pub trait PluginHandler: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Handle a command
    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value>;

    /// Get supported commands
    fn commands(&self) -> Vec<&str>;
}

/// Plugin router for dispatching commands to plugins
pub struct PluginRouter {
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn PluginHandler>>,
    /// Global scope configuration
    scope: ScopeConfig,
    /// Event callback for plugins to emit events to frontend
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
}

impl Default for PluginRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRouter {
    /// Create a new plugin router with default plugins
    pub fn new() -> Self {
        let event_callback: Arc<RwLock<Option<PluginEventCallback>>> = Arc::new(RwLock::new(None));

        let mut router = Self {
            plugins: HashMap::new(),
            scope: ScopeConfig::default(),
            event_callback: event_callback.clone(),
        };

        // Register built-in plugins
        router.register("fs", Arc::new(fs::FsPlugin::new()));
        router.register("clipboard", Arc::new(clipboard::ClipboardPlugin::new()));
        router.register("shell", Arc::new(shell::ShellPlugin::new()));
        router.register("dialog", Arc::new(dialog::DialogPlugin::new()));

        // Create process plugin with shared event callback
        let process_plugin = process::ProcessPlugin::with_event_callback(event_callback);
        router.register("process", Arc::new(process_plugin));

        router
    }

    /// Create with custom scope configuration
    pub fn with_scope(scope: ScopeConfig) -> Self {
        let mut router = Self::new();
        router.scope = scope;
        router
    }

    /// Set the event callback for plugins to emit events
    ///
    /// This callback will be invoked when plugins (like ProcessPlugin) need
    /// to send events to the frontend (e.g., process stdout/stderr output).
    pub fn set_event_callback(&self, callback: PluginEventCallback) {
        let mut cb = self.event_callback.write().unwrap();
        *cb = Some(callback);
    }

    /// Clear the event callback
    pub fn clear_event_callback(&self) {
        let mut cb = self.event_callback.write().unwrap();
        *cb = None;
    }

    /// Emit an event through the callback (if set)
    pub fn emit_event(&self, event_name: &str, data: Value) {
        if let Some(callback) = self.event_callback.read().unwrap().as_ref() {
            callback(event_name, data);
        }
    }

    /// Register a plugin
    pub fn register(&mut self, name: impl Into<String>, plugin: Arc<dyn PluginHandler>) {
        self.plugins.insert(name.into(), plugin);
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn PluginHandler>> {
        self.plugins.remove(name)
    }

    /// Handle a plugin command
    pub fn handle(&self, request: PluginRequest) -> PluginResponse {
        // Check if plugin is enabled
        if !self.scope.is_plugin_enabled(&request.plugin) {
            return PluginResponse::err(
                format!("Plugin '{}' is disabled", request.plugin),
                "PLUGIN_DISABLED",
            )
            .with_id(request.id);
        }

        let plugin = match self.plugins.get(&request.plugin) {
            Some(p) => p,
            None => {
                return PluginResponse::err(
                    format!("Plugin '{}' not found", request.plugin),
                    "PLUGIN_NOT_FOUND",
                )
                .with_id(request.id);
            }
        };

        match plugin.handle(&request.command, request.args.clone(), &self.scope) {
            Ok(data) => PluginResponse::ok(data).with_id(request.id),
            Err(e) => PluginResponse::err(e.message(), e.code()).with_id(request.id),
        }
    }

    /// Check if a plugin is registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get list of registered plugin names
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Get the scope configuration
    pub fn scope(&self) -> &ScopeConfig {
        &self.scope
    }

    /// Get mutable scope configuration
    pub fn scope_mut(&mut self) -> &mut ScopeConfig {
        &mut self.scope
    }

    /// Update scope configuration
    pub fn set_scope(&mut self, scope: ScopeConfig) {
        self.scope = scope;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_request_parse() {
        let req = PluginRequest::from_invoke("plugin:fs|read_file", serde_json::json!({}));
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.plugin, "fs");
        assert_eq!(req.command, "read_file");
    }

    #[test]
    fn test_plugin_request_parse_invalid() {
        let req = PluginRequest::from_invoke("not_a_plugin", serde_json::json!({}));
        assert!(req.is_none());

        let req = PluginRequest::from_invoke("plugin:no_command", serde_json::json!({}));
        assert!(req.is_none());
    }

    #[test]
    fn test_plugin_response_ok() {
        let resp = PluginResponse::ok(serde_json::json!({"result": "success"}));
        assert!(resp.success);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_plugin_response_err() {
        let resp = PluginResponse::err("File not found", "NOT_FOUND");
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("File not found".to_string()));
        assert_eq!(resp.code, Some("NOT_FOUND".to_string()));
    }

    #[test]
    fn test_router_has_default_plugins() {
        let router = PluginRouter::new();
        assert!(router.has_plugin("fs"));
        assert!(router.has_plugin("clipboard"));
        assert!(router.has_plugin("shell"));
        assert!(router.has_plugin("dialog"));
    }
}
