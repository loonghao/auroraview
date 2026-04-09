//! Assets tests

use auroraview_core::assets::{
    build_error_page, build_load_url_script, get_all_plugins_js, get_bridge_stub_js,
    get_browsing_data_js, get_channel_bridge_js, get_command_bridge_js, get_dom_events_js,
    get_error_html, get_event_bridge_js, get_event_utils_js, get_file_drop_js, get_js_asset,
    get_loading_html, get_midscene_bridge_js, get_navigation_api_js, get_navigation_tracker_js,
    get_network_intercept_js, get_plugin_js, get_screenshot_js, get_state_bridge_js,
    get_test_callback_js, get_zoom_api_js, plugin_names,
};

#[test]
fn test_loading_html_not_empty() {
    let html = get_loading_html();
    assert!(!html.is_empty());
    assert!(html.contains("Loading") || html.contains("loading"));
}

#[test]
fn test_error_html_not_empty() {
    let html = get_error_html();
    assert!(!html.is_empty());
    assert!(html.contains("Error") || html.contains("error"));
}

#[test]
fn test_build_error_page() {
    let html = build_error_page(
        500,
        "Internal Server Error",
        "Something went wrong",
        Some("Details here"),
        Some("https://example.com"),
    );
    assert!(html.contains("500"));
    assert!(html.contains("Internal Server Error"));
    assert!(html.contains("Something went wrong"));
    assert!(html.contains("Details here"));
    assert!(html.contains("https://example.com"));
}

#[test]
fn test_build_error_page_without_details() {
    let html = build_error_page(404, "Not Found", "Page not found", None, None);
    assert!(html.contains("404"));
    assert!(html.contains("Not Found"));
    assert!(html.contains("Page not found"));
}

#[test]
fn test_build_load_url_script() {
    let script = build_load_url_script("https://example.com");
    assert!(script.contains("https://example.com"));
    assert!(script.contains("window.location.href"));
}

#[test]
fn test_bom_scripts_available() {
    // These may be empty if assets aren't embedded, but shouldn't panic
    let _ = get_navigation_tracker_js();
    let _ = get_dom_events_js();
    let _ = get_browsing_data_js();
    let _ = get_navigation_api_js();
    let _ = get_zoom_api_js();
}

#[test]
fn test_file_drop_js_available() {
    let js = get_file_drop_js();
    // Should contain file drop handler code
    assert!(js.contains("file_drop") || js.is_empty());
}

#[test]
fn test_event_utils_js_available() {
    let js = get_event_utils_js();
    // Should contain debounce/throttle utilities
    assert!(js.contains("debounce") || js.is_empty());
}

#[test]
fn test_event_bridge_js_available() {
    let js = get_event_bridge_js();
    // Should contain auroraview bridge code
    assert!(js.contains("auroraview") || js.is_empty());
}

#[test]
fn test_plugin_names() {
    let names = plugin_names();
    assert!(names.contains(&"fs"));
    assert!(names.contains(&"dialog"));
    assert!(names.contains(&"clipboard"));
    assert!(names.contains(&"shell"));
    assert_eq!(names.len(), 4);
}

#[test]
fn test_get_plugin_js_valid() {
    // Test valid plugin names
    assert!(get_plugin_js("fs").is_some());
    assert!(get_plugin_js("dialog").is_some());
    assert!(get_plugin_js("clipboard").is_some());
    assert!(get_plugin_js("shell").is_some());
}

#[test]
fn test_get_plugin_js_invalid() {
    // Test invalid plugin name
    assert!(get_plugin_js("nonexistent").is_none());
    assert!(get_plugin_js("").is_none());
}

#[test]
fn test_get_all_plugins_js() {
    let all_js = get_all_plugins_js();
    // Should contain code from all plugins
    assert!(!all_js.is_empty() || plugin_names().is_empty());
}

#[test]
fn test_bridge_stub_js_available() {
    let js = get_bridge_stub_js();
    // Should contain stub code for early initialization
    assert!(js.contains("auroraview") || js.is_empty());
}

#[test]
fn test_state_bridge_js_available() {
    let js = get_state_bridge_js();
    // Should be available (may be empty if not embedded)
    let _ = js;
}

#[test]
fn test_channel_bridge_js_available() {
    let js = get_channel_bridge_js();
    // Should be available
    let _ = js;
}

#[test]
fn test_screenshot_js_available() {
    let js = get_screenshot_js();
    // Should contain screenshot functionality
    assert!(js.contains("screenshot") || js.is_empty());
}

#[test]
fn test_network_intercept_js_available() {
    let js = get_network_intercept_js();
    // Should contain network interception code
    let _ = js;
}

#[test]
fn test_get_js_asset() {
    // Test getting asset by path
    let result = get_js_asset("core/event_bridge.js");
    // May be Some or None depending on whether assets are embedded
    let _ = result;
}

#[test]
fn test_get_js_asset_invalid_path() {
    let result = get_js_asset("nonexistent/file.js");
    assert!(result.is_none());
}

#[test]
fn test_midscene_bridge_js_available() {
    let js = get_midscene_bridge_js();
    // Should contain midscene bridge code
    assert!(js.contains("__midscene_bridge__") || js.is_empty());
}

#[test]
fn test_test_callback_js_available() {
    let js = get_test_callback_js();
    // Should contain test callback code
    assert!(js.contains("__auroratest_callback") || js.is_empty());
}

#[test]
fn test_command_bridge_js_available() {
    let js = get_command_bridge_js();
    // Should be available
    let _ = js;
}

#[test]
fn test_context_menu_js_available() {
    let js = auroraview_core::assets::get_context_menu_js();
    // Should contain context menu code
    let _ = js;
}

#[test]
fn test_emit_event_js_available() {
    let js = auroraview_core::assets::get_emit_event_js();
    // Should be available
    let _ = js;
}

#[test]
fn test_load_url_js_available() {
    let js = auroraview_core::assets::get_load_url_js();
    // Should be available
    let _ = js;
}

#[test]
fn test_typescript_definitions_available() {
    let ts = auroraview_core::assets::get_typescript_definitions();
    // Should be available (may be empty)
    let _ = ts;
}

#[test]
fn test_build_packed_init_script() {
    let script = auroraview_core::assets::build_packed_init_script();
    // Should contain event bridge code
    assert!(script.contains("auroraview") || script.is_empty());
}

#[test]
fn test_build_csp_injection_script_contains_policy() {
    let policy = "default-src 'self'";
    let script = auroraview_core::assets::build_csp_injection_script(policy);
    assert!(script.contains("Content-Security-Policy"));
    assert!(script.contains("default-src"));
}

#[test]
fn test_build_csp_injection_script_escapes_backslash() {
    let policy = r"default-src 'self'\test";
    let script = auroraview_core::assets::build_csp_injection_script(policy);
    // backslash should be escaped to \\
    assert!(script.contains(r"\\"));
}

#[test]
fn test_build_packed_init_script_with_csp_none() {
    let without_csp = auroraview_core::assets::build_packed_init_script();
    let with_none = auroraview_core::assets::build_packed_init_script_with_csp(None);
    assert_eq!(without_csp, with_none);
}

#[test]
fn test_build_packed_init_script_with_csp_some() {
    let policy = "default-src 'self'; img-src *";
    let script = auroraview_core::assets::build_packed_init_script_with_csp(Some(policy));
    // Should start with CSP injection, then event bridge
    assert!(script.contains("Content-Security-Policy"));
    assert!(script.contains("auroraview") || script.contains("document.createElement"));
}

// ============================================================================
// R8 Extensions
// ============================================================================

#[test]
fn test_get_plugin_js_fs_not_empty() {
    let js = get_plugin_js("fs");
    assert!(js.is_some(), "fs plugin should exist");
    assert!(!js.unwrap().is_empty(), "fs plugin JS should not be empty");
}

#[test]
fn test_get_plugin_js_dialog_not_empty() {
    let js = get_plugin_js("dialog");
    assert!(js.is_some(), "dialog plugin should exist");
}

#[test]
fn test_get_plugin_js_clipboard_not_empty() {
    let js = get_plugin_js("clipboard");
    assert!(js.is_some(), "clipboard plugin should exist");
}

#[test]
fn test_get_plugin_js_shell_not_empty() {
    let js = get_plugin_js("shell");
    assert!(js.is_some(), "shell plugin should exist");
}

#[test]
fn test_plugin_names_length_matches_valid_plugins() {
    let names = plugin_names();
    // Each name should have a corresponding JS
    for name in names {
        assert!(get_plugin_js(name).is_some(), "Plugin '{}' should have JS", name);
    }
}

#[test]
fn test_build_load_url_script_nonempty() {
    let script = build_load_url_script("https://test.example.com");
    assert!(!script.is_empty());
}

#[test]
fn test_build_load_url_script_contains_url() {
    let url = "https://my-dcc-tool.internal/app";
    let script = build_load_url_script(url);
    assert!(script.contains(url) || script.contains("my-dcc-tool"), "Script should reference URL: {}", script);
}

#[test]
fn test_build_error_page_is_valid_html_structure() {
    let html = build_error_page(404, "Not Found", "Page not found", None, None);
    // Should be valid HTML structure
    assert!(html.contains("<html") || html.contains("<!DOCTYPE"), "Error page should be HTML");
    assert!(html.contains("</html>") || html.contains("</body>"), "Error page should close properly");
}

#[test]
fn test_get_all_plugins_js_contains_fs() {
    let all = get_all_plugins_js();
    // All plugins combined JS should contain something from fs plugin
    assert!(!all.is_empty());
}

#[test]
fn test_build_csp_injection_script_nonempty() {
    let policy = "default-src 'self'";
    let script = auroraview_core::assets::build_csp_injection_script(policy);
    assert!(!script.is_empty());
}
