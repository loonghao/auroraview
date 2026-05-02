// Parameter and output types for AuroraViewMcpServer tools.

use schemars;
use serde::{Deserialize, Serialize};

// --- Parameter types ---

/// Parameters for the `screenshot` tool.
///
/// Captures the current visible area of a `WebView` and returns
/// the image as a base64‑encoded PNG.
#[derive(Debug, Deserialize, schemars::JsonSchema, Default)]
pub struct ScreenshotParams {
    /// Optional `WebView` ID. If omitted, captures the first available `WebView`.
    pub id: Option<String>,
}

/// Parameters for the `load_url` tool.
///
/// Navigates the target `WebView` to the given URL.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadUrlParams {
    /// The URL to load (http://, https://, or file://).
    pub url: String,
    /// Target `WebView` ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadHtmlParams {
    /// Raw HTML content to load.
    pub html: String,
    /// Target `WebView` ID. Uses first available if omitted.
    pub id: Option<String>,
}

/// Parameters for the `eval_js` tool.
///
/// Evaluates a JavaScript expression in the target `WebView`
/// and returns the JSON‑serializable result.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvalJsParams {
    /// JavaScript expression to evaluate.
    pub script: String,
    /// Target `WebView` ID. Uses first available if omitted.
    pub id: Option<String>,
}

/// Parameters for the `send_event` tool.
///
/// Emits an event that the front‑end can subscribe to via
/// `window.auroraview.on(event, handler)`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendEventParams {
    /// Event name.
    pub event: String,
    /// Event payload (JSON).
    pub data: Option<serde_json::Value>,
    /// Target `WebView` ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetHwndParams {
    /// `WebView` ID. Uses first available if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CloseWebViewParams {
    /// `WebView` ID to close.
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWebViewParams {
    /// Title for the new `WebView` window.
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

/// Output of the `list_webviews` tool.
///
/// Returns the current set of `WebView` instances managed by
/// the `AuroraViewMcpServer`.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListWebViewsOutput {
    pub count: usize,
    /// Capacity limit, if set. `None` means unlimited.
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
