use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a WebView instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebViewId(pub String);

impl WebViewId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::str::FromStr for WebViewId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for WebViewId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WebViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a WebView instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewInfo {
    pub id: WebViewId,
    pub title: String,
    pub url: String,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    /// Raw HWND on Windows (0 if not available).
    pub hwnd: u64,
}

/// Configuration for creating a new WebView.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewConfig {
    pub title: Option<String>,
    pub url: Option<String>,
    pub html: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub visible: Option<bool>,
    pub debug: Option<bool>,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            title: Some("AuroraView".to_string()),
            url: None,
            html: None,
            width: Some(800),
            height: Some(600),
            visible: Some(true),
            debug: Some(false),
        }
    }
}

/// Screenshot result containing image data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotData {
    /// Base64-encoded PNG image data.
    pub data: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

impl ScreenshotData {
    pub fn new_placeholder(width: u32, height: u32) -> Self {
        Self {
            data: String::new(),
            width,
            height,
            format: "png".to_string(),
        }
    }
}

/// Result of JavaScript evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsResult {
    pub value: serde_json::Value,
    pub error: Option<String>,
}

impl JsResult {
    pub fn ok(value: serde_json::Value) -> Self {
        Self { value, error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            value: serde_json::Value::Null,
            error: Some(msg.into()),
        }
    }
}

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub enable_mdns: bool,
    /// Maximum number of concurrent WebView instances.
    /// `None` means no limit.
    pub max_webviews: Option<usize>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7890,
            service_name: "auroraview-mcp".to_string(),
            enable_mdns: true,
            max_webviews: None,
        }
    }
}

impl McpServerConfig {
    /// Set the port number.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the bind host.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Enable or disable mDNS broadcast.
    pub fn with_mdns(mut self, enabled: bool) -> Self {
        self.enable_mdns = enabled;
        self
    }

    /// Set the mDNS service name.
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Set the maximum number of concurrent WebView instances.
    pub fn with_max_webviews(mut self, max: usize) -> Self {
        self.max_webviews = Some(max);
        self
    }

    /// Validate the configuration.
    ///
    /// Returns an error message if any field is invalid.
    ///
    /// Checks:
    /// - `port` must be in the valid range 1–65535 (0 is reserved)
    /// - `host` must not be empty
    /// - `service_name` must not be empty
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("port must be in range 1–65535 (got 0)".to_string());
        }
        if self.host.trim().is_empty() {
            return Err("host must not be empty".to_string());
        }
        if self.service_name.trim().is_empty() {
            return Err("service_name must not be empty".to_string());
        }
        Ok(())
    }

    /// Return `true` if the configuration is valid.
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}
