//! MCP Server implementation for AuroraView.
//!
//! Exposes AuroraView capabilities as standard MCP tools via HTTP/SSE transport.
//! Uses `rmcp` crate with `StreamableHttpService` for HTTP-based MCP communication.

use rmcp::{handler::server::wrapper::Parameters, schemars::JsonSchema, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{debug, error, info, warn};

use crate::agui::AguiBus;
use crate::cdp::{CdpClient, CdpError};
use crate::registry::WebViewRegistry;
use crate::{CdpAdapterConfig, DEFAULT_CDP_TIMEOUT};

// ---------------------------------------------------------------------------
// Tool parameter structs
// ---------------------------------------------------------------------------

/// Parameters for the `screenshot` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScreenshotParams {
    /// Image format: "png", "jpeg", or "webp". Defaults to "png".
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "png".to_owned()
}

/// Parameters for the `eval_js` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvalJsParams {
    /// JavaScript expression to evaluate in the WebView context.
    pub script: String,
}

/// Parameters for the `load_url` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LoadUrlParams {
    /// URL to load in the WebView.
    pub url: String,
}

/// Parameters for the `send_event` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendEventParams {
    /// Event name to emit in the WebView.
    pub event: String,
    /// Event payload (JSON value).
    pub data: serde_json::Value,
}

/// Parameters for the `get_hwnd` tool (placeholder).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetHwndParams {}

/// Parameters for the `list_webviews` tool (placeholder).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListWebviewsParams {}

/// Parameters for the `create_webview` tool (placeholder).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateWebviewParams {
    /// WebView configuration (JSON).
    pub config: serde_json::Value,
}

/// Parameters for the `close_webview` tool (placeholder).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CloseWebviewParams {
    /// WebView ID to close.
    pub id: String,
}

// ---------------------------------------------------------------------------
// McpServer — rmcp ServerHandler implementation
// ---------------------------------------------------------------------------

/// MCP Server that bridges rmcp protocol to a running AuroraView CDP endpoint.
///
/// `McpServer` implements the `rmcp` crate's `ServerHandler` trait and exposes
/// AuroraView capabilities as standard MCP tools (screenshot, eval_js, load_url, etc.).
///
/// It reuses a single CDP connection for all tool calls (lazily established on first use).
///
/// # Example
///
/// ```rust,ignore
/// let config = CdpAdapterConfig::localhost(9222, "0.5.2");
/// let server = McpServer::new(config);
/// let bus = AguiBus::new();
/// let server = server.with_agui_bus(bus);
/// ```
#[derive(Clone)]
pub struct McpServer {
    /// CDP adapter configuration (HTTP/WS endpoints).
    config: CdpAdapterConfig,
    /// Registry of active WebView instances.
    registry: WebViewRegistry,
    /// AG-UI event bus (None if not enabled).
    agui_bus: Option<AguiBus>,
    /// Lazily initialized CDP client (shared across tool calls).
    client: Arc<OnceCell<CdpClient>>,
}

impl McpServer {
    /// Create a new MCP server that will connect to the given CDP endpoint.
    ///
    /// The server starts without an AG-UI bus. Use `with_agui_bus()` to enable
    /// AG-UI event streaming.
    pub fn new(config: CdpAdapterConfig) -> Self {
        Self {
            config,
            registry: WebViewRegistry::new(),
            agui_bus: None,
            client: Arc::new(OnceCell::new()),
        }
    }

    /// Set the AG-UI event bus.
    ///
    /// Call this to enable AG-UI event streaming via `/agui/events` SSE endpoint.
    /// The bus is used to emit events that frontend clients can subscribe to.
    #[must_use]
    pub fn with_agui_bus(mut self, bus: AguiBus) -> Self {
        self.agui_bus = Some(bus);
        self
    }

    /// Get or create a shared CDP client (lazily initialized on first use).
    ///
    /// This method is called internally by tool handlers. It establishes a CDP
    /// connection on first call, then reuses the same connection for subsequent calls.
    async fn get_client(&self) -> Result<CdpClient, CdpError> {
        let start = std::time::Instant::now();
        let client_ref = self
            .client
            .get_or_try_init(|| async { CdpClient::connect(&self.config.http_endpoint).await })
            .await?;
        debug!(elapsed = ?start.elapsed(), "get_client() completed");
        Ok(client_ref.clone())
    }

    /// Return a reference to the WebView registry.
    ///
    /// The registry tracks all registered WebView instances. Currently a placeholder
    /// (will be used when `create_webview` tool is implemented).
    #[must_use]
    pub fn registry(&self) -> &WebViewRegistry {
        &self.registry
    }
}

#[tool_router(server_handler)]
impl McpServer {
    /// Capture a screenshot of the current WebView.
    ///
    /// Returns the image as a base64-encoded data URI.
    #[tool(description = "Capture a screenshot of the current WebView")]
    async fn screenshot(
        &self,
        Parameters(params): Parameters<ScreenshotParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let bytes = client
            .capture_screenshot(&params.format, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "screenshot failed");
                rmcp::ErrorData::internal_error(format!("screenshot failed: {e}"), None)
            })?;
        debug!(format = %params.format, size = bytes.len(), "screenshot captured");
        let mime = match params.format.as_str() {
            "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            _ => "image/png",
        };
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(format!("data:{mime};base64,{b64}"))
    }

    /// Evaluate a JavaScript expression in the WebView context.
    ///
    /// Returns the JSON-serialized result of the expression.
    #[tool(description = "Evaluate JavaScript in the WebView and return the result")]
    async fn eval_js(
        &self,
        Parameters(params): Parameters<EvalJsParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let value = client
            .evaluate_script(&params.script, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, script = %params.script, "eval_js failed");
                rmcp::ErrorData::internal_error(format!("eval_js failed: {e}"), None)
            })?;
        debug!(script = %params.script, "eval_js completed");
        Ok(serde_json::to_string(&value).unwrap_or_else(|_| "null".to_owned()))
    }

    /// Navigate the WebView to a URL.
    #[tool(description = "Navigate the WebView to a URL")]
    async fn load_url(
        &self,
        Parameters(params): Parameters<LoadUrlParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .navigate_to(&params.url, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, url = %params.url, "load_url failed");
                rmcp::ErrorData::internal_error(format!("load_url failed: {e}"), None)
            })?;
        info!(url = %params.url, "URL loaded");
        Ok(format!("navigated to {}", params.url))
    }

    /// Send an event to the WebView.
    ///
    /// Emits `event` with `data` via `window.auroraview.trigger()`.
    #[tool(description = "Send an event to the WebView via window.auroraview.trigger()")]
    async fn send_event(
        &self,
        Parameters(params): Parameters<SendEventParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let data_str = serde_json::to_string(&params.data).map_err(|e| {
            warn!(error = %e, "JSON serialize failed");
            rmcp::ErrorData::internal_error(format!("JSON serialize failed: {e}"), None)
        })?;
        let script = format!("if(window.auroraview && window.auroraview.trigger){{ window.auroraview.trigger('{}', {}); }} else {{ console.error('[AuroraView] Event bridge not ready'); }}", params.event.replace('\'', "\\'"), data_str);
        client
            .evaluate_script(&script, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, event = %params.event, "send_event failed");
                rmcp::ErrorData::internal_error(format!("send_event failed: {e}"), None)
            })?;
        debug!(event = %params.event, "event sent");
        Ok(format!("event '{}' sent", params.event))
    }

    /// Get the native window handle (HWND on Windows, WID on Linux, NSView* on macOS).
    ///
    /// **TODO**: Requires AuroraView core to expose a CDP extension API:
    /// - Method: `Browser.getWindowHandle`
    /// - Params: `{ "viewId": <string> }`
    /// - Returns: `{ "handle": <string> }` (hexadecimal representation)
    ///
    /// Currently a placeholder; will be implemented when AuroraView core
    /// adds CDP extension support (target: Q3 2026).
    #[tool(description = "(PLACHOLDER) Get native window handle (HWND/WID/NSView)")]
    async fn get_hwnd(
        &self,
        Parameters(_): Parameters<GetHwndParams>,
    ) -> Result<String, rmcp::ErrorData> {
        Err(rmcp::ErrorData::internal_error(
            "get_hwnd not yet implemented: requires AuroraView core CDP extension API",
            None,
        ))
    }

    /// List all active WebView instances.
    ///
    /// **TODO**: Requires AuroraView core to expose a CDP extension API:
    /// - Method: `Browser.getWebViews`
    /// - Params: `{}`
    /// - Returns: `[ { "id": <string>, "url": <string>, "title": <string> } ]`
    ///
    /// Currently a placeholder; will be implemented when AuroraView core
    /// adds CDP extension support (target: Q3 2026).
    #[tool(description = "(PLACEHOLDER) List all active WebView instances")]
    async fn list_webviews(
        &self,
        Parameters(_): Parameters<ListWebviewsParams>,
    ) -> Result<String, rmcp::ErrorData> {
        Err(rmcp::ErrorData::internal_error(
            "list_webviews not yet implemented: requires AuroraView core API",
            None,
        ))
    }

    /// Create a new WebView instance.
    ///
    /// **TODO**: Requires AuroraView core to expose a CDP extension API:
    /// - Method: `Browser.newWebView`
    /// - Params: `{ "url": <string>, "width": <int>, "height": <int>, "title": <string> }`
    /// - Returns: `{ "id": <string>, "handle": <string> }`
    ///
    /// Currently a placeholder; will be implemented when AuroraView core
    /// adds CDP extension support (target: Q3 2026).
    #[tool(description = "(PLACEHOLDER) Create a new WebView instance")]
    async fn create_webview(
        &self,
        Parameters(_params): Parameters<CreateWebviewParams>,
    ) -> Result<String, rmcp::ErrorData> {
        Err(rmcp::ErrorData::internal_error(
            "create_webview not yet implemented: requires AuroraView core CDP extension API",
            None,
        ))
    }

    /// Close a WebView instance by ID.
    ///
    /// **TODO**: Requires AuroraView core to expose a CDP extension API:
    /// - Method: `Browser.closeWebView`
    /// - Params: `{ "id": <string> }`
    /// - Returns: `{}`
    ///
    /// Currently a placeholder; will be implemented when AuroraView core
    /// adds CDP extension support (target: Q3 2026).
    #[tool(description = "(PLACEHOLDER) Close a WebView instance by ID")]
    async fn close_webview(
        &self,
        Parameters(params): Parameters<CloseWebviewParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let _ = params; // Suppress unused variable warning
        Err(rmcp::ErrorData::internal_error(
            "close_webview not yet implemented: requires AuroraView core CDP extension API",
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_params_default_format() {
        let p: ScreenshotParams = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(p.format, "png");
    }

    #[test]
    fn screenshot_params_custom_format() {
        let p: ScreenshotParams = serde_json::from_str(r#"{"format": "jpeg"}"#).unwrap();
        assert_eq!(p.format, "jpeg");
    }

    #[test]
    fn eval_js_params() {
        let p: EvalJsParams = serde_json::from_str(r#"{"script": "document.title"}"#).unwrap();
        assert_eq!(p.script, "document.title");
    }

    #[test]
    fn load_url_params() {
        let p: LoadUrlParams = serde_json::from_str(r#"{"url": "https://example.com"}"#).unwrap();
        assert_eq!(p.url, "https://example.com");
    }

    #[test]
    fn send_event_params_default() {
        let json = r#"{"event": "test_event", "data": {"key": "value"}}"#;
        let p: SendEventParams = serde_json::from_str(json).unwrap();
        assert_eq!(p.event, "test_event");
        assert_eq!(p.data, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn get_hwnd_params_empty() {
        let p: GetHwndParams = serde_json::from_str(r#"{}"#).unwrap();
        let _ = p; // empty struct
    }

    #[test]
    fn list_webviews_params_empty() {
        let p: ListWebviewsParams = serde_json::from_str(r#"{}"#).unwrap();
        let _ = p; // empty struct
    }

    #[test]
    fn create_webview_params() {
        let json = r#"{"config": {"url": "https://example.com"}}"#;
        let p: CreateWebviewParams = serde_json::from_str(json).unwrap();
        assert_eq!(p.config, serde_json::json!({"url": "https://example.com"}));
    }

    #[test]
    fn close_webview_params() {
        let json = r#"{"id": "view-123"}"#;
        let p: CloseWebviewParams = serde_json::from_str(json).unwrap();
        assert_eq!(p.id, "view-123");
    }

    #[test]
    fn mcp_server_new_creates_instance() {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        let server = McpServer::new(config);
        assert_eq!(server.registry().len(), 0);
    }

    #[test]
    fn mcp_server_with_agui_bus() {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        let server = McpServer::new(config);
        let bus = AguiBus::new();
        let server = server.with_agui_bus(bus);
        // Should not panic
        let _ = server;
    }

    #[test]
    fn mcp_server_registry() {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        let server = McpServer::new(config);
        let registry = server.registry();
        assert_eq!(registry.len(), 0);
    }

    // ---------------------------------------------------------------------------
    // Placeholder tool behavior tests
    // ---------------------------------------------------------------------------

    /// Helper to create a test server (won't actually connect to CDP).
    fn test_server() -> McpServer {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        McpServer::new(config)
    }

    #[tokio::test]
    async fn get_hwnd_returns_not_implemented() {
        let server = test_server();
        let params = Parameters(GetHwndParams {});
        let result = server.get_hwnd(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not yet implemented"), "Expected 'not yet implemented' in error: {msg}");
    }

    #[tokio::test]
    async fn list_webviews_returns_not_implemented() {
        let server = test_server();
        let params = Parameters(ListWebviewsParams {});
        let result = server.list_webviews(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not yet implemented"), "Expected 'not yet implemented' in error: {msg}");
    }

    #[tokio::test]
    async fn create_webview_returns_not_implemented() {
        let server = test_server();
        let params = Parameters(CreateWebviewParams {
            config: serde_json::json!({}),
        });
        let result = server.create_webview(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not yet implemented"), "Expected 'not yet implemented' in error: {msg}");
    }

    #[tokio::test]
    async fn close_webview_returns_not_implemented() {
        let server = test_server();
        let params = Parameters(CloseWebviewParams {
            id: "test-id".to_owned(),
        });
        let result = server.close_webview(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not yet implemented"), "Expected 'not yet implemented' in error: {msg}");
    }
}
