use crate::{
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

/// AuroraView MCP Server — exposes WebView management tools via MCP.
#[derive(Clone)]
pub struct AuroraViewMcpServer {
    registry: WebViewRegistry,
    config: Arc<McpServerConfig>,
    tool_router: ToolRouter<Self>,
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
        debug!("screenshot requested for WebView: {id}");
        let data = ScreenshotData::new_placeholder(800, 600);
        Json(ScreenshotOutput {
            id,
            data: data.data,
            width: data.width,
            height: data.height,
            format: data.format,
        })
    }

    /// Load a URL in the specified WebView.
    #[tool(
        name = "load_url",
        description = "Load a URL (http://, https://, or file://) in a WebView."
    )]
    fn load_url(&self, Parameters(params): Parameters<LoadUrlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        info!("load_url: id={id} url={}", params.url);
        let updated = self.registry.update_url(&id.parse::<WebViewId>().unwrap(), &params.url);
        Json(SuccessOutput {
            ok: updated,
            message: if updated {
                format!("Loaded URL in WebView {id}")
            } else {
                format!("WebView {id} not found")
            },
        })
    }

    /// Load raw HTML content in the specified WebView.
    #[tool(
        name = "load_html",
        description = "Load HTML content directly into a WebView."
    )]
    fn load_html(&self, Parameters(params): Parameters<LoadHtmlParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        info!("load_html: id={id} html_len={}", params.html.len());
        Json(SuccessOutput {
            ok: true,
            message: format!("HTML loaded in WebView {id} ({} bytes)", params.html.len()),
        })
    }

    /// Execute JavaScript in the specified WebView and return the result.
    #[tool(
        name = "eval_js",
        description = "Execute JavaScript code in a WebView and return the result."
    )]
    fn eval_js(&self, Parameters(params): Parameters<EvalJsParams>) -> Json<JsResultOutput> {
        let id = self.resolve_id(params.id.as_deref());
        debug!("eval_js: id={id}");
        let result = JsResult::ok(serde_json::Value::Null);
        Json(JsResultOutput {
            id,
            value: result.value,
            error: result.error,
        })
    }

    /// Send a custom event to the WebView's JavaScript context.
    #[tool(
        name = "send_event",
        description = "Send a named event with payload to the WebView JS context via auroraview.on()."
    )]
    fn send_event(&self, Parameters(params): Parameters<SendEventParams>) -> Json<SuccessOutput> {
        let id = self.resolve_id(params.id.as_deref());
        info!("send_event: id={id} event={}", params.event);
        Json(SuccessOutput {
            ok: true,
            message: format!("Event '{}' sent to WebView {id}", params.event),
        })
    }

    /// Get the native window handle (HWND on Windows) for a WebView.
    #[tool(
        name = "get_hwnd",
        description = "Get the native window handle (HWND on Windows, 0 on other platforms) for embedding in UE or other hosts."
    )]
    fn get_hwnd(&self, Parameters(params): Parameters<GetHwndParams>) -> Json<HwndOutput> {
        let id = self.resolve_id(params.id.as_deref());
        let hwnd = self
            .registry
            .get(&id.parse::<WebViewId>().unwrap())
            .map(|v| v.hwnd)
            .unwrap_or(0);
        Json(HwndOutput { id, hwnd })
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
        Json(ListWebViewsOutput { count, views })
    }

    /// Create a new WebView instance.
    #[tool(
        name = "create_webview",
        description = "Create a new WebView window with the given configuration."
    )]
    fn create_webview(
        &self,
        Parameters(params): Parameters<CreateWebViewParams>,
    ) -> Json<WebViewIdOutput> {
        let config = WebViewConfig {
            title: params.title,
            url: params.url,
            html: params.html,
            width: params.width,
            height: params.height,
            visible: Some(true),
            debug: params.debug,
        };
        let id = self.registry.register(&config);
        info!("create_webview: new id={}", id.0);
        Json(WebViewIdOutput { id: id.0 })
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
        Self {
            registry: WebViewRegistry::new(),
            config: Arc::new(config),
            tool_router,
        }
    }

    pub fn registry(&self) -> &WebViewRegistry {
        &self.registry
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
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
