//! Tool parameter structs for `McpServer`.
//!
//! This module contains all the parameter structs used by the MCP tools.
//! Each struct derives `Debug`, `Deserialize`, and `JsonSchema` for rmcp.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

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
    /// JavaScript expression to evaluate in the `WebView` context.
    pub script: String,
}

/// Parameters for the `load_url` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LoadUrlParams {
    /// URL to load in the `WebView`.
    pub url: String,
}

/// Parameters for the `send_event` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendEventParams {
    /// Event name to emit in the `WebView`.
    pub event: String,
    /// Event payload (JSON value).
    pub data: Value,
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
