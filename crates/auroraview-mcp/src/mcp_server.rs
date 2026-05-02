//! MCP Server implementation for AuroraView.
//!
//! Exposes AuroraView capabilities as standard MCP tools via HTTP/SSE transport.
//! Uses `rmcp` crate with `StreamableHttpService` for HTTP-based MCP communication.

use rmcp::{
    handler::server::wrapper::Parameters,
    tool, tool_router,
    schemars::JsonSchema,
};
use serde::Deserialize;
use tracing::info;

use crate::cdp::{CdpClient, CdpError};
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

// ---------------------------------------------------------------------------
// McpServer — rmcp ServerHandler implementation
// ---------------------------------------------------------------------------

/// MCP Server that bridges rmcp protocol to a running AuroraView CDP endpoint.
///
/// Creates a new CDP connection for each tool call (MVP approach).
pub struct McpServer {
    config: CdpAdapterConfig,
}

impl McpServer {
    /// Create a new MCP server that will connect to the given CDP endpoint.
    pub fn new(config: CdpAdapterConfig) -> Self {
        Self { config }
    }

    /// Create a CDP client for a tool call.
    async fn create_client(&self) -> Result<CdpClient, CdpError> {
        CdpClient::connect(&self.config.http_endpoint).await
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
        let mut client = self.create_client().await.map_err(|e| {
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
        let b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &bytes,
        );
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
        let mut client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let value = client
            .evaluate_script(&params.script, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("eval_js failed: {e}"), None)
            })?;
        Ok(serde_json::to_string(&value).unwrap_or_else(|_| "null".to_owned()))
    }

    /// Navigate the WebView to a URL.
    #[tool(description = "Navigate the WebView to a URL")]
    async fn load_url(
        &self,
        Parameters(params): Parameters<LoadUrlParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let mut client = self.create_client().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .navigate_to(&params.url, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("load_url failed: {e}"), None)
            })?;
        Ok(format!("navigated to {}", params.url))
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
        let p: ScreenshotParams =
            serde_json::from_str(r#"{"format": "jpeg"}"#).unwrap();
        assert_eq!(p.format, "jpeg");
    }

    #[test]
    fn eval_js_params() {
        let p: EvalJsParams =
            serde_json::from_str(r#"{"script": "document.title"}"#).unwrap();
        assert_eq!(p.script, "document.title");
    }

    #[test]
    fn load_url_params() {
        let p: LoadUrlParams =
            serde_json::from_str(r#"{"url": "https://example.com"}"#).unwrap();
        assert_eq!(p.url, "https://example.com");
    }
}
