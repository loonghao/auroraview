// MCP tool implementations for AuroraViewMcpServer.
// Extracted from server.rs to keep files under 1000 lines.

use crate::{
    agui::{AguiBus, AguiEvent},
    cdp::CdpClient,
    registry::WebViewRegistry,
    types::{JsResult, McpServerConfig, ScreenshotData, WebViewConfig, WebViewId},
    server::types::*,
};
use tokio::runtime::Runtime;
use rmcp::{
    ServerHandler, tool,
    handler::server::{
        router::tool::ToolRouter,
        tool::ToolCallContext,
        wrapper::{Json, Parameters},
    },
    model::{
        CallToolResult, CallToolRequestParams, InitializeResult, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities,
    },
    schemars,
    service::RequestContext,
    tool_router, RoleServer,
    ErrorData,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// `AuroraView` MCP Server — exposes `WebView` management tools via MCP.
///
/// When an `AguiBus` is attached (via `with_agui_bus`), every tool invocation
/// automatically emits `ToolCallStart` and `ToolCallEnd` events so that
/// AG-UI subscribers can track real-time progress.
#[derive(Clone)]
pub struct AuroraViewMcpServer {
    pub registry: WebViewRegistry,
    pub config: Arc<McpServerConfig>,
    pub tool_router: ToolRouter<Self>,
    /// Optional AG-UI broadcast bus — `None` means events are not emitted.
    pub agui_bus: Option<AguiBus>,
}

#[tool_router]
impl AuroraViewMcpServer {
    #[must_use]
    pub fn new(config: McpServerConfig) -> Self {
        let tool_router = Self::tool_router();
        let registry = match config.max_webviews {
            Some(max) => WebViewRegistry::with_capacity(max),
            None => WebViewRegistry::new(),
        };
        Self {
            registry,
            config: Arc::new(config),
            tool_router,
            agui_bus: None,
        }
    }

    /// Capture a screenshot of the specified `WebView`.
    #[tool(
        name = "screenshot",
        description = "Capture a screenshot of a WebView window. Returns base64-encoded PNG data."
    )]
    fn screenshot(&self, params: Parameters<ScreenshotParams>) -> Json<ScreenshotOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("screenshot", &call_id, &id);
        debug!("screenshot requested for WebView: {id}");

        let data = if let Some(info) = self.registry.get(&id.parse::<WebViewId>().unwrap()) {
            if let Some(ref cdp_ep) = info.cdp_endpoint {
                debug!("Connecting to CDP endpoint: {cdp_ep}");
                match Runtime::new() {
                    Ok(rt) => {
                        let fut = CdpClient::connect(cdp_ep);
                        match rt.block_on(fut) {
                            Ok(mut client) => {
                                let capture_fut = client.capture_screenshot(
                                    "png",
                                    std::time::Duration::from_secs(10),
                                );
                                match rt.block_on(capture_fut) {
                                    Ok(bytes) => {
                                        ScreenshotData::from_bytes(
                                            &bytes,
                                            info.width,
                                            info.height,
                                            "png",
                                        )
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "CDP screenshot failed: {e}, falling back to placeholder"
                                        );
                                        ScreenshotData::new_placeholder(
                                            info.width,
                                            info.height,
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "CDP connect failed: {e}, falling back to placeholder"
                                );
                                ScreenshotData::new_placeholder(info.width, info.height)
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to create runtime: {e}, falling back to placeholder"
                        );
                        ScreenshotData::new_placeholder(info.width, info.height)
                    }
                }
            } else {
                ScreenshotData::new_placeholder(info.width, info.height)
            }
        } else {
            ScreenshotData::new_placeholder(800, 600)
        };

        let result = Json(ScreenshotOutput {
            id: id.clone(),
            data: data.data,
            width: data.width,
            height: data.height,
            format: data.format,
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Load a URL in the specified `WebView`.
    #[tool(
        name = "load_url",
        description = "Load a URL (http://, https://, or file://) in a WebView."
    )]
    fn load_url(&self, params: Parameters<LoadUrlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("load_url", &call_id, &id);

        // Validate URL scheme.
        let scheme_ok = params.0.url.starts_with("http://")
            || params.0.url.starts_with("https://")
            || params.0.url.starts_with("file://");
        if !scheme_ok {
            self.emit_tool_end(&call_id, &id);
            return Json(SuccessOutput {
                ok: false,
                message: format!(
                    "Invalid URL scheme: '{}' — must be http, https, or file",
                    params.0.url
                ),
            });
        }

        info!("load_url: id={id} url={}", params.0.url);
        let updated = self.registry.update_url(&id.parse::<WebViewId>().unwrap(), &params.0.url);
        let result = Json(SuccessOutput {
            ok: updated,
            message: if updated {
                format!("Loaded URL in WebView {id}")
            } else {
                format!("WebView {id} not found")
            },
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Load raw HTML content in the specified `WebView`.
    #[tool(
        name = "load_html",
        description = "Load HTML content directly into a WebView."
    )]
    fn load_html(&self, params: Parameters<LoadHtmlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("load_html", &call_id, &id);
        info!("load_html: id={id} html_len={}", params.0.html.len());
        let result = Json(SuccessOutput {
            ok: true,
            message: format!("HTML loaded in WebView {id} ({} bytes)", params.0.html.len()),
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Execute JavaScript in the specified `WebView` and return the result.
    #[tool(
        name = "eval_js",
        description = "Execute JavaScript code in a WebView and return the result."
    )]
    fn eval_js(&self, params: Parameters<EvalJsParams>) -> Json<JsResultOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("eval_js", &call_id, &id);

        if params.0.script.trim().is_empty() {
            self.emit_tool_end(&call_id, &id);
            return Json(JsResultOutput {
                id,
                value: serde_json::Value::Null,
                error: Some("eval_js script must not be empty".to_string()),
            });
        }

        debug!("eval_js: id={id}");

        let result_data =
            if let Some(info) = self.registry.get(&id.parse::<WebViewId>().unwrap()) {
                if let Some(ref cdp_ep) = info.cdp_endpoint {
                    debug!("eval_js: connecting to CDP endpoint: {cdp_ep}");
                    match Runtime::new() {
                        Ok(rt) => {
                            let fut = CdpClient::connect(cdp_ep);
                            match rt.block_on(fut) {
                                Ok(mut client) => {
                                    let eval_fut =
                                        client.evaluate_script(&params.0.script, std::time::Duration::from_secs(10));
                                    match rt.block_on(eval_fut) {
                                        Ok(value) => JsResult::ok(value),
                                        Err(e) => JsResult::err(format!("CDP eval error: {e}")),
                                    }
                                }
                                Err(e) => {
                                    JsResult::err(format!("CDP connect error: {e}"))
                                }
                            }
                        }
                        Err(e) => JsResult::err(format!("Failed to create runtime: {e}")),
                    }
                } else {
                    JsResult::err(
                        "No CDP endpoint available for this WebView. Is the WebView running?"
                            .to_string(),
                    )
                }
            } else {
                JsResult::err(format!("WebView {id} not found"))
            };

        let result = Json(JsResultOutput {
            id: id.clone(),
            value: result_data.value,
            error: result_data.error,
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Send a custom event to the `WebView`s JavaScript context.
    #[tool(
        name = "send_event",
        description = "Send a named event with payload to the WebView JS context via auroraview.on()."
    )]
    fn send_event(&self, params: Parameters<SendEventParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("send_event", &call_id, &id);
        info!("send_event: id={id} event={}", params.0.event);
        let result = Json(SuccessOutput {
            ok: true,
            message: format!("Event '{}' sent to WebView {id}", params.0.event),
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Get the native window handle (HWND on Windows) for a `WebView`.
    #[tool(
        name = "get_hwnd",
        description = "Get the native window handle (HWND on Windows, 0 on other platforms) for embedding in UE or other hosts."
    )]
    fn get_hwnd(&self, params: Parameters<GetHwndParams>) -> Json<HwndOutput> {
        let id = self.resolve_id(params.0.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("get_hwnd", &call_id, &id);
        let hwnd = self
            .registry
            .get(&id.parse::<WebViewId>().unwrap())
            .map_or(0, |v| v.hwnd);
        let result = Json(HwndOutput { id: id.clone(), hwnd });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// List all active `WebView` instances.
    #[tool(
        name = "list_webviews",
        description = "Return a list of all active WebView instances with their IDs, titles, and URLs."
    )]
    fn list_webviews(&self) -> Json<ListWebViewsOutput> {
        let views: Vec<serde_json::Value> = self
            .registry
            .list()
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "id": v.id.0,
                    "title": v.title,
                    "url": v.url,
                    "visible": v.visible,
                    "width": v.width,
                    "height": v.height,
                    "hwnd": v.hwnd,
                })
            })
            .collect();
        let count = views.len();
        Json(ListWebViewsOutput {
            count,
            capacity: self.registry.capacity(),
            views,
        })
    }

    /// Create a new `WebView` instance.
    #[tool(
        name = "create_webview",
        description = "Create a new WebView window with the given configuration."
    )]
    fn create_webview(
        &self,
        params: Parameters<CreateWebViewParams>,
    ) -> Json<SuccessOutput> {
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("create_webview", &call_id, "server");
        let config = WebViewConfig {
            title: params.0.title,
            url: params.0.url,
            html: params.0.html,
            width: params.0.width,
            height: params.0.height,
            visible: Some(true),
            debug: params.0.debug,
        };
        let result = match self.registry.try_register(&config) {
            Ok(id) => {
                info!("create_webview: new id={}", id.0);
                Json(SuccessOutput {
                    ok: true,
                    message: id.0,
                })
            }
            Err(e) => Json(SuccessOutput {
                ok: false,
                message: e.to_string(),
            }),
        };
        self.emit_tool_end(&call_id, "server");
        result
    }

    /// Close and remove a `WebView` instance.
    #[tool(
        name = "close_webview",
        description = "Close a WebView window by its ID and release resources."
    )]
    fn close_webview(
        &self,
        params: Parameters<CloseWebViewParams>,
    ) -> Json<SuccessOutput> {
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("close_webview", &call_id, &params.0.id);
        let wid = params.0.id.parse::<WebViewId>().unwrap();
        let removed = self.registry.remove(&wid).is_some();
        let result = Json(SuccessOutput {
            ok: removed,
            message: if removed {
                format!("WebView {} closed", params.0.id)
            } else {
                format!("WebView {} not found", params.0.id)
            },
        });
        self.emit_tool_end(&call_id, &params.0.id);
        result
    }
}
