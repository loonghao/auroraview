use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a `WebView` instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebViewId(pub String);

impl WebViewId {
    #[must_use]
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

/// Information about a `WebView` instance.
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
    /// CDP endpoint for `DevTools` (e.g. `http://127.0.0.1:9222`).
    /// `None` if CDP is not enabled for this `WebView`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdp_endpoint: Option<String>,
}

impl std::fmt::Display for WebViewInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WebViewInfo {{ id: {}, title: \"{}\", url: \"{}\", visible: {}, {}x{}, hwnd: {} }}",
            self.id, self.title, self.url, self.visible, self.width, self.height, self.hwnd
        )
    }
}

/// Configuration for creating a new `WebView`.
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

impl std::fmt::Display for WebViewConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WebViewConfig {{ title: {:?}, url: {:?}, html: {:?}, {}x{}, visible: {}, debug: {} }}",
            self.title,
            self.url,
            self.html,
            self.width.unwrap_or(800),
            self.height.unwrap_or(600),
            self.visible.unwrap_or(true),
            self.debug.unwrap_or(false)
        )
    }
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

impl std::fmt::Display for ScreenshotData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScreenshotData {{ {}x{}, format: {} }}",
            self.width, self.height, self.format
        )
    }
}

impl ScreenshotData {
    #[must_use]
    pub fn new_placeholder(width: u32, height: u32) -> Self {
        Self {
            data: String::new(),
            width,
            height,
            format: "png".to_string(),
        }
    }

    /// Create `ScreenshotData` from raw image bytes (PNG/JPEG/WebP).
    #[must_use]
    pub fn from_bytes(bytes: &[u8], width: u32, height: u32, format: &str) -> Self {
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        Self {
            data,
            width,
            height,
            format: format.to_string(),
        }
    }
}

/// Result of JavaScript evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsResult {
    pub value: serde_json::Value,
    pub error: Option<String>,
}

impl std::fmt::Display for JsResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref err) = self.error {
            write!(f, "JsResult::Err({err})")
        } else {
            write!(f, "JsResult::Ok({})", self.value)
        }
    }
}

impl JsResult {
    #[must_use]
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
    /// Enable OAuth 2.0 authentication for MCP endpoints.
    /// When enabled, clients must authenticate via OAuth 2.0
    /// to access MCP tools.
    #[serde(default)]
    pub enable_oauth: bool,
    /// Maximum number of concurrent `WebView` instances.
    /// `None` means no limit.
    #[serde(default)]
    pub max_webviews: Option<usize>,
}

impl std::fmt::Display for McpServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "McpServerConfig {{ host: {}, port: {}, service_name: \"{}\", mdns: {}, oauth: {}, max_webviews: {:?} }}",
            self.host,
            self.port,
            self.service_name,
            self.enable_mdns,
            self.enable_oauth,
            self.max_webviews
        )
    }
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7890,
            service_name: "auroraview-mcp".to_string(),
            enable_mdns: true,
            enable_oauth: false,
            max_webviews: None,
        }
    }
}

impl McpServerConfig {
    /// Set the port number.
    #[must_use]
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
    #[must_use]
    pub fn with_mdns(mut self, enabled: bool) -> Self {
        self.enable_mdns = enabled;
        self
    }

    /// Enable or disable OAuth 2.0 authentication.
    ///
    /// When enabled, clients must authenticate via OAuth 2.0
    /// to access MCP endpoints.
    #[must_use]
    pub fn with_oauth(mut self, enabled: bool) -> Self {
        self.enable_oauth = enabled;
        self
    }

    /// Set the mDNS service name.
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Set the maximum number of concurrent `WebView` instances.
    #[must_use]
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
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Build a fully-configured instance in a single call.
    ///
    /// Equivalent to chaining `with_port`, `with_host`, `with_service_name`,
    /// `with_mdns`, `with_oauth`, and `with_max_webviews` on a default config.
    ///
    /// # Example
    ///
    /// ```rust
    /// use auroraview_mcp::types::McpServerConfig;
    ///
    /// let cfg = McpServerConfig::with_all(
    ///     7891,
    ///     "0.0.0.0",
    ///     "my-mcp",
    ///     true,
    ///     false,
    ///     Some(10),
    /// );
    /// assert_eq!(cfg.port, 7891);
    /// assert_eq!(cfg.max_webviews, Some(10));
    /// assert!(!cfg.enable_oauth);
    /// ```
    pub fn with_all(
        port: u16,
        host: impl Into<String>,
        service_name: impl Into<String>,
        enable_mdns: bool,
        enable_oauth: bool,
        max_webviews: Option<usize>,
    ) -> Self {
        let mut cfg = Self::default()
            .with_port(port)
            .with_host(host)
            .with_service_name(service_name)
            .with_mdns(enable_mdns)
            .with_oauth(enable_oauth);
        if let Some(max) = max_webviews {
            cfg = cfg.with_max_webviews(max);
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // WebViewId tests
    #[test]
    fn webview_id_new_creates_unique_ids() {
        let id1 = WebViewId::new();
        let id2 = WebViewId::new();
        assert_ne!(id1, id2, "WebViewId::new() should create unique IDs");
    }

    #[test]
    fn webview_id_from_str() {
        let id = WebViewId::from_str("test-id-123").unwrap();
        assert_eq!(id, WebViewId("test-id-123".to_string()));
    }

    #[test]
    fn webview_id_display() {
        let id = WebViewId("test-id".to_string());
        assert_eq!(format!("{id}"), "test-id");
    }

    // ScreenshotData tests
    #[test]
    fn screenshot_data_new_placeholder() {
        let data = ScreenshotData::new_placeholder(800, 600);
        assert_eq!(data.width, 800);
        assert_eq!(data.height, 600);
        assert_eq!(data.format, "png");
        assert!(data.data.is_empty());
    }

    #[test]
    fn screenshot_data_from_bytes() {
        let bytes = b"fake png data";
        let data = ScreenshotData::from_bytes(bytes, 1024, 768, "png");
        assert_eq!(data.width, 1024);
        assert_eq!(data.height, 768);
        assert_eq!(data.format, "png");
        assert!(!data.data.is_empty());
        // Verify it's valid base64
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&data.data)
            .expect("should be valid base64");
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn screenshot_data_display() {
        let data = ScreenshotData::new_placeholder(800, 600);
        let display = format!("{data}");
        assert!(display.contains("800"));
        assert!(display.contains("600"));
        assert!(display.contains("png"));
    }

    // JsResult tests
    #[test]
    fn js_result_ok() {
        let result = JsResult::ok(serde_json::json!({"key": "value"}));
        assert_eq!(result.value, serde_json::json!({"key": "value"}));
        assert!(result.error.is_none());
    }

    #[test]
    fn js_result_err() {
        let result = JsResult::err("something went wrong");
        assert_eq!(result.value, serde_json::Value::Null);
        assert_eq!(result.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn js_result_display_ok() {
        let result = JsResult::ok(serde_json::json!("hello"));
        let display = format!("{result}");
        assert!(display.contains("Ok"));
        assert!(display.contains("hello"));
    }

    #[test]
    fn js_result_display_err() {
        let result = JsResult::err("fail");
        let display = format!("{result}");
        assert!(display.contains("Err"));
        assert!(display.contains("fail"));
    }

    // McpServerConfig tests
    #[test]
    fn mcp_server_config_default() {
        let cfg = McpServerConfig::default();
        assert_eq!(cfg.port, 7890);
        assert_eq!(cfg.host, "127.0.0.1");
        assert!(cfg.enable_mdns);
        assert!(!cfg.enable_oauth);
        assert_eq!(cfg.max_webviews, None);
    }

    #[test]
    fn mcp_server_config_with_all() {
        let cfg = McpServerConfig::with_all(7891, "0.0.0.0", "my-mcp", true, false, Some(10));
        assert_eq!(cfg.port, 7891);
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.service_name, "my-mcp");
        assert!(cfg.enable_mdns);
        assert!(!cfg.enable_oauth);
        assert_eq!(cfg.max_webviews, Some(10));
    }

    #[test]
    fn mcp_server_config_validate_invalid_port() {
        let cfg = McpServerConfig {
            port: 0,
            ..McpServerConfig::default()
        };
        let err = cfg.validate().expect_err("port 0 should be invalid");
        assert!(err.contains("port"), "error should mention 'port': {err}");
    }

    #[test]
    fn mcp_server_config_validate_empty_host() {
        let cfg = McpServerConfig {
            host: "   ".to_string(),
            ..McpServerConfig::default()
        };
        let err = cfg.validate().expect_err("empty host should be invalid");
        assert!(err.contains("host"), "error should mention 'host': {err}");
    }

    #[test]
    fn mcp_server_config_validate_empty_service_name() {
        let cfg = McpServerConfig {
            service_name: "".to_string(),
            ..McpServerConfig::default()
        };
        let err = cfg
            .validate()
            .expect_err("empty service_name should be invalid");
        assert!(
            err.contains("service_name"),
            "error should mention 'service_name': {err}"
        );
    }

    #[test]
    fn mcp_server_config_is_valid() {
        let cfg = McpServerConfig::default();
        assert!(cfg.is_valid());

        let invalid_cfg = McpServerConfig {
            port: 0,
            ..McpServerConfig::default()
        };
        assert!(!invalid_cfg.is_valid());
    }

    #[test]
    fn mcp_server_config_display() {
        let cfg = McpServerConfig::default();
        let display = format!("{cfg}");
        assert!(display.contains("7890"));
        assert!(display.contains("127.0.0.1"));
        assert!(display.contains("auroraview-mcp"));
    }
}
