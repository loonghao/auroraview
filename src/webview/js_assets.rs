//! JavaScript assets management
//!
//! This module manages all JavaScript code that is injected into the WebView.
//! JavaScript files are stored in `assets/js/` and embedded at compile time
//! using the `include_str!` macro.
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
use std::collections::HashMap;

#[cfg(feature = "templates")]
use crate::webview::js_templates::{
    ApiMethodEntry, ApiRegistrationTemplate, EmitEventTemplate, LoadUrlTemplate,
};
#[cfg(feature = "templates")]
use askama::Template;

/// HTML asset registry
///
/// All HTML files are embedded at compile time for use in WebView
fn get_html_registry() -> HashMap<&'static str, &'static str> {
    let mut registry = HashMap::new();

    // Loading screen
    registry.insert("loading.html", include_str!("../assets/html/loading.html"));

    registry
}

/// Get loading screen HTML
pub fn get_loading_html() -> &'static str {
    get_html_registry()
        .get("loading.html")
        .expect("loading.html should be in registry")
}

/// JavaScript asset registry
///
/// All JavaScript files are embedded at compile time and registered in a HashMap
/// for dynamic access by path.
fn get_js_registry() -> HashMap<&'static str, &'static str> {
    let mut registry = HashMap::new();

    // Core scripts
    registry.insert(
        "core/event_bridge.js",
        include_str!("../assets/js/core/event_bridge.js"),
    );

    // Feature scripts
    registry.insert(
        "features/context_menu.js",
        include_str!("../assets/js/features/context_menu.js"),
    );

    // Runtime templates
    registry.insert(
        "runtime/emit_event.js",
        include_str!("../assets/js/runtime/emit_event.js"),
    );
    registry.insert(
        "runtime/load_url.js",
        include_str!("../assets/js/runtime/load_url.js"),
    );

    registry
}

/// Get JavaScript code by path
///
/// Dynamically loads JavaScript assets by their relative path from `assets/js/`.
/// All assets are still embedded at compile time using `include_str!`.
///
/// # Arguments
///
/// * `path` - Relative path from `assets/js/`, e.g., "core/event_bridge.js"
///
/// # Returns
///
/// The JavaScript code as a static string slice, or None if path not found
///
/// # Example
///
/// ```rust,ignore
/// use crate::webview::js_assets;
///
/// let event_bridge = js_assets::get_js_code("core/event_bridge.js").unwrap();
/// let context_menu = js_assets::get_js_code("features/context_menu.js").unwrap();
/// ```
pub fn get_js_code(path: &str) -> Option<&'static str> {
    static REGISTRY: std::sync::OnceLock<HashMap<&'static str, &'static str>> =
        std::sync::OnceLock::new();
    let registry = REGISTRY.get_or_init(get_js_registry);
    registry.get(path).copied()
}

/// Legacy constants for backward compatibility
///
/// These are kept for existing code that uses them directly.
/// New code should use `get_js_code()` instead.
pub const EVENT_BRIDGE: &str = include_str!("../assets/js/core/event_bridge.js");
pub const CONTEXT_MENU_DISABLE: &str = include_str!("../assets/js/features/context_menu.js");

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
    let mut script = String::with_capacity(8192); // Pre-allocate reasonable size

    // Core scripts (always included)
    script.push_str(EVENT_BRIDGE);
    script.push('\n');

    // Optional features based on configuration
    if !config.context_menu {
        script.push_str(CONTEXT_MENU_DISABLE);
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
pub fn get_event_bridge() -> &'static str {
    EVENT_BRIDGE
}

/// Get context menu disable script only
///
/// Returns just the context menu disable script.
/// Useful for dynamic injection after WebView creation.
#[allow(dead_code)]
pub fn get_context_menu_disable() -> &'static str {
    CONTEXT_MENU_DISABLE
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
/// All assets are still embedded at compile time using `include_str!`.
///
/// # Arguments
///
/// * `asset` - The type of asset to retrieve
///
/// # Returns
///
/// The JavaScript code as a static string slice
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
pub fn get_asset(asset: JsAsset) -> &'static str {
    match asset {
        JsAsset::EventBridge => EVENT_BRIDGE,
        JsAsset::ContextMenuDisable => CONTEXT_MENU_DISABLE,
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
        script.push_str(get_asset(*asset));
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
        get_js_code("runtime/emit_event.js")
            .expect("emit_event.js template not found")
            .replace("{EVENT_NAME}", event_name)
            .replace("{EVENT_DATA}", event_data)
    })
}

#[cfg(not(feature = "templates"))]
pub fn build_emit_event_script(event_name: &str, event_data: &str) -> String {
    get_js_code("runtime/emit_event.js")
        .expect("emit_event.js template not found")
        .replace("{EVENT_NAME}", event_name)
        .replace("{EVENT_DATA}", event_data)
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
        get_js_code("runtime/load_url.js")
            .expect("load_url.js template not found")
            .replace("{URL}", url)
    })
}

#[cfg(not(feature = "templates"))]
pub fn build_load_url_script(url: &str) -> String {
    get_js_code("runtime/load_url.js")
        .expect("load_url.js template not found")
        .replace("{URL}", url)
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
    fn test_get_js_registry_contains_all_assets() {
        // Test that registry contains all expected assets
        let registry = get_js_registry();

        assert!(registry.contains_key("core/event_bridge.js"));
        assert!(registry.contains_key("features/context_menu.js"));
        assert!(registry.contains_key("runtime/emit_event.js"));
        assert!(registry.contains_key("runtime/load_url.js"));
        assert_eq!(registry.len(), 4);
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
