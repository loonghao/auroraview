//! MCP Server implementation for AuroraView.
//!
//! Exposes AuroraView capabilities as standard MCP tools via HTTP/SSE transport.
//! Uses `rmcp` crate with `StreamableHttpService` for HTTP-based MCP communication.

use rmcp::{handler::server::wrapper::Parameters, schemars::JsonSchema, tool, tool_router};
use serde::Deserialize;
use tracing::info;

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
/// Creates a new CDP connection for each tool call (MVP approach).
#[derive(Clone)]
pub struct McpServer {
    config: CdpAdapterConfig,
    registry: WebViewRegistry,
    agui_bus: Option<AguiBus>,
}

impl McpServer {
    /// Create a new MCP server that will connect to the given CDP endpoint.
    pub fn new(config: CdpAdapterConfig) -> Self {
        Self {
            config,
            registry: WebViewRegistry::new(),
            agui_bus: None,
        }
    }

    /// Set the AG-UI event bus.
    #[must_use]
    pub fn with_agui_bus(mut self, bus: AguiBus) -> Self {
        self.agui_bus = Some(bus);
        self
    }

    /// Create a CDP client for a tool call.
    async fn create_client(&self) -> Result<CdpClient, CdpError> {
        CdpClient::connect(&self.config.http_endpoint).await
    }

    /// Return a reference to the WebView registry.
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
        let client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let bytes = client
            .capture_screenshot(&params.format, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("screenshot failed: {e}"), None)
            })?;
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
        let client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let value = client
            .evaluate_script(&params.script, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("eval_js failed: {e}"), None))?;
        Ok(serde_json::to_string(&value).unwrap_or_else(|_| "null".to_owned()))
    }

    /// Navigate the WebView to a URL.
    #[tool(description = "Navigate the WebView to a URL")]
    async fn load_url(
        &self,
        Parameters(params): Parameters<LoadUrlParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .navigate_to(&params.url, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(format!("load_url failed: {e}"), None))?;
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
        let client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let data_str = serde_json::to_string(&params.data).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("JSON serialize failed: {e}"), None)
        })?;
        let script = format!("if(window.auroraview && window.auroraview.trigger){{ window.auroraview.trigger('{}', {}); }} else {{ console.error('[AuroraView] Event bridge not ready'); }}", params.event.replace('\'', "\\'"), data_str);
        client
            .evaluate_script(&script, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("send_event failed: {e}"), None)
            })?;
        Ok(format!("event '{}' sent", params.event))
    }

    /// Get the native window handle (HWND on Windows) of the WebView.
    ///
    /// **TODO**: Requires AuroraView core to expose a CDP extension API
    /// (e.g., `AuroraView.getHwnd()`). Currently a placeholder.
    #[tool(description = "Get the native window handle of the WebView (TODO: not yet implemented)")]
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
    /// **TODO**: Requires AuroraView core to expose an API to list WebViews.
    /// Currently a placeholder.
    #[tool(description = "List all WebView instances (TODO: not yet implemented)")]
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
    /// **TODO**: Requires AuroraView core to expose a CDP extension API
    /// for creating new WebViews. Currently a placeholder.
    #[tool(description = "Create a new WebView instance (TODO: not yet implemented)")]
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
    /// **TODO**: Requires AuroraView core to expose a CDP extension API
    /// for closing WebViews. Currently a placeholder.
    #[tool(description = "Close a WebView instance by ID (TODO: not yet implemented)")]
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

// ---------------------------------------------------------------------------
// Server start helper
// ---------------------------------------------------------------------------

/// Start the MCP server with HTTP transport.
///
/// # Arguments
/// - `config`: CDP adapter configuration.
/// - `bind_addr`: Socket address to listen on (e.g. "0.0.0.0:7890").
pub async fn start_mcp_server(
    config: CdpAdapterConfig,
    bind_addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _server = McpServer::new(config);

    info!(addr = %bind_addr, "starting AuroraView MCP server");

    // TODO: Wire MCP service to HTTP listener using axum/tower-http
    // The `server.serve(...)` needs a transport that implements
    // `IntoTransport<RoleServer, ...>`.
    // Example with axum:
    // let service = server.serve(tower::service_fn(|req| ...));
    // let app = axum::Router::new().route("/mcp", axum::routing::post(...));
    // let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    // axum::serve(listener, app).await?;

    // For now, just log that the server is ready
    info!("AuroraView MCP server started (HTTP transport TODO)");

    Ok(())
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
}
