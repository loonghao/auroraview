//! JavaScript assets management
//!
//! This module manages all JavaScript code that is injected into the WebView.
//! JavaScript files are stored in `auroraview-core/src/assets/js/` and embedded
//! at compile time using the `rust_embed` crate in auroraview-core.
//!
//! ## Architecture
//!
//! - **Core scripts**: Always included, provide fundamental functionality
//! - **Feature scripts**: Conditionally included based on WebViewConfig
//!
//! ## Template Support
//!
//! When the `templates` feature is enabled, this module uses Askama templates
//! for type-safe JavaScript code generation. Otherwise, it falls back to
//! simple string replacement.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::webview::js_assets;
//! use crate::webview::WebViewConfig;
//!
//! let config = WebViewConfig::default();
//! let init_script = js_assets::build_init_script(&config);
//! ```

use crate::webview::WebViewConfig;

#[cfg(feature = "templates")]
use crate::webview::js_templates::{
    ApiMethodEntry, ApiRegistrationTemplate, EmitEventTemplate, LoadUrlTemplate,
};
#[cfg(feature = "templates")]
use askama::Template;

// Re-export from auroraview-core for convenience
pub use auroraview_core::assets::{
    get_all_plugins_js, get_bridge_stub_js, get_browsing_data_js, get_channel_bridge_js,
    get_clipboard_plugin_js, get_command_bridge_js, get_context_menu_js, get_dialog_plugin_js,
    get_dom_events_js, get_emit_event_js, get_event_bridge_js, get_event_utils_js,
    get_file_drop_js, get_fs_plugin_js, get_js_asset, get_load_url_js, get_loading_html,
    get_loading_html as get_loading_html_string, get_navigation_api_js, get_navigation_tracker_js,
    get_network_intercept_js, get_plugin_js, get_screenshot_js, get_shell_plugin_js,
    get_state_bridge_js, get_test_callback_js, get_typescript_definitions, get_zoom_api_js,
    plugin_names,
};

/// Get JavaScript code by path
///
/// Dynamically loads JavaScript assets by their relative path from `assets/js/`.
/// All assets are embedded at compile time in auroraview-core.
///
/// # Arguments
///
/// * `path` - Relative path from `assets/js/`, e.g., "core/event_bridge.js"
///
/// # Returns
///
/// The JavaScript code as a String, or None if path not found
pub fn get_js_code(path: &str) -> Option<String> {
    get_js_asset(path)
}

/// Get event bridge JavaScript (lazy loaded from core)
pub fn event_bridge() -> String {
    get_event_bridge_js()
}

/// Get context menu disable JavaScript (lazy loaded from core)
pub fn context_menu_disable() -> String {
    get_context_menu_js()
}

/// Build complete initialization script based on configuration
///
/// This function assembles the final JavaScript initialization script
/// by combining core scripts and optional feature scripts based on
/// the provided WebViewConfig.
///
/// # Arguments
///
/// * `config` - WebView configuration
///
/// # Returns
///
/// Complete JavaScript initialization script as a String
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::{WebViewConfig, js_assets};
///
/// let mut config = WebViewConfig::default();
/// config.context_menu = false;
///
/// let script = js_assets::build_init_script(&config);
/// // script now contains event_bridge.js + context_menu.js
/// ```
pub fn build_init_script(config: &WebViewConfig) -> String {
    let mut script = String::with_capacity(32768); // Pre-allocate reasonable size

    // Core scripts (always included) - loaded from auroraview-core
    script.push_str(&event_bridge());
    script.push('\n');

    // BOM (Browser Object Model) scripts - always included
    // These provide navigation tracking, DOM events, and utility functions

    // Navigation tracker - handles popstate, pushState, loading progress
    script.push_str(&get_navigation_tracker_js());
    script.push('\n');

    // DOM events tracker - handles title/URL changes, visibility, focus
    script.push_str(&get_dom_events_js());
    script.push('\n');

    // Browsing data utilities - clear localStorage, sessionStorage, cookies
    script.push_str(&get_browsing_data_js());
    script.push('\n');

    // Navigation API utilities - canGoBack, canGoForward, isLoading
    script.push_str(&get_navigation_api_js());
    script.push('\n');

    // Zoom API utilities - setZoom, getZoom, zoomIn, zoomOut
    script.push_str(&get_zoom_api_js());
    script.push('\n');

    // File drop handler - drag and drop file support
    script.push_str(&get_file_drop_js());
    script.push('\n');

    // State bridge for Python ↔ JavaScript state sync
    script.push_str(&get_state_bridge_js());
    script.push('\n');

    // Command bridge for RPC-style invocation
    script.push_str(&get_command_bridge_js());
    script.push('\n');

    // Channel bridge for streaming data
    script.push_str(&get_channel_bridge_js());
    script.push('\n');

    // Event utilities - debounce, throttle, once
    script.push_str(&get_event_utils_js());
    script.push('\n');

    // Screenshot support - always included for testing framework
    script.push_str(&get_screenshot_js());
    script.push('\n');

    // Network interception - always included for testing framework
    script.push_str(&get_network_intercept_js());
    script.push('\n');

    // Test callback bridge - always included for testing framework
    script.push_str(&get_test_callback_js());
    script.push('\n');

    // Optional features based on configuration
    if !config.context_menu {
        script.push_str(&context_menu_disable());
        script.push('\n');
    }

    // Plugin JavaScript APIs (fs, dialog, clipboard, shell)
    if config.enable_plugins {
        if config.enabled_plugin_names.is_empty() {
            // All plugins enabled
            script.push_str(&get_all_plugins_js());
        } else {
            // Only specific plugins
            for name in &config.enabled_plugin_names {
                if let Some(js) = get_plugin_js(name) {
                    if !js.is_empty() {
                        script.push_str(&js);
                        script.push('\n');
                    }
                }
            }
        }
        script.push('\n');
    }

    // API method registration
    if !config.api_methods.is_empty() {
        script.push_str(&build_api_registration_script(&config.api_methods));
        script.push('\n');
    }

    script
}

/// Build API registration script
///
/// Generates JavaScript code to register API methods using the
/// window.auroraview._registerApiMethods helper function.
///
/// When the `templates` feature is enabled, uses Askama templates for
/// type-safe code generation. Otherwise, falls back to manual string building.
///
/// # Arguments
///
/// * `api_methods` - Map of namespace to method names
///
/// # Returns
///
/// JavaScript code that registers all API methods
#[cfg(feature = "templates")]
fn build_api_registration_script(
    api_methods: &std::collections::HashMap<String, Vec<String>>,
) -> String {
    let entries: Vec<ApiMethodEntry> = api_methods
        .iter()
        .map(|(namespace, methods)| ApiMethodEntry {
            namespace: namespace.replace('\'', "\\'"),
            methods: methods.iter().map(|m| m.replace('\'', "\\'")).collect(),
        })
        .collect();

    let template = ApiRegistrationTemplate {
        api_methods: entries,
    };
    template.render().unwrap_or_else(|e| {
        eprintln!(
            "[AuroraView] Failed to render API registration template: {}",
            e
        );
        String::new()
    })
}

#[cfg(not(feature = "templates"))]
fn build_api_registration_script(
    api_methods: &std::collections::HashMap<String, Vec<String>>,
) -> String {
    let mut script = String::new();

    script.push_str("// Auto-generated API method registration\n");
    script.push_str("(function() {\n");
    script.push_str("    if (!window.auroraview || !window.auroraview._registerApiMethods) {\n");
    script.push_str("        console.error('[AuroraView] Event bridge not initialized!');\n");
    script.push_str("        return;\n");
    script.push_str("    }\n\n");

    for (namespace, methods) in api_methods {
        if methods.is_empty() {
            continue;
        }

        // Build JSON array of method names
        let methods_json: Vec<String> = methods
            .iter()
            .map(|m| format!("'{}'", m.replace('\'', "\\'")))
            .collect();

        script.push_str(&format!(
            "    window.auroraview._registerApiMethods('{}', [{}]);\n",
            namespace.replace('\'', "\\'"),
            methods_json.join(", ")
        ));
    }

    script.push_str("})();\n");

    script
}

/// Get event bridge script only
///
/// Returns just the core event bridge without any optional features.
/// Useful for minimal WebView setups.
#[allow(dead_code)]
pub fn get_event_bridge() -> String {
    event_bridge()
}

/// Get context menu disable script only
///
/// Returns just the context menu disable script.
/// Useful for dynamic injection after WebView creation.
#[allow(dead_code)]
pub fn get_context_menu_disable() -> String {
    context_menu_disable()
}

/// JavaScript asset types
///
/// Enum representing all available JavaScript assets.
/// Used with `get_asset()` for dynamic loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum JsAsset {
    /// Core event bridge (window.auroraview API)
    EventBridge,
    /// Context menu disable script
    ContextMenuDisable,
}

/// Get a JavaScript asset by type
///
/// This function provides a dynamic way to load JavaScript assets at runtime.
/// All assets are embedded at compile time in auroraview-core.
///
/// # Arguments
///
/// * `asset` - The type of asset to retrieve
///
/// # Returns
///
/// The JavaScript code as a String
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::js_assets::{get_asset, JsAsset};
///
/// let event_bridge = get_asset(JsAsset::EventBridge);
/// let context_menu = get_asset(JsAsset::ContextMenuDisable);
/// ```
#[allow(dead_code)]
pub fn get_asset(asset: JsAsset) -> String {
    match asset {
        JsAsset::EventBridge => event_bridge(),
        JsAsset::ContextMenuDisable => context_menu_disable(),
    }
}

/// Get multiple JavaScript assets and combine them
///
/// This function allows you to dynamically select and combine multiple
/// JavaScript assets into a single script.
///
/// # Arguments
///
/// * `assets` - Slice of asset types to include
///
/// # Returns
///
/// Combined JavaScript code as a String
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::js_assets::{get_assets, JsAsset};
///
/// let script = get_assets(&[
///     JsAsset::EventBridge,
///     JsAsset::ContextMenuDisable,
/// ]);
/// ```
#[allow(dead_code)]
pub fn get_assets(assets: &[JsAsset]) -> String {
    let mut script = String::with_capacity(8192);

    for asset in assets {
        script.push_str(&get_asset(*asset));
        script.push('\n');
    }

    script
}

/// Generate script to emit an event to JavaScript
///
/// Creates JavaScript code that uses window.auroraview.trigger() to dispatch
/// an event from Rust/Python to JavaScript listeners.
///
/// When the `templates` feature is enabled, uses Askama templates for
/// type-safe code generation.
///
/// # Arguments
///
/// * `event_name` - Name of the event to trigger
/// * `event_data` - JSON string of event data (must be properly escaped)
///
/// # Returns
///
/// JavaScript code as a String
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::js_assets;
///
/// let json_data = r#"{"message": "hello"}"#;
/// let escaped = json_data.replace('\\', "\\\\").replace('\'', "\\'");
/// let script = js_assets::build_emit_event_script("my_event", &escaped);
/// ```
#[cfg(feature = "templates")]
pub fn build_emit_event_script(event_name: &str, event_data: &str) -> String {
    let template = EmitEventTemplate {
        event_name,
        event_data,
    };
    template.render().unwrap_or_else(|e| {
        eprintln!("[AuroraView] Failed to render emit event template: {}", e);
        // Fallback to legacy method
        get_emit_event_js()
            .replace("{EVENT_NAME}", event_name)
            .replace("{EVENT_DATA}", event_data)
    })
}

#[cfg(not(feature = "templates"))]
pub fn build_emit_event_script(event_name: &str, event_data: &str) -> String {
    get_emit_event_js()
        .replace("{EVENT_NAME}", event_name)
        .replace("{EVENT_DATA}", event_data)
}

/// Generate script for async JavaScript execution with result callback
///
/// Creates JavaScript code that:
/// 1. Executes the user's script
/// 2. Captures the result (or error)
/// 3. Sends the result back to Python via IPC
///
/// # Arguments
///
/// * `script` - JavaScript code to execute
/// * `callback_id` - Unique ID to correlate request with response
///
/// # Returns
///
/// JavaScript code as a String that wraps the user script
pub fn build_eval_js_async_script(script: &str, callback_id: u64) -> String {
    // Escape the script for embedding in a string literal
    let escaped_script = script
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    format!(
        r#"(function() {{
    'use strict';
    var callbackId = {callback_id};
    var result = null;
    var error = null;

    try {{
        // Execute the user script and capture result
        result = (function() {{
            return eval(`{escaped_script}`);
        }})();
    }} catch (e) {{
        error = {{
            message: e.message || String(e),
            name: e.name || 'Error',
            stack: e.stack || null
        }};
    }}

    // Send result back to Python via IPC
    try {{
        var payload = {{
            type: 'js_callback_result',
            callback_id: callbackId,
            result: result,
            error: error
        }};
        window.ipc.postMessage(JSON.stringify(payload));
    }} catch (ipcError) {{
        console.error('[AuroraView] Failed to send eval_js_async result:', ipcError);
    }}
}})();"#,
        callback_id = callback_id,
        escaped_script = escaped_script
    )
}

/// Generate script to load a URL
///
/// Creates JavaScript code that navigates the WebView to a new URL
/// by setting window.location.href.
///
/// When the `templates` feature is enabled, uses Askama templates for
/// type-safe code generation.
///
/// # Arguments
///
/// * `url` - Target URL to navigate to
///
/// # Returns
///
/// JavaScript code as a String
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::js_assets;
///
/// let script = js_assets::build_load_url_script("https://example.com");
/// ```
#[cfg(feature = "templates")]
pub fn build_load_url_script(url: &str) -> String {
    let template = LoadUrlTemplate { url };
    template.render().unwrap_or_else(|e| {
        eprintln!("[AuroraView] Failed to render load URL template: {}", e);
        // Fallback to legacy method
        get_load_url_js().replace("{URL}", url)
    })
}

#[cfg(not(feature = "templates"))]
pub fn build_load_url_script(url: &str) -> String {
    get_load_url_js().replace("{URL}", url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_init_script_default() {
        let config = WebViewConfig::default();
        let script = build_init_script(&config);

        // Should include event bridge
        assert!(script.contains("window.auroraview"));
        // Should NOT include context menu disable (default is true)
        assert!(!script.contains("contextmenu"));
    }

    #[test]
    fn test_build_init_script_no_context_menu() {
        let config = WebViewConfig {
            context_menu: false,
            ..Default::default()
        };
        let script = build_init_script(&config);

        // Should include context menu disable
        assert!(script.contains("contextmenu"));
        assert!(script.contains("preventDefault"));
    }

    #[test]
    fn test_individual_scripts() {
        // Test that individual getters work
        assert!(get_event_bridge().contains("window.auroraview"));
        assert!(get_context_menu_disable().contains("contextmenu"));
    }

    #[test]
    fn test_get_asset() {
        // Test dynamic asset loading
        let event_bridge = get_asset(JsAsset::EventBridge);
        assert!(event_bridge.contains("window.auroraview"));

        let context_menu = get_asset(JsAsset::ContextMenuDisable);
        assert!(context_menu.contains("contextmenu"));
    }

    #[test]
    fn test_get_assets() {
        // Test combining multiple assets
        let script = get_assets(&[JsAsset::EventBridge, JsAsset::ContextMenuDisable]);

        assert!(script.contains("window.auroraview"));
        assert!(script.contains("contextmenu"));
    }

    #[test]
    fn test_get_assets_empty() {
        // Test empty asset list
        let script = get_assets(&[]);
        assert_eq!(script, "");
    }

    #[test]
    fn test_get_assets_single() {
        // Test single asset
        let script = get_assets(&[JsAsset::EventBridge]);
        assert!(script.contains("window.auroraview"));
        assert!(!script.contains("contextmenu"));
    }

    #[test]
    fn test_get_js_code() {
        // Test dynamic path-based loading
        let event_bridge = get_js_code("core/event_bridge.js").unwrap();
        assert!(event_bridge.contains("window.auroraview"));

        let context_menu = get_js_code("features/context_menu.js").unwrap();
        assert!(context_menu.contains("contextmenu"));

        let emit_event = get_js_code("runtime/emit_event.js").unwrap();
        assert!(emit_event.contains("{EVENT_NAME}"));

        let load_url = get_js_code("runtime/load_url.js").unwrap();
        assert!(load_url.contains("{URL}"));
    }

    #[test]
    fn test_get_js_code_not_found() {
        // Test non-existent path
        let result = get_js_code("nonexistent/file.js");
        assert!(result.is_none());
    }

    #[test]
    fn test_build_scripts_use_registry() {
        // Test that build functions use the registry
        let emit_script = build_emit_event_script("test_event", r#"{"data": "test"}"#);
        assert!(emit_script.contains("test_event"));
        assert!(emit_script.contains(r#"{"data": "test"}"#));

        let load_script = build_load_url_script("https://example.com");
        assert!(load_script.contains("https://example.com"));
    }

    #[test]
    fn test_build_api_registration_script() {
        // Test API registration script generation
        let mut api_methods = std::collections::HashMap::new();
        api_methods.insert(
            "test".to_string(),
            vec!["method1".to_string(), "method2".to_string()],
        );

        let script = build_api_registration_script(&api_methods);

        assert!(script.contains("window.auroraview._registerApiMethods"));
        assert!(script.contains("'test'"));
        assert!(script.contains("'method1'"));
        assert!(script.contains("'method2'"));
    }

    #[test]
    fn test_build_api_registration_script_empty_methods() {
        // Test with empty methods list
        let mut api_methods = std::collections::HashMap::new();
        api_methods.insert("test".to_string(), vec![]);

        let script = build_api_registration_script(&api_methods);

        // Should not include the namespace with empty methods
        assert!(!script.contains("'test'"));
    }

    #[test]
    fn test_build_api_registration_script_special_chars() {
        // Test escaping of special characters
        let mut api_methods = std::collections::HashMap::new();
        api_methods.insert("test'namespace".to_string(), vec!["method'1".to_string()]);

        let script = build_api_registration_script(&api_methods);

        // Should escape single quotes
        assert!(script.contains("\\'"));
    }

    #[test]
    fn test_build_init_script_with_api_methods() {
        // Test init script with API methods
        let mut config = WebViewConfig::default();
        let mut api_methods = std::collections::HashMap::new();
        api_methods.insert("test".to_string(), vec!["method1".to_string()]);
        config.api_methods = api_methods;

        let script = build_init_script(&config);

        assert!(script.contains("window.auroraview._registerApiMethods"));
        assert!(script.contains("'test'"));
        assert!(script.contains("'method1'"));
    }

    #[test]
    fn test_all_core_assets_available() {
        // Test that all core assets are available from auroraview-core
        assert!(!get_event_bridge_js().is_empty());
        assert!(!get_state_bridge_js().is_empty());
        assert!(!get_command_bridge_js().is_empty());
        assert!(!get_channel_bridge_js().is_empty());
        assert!(!get_context_menu_js().is_empty());
        assert!(!get_emit_event_js().is_empty());
        assert!(!get_load_url_js().is_empty());
    }

    #[test]
    fn test_emit_event_script_template_replacement() {
        // Test that template placeholders are replaced correctly
        let script = build_emit_event_script("my_event", r#"{"key": "value"}"#);

        assert!(!script.contains("{EVENT_NAME}"));
        assert!(!script.contains("{EVENT_DATA}"));
        assert!(script.contains("my_event"));
        assert!(script.contains(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_load_url_script_template_replacement() {
        // Test that URL template is replaced correctly
        let script = build_load_url_script("https://example.com/path");

        assert!(!script.contains("{URL}"));
        assert!(script.contains("https://example.com/path"));
    }
}
