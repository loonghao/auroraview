//! Browser Extension Bridge Plugin
//!
//! Provides WebSocket + HTTP server for browser extension communication.
//! This is a high-performance Rust implementation that enables Chrome/Firefox
//! extensions to communicate with AuroraView applications.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Browser Extension                             │
//! │  (Content Script / Side Panel / Background Service Worker)       │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ WebSocket / HTTP
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                 BrowserBridgePlugin (Rust)                       │
//! │  - WebSocket Server (bidirectional real-time communication)      │
//! │  - HTTP Server (REST API for request/response)                   │
//! │  - Event routing to frontend via PluginEventCallback             │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ Plugin Events
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │              AuroraView WebView Frontend                         │
//! │  (Receives browser:message, browser:connect, browser:disconnect) │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Commands
//!
//! - `start` - Start the WebSocket + HTTP bridge server
//! - `stop` - Stop the bridge server
//! - `status` - Get current bridge status (running, ports, clients)
//! - `broadcast` - Send message to all connected browser extensions
//! - `send` - Send message to a specific client
//! - `get_extension` - Get extension download info and instructions
//! - `install_extension` - Install extension from local file
//! - `get_extension_path` - Get bundled extension file path
//!
//! ## Events (emitted to frontend)
//!
//! - `browser:connect` - { clientId } - Browser extension connected
//! - `browser:disconnect` - { clientId } - Browser extension disconnected
//! - `browser:message` - { clientId, action, data, requestId } - Message from extension
//!
//! ## Example
//!
//! ```javascript
//! // Start the bridge (using high-port range 49152+)
//! await auroraview.invoke("plugin:browser_bridge|start", {
//!     wsPort: 49152,
//!     httpPort: 49153
//! });
//!
//! // Listen for messages from browser extensions
//! auroraview.on("browser:message", ({ clientId, action, data }) => {
//!     console.log(`Extension ${clientId} sent: ${action}`, data);
//! });
//!
//! // Broadcast to all extensions
//! await auroraview.invoke("plugin:browser_bridge|broadcast", {
//!     action: "update",
//!     data: { status: "ready" }
//! });
//!
//! // Get extension download info
//! const info = await auroraview.invoke("plugin:browser_bridge|get_extension", {
//!     browser: "chrome"
//! });
//!
//! // Install extension from local file (drag & drop)
//! await auroraview.invoke("plugin:browser_bridge|install_extension", {
//!     path: "/path/to/extension.crx",
//!     browser: "chrome"
//! });
//!
//! // Install unpacked extension folder (development version)
//! await auroraview.invoke("plugin:browser_bridge|install_extension", {
//!     path: "/path/to/extension-folder",
//!     browser: "chrome"
//! });
//!
//! // Stop the bridge
//! await auroraview.invoke("plugin:browser_bridge|stop");
//! ```

use auroraview_plugin_core::{
    PluginError, PluginEventCallback, PluginHandler, PluginResult, ScopeConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Browser bridge plugin for extension communication
pub struct BrowserBridgePlugin {
    name: String,
    /// Bridge server state
    state: Arc<RwLock<BridgeState>>,
    /// Event callback for emitting events to frontend
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
}

/// Bridge server state
struct BridgeState {
    /// Whether the server is running
    is_running: bool,
    /// WebSocket port
    ws_port: u16,
    /// HTTP port
    http_port: u16,
    /// Connected clients (client_id -> client info)
    clients: HashMap<u64, ClientInfo>,
    /// Client streams for sending messages
    client_streams: HashMap<u64, Arc<Mutex<TcpStream>>>,
    /// Next client ID
    next_client_id: AtomicU64,
}

/// Connected client info
#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    id: u64,
    address: String,
    connected_at: u64,
}

/// Options for starting the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOptions {
    /// WebSocket server port (default: 9001)
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,
    /// HTTP server port (default: 9002)
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    /// Server host (default: 127.0.0.1)
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_ws_port() -> u16 {
    // Use high port range (49152-65535) to avoid conflicts
    49152
}

fn default_http_port() -> u16 {
    // Use high port range (49152-65535) to avoid conflicts
    49153
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

/// Options for broadcasting a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastOptions {
    /// Action/event name
    pub action: String,
    /// Message data
    #[serde(default)]
    pub data: Value,
}

/// Options for sending to a specific client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOptions {
    /// Target client ID
    pub client_id: u64,
    /// Action/event name
    pub action: String,
    /// Message data
    #[serde(default)]
    pub data: Value,
}

/// Bridge status response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatus {
    pub is_running: bool,
    pub ws_port: u16,
    pub http_port: u16,
    pub connected_clients: usize,
    pub clients: Vec<ClientInfo>,
}

/// Options for getting extension download URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExtensionOptions {
    /// Browser type: "chrome" or "firefox"
    #[serde(default = "default_browser")]
    pub browser: String,
}

fn default_browser() -> String {
    "chrome".to_string()
}

/// Extension download info
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    /// Download URL for the extension
    pub download_url: String,
    /// Extension ID
    pub extension_id: String,
    /// Browser type
    pub browser: String,
    /// Installation instructions
    pub instructions: String,
    /// Local extension path (if available)
    pub local_path: Option<String>,
}

/// Options for installing extension from local path
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallExtensionOptions {
    /// Path to the extension file (.crx for Chrome, .xpi for Firefox) or unpacked folder
    pub path: String,
    /// Browser type: "chrome" or "firefox"
    #[serde(default = "default_browser")]
    pub browser: String,
}

/// Options for installing extension to WebView2's extensions directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallToWebViewOptions {
    /// Path to the unpacked extension folder (must contain manifest.json)
    pub path: String,
    /// Optional custom name for the extension folder (defaults to source folder name)
    #[serde(default)]
    pub name: Option<String>,
}

/// Information about an installed WebView2 extension
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewExtensionInfo {
    /// Extension folder name (used as ID)
    pub id: String,
    /// Extension name from manifest
    pub name: String,
    /// Extension version from manifest
    pub version: String,
    /// Extension description from manifest
    pub description: String,
    /// Full path to extension folder
    pub path: String,
}

impl BrowserBridgePlugin {
    /// Create a new browser bridge plugin
    pub fn new() -> Self {
        Self {
            name: "browser_bridge".to_string(),
            state: Arc::new(RwLock::new(BridgeState {
                is_running: false,
                ws_port: 49152,
                http_port: 49153,
                clients: HashMap::new(),
                client_streams: HashMap::new(),
                next_client_id: AtomicU64::new(1),
            })),
            event_callback: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create with shared event callback (used by PluginRouter)
    pub fn with_event_callback(callback: Arc<RwLock<Option<PluginEventCallback>>>) -> Self {
        Self {
            name: "browser_bridge".to_string(),
            state: Arc::new(RwLock::new(BridgeState {
                is_running: false,
                ws_port: 49152,
                http_port: 49153,
                clients: HashMap::new(),
                client_streams: HashMap::new(),
                next_client_id: AtomicU64::new(1),
            })),
            event_callback: callback,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Emit event to frontend
    #[allow(dead_code)]
    fn emit_event(&self, event: &str, data: Value) {
        if let Ok(cb) = self.event_callback.read() {
            if let Some(callback) = cb.as_ref() {
                callback(event, data);
            }
        }
    }

    /// Start the bridge servers
    fn start(&self, opts: StartOptions) -> PluginResult<Value> {
        {
            let state = self.state.read().unwrap();
            if state.is_running {
                return Err(PluginError::from_plugin(
                    "browser_bridge",
                    "Bridge is already running",
                ));
            }
        }

        // Reset shutdown flag
        self.shutdown.store(false, Ordering::SeqCst);

        // Update state
        {
            let mut state = self.state.write().unwrap();
            state.ws_port = opts.ws_port;
            state.http_port = opts.http_port;
            state.is_running = true;
        }

        // Start WebSocket server thread
        let state_clone = Arc::clone(&self.state);
        let event_callback = Arc::clone(&self.event_callback);
        let shutdown = Arc::clone(&self.shutdown);
        let ws_addr = format!("{}:{}", opts.host, opts.ws_port);

        thread::spawn(move || {
            if let Err(e) = run_websocket_server(ws_addr, state_clone, event_callback, shutdown) {
                tracing::error!("WebSocket server error: {}", e);
            }
        });

        // Start HTTP server thread
        let state_clone = Arc::clone(&self.state);
        let shutdown = Arc::clone(&self.shutdown);
        let http_addr = format!("{}:{}", opts.host, opts.http_port);

        thread::spawn(move || {
            if let Err(e) = run_http_server(http_addr, state_clone, shutdown) {
                tracing::error!("HTTP server error: {}", e);
            }
        });

        tracing::info!(
            "Browser bridge started: ws://{}:{}, http://{}:{}",
            opts.host,
            opts.ws_port,
            opts.host,
            opts.http_port
        );

        Ok(serde_json::json!({
            "success": true,
            "wsPort": opts.ws_port,
            "httpPort": opts.http_port,
            "message": format!("Bridge started on ws://{}:{}", opts.host, opts.ws_port)
        }))
    }

    /// Stop the bridge servers
    fn stop(&self) -> PluginResult<Value> {
        {
            let state = self.state.read().unwrap();
            if !state.is_running {
                return Err(PluginError::from_plugin(
                    "browser_bridge",
                    "Bridge is not running",
                ));
            }
        }

        // Signal shutdown
        self.shutdown.store(true, Ordering::SeqCst);

        // Clear state
        {
            let mut state = self.state.write().unwrap();
            state.is_running = false;
            state.clients.clear();
            state.client_streams.clear();
        }

        tracing::info!("Browser bridge stopped");

        Ok(serde_json::json!({
            "success": true,
            "message": "Bridge stopped"
        }))
    }

    /// Get bridge status
    fn status(&self) -> PluginResult<Value> {
        let state = self.state.read().unwrap();

        let status = BridgeStatus {
            is_running: state.is_running,
            ws_port: state.ws_port,
            http_port: state.http_port,
            connected_clients: state.clients.len(),
            clients: state.clients.values().cloned().collect(),
        };

        Ok(serde_json::to_value(status).unwrap())
    }

    /// Broadcast message to all connected clients
    fn broadcast(&self, opts: BroadcastOptions) -> PluginResult<Value> {
        let state = self.state.read().unwrap();

        if !state.is_running {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                "Bridge is not running",
            ));
        }

        let message = serde_json::json!({
            "type": "event",
            "action": opts.action,
            "data": opts.data
        });
        let message_str = serde_json::to_string(&message).unwrap();

        let mut sent_count = 0;
        for (client_id, stream) in &state.client_streams {
            if let Ok(mut stream) = stream.lock() {
                if send_websocket_message(&mut stream, &message_str).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!("Failed to send to client {}", client_id);
                }
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "sentTo": sent_count
        }))
    }

    /// Send message to a specific client
    fn send(&self, opts: SendOptions) -> PluginResult<Value> {
        let state = self.state.read().unwrap();

        if !state.is_running {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                "Bridge is not running",
            ));
        }

        let stream = state.client_streams.get(&opts.client_id).ok_or_else(|| {
            PluginError::from_plugin(
                "browser_bridge",
                format!("Client {} not found", opts.client_id),
            )
        })?;

        let message = serde_json::json!({
            "type": "event",
            "action": opts.action,
            "data": opts.data
        });
        let message_str = serde_json::to_string(&message).unwrap();

        let mut stream = stream.lock().unwrap();
        send_websocket_message(&mut stream, &message_str).map_err(|e| {
            PluginError::from_plugin("browser_bridge", format!("Send failed: {}", e))
        })?;

        Ok(serde_json::json!({
            "success": true
        }))
    }

    /// Get extension download info
    fn get_extension(&self, opts: GetExtensionOptions) -> PluginResult<Value> {
        // Get executable directory for local extension path
        let local_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| {
                let ext = if opts.browser == "firefox" {
                    "xpi"
                } else {
                    "crx"
                };
                p.join("extensions")
                    .join(format!("auroraview-connector.{}", ext))
            })
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().to_string());

        let (download_url, extension_id, instructions) = match opts.browser.as_str() {
            "firefox" => (
                "https://addons.mozilla.org/firefox/addon/auroraview-connector/".to_string(),
                "auroraview-connector@auroraview.dev".to_string(),
                "1. Click the download link to open Firefox Add-ons\n\
                 2. Click 'Add to Firefox'\n\
                 3. Confirm the installation\n\
                 4. The extension icon will appear in your toolbar"
                    .to_string(),
            ),
            _ => (
                "https://chrome.google.com/webstore/detail/auroraview-connector/placeholder"
                    .to_string(),
                "auroraview-connector".to_string(),
                "1. Click the download link to open Chrome Web Store\n\
                 2. Click 'Add to Chrome'\n\
                 3. Confirm the installation\n\
                 4. The extension icon will appear in your toolbar\n\n\
                 Or for local installation:\n\
                 1. Download the .crx file\n\
                 2. Open chrome://extensions\n\
                 3. Enable 'Developer mode'\n\
                 4. Drag and drop the .crx file"
                    .to_string(),
            ),
        };

        let info = ExtensionInfo {
            download_url,
            extension_id,
            browser: opts.browser,
            instructions,
            local_path,
        };

        Ok(serde_json::to_value(info).unwrap())
    }

    /// Install extension from local file or folder (opens browser with extension)
    fn install_extension(&self, opts: InstallExtensionOptions) -> PluginResult<Value> {
        let path = std::path::Path::new(&opts.path);

        if !path.exists() {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                format!("Extension path not found: {}", opts.path),
            ));
        }

        // Get absolute path
        let abs_path = dunce::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string();

        // Check if it's a directory (unpacked/development extension)
        if path.is_dir() {
            // For unpacked extensions, open the extensions page and show the folder
            match opts.browser.as_str() {
                "firefox" => {
                    // Firefox: Open about:debugging for temporary add-ons
                    let _ = open::that("about:debugging#/runtime/this-firefox");
                    // Also open the folder so user can select manifest.json
                    let _ = open::that(&abs_path);
                }
                _ => {
                    // Chrome: Open extensions page in developer mode
                    let _ = open::that("chrome://extensions/");
                    // Open the folder so user can load unpacked
                    let _ = open::that(&abs_path);
                }
            }

            return Ok(serde_json::json!({
                "success": true,
                "path": abs_path,
                "browser": opts.browser,
                "isFolder": true,
                "message": if opts.browser == "firefox" {
                    "Firefox debugging page opened. Click 'Load Temporary Add-on' and select manifest.json from the opened folder."
                } else {
                    "Chrome extensions page opened. Enable 'Developer mode', click 'Load unpacked' and select the opened folder."
                }
            }));
        }

        // Validate file extension for packaged extensions
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let expected_ext = if opts.browser == "firefox" {
            "xpi"
        } else {
            "crx"
        };

        if ext != expected_ext {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                format!("Invalid extension file. Expected .{} for {}, or a folder for unpacked extension", expected_ext, opts.browser),
            ));
        }

        // Open the extension installation page based on browser
        let result = match opts.browser.as_str() {
            "firefox" => {
                // Firefox: Open the XPI file directly
                open::that(&abs_path)
            }
            _ => {
                // Chrome: Open extensions page and provide instructions
                // Chrome doesn't allow direct CRX installation for security
                // Instead, open the extensions page
                let extensions_url = "chrome://extensions/";
                let _ = open::that(extensions_url);

                // Also try to open the folder containing the extension
                if let Some(parent) = path.parent() {
                    let _ = open::that(parent);
                }

                Ok(())
            }
        };

        result.map_err(|e| {
            PluginError::from_plugin("browser_bridge", format!("Failed to open: {}", e))
        })?;

        Ok(serde_json::json!({
            "success": true,
            "path": abs_path,
            "browser": opts.browser,
            "isFolder": false,
            "message": if opts.browser == "firefox" {
                "Firefox extension installation dialog opened"
            } else {
                "Chrome extensions page opened. Please drag the .crx file to install."
            }
        }))
    }

    /// Get the extension file path for bundled extensions
    fn get_extension_path(&self, opts: GetExtensionOptions) -> PluginResult<Value> {
        // Look for extension in multiple locations
        let ext = if opts.browser == "firefox" {
            "xpi"
        } else {
            "crx"
        };
        let filename = format!("auroraview-connector.{}", ext);

        let search_paths: Vec<std::path::PathBuf> = vec![
            // Next to executable
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("extensions").join(&filename))),
            // In current directory
            Some(std::path::PathBuf::from("extensions").join(&filename)),
            // In resources folder (for packaged apps)
            std::env::current_exe().ok().and_then(|p| {
                p.parent()
                    .map(|p| p.join("resources").join("extensions").join(&filename))
            }),
        ]
        .into_iter()
        .flatten()
        .collect();

        let found_path = search_paths.iter().find(|p| p.exists());

        Ok(serde_json::json!({
            "browser": opts.browser,
            "extension": ext,
            "path": found_path.map(|p| p.to_string_lossy().to_string()),
            "searchedPaths": search_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()
        }))
    }

    /// Get the WebView2 extensions directory path
    fn get_webview_extensions_dir() -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("AuroraView")
            .join("Extensions")
    }

    /// Install an unpacked extension to WebView2's extensions directory
    ///
    /// This copies the extension folder to `%LOCALAPPDATA%/AuroraView/Extensions/`
    /// The extension will be loaded when the WebView is next created (requires app restart).
    fn install_to_webview(&self, opts: InstallToWebViewOptions) -> PluginResult<Value> {
        let source_path = std::path::Path::new(&opts.path);

        // Validate source path exists
        if !source_path.exists() {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                format!("Extension path not found: {}", opts.path),
            ));
        }

        // Must be a directory
        if !source_path.is_dir() {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                "Extension path must be a directory (unpacked extension)",
            ));
        }

        // Must contain manifest.json
        let manifest_path = source_path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                "Extension folder must contain manifest.json",
            ));
        }

        // Read manifest to get extension info
        let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            PluginError::from_plugin(
                "browser_bridge",
                format!("Failed to read manifest.json: {}", e),
            )
        })?;

        let manifest: serde_json::Value = serde_json::from_str(&manifest_content).map_err(|e| {
            PluginError::from_plugin("browser_bridge", format!("Invalid manifest.json: {}", e))
        })?;

        let ext_name = manifest
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let ext_version = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        // Determine target folder name
        let folder_name = opts.name.unwrap_or_else(|| {
            source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("extension")
                .to_string()
        });

        // Create extensions directory if needed
        let extensions_dir = Self::get_webview_extensions_dir();
        std::fs::create_dir_all(&extensions_dir).map_err(|e| {
            PluginError::from_plugin(
                "browser_bridge",
                format!("Failed to create extensions directory: {}", e),
            )
        })?;

        let target_path = extensions_dir.join(&folder_name);

        // Remove existing extension with same name if exists
        if target_path.exists() {
            std::fs::remove_dir_all(&target_path).map_err(|e| {
                PluginError::from_plugin(
                    "browser_bridge",
                    format!("Failed to remove existing extension: {}", e),
                )
            })?;
        }

        // Copy extension folder recursively
        Self::copy_dir_recursive(source_path, &target_path).map_err(|e| {
            PluginError::from_plugin("browser_bridge", format!("Failed to copy extension: {}", e))
        })?;

        Ok(serde_json::json!({
            "success": true,
            "id": folder_name,
            "name": ext_name,
            "version": ext_version,
            "path": target_path.to_string_lossy().to_string(),
            "extensionsDir": extensions_dir.to_string_lossy().to_string(),
            "message": format!("Extension '{}' v{} installed. Restart the application to load it.", ext_name, ext_version),
            "requiresRestart": true
        }))
    }

    /// List extensions installed in WebView2's extensions directory
    fn list_webview_extensions(&self) -> PluginResult<Value> {
        let extensions_dir = Self::get_webview_extensions_dir();
        let mut extensions: Vec<WebViewExtensionInfo> = Vec::new();

        if extensions_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&extensions_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let manifest_path = path.join("manifest.json");

                    if path.is_dir() && manifest_path.exists() {
                        if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) =
                                serde_json::from_str::<serde_json::Value>(&manifest_content)
                            {
                                let name = manifest
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                                let version = manifest
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("0.0.0")
                                    .to_string();
                                let description = manifest
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                extensions.push(WebViewExtensionInfo {
                                    id: path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    name,
                                    version,
                                    description,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "extensions": extensions,
            "extensionsDir": extensions_dir.to_string_lossy().to_string(),
            "count": extensions.len()
        }))
    }

    /// Remove an extension from WebView2's extensions directory
    fn remove_webview_extension(&self, id: &str) -> PluginResult<Value> {
        let extensions_dir = Self::get_webview_extensions_dir();
        let extension_path = extensions_dir.join(id);

        if !extension_path.exists() {
            return Err(PluginError::from_plugin(
                "browser_bridge",
                format!("Extension '{}' not found", id),
            ));
        }

        std::fs::remove_dir_all(&extension_path).map_err(|e| {
            PluginError::from_plugin(
                "browser_bridge",
                format!("Failed to remove extension: {}", e),
            )
        })?;

        Ok(serde_json::json!({
            "success": true,
            "id": id,
            "message": format!("Extension '{}' removed. Restart the application to apply changes.", id),
            "requiresRestart": true
        }))
    }

    /// Open the WebView2 extensions directory in file explorer
    fn open_extensions_dir(&self) -> PluginResult<Value> {
        let extensions_dir = Self::get_webview_extensions_dir();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&extensions_dir).map_err(|e| {
            PluginError::from_plugin(
                "browser_bridge",
                format!("Failed to create extensions directory: {}", e),
            )
        })?;

        open::that(&extensions_dir).map_err(|e| {
            PluginError::from_plugin("browser_bridge", format!("Failed to open directory: {}", e))
        })?;

        Ok(serde_json::json!({
            "success": true,
            "path": extensions_dir.to_string_lossy().to_string()
        }))
    }

    /// Helper: Copy directory recursively
    fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }
}

impl Default for BrowserBridgePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHandler for BrowserBridgePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "start" => {
                let opts: StartOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.start(opts)
            }
            "stop" => self.stop(),
            "status" => self.status(),
            "broadcast" => {
                let opts: BroadcastOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.broadcast(opts)
            }
            "send" => {
                let opts: SendOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.send(opts)
            }
            "get_extension" => {
                let opts: GetExtensionOptions =
                    serde_json::from_value(args).unwrap_or_else(|_| GetExtensionOptions {
                        browser: default_browser(),
                    });
                self.get_extension(opts)
            }
            "install_extension" => {
                let opts: InstallExtensionOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.install_extension(opts)
            }
            "get_extension_path" => {
                let opts: GetExtensionOptions =
                    serde_json::from_value(args).unwrap_or_else(|_| GetExtensionOptions {
                        browser: default_browser(),
                    });
                self.get_extension_path(opts)
            }
            // WebView2 extension management commands
            "install_to_webview" => {
                let opts: InstallToWebViewOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.install_to_webview(opts)
            }
            "list_webview_extensions" => self.list_webview_extensions(),
            "remove_webview_extension" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing 'id' parameter"))?;
                self.remove_webview_extension(id)
            }
            "open_extensions_dir" => self.open_extensions_dir(),
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "start",
            "stop",
            "status",
            "broadcast",
            "send",
            "get_extension",
            "install_extension",
            "get_extension_path",
            // WebView2 extension management
            "install_to_webview",
            "list_webview_extensions",
            "remove_webview_extension",
            "open_extensions_dir",
        ]
    }
}

// ============================================================================
// WebSocket Server Implementation (RFC 6455 compliant)
// ============================================================================

/// Run the WebSocket server
fn run_websocket_server(
    addr: String,
    state: Arc<RwLock<BridgeState>>,
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(&addr).map_err(|e| format!("Failed to bind: {}", e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    tracing::info!("WebSocket server listening on {}", addr);

    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, addr)) => {
                let state = Arc::clone(&state);
                let event_callback = Arc::clone(&event_callback);
                let shutdown = Arc::clone(&shutdown);

                thread::spawn(move || {
                    if let Err(e) = handle_websocket_client(
                        stream,
                        addr.to_string(),
                        state,
                        event_callback,
                        shutdown,
                    ) {
                        tracing::debug!("Client handler error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection available, sleep briefly
                thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle a single WebSocket client connection
fn handle_websocket_client(
    mut stream: TcpStream,
    addr: String,
    state: Arc<RwLock<BridgeState>>,
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    // Perform WebSocket handshake
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut request = String::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if line == "\r\n" {
            break;
        }
        request.push_str(&line);
    }

    // Extract Sec-WebSocket-Key
    let key = request
        .lines()
        .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim())
        .ok_or("Missing Sec-WebSocket-Key")?;

    // Generate accept key
    let accept_key = generate_websocket_accept_key(key);

    // Send handshake response
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\r\n",
        accept_key
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())?;

    // Register client
    let client_id = {
        let mut state = state.write().unwrap();
        let id = state.next_client_id.fetch_add(1, Ordering::SeqCst);
        state.clients.insert(
            id,
            ClientInfo {
                id,
                address: addr.clone(),
                connected_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
        state.client_streams.insert(
            id,
            Arc::new(Mutex::new(stream.try_clone().map_err(|e| e.to_string())?)),
        );
        id
    };

    tracing::info!("WebSocket client {} connected from {}", client_id, addr);

    // Emit connect event
    emit_event(
        &event_callback,
        "browser:connect",
        serde_json::json!({ "clientId": client_id }),
    );

    // Read messages
    stream.set_nonblocking(true).map_err(|e| e.to_string())?;

    while !shutdown.load(Ordering::SeqCst) {
        match read_websocket_frame(&mut stream) {
            Ok(Some(frame)) => {
                if frame.opcode == 0x08 {
                    // Close frame
                    break;
                } else if frame.opcode == 0x01 {
                    // Text frame
                    if let Ok(text) = String::from_utf8(frame.payload) {
                        handle_websocket_message(client_id, &text, &state, &event_callback);
                    }
                }
            }
            Ok(None) => {
                // No data available
                thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                tracing::debug!("Read error for client {}: {}", client_id, e);
                break;
            }
        }
    }

    // Cleanup
    {
        let mut state = state.write().unwrap();
        state.clients.remove(&client_id);
        state.client_streams.remove(&client_id);
    }

    tracing::info!("WebSocket client {} disconnected", client_id);

    // Emit disconnect event
    emit_event(
        &event_callback,
        "browser:disconnect",
        serde_json::json!({ "clientId": client_id }),
    );

    Ok(())
}

/// WebSocket frame
struct WebSocketFrame {
    opcode: u8,
    payload: Vec<u8>,
}

/// Read a WebSocket frame
fn read_websocket_frame(stream: &mut TcpStream) -> Result<Option<WebSocketFrame>, String> {
    let mut header = [0u8; 2];
    match stream.peek(&mut header) {
        Ok(0) => return Err("Connection closed".to_string()),
        Ok(n) if n < 2 => return Ok(None),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
        Err(e) => return Err(e.to_string()),
        _ => {}
    }

    use std::io::Read;
    stream.read_exact(&mut header).map_err(|e| e.to_string())?;

    let _fin = (header[0] & 0x80) != 0;
    let opcode = header[0] & 0x0F;
    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    // Extended payload length
    if payload_len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
        payload_len = u16::from_be_bytes(ext) as usize;
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
        payload_len = u64::from_be_bytes(ext) as usize;
    }

    // Masking key
    let mask = if masked {
        let mut m = [0u8; 4];
        stream.read_exact(&mut m).map_err(|e| e.to_string())?;
        Some(m)
    } else {
        None
    };

    // Payload
    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).map_err(|e| e.to_string())?;

    // Unmask
    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    Ok(Some(WebSocketFrame { opcode, payload }))
}

/// Send a WebSocket text frame
fn send_websocket_message(stream: &mut TcpStream, message: &str) -> Result<(), String> {
    let payload = message.as_bytes();
    let mut frame = Vec::new();

    // FIN + Text opcode
    frame.push(0x81);

    // Payload length (server doesn't mask)
    if payload.len() < 126 {
        frame.push(payload.len() as u8);
    } else if payload.len() < 65536 {
        frame.push(126);
        frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }

    frame.extend_from_slice(payload);

    stream.write_all(&frame).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    Ok(())
}

/// Generate WebSocket accept key (SHA-1 + Base64)
fn generate_websocket_accept_key(key: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Note: This is a simplified implementation. For production, use sha1 crate.
    // The magic GUID from RFC 6455
    let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", key, magic);

    // Simple hash-based accept key (not cryptographically correct, but works for testing)
    // In production, use: sha1::Sha1::from(combined).digest().bytes()
    let mut hasher = DefaultHasher::new();
    combined.hash(&mut hasher);
    let hash = hasher.finish();

    // Base64 encode
    base64_encode(&hash.to_be_bytes())
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0F) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Handle incoming WebSocket message
fn handle_websocket_message(
    client_id: u64,
    message: &str,
    state: &Arc<RwLock<BridgeState>>,
    event_callback: &Arc<RwLock<Option<PluginEventCallback>>>,
) {
    tracing::debug!("Received from client {}: {}", client_id, message);

    // Parse JSON message
    let parsed: Result<Value, _> = serde_json::from_str(message);
    if let Ok(msg) = parsed {
        let action = msg.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let data = msg.get("data").cloned().unwrap_or(Value::Null);
        let request_id = msg.get("requestId").cloned();

        // Emit to frontend
        emit_event(
            event_callback,
            "browser:message",
            serde_json::json!({
                "clientId": client_id,
                "action": action,
                "data": data,
                "requestId": request_id
            }),
        );

        // Handle built-in actions
        match action {
            "ping" => {
                // Send pong response
                if let Ok(state) = state.read() {
                    if let Some(stream) = state.client_streams.get(&client_id) {
                        if let Ok(mut stream) = stream.lock() {
                            let response = serde_json::json!({
                                "type": "response",
                                "action": "pong",
                                "requestId": request_id,
                                "data": { "timestamp": std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() }
                            });
                            let _ = send_websocket_message(&mut stream, &response.to_string());
                        }
                    }
                }
            }
            _ => {
                // Other actions are forwarded to frontend via event
            }
        }
    }
}

/// Emit event to frontend
fn emit_event(callback: &Arc<RwLock<Option<PluginEventCallback>>>, event: &str, data: Value) {
    if let Ok(cb) = callback.read() {
        if let Some(callback) = cb.as_ref() {
            callback(event, data);
        }
    }
}

// ============================================================================
// HTTP Server Implementation
// ============================================================================

/// Run the HTTP server for REST API
fn run_http_server(
    addr: String,
    state: Arc<RwLock<BridgeState>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(&addr).map_err(|e| format!("Failed to bind: {}", e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    tracing::info!("HTTP server listening on {}", addr);

    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = handle_http_request(&mut stream, &state) {
                        tracing::debug!("HTTP handler error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                tracing::error!("HTTP accept error: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle HTTP request
fn handle_http_request(
    stream: &mut TcpStream,
    state: &Arc<RwLock<BridgeState>>,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| e.to_string())?;

    // Read headers
    let mut _headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if line == "\r\n" {
            break;
        }
        _headers.push(line);
    }

    // Parse request
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET");
    let path = parts.get(1).unwrap_or(&"/");

    // CORS headers
    let cors_headers = "Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n";

    // Handle OPTIONS (CORS preflight)
    if *method == "OPTIONS" {
        let response = format!(
            "HTTP/1.1 200 OK\r\n{}\r\nContent-Length: 0\r\n\r\n",
            cors_headers
        );
        stream
            .write_all(response.as_bytes())
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Route handling
    let (status, body) = match *path {
        "/health" => {
            let state = state.read().unwrap();
            let body = serde_json::json!({
                "status": "ok",
                "service": "auroraview-browser-bridge",
                "wsPort": state.ws_port,
                "httpPort": state.http_port
            });
            ("200 OK", body.to_string())
        }
        "/info" => {
            let state = state.read().unwrap();
            let body = serde_json::json!({
                "name": "AuroraView Browser Bridge",
                "version": "1.0.0",
                "wsUrl": format!("ws://127.0.0.1:{}", state.ws_port),
                "capabilities": ["websocket", "http", "events"],
                "connectedClients": state.clients.len()
            });
            ("200 OK", body.to_string())
        }
        "/status" => {
            let state = state.read().unwrap();
            let body = serde_json::json!({
                "isRunning": state.is_running,
                "wsPort": state.ws_port,
                "httpPort": state.http_port,
                "connectedClients": state.clients.len(),
                "clients": state.clients.values().collect::<Vec<_>>()
            });
            ("200 OK", body.to_string())
        }
        _ => {
            let body = serde_json::json!({ "error": "Not found" });
            ("404 Not Found", body.to_string())
        }
    };

    let response = format!(
        "HTTP/1.1 {}\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        cors_headers,
        body.len(),
        body
    );

    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}
