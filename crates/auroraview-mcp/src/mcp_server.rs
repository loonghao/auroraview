//! MCP Server implementation for `AuroraView`.
//!
//! Exposes `AuroraView` capabilities as standard MCP tools via HTTP/SSE transport.
//! Uses `rmcp` crate with `StreamableHttpService` for HTTP-based MCP communication.

use rmcp::{handler::server::wrapper::Parameters, schemars::JsonSchema, tool, tool_router};
use serde::Deserialize;
use serde_json::Value;
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

/// Parameters for the `set_attribute` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetAttributeParams {
    /// CSS selector to find the target element.
    pub selector: String,
    /// Attribute name to set.
    pub name: String,
    /// Attribute value to set.
    pub value: String,
}

/// Parameters for the `remove_attribute` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveAttributeParams {
    /// CSS selector to find the target element.
    pub selector: String,
    /// Attribute name to remove.
    pub name: String,
}

/// Parameters for the `call_function` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CallFunctionParams {
    /// JavaScript expression that evaluates to an object.
    pub object_expr: String,
    /// Function declaration to call on the object.
    pub function: String,
}

/// Parameters for the `clear_cache` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClearCacheParams {}

/// Parameters for the `set_cache_disabled` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCacheDisabledParams {
    /// If `true`, disable browser cache; if `false`, enable cache.
    pub disabled: bool,
}

/// Parameters for the `set_download_behavior` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDownloadBehaviorParams {
    /// Download behavior: `"deny"`, `"allow"`, or `"default"`.
    pub behavior: String,
    /// Required when `behavior` is `"allow"`.
    pub download_path: Option<String>,
}

/// Parameters for the `set_device_metrics_override` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDeviceMetricsOverrideParams {
    /// Override width (pixels). Set to `0` to clear override.
    pub width: i64,
    /// Override height (pixels). Set to `0` to clear override.
    pub height: i64,
    /// Device pixel ratio (e.g., `1.0`, `2.0` for Retina).
    pub device_scale_factor: f64,
    /// Whether the emulated device is mobile.
    pub mobile: bool,
}

/// Parameters for the `set_ignore_certificate_errors` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetIgnoreCertificateErrorsParams {
    /// If `true`, ignore all SSL certificate errors (DEV ONLY).
    pub ignore: bool,
}

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

impl Default for McpServer {
    fn default() -> Self {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        Self {
            config,
            registry: WebViewRegistry::new(),
            agui_bus: None,
            client: Arc::new(OnceCell::new()),
        }
    }
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

    /// Update the CDP endpoint URL for the server.
    ///
    /// This allows dynamically changing the CDP endpoint (e.g., when a new
    /// WebView is created or the CDP port changes).
    /// The next `get_client()` call will establish a new connection.
    #[must_use]
    pub fn with_cdp_endpoint(mut self, endpoint: String) -> Self {
        self.config.http_endpoint = endpoint;
        // Reset the cached client so next call reconnects
        self.client = Arc::new(OnceCell::new());
        self
    }

    /// Get or create a shared CDP client (lazily initialized on first use).
    ///
    /// This method is called internally by tool handlers. It establishes a CDP
    /// connection on first call, then reuses the same connection for subsequent calls.
    ///
    /// # Errors
    ///
    /// Returns `CdpError` with detailed context if:
    /// - The CDP endpoint is not reachable (check if AuroraView is running)
    /// - The WebSocket connection fails (check firewall/permissions)
    /// - The CDP endpoint returns invalid responses (possible version mismatch)
    async fn get_client(&self) -> Result<CdpClient, CdpError> {
        let start = std::time::Instant::now();
        let endpoint = &self.config.http_endpoint;
        let client_ref = self
            .client
            .get_or_try_init(|| async { CdpClient::connect(endpoint).await })
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    %endpoint,
                    "CDP client initialization failed"
                );
                warn!(
                    %endpoint,
                    "Troubleshooting: \
                     1) Is AuroraView running with CDP enabled? \
                     2) Is the port correct? \
                     3) Check firewall allows connections to {}",
                    endpoint
                );
                e
            })?;
        debug!(
            elapsed = ?start.elapsed(),
            %endpoint,
            "get_client() completed"
        );
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

    /// Set an attribute on a DOM element.
    #[tool(description = "Set an attribute on a DOM element matching the CSS selector")]
    async fn set_attribute(
        &self,
        Parameters(params): Parameters<SetAttributeParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;

        // First, get the document root
        let doc = client.get_document(DEFAULT_CDP_TIMEOUT).await.map_err(|e| {
            warn!(error = %e, "get_document failed");
            rmcp::ErrorData::internal_error(format!("get_document failed: {e}"), None)
        })?;
        let root_id = doc
            .get("root")
            .and_then(|r| r.get("nodeId"))
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                rmcp::ErrorData::internal_error("Failed to parse document root nodeId", None)
            })?;

        // Find the target element
        let node_id = client
            .query_selector(root_id, &params.selector, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, selector = %params.selector, "query_selector failed");
                rmcp::ErrorData::internal_error(format!("query_selector failed: {e}"), None)
            })?;

        let node_id = node_id.ok_or_else(|| {
            rmcp::ErrorData::invalid_params(
                format!("No element found matching '{}'", params.selector),
                None,
            )
        })?;

        // Set the attribute
        client
            .set_attribute_value(node_id, &params.name, &params.value, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "set_attribute_value failed");
                rmcp::ErrorData::internal_error(format!("set_attribute_value failed: {e}"), None)
            })?;

        debug!(selector = %params.selector, name = %params.name, "attribute set");
        Ok(format!("Attribute '{}' set on '{}'", params.name, params.selector))
    }

    /// Remove an attribute from a DOM element.
    #[tool(description = "Remove an attribute from a DOM element matching the CSS selector")]
    async fn remove_attribute(
        &self,
        Parameters(params): Parameters<RemoveAttributeParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;

        // First, get the document root
        let doc = client.get_document(DEFAULT_CDP_TIMEOUT).await.map_err(|e| {
            warn!(error = %e, "get_document failed");
            rmcp::ErrorData::internal_error(format!("get_document failed: {e}"), None)
        })?;
        let root_id = doc
            .get("root")
            .and_then(|r| r.get("nodeId"))
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                rmcp::ErrorData::internal_error("Failed to parse document root nodeId", None)
            })?;

        // Find the target element
        let node_id = client
            .query_selector(root_id, &params.selector, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, selector = %params.selector, "query_selector failed");
                rmcp::ErrorData::internal_error(format!("query_selector failed: {e}"), None)
            })?;

        let node_id = node_id.ok_or_else(|| {
            rmcp::ErrorData::invalid_params(
                format!("No element found matching '{}'", params.selector),
                None,
            )
        })?;

        // Remove the attribute
        client
            .remove_attribute(node_id, &params.name, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "remove_attribute failed");
                rmcp::ErrorData::internal_error(format!("remove_attribute failed: {e}"), None)
            })?;

        debug!(selector = %params.selector, name = %params.name, "attribute removed");
        Ok(format!("Attribute '{}' removed from '{}'", params.name, params.selector))
    }

    /// Call a JavaScript function on an object.
    #[tool(description = "Call a JavaScript function on the object returned by `object_expr`")]
    async fn call_function(
        &self,
        Parameters(params): Parameters<CallFunctionParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;

        // Evaluate the object expression to get the object ID
        let obj_result = client
            .evaluate_script(&params.object_expr, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, expr = %params.object_expr, "evaluate_script failed");
                rmcp::ErrorData::internal_error(format!("evaluate_script failed: {e}"), None)
            })?;

        let object_id = obj_result
            .get("objectId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                rmcp::ErrorData::invalid_params(
                    format!("Expression did not return an object: '{}'", params.object_expr),
                    None,
                )
            })?;

        // Call the function on the object
        let result = client
            .call_function_on(object_id, &params.function, None, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "call_function_on failed");
                rmcp::ErrorData::internal_error(format!("call_function_on failed: {e}"), None)
            })?;

        debug!(expr = %params.object_expr, func = %params.function, "function called");
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "null".to_owned()))
    }

    /// Clear the browser cache.
    #[tool(description = "Clear the browser cache (network requests)")]
    async fn clear_cache(
        &self,
        Parameters(_): Parameters<ClearCacheParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;

        // Enable network first (required before clearing cache)
        let _ = client.network_enable(DEFAULT_CDP_TIMEOUT).await;

        client
            .clear_browser_cache(DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "clear_browser_cache failed");
                rmcp::ErrorData::internal_error(format!("clear_browser_cache failed: {e}"), None)
            })?;

        // Optionally disable network to clean up
        let _ = client.network_disable(DEFAULT_CDP_TIMEOUT).await;

        info!("Browser cache cleared");
        Ok("Browser cache cleared".to_owned())
    }

    /// Disable or enable browser cache.
    #[tool(description = "Disable or enable browser cache (true = disable, false = enable)")]
    async fn set_cache_disabled(
        &self,
        Parameters(params): Parameters<SetCacheDisabledParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .set_cache_disabled(params.disabled, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "set_cache_disabled failed");
                rmcp::ErrorData::internal_error(format!("set_cache_disabled failed: {e}"), None)
            })?;
        let msg = if params.disabled { "Browser cache disabled" } else { "Browser cache enabled" };
        info!(%params.disabled, "Cache disabled/enabled");
        Ok(msg.to_owned())
    }

    /// Control download behavior.
    #[tool(description = "Control download behavior: 'deny', 'allow', or 'default'")]
    async fn set_download_behavior(
        &self,
        Parameters(params): Parameters<SetDownloadBehaviorParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        let download_path = params.download_path.as_deref();
        client
            .set_download_behavior(&params.behavior, download_path, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, behavior = %params.behavior, "set_download_behavior failed");
                rmcp::ErrorData::internal_error(format!("set_download_behavior failed: {e}"), None)
            })?;
        info!(behavior = %params.behavior, ?params.download_path, "Download behavior set");
        Ok(format!("Download behavior set to '{}'", params.behavior))
    }

    /// Override device metrics (screen size, pixel ratio, mobile emulation).
    #[tool(description = "Override device metrics for responsive testing (set all to 0 to clear)")]
    async fn set_device_metrics_override(
        &self,
        Parameters(params): Parameters<SetDeviceMetricsOverrideParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .set_device_metrics_override(
                params.width,
                params.height,
                params.device_scale_factor,
                params.mobile,
                DEFAULT_CDP_TIMEOUT,
            )
            .await
            .map_err(|e| {
                warn!(error = %e, "set_device_metrics_override failed");
                rmcp::ErrorData::internal_error(format!("set_device_metrics_override failed: {e}"), None)
            })?;
        info!(width = params.width, height = params.height, "Device metrics overridden");
        Ok(format!("Device metrics overridden: {}x{} @ {}x", params.width, params.height, params.device_scale_factor))
    }

    /// Ignore SSL certificate errors (DEV ONLY).
    #[tool(description = "(DEV ONLY) Ignore SSL certificate errors - USE WITH CAUTION")]
    async fn set_ignore_certificate_errors(
        &self,
        Parameters(params): Parameters<SetIgnoreCertificateErrorsParams>,
    ) -> Result<String, rmcp::ErrorData> {
        let client = self.get_client().await.map_err(|e| {
            error!(error = %e, "CDP connect failed");
            rmcp::ErrorData::internal_error(format!("CDP connect failed: {e}"), None)
        })?;
        client
            .set_ignore_certificate_errors(params.ignore, DEFAULT_CDP_TIMEOUT)
            .await
            .map_err(|e| {
                warn!(error = %e, "set_ignore_certificate_errors failed");
                rmcp::ErrorData::internal_error(format!("set_ignore_certificate_errors failed: {e}"), None)
            })?;
        let msg = if params.ignore { "SSL certificate errors will be ignored (DEV ONLY)" } else { "SSL certificate errors will be enforced" };
        warn!(%params.ignore, "SSL certificate error handling changed");
        Ok(msg.to_owned())
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

    // ---------------------------------------------------------------------------
    // Tests for new MCP tools (set_attribute, remove_attribute, call_function, clear_cache)
    // ---------------------------------------------------------------------------

    #[test]
    fn set_attribute_params() {
        let json = serde_json::json!({
            "selector": "#my-id",
            "name": "data-custom",
            "value": "test-value"
        });
        let p: SetAttributeParams = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(p.selector, "#my-id");
        assert_eq!(p.name, "data-custom");
        assert_eq!(p.value, "test-value");
    }

    #[test]
    fn remove_attribute_params() {
        let json = serde_json::json!({
            "selector": ".my-class",
            "name": "disabled"
        });
        let p: RemoveAttributeParams = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(p.selector, ".my-class");
        assert_eq!(p.name, "disabled");
    }

    #[test]
    fn call_function_params() {
        let json = serde_json::json!({
            "object_expr": "document.body",
            "function": "function() { return this.tagName; }"
        });
        let p: CallFunctionParams = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(p.object_expr, "document.body");
        assert_eq!(p.function, "function() { return this.tagName; }");
    }

    #[test]
    fn clear_cache_params_empty() {
        let p: ClearCacheParams = serde_json::from_str(r#"{}"#).unwrap();
        let _ = p; // empty struct
    }

    #[test]
    fn set_cache_disabled_params() {
        let p: SetCacheDisabledParams = serde_json::from_str(r#"{"disabled": true}"#).unwrap();
        assert!(p.disabled);
        let p: SetCacheDisabledParams = serde_json::from_str(r#"{"disabled": false}"#).unwrap();
        assert!(!p.disabled);
    }

    #[test]
    fn set_download_behavior_params() {
        let p: SetDownloadBehaviorParams =
            serde_json::from_str(r#"{"behavior": "allow"}"#).unwrap();
        assert_eq!(p.behavior, "allow");
        assert!(p.download_path.is_none());

        let p: SetDownloadBehaviorParams =
            serde_json::from_str(r#"{"behavior": "allow", "download_path": "/tmp"}"#).unwrap();
        assert_eq!(p.behavior, "allow");
        assert_eq!(p.download_path, Some("/tmp".to_owned()));
    }

    #[test]
    fn set_device_metrics_override_params() {
        let p: SetDeviceMetricsOverrideParams =
            serde_json::from_str(r#"{"width": 1920, "height": 1080, "device_scale_factor": 1.0, "mobile": false}"#).unwrap();
        assert_eq!(p.width, 1920);
        assert_eq!(p.height, 1080);
        assert_eq!(p.device_scale_factor, 1.0);
        assert!(!p.mobile);
    }

    #[test]
    fn set_ignore_certificate_errors_params() {
        let p: SetIgnoreCertificateErrorsParams =
            serde_json::from_str(r#"{"ignore": true}"#).unwrap();
        assert!(p.ignore);
        let p: SetIgnoreCertificateErrorsParams =
            serde_json::from_str(r#"{"ignore": false}"#).unwrap();
        assert!(!p.ignore);
    }

    #[test]
    fn mcp_server_with_cdp_endpoint() {
        let config = CdpAdapterConfig::localhost(9222, "0.5.2");
        let server = McpServer::new(config);
        let server = server.with_cdp_endpoint("http://127.0.0.1:9223".to_owned());
        // Should not panic - the endpoint was updated
        let _ = server;
    }
}
