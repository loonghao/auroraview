//! DevTools and CDP (Chrome DevTools Protocol) support
//!
//! This module provides developer tools integration for the browser,
//! including:
//! - DevTools panel toggle
//! - CDP remote debugging support
//! - JavaScript console access
//! - Network inspection hooks

use serde::{Deserialize, Serialize};

/// DevTools configuration
#[derive(Debug, Clone)]
pub struct DevToolsConfig {
    /// Enable DevTools access (F12)
    pub enabled: bool,
    /// Remote debugging port for CDP (0 = disabled)
    pub remote_debugging_port: u16,
    /// Auto-open DevTools on launch
    pub auto_open: bool,
    /// DevTools dock position
    pub dock_side: DockSide,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            remote_debugging_port: 0,
            auto_open: false,
            dock_side: DockSide::Right,
        }
    }
}

/// DevTools dock position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockSide {
    /// Dock to the right side
    Right,
    /// Dock to the bottom
    Bottom,
    /// Dock to the left side
    Left,
    /// Undock into a separate window
    Undocked,
}

/// DevTools state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevToolsState {
    /// Whether DevTools is currently open
    pub is_open: bool,
    /// Current dock side
    pub dock_side: Option<DockSide>,
    /// Selected panel (elements, console, network, etc.)
    pub selected_panel: Option<String>,
}

/// CDP (Chrome DevTools Protocol) session info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpSessionInfo {
    /// WebSocket URL for DevTools connection
    pub websocket_debugger_url: String,
    /// DevTools frontend URL
    pub devtools_frontend_url: String,
    /// Browser version
    pub browser: String,
    /// Protocol version
    pub protocol_version: String,
    /// User agent
    pub user_agent: String,
    /// V8 version
    pub v8_version: String,
    /// WebKit version
    pub webkit_version: String,
}

/// Console message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleMessageType {
    Log,
    Debug,
    Info,
    Warning,
    Error,
}

/// Console message from DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleMessage {
    /// Message type
    pub message_type: ConsoleMessageType,
    /// Message text
    pub text: String,
    /// Source URL
    pub source: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Stack trace (if error)
    pub stack_trace: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

/// Network request info for DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkRequestInfo {
    /// Request ID
    pub request_id: String,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Request headers
    pub headers: std::collections::HashMap<String, String>,
    /// Post data (if any)
    pub post_data: Option<String>,
    /// Resource type
    pub resource_type: String,
    /// Timestamp
    pub timestamp: f64,
}

/// Network response info for DevTools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkResponseInfo {
    /// Request ID
    pub request_id: String,
    /// Status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// MIME type
    pub mime_type: String,
    /// Content length
    pub content_length: Option<u64>,
    /// Whether response came from cache
    pub from_cache: bool,
    /// Timestamp
    pub timestamp: f64,
}

/// DevTools manager for a tab
pub struct DevToolsManager {
    /// Configuration
    config: DevToolsConfig,
    /// Current state
    state: DevToolsState,
    /// Console messages
    console_messages: Vec<ConsoleMessage>,
    /// Network requests (request_id -> info)
    network_requests: std::collections::HashMap<String, NetworkRequestInfo>,
}

impl DevToolsManager {
    /// Create a new DevTools manager
    pub fn new(config: DevToolsConfig) -> Self {
        Self {
            config,
            state: DevToolsState::default(),
            console_messages: Vec::new(),
            network_requests: std::collections::HashMap::new(),
        }
    }

    /// Check if DevTools is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if DevTools is currently open
    pub fn is_open(&self) -> bool {
        self.state.is_open
    }

    /// Get remote debugging port
    pub fn remote_debugging_port(&self) -> u16 {
        self.config.remote_debugging_port
    }

    /// Open DevTools
    pub fn open(&mut self) {
        self.state.is_open = true;
    }

    /// Close DevTools
    pub fn close(&mut self) {
        self.state.is_open = false;
    }

    /// Toggle DevTools
    pub fn toggle(&mut self) {
        self.state.is_open = !self.state.is_open;
    }

    /// Set dock side
    pub fn set_dock_side(&mut self, side: DockSide) {
        self.state.dock_side = Some(side);
    }

    /// Add console message
    pub fn add_console_message(&mut self, message: ConsoleMessage) {
        // Keep last 1000 messages
        if self.console_messages.len() > 1000 {
            self.console_messages.remove(0);
        }
        self.console_messages.push(message);
    }

    /// Get console messages
    pub fn console_messages(&self) -> &[ConsoleMessage] {
        &self.console_messages
    }

    /// Clear console messages
    pub fn clear_console(&mut self) {
        self.console_messages.clear();
    }

    /// Track network request
    pub fn add_network_request(&mut self, request: NetworkRequestInfo) {
        self.network_requests.insert(request.request_id.clone(), request);
    }

    /// Get network requests
    pub fn network_requests(&self) -> &std::collections::HashMap<String, NetworkRequestInfo> {
        &self.network_requests
    }

    /// Clear network requests
    pub fn clear_network(&mut self) {
        self.network_requests.clear();
    }

    /// Get current state
    pub fn state(&self) -> &DevToolsState {
        &self.state
    }

    /// Get config
    pub fn config(&self) -> &DevToolsConfig {
        &self.config
    }
}

/// CDP protocol commands
pub mod cdp {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// CDP command request
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CdpRequest {
        /// Command ID
        pub id: u64,
        /// Method name (e.g., "Page.navigate", "Runtime.evaluate")
        pub method: String,
        /// Parameters
        #[serde(default)]
        pub params: Value,
    }

    /// CDP command response
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CdpResponse {
        /// Command ID (matches request)
        pub id: u64,
        /// Result (if success)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        /// Error (if failed)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<CdpError>,
    }

    /// CDP error
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CdpError {
        /// Error code
        pub code: i32,
        /// Error message
        pub message: String,
        /// Additional data
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<String>,
    }

    /// CDP event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CdpEvent {
        /// Event method (e.g., "Page.loadEventFired", "Console.messageAdded")
        pub method: String,
        /// Event parameters
        #[serde(default)]
        pub params: Value,
    }

    /// Common CDP domains
    pub mod domains {
        /// Page domain - Page navigation and lifecycle
        pub const PAGE: &str = "Page";
        /// Runtime domain - JavaScript runtime
        pub const RUNTIME: &str = "Runtime";
        /// Network domain - Network activity
        pub const NETWORK: &str = "Network";
        /// DOM domain - DOM tree access
        pub const DOM: &str = "DOM";
        /// Console domain - Console messages
        pub const CONSOLE: &str = "Console";
        /// Debugger domain - JavaScript debugging
        pub const DEBUGGER: &str = "Debugger";
        /// Profiler domain - Performance profiling
        pub const PROFILER: &str = "Profiler";
        /// Target domain - Target/session management
        pub const TARGET: &str = "Target";
        /// Browser domain - Browser-level operations
        pub const BROWSER: &str = "Browser";
    }

    /// Common CDP methods
    pub mod methods {
        // Page domain
        pub const PAGE_NAVIGATE: &str = "Page.navigate";
        pub const PAGE_RELOAD: &str = "Page.reload";
        pub const PAGE_GET_FRAME_TREE: &str = "Page.getFrameTree";
        pub const PAGE_CAPTURE_SCREENSHOT: &str = "Page.captureScreenshot";

        // Runtime domain
        pub const RUNTIME_EVALUATE: &str = "Runtime.evaluate";
        pub const RUNTIME_CALL_FUNCTION_ON: &str = "Runtime.callFunctionOn";
        pub const RUNTIME_GET_PROPERTIES: &str = "Runtime.getProperties";

        // Network domain
        pub const NETWORK_ENABLE: &str = "Network.enable";
        pub const NETWORK_DISABLE: &str = "Network.disable";
        pub const NETWORK_SET_EXTRA_HEADERS: &str = "Network.setExtraHTTPHeaders";

        // DOM domain
        pub const DOM_GET_DOCUMENT: &str = "DOM.getDocument";
        pub const DOM_QUERY_SELECTOR: &str = "DOM.querySelector";
        pub const DOM_QUERY_SELECTOR_ALL: &str = "DOM.querySelectorAll";

        // Target domain
        pub const TARGET_GET_TARGETS: &str = "Target.getTargets";
        pub const TARGET_CREATE_TARGET: &str = "Target.createTarget";
        pub const TARGET_CLOSE_TARGET: &str = "Target.closeTarget";

        // Browser domain
        pub const BROWSER_GET_VERSION: &str = "Browser.getVersion";
        pub const BROWSER_CLOSE: &str = "Browser.close";
    }

    impl CdpRequest {
        /// Create a new CDP request
        pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
            Self {
                id,
                method: method.into(),
                params,
            }
        }

        /// Create a simple request without parameters
        pub fn simple(id: u64, method: impl Into<String>) -> Self {
            Self::new(id, method, Value::Null)
        }
    }

    impl CdpResponse {
        /// Create a success response
        pub fn success(id: u64, result: Value) -> Self {
            Self {
                id,
                result: Some(result),
                error: None,
            }
        }

        /// Create an error response
        pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
            Self {
                id,
                result: None,
                error: Some(CdpError {
                    code,
                    message: message.into(),
                    data: None,
                }),
            }
        }

        /// Check if response is success
        pub fn is_success(&self) -> bool {
            self.error.is_none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_config_default() {
        let config = DevToolsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.remote_debugging_port, 0);
        assert!(!config.auto_open);
        assert_eq!(config.dock_side, DockSide::Right);
    }

    #[test]
    fn test_devtools_manager() {
        let config = DevToolsConfig {
            enabled: true,
            remote_debugging_port: 9222,
            ..Default::default()
        };
        let mut manager = DevToolsManager::new(config);

        assert!(!manager.is_open());
        manager.open();
        assert!(manager.is_open());
        manager.toggle();
        assert!(!manager.is_open());
        assert_eq!(manager.remote_debugging_port(), 9222);
    }

    #[test]
    fn test_console_messages() {
        let mut manager = DevToolsManager::new(DevToolsConfig::default());

        manager.add_console_message(ConsoleMessage {
            message_type: ConsoleMessageType::Log,
            text: "Test message".to_string(),
            source: None,
            line: None,
            column: None,
            stack_trace: None,
            timestamp: 0,
        });

        assert_eq!(manager.console_messages().len(), 1);
        manager.clear_console();
        assert!(manager.console_messages().is_empty());
    }

    #[test]
    fn test_cdp_request() {
        use cdp::CdpRequest;
        use serde_json::json;

        let req = CdpRequest::new(1, "Page.navigate", json!({"url": "https://example.com"}));
        assert_eq!(req.id, 1);
        assert_eq!(req.method, "Page.navigate");
    }

    #[test]
    fn test_cdp_response() {
        use cdp::CdpResponse;
        use serde_json::json;

        let success = CdpResponse::success(1, json!({"frameId": "123"}));
        assert!(success.is_success());

        let error = CdpResponse::error(2, -32601, "Method not found");
        assert!(!error.is_success());
    }
}
