//! MCP Server configuration

use serde::{Deserialize, Serialize};

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name (shown to AI assistants)
    pub name: String,

    /// Server version
    pub version: String,

    /// Host to bind to (default: 127.0.0.1)
    pub host: String,

    /// Port to listen on (0 = auto-assign)
    pub port: u16,

    /// Auto-expose bound API methods as tools
    pub auto_expose_api: bool,

    /// Expose event emission capability
    pub expose_events: bool,

    /// Expose DOM manipulation tools
    pub expose_dom: bool,

    /// Expose debug tools (console logs, etc.)
    pub expose_debug: bool,

    /// Allowed CORS origins
    pub allowed_origins: Vec<String>,

    /// Require authentication
    pub require_auth: bool,

    /// Authentication token (if require_auth is true)
    pub auth_token: Option<String>,

    /// Maximum concurrent SSE connections
    pub max_connections: usize,

    /// SSE heartbeat interval in seconds
    pub heartbeat_interval: u64,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Execute tool handlers directly in tokio thread (default: true)
    ///
    /// When true (default), tool handlers are executed directly in the tokio
    /// runtime thread using Python's GIL. This is simpler and works well for
    /// most use cases.
    ///
    /// When false, tool calls are routed through the WebView's message queue
    /// to execute on the main/event-loop thread. This is only needed for tools
    /// that must interact with the WebView UI directly.
    pub direct_execution: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            name: "auroraview-embedded".to_string(),
            version: "1.0.0".to_string(),
            host: "127.0.0.1".to_string(),
            port: 0, // Auto-assign
            auto_expose_api: true,
            expose_events: true,
            expose_dom: true,
            expose_debug: true,
            allowed_origins: vec!["*".to_string()],
            require_auth: false,
            auth_token: None,
            max_connections: 10,
            heartbeat_interval: 15,
            timeout: 30,
            direct_execution: true, // Default: execute directly in tokio thread
        }
    }
}

impl McpConfig {
    /// Create a new config with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the server port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Enable authentication with the given token
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.require_auth = true;
        self.auth_token = Some(token.into());
        self
    }

    /// Disable auto-expose of API methods
    pub fn without_auto_expose(mut self) -> Self {
        self.auto_expose_api = false;
        self
    }

    /// Disable DOM tools
    pub fn without_dom(mut self) -> Self {
        self.expose_dom = false;
        self
    }

    /// Disable debug tools
    pub fn without_debug(mut self) -> Self {
        self.expose_debug = false;
        self
    }
}
