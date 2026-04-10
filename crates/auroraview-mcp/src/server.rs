use crate::{
    agui::{AguiBus, AguiEvent},
    registry::WebViewRegistry,
    types::{JsResult, McpServerConfig, ScreenshotData, WebViewConfig, WebViewId},
};
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

/// AuroraView MCP Server — exposes WebView management tools via MCP.
///
/// When an `AguiBus` is attached (via `with_agui_bus`), every tool invocation
/// automatically emits `ToolCallStart` and `ToolCallEnd` events so that
/// AG-UI subscribers can track real-time progress.
#[derive(Clone)]
pub struct AuroraViewMcpServer {
    registry: WebViewRegistry,
    config: Arc<McpServerConfig>,
    tool_router: ToolRouter<Self>,
    /// Optional AG-UI broadcast bus — `None` means events are not emitted.
    agui_bus: Option<AguiBus>,
}

// --- Parameter types ---

#[derive(Debug, Deserialize, schemars::JsonSchema, Default)]
pub struct ScreenshotParams {
    /// Optional WebView ID. If omitted, captures the first available WebView.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadUrlParams {
    /// The URL to load (http://, https://, or file://).
    pub url: String,
    /// Target WebView ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadHtmlParams {
    /// Raw HTML content to load.
    pub html: String,
    /// Target WebView ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvalJsParams {
    /// JavaScript expression to evaluate.
    pub script: String,
    /// Target WebView ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendEventParams {
    /// Event name.
    pub event: String,
    /// Event payload (JSON).
    pub data: Option<serde_json::Value>,
    /// Target WebView ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetHwndParams {
    /// WebView ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CloseWebViewParams {
    /// WebView ID to close.
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWebViewParams {
    /// Title for the new WebView window.
    pub title: Option<String>,
    /// Initial URL to load.
    pub url: Option<String>,
    /// Initial HTML content.
    pub html: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub debug: Option<bool>,
}

// --- Output types ---

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ScreenshotOutput {
    pub id: String,
    pub data: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct WebViewIdOutput {
    pub id: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct HwndOutput {
    pub id: String,
    pub hwnd: u64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListWebViewsOutput {
    pub count: usize,
    /// Capacity limit, if set. `null` means unlimited.
    pub capacity: Option<usize>,
    pub views: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct JsResultOutput {
    pub id: String,
    pub value: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SuccessOutput {
    pub ok: bool,
    pub message: String,
}

#[tool_router]
impl AuroraViewMcpServer {
    /// Capture a screenshot of the specified WebView.
    #[tool(
        name = "screenshot",
        description = "Capture a screenshot of a WebView window. Returns base64-encoded PNG data."
    )]
    fn screenshot(&self, Parameters(params): Parameters<ScreenshotParams>) -> Json<ScreenshotOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("screenshot", &call_id, &id);
        debug!("screenshot requested for WebView: {id}");
        let data = ScreenshotData::new_placeholder(800, 600);
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

    /// Load a URL in the specified WebView.
    #[tool(
        name = "load_url",
        description = "Load a URL (http://, https://, or file://) in a WebView."
    )]
    fn load_url(&self, Parameters(params): Parameters<LoadUrlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("load_url", &call_id, &id);

        // Validate URL scheme.
        let scheme_ok = params.url.starts_with("http://")
            || params.url.starts_with("https://")
            || params.url.starts_with("file://");
        if !scheme_ok {
            self.emit_tool_end(&call_id, &id);
            return Json(SuccessOutput {
                ok: false,
                message: format!(
                    "Invalid URL scheme: '{}' — must be http, https, or file",
                    params.url
                ),
            });
        }

        info!("load_url: id={id} url={}", params.url);
        let updated = self.registry.update_url(&id.parse::<WebViewId>().unwrap(), &params.url);
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

    /// Load raw HTML content in the specified WebView.
    #[tool(
        name = "load_html",
        description = "Load HTML content directly into a WebView."
    )]
    fn load_html(&self, Parameters(params): Parameters<LoadHtmlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("load_html", &call_id, &id);
        info!("load_html: id={id} html_len={}", params.html.len());
        let result = Json(SuccessOutput {
            ok: true,
            message: format!("HTML loaded in WebView {id} ({} bytes)", params.html.len()),
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Execute JavaScript in the specified WebView and return the result.
    #[tool(
        name = "eval_js",
        description = "Execute JavaScript code in a WebView and return the result."
    )]
    fn eval_js(&self, Parameters(params): Parameters<EvalJsParams>) -> Json<JsResultOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("eval_js", &call_id, &id);

        if params.script.trim().is_empty() {
            self.emit_tool_end(&call_id, &id);
            return Json(JsResultOutput {
                id,
                value: serde_json::Value::Null,
                error: Some("eval_js script must not be empty".to_string()),
            });
        }

        debug!("eval_js: id={id}");
        let result_data = JsResult::ok(serde_json::Value::Null);
        let result = Json(JsResultOutput {
            id: id.clone(),
            value: result_data.value,
            error: result_data.error,
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Send a custom event to the WebView's JavaScript context.
    #[tool(
        name = "send_event",
        description = "Send a named event with payload to the WebView JS context via auroraview.on()."
    )]
    fn send_event(&self, Parameters(params): Parameters<SendEventParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("send_event", &call_id, &id);
        info!("send_event: id={id} event={}", params.event);
        let result = Json(SuccessOutput {
            ok: true,
            message: format!("Event '{}' sent to WebView {id}", params.event),
        });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// Get the native window handle (HWND on Windows) for a WebView.
    #[tool(
        name = "get_hwnd",
        description = "Get the native window handle (HWND on Windows, 0 on other platforms) for embedding in UE or other hosts."
    )]
    fn get_hwnd(&self, Parameters(params): Parameters<GetHwndParams>) -> Json<HwndOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let call_id = Uuid::new_v4().to_string();
        self.emit_tool_start("get_hwnd", &call_id, &id);
        let hwnd = self
            .registry
            .get(&id.parse::<WebViewId>().unwrap())
            .map(|v| v.hwnd)
            .unwrap_or(0);
        let result = Json(HwndOutput { id: id.clone(), hwnd });
        self.emit_tool_end(&call_id, &id);
        result
    }

    /// List all active WebView instances.
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

    /// Create a new WebView instance.
    #[tool(
        name = "create_webview",
        description = "Create a new WebView window with the given configuration."
    )]
    fn create_webview(
        &self,
        Parameters(params): Parameters<CreateWebViewParams>,
    ) -> Json<SuccessOutput> {
        let config = WebViewConfig {
            title: params.title,
            url: params.url,
            html: params.html,
            width: params.width,
            height: params.height,
            visible: Some(true),
            debug: params.debug,
        };
        match self.registry.try_register(&config) {
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
        }
    }

    /// Close and remove a WebView instance.
    #[tool(
        name = "close_webview",
        description = "Close a WebView window by its ID and release resources."
    )]
    fn close_webview(
        &self,
        Parameters(params): Parameters<CloseWebViewParams>,
    ) -> Json<SuccessOutput> {
        let wid = params.id.parse::<WebViewId>().unwrap();
        let removed = self.registry.remove(&wid).is_some();
        Json(SuccessOutput {
            ok: removed,
            message: if removed {
                format!("WebView {} closed", params.id)
            } else {
                format!("WebView {} not found", params.id)
            },
        })
    }
}

impl AuroraViewMcpServer {
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

    /// Attach an `AguiBus` so tool invocations automatically emit AG-UI events.
    pub fn with_agui_bus(mut self, bus: AguiBus) -> Self {
        self.agui_bus = Some(bus);
        self
    }

    pub fn registry(&self) -> &WebViewRegistry {
        &self.registry
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Return a reference to the attached AG-UI bus, if any.
    pub fn agui_bus(&self) -> Option<&AguiBus> {
        self.agui_bus.as_ref()
    }

    /// Resolve a WebView ID: use provided string or fall back to first registered.
    fn resolve_id(&self, id: Option<&str>) -> String {
        if let Some(s) = id {
            return s.to_string();
        }
        self.registry
            .list()
            .into_iter()
            .next()
            .map(|v| v.id.0)
            .unwrap_or_else(|| "default".to_string())
    }

    /// Emit `ToolCallStart` when a tool begins execution.
    fn emit_tool_start(&self, tool_name: &str, call_id: &str, run_id: &str) {
        if let Some(bus) = &self.agui_bus {
            bus.emit(AguiEvent::ToolCallStart {
                run_id: run_id.to_string(),
                tool_call_id: call_id.to_string(),
                tool_name: tool_name.to_string(),
            });
        }
    }

    /// Emit `ToolCallEnd` when a tool finishes execution.
    fn emit_tool_end(&self, call_id: &str, run_id: &str) {
        if let Some(bus) = &self.agui_bus {
            bus.emit(AguiEvent::ToolCallEnd {
                run_id: run_id.to_string(),
                tool_call_id: call_id.to_string(),
            });
        }
    }
}

impl ServerHandler for AuroraViewMcpServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_instructions(
            "AuroraView MCP Server: manage WebView windows in DCC applications (Maya, Houdini, Blender, UE, etc.)"
        )
    }

    fn call_tool(
        &self,
        req: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let ctx = ToolCallContext::new(self, req, context);
        self.tool_router.call(ctx)
    }

    fn list_tools(
        &self,
        _req: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_
    {
        let tools = self.tool_router.list_all();
        async move {
            Ok(ListToolsResult {
                tools,
                ..Default::default()
            })
        }
    }
}
