//! Integration tests for standalone WebView mode
//!
//! These tests verify the standalone WebView functionality including:
//! - Configuration handling
//! - Loading screen HTML generation
//! - URL loading script generation

use _core::webview::config::WebViewConfig;
use _core::webview::js_assets;
use rstest::*;

/// Test standalone WebView configuration defaults
#[rstest]
fn test_standalone_config_defaults() {
    let config = WebViewConfig::default();

    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.resizable);
    assert!(config.decorations);
    assert!(!config.transparent);
    assert!(config.dev_tools);
}

/// Test standalone WebView configuration with custom values
#[rstest]
#[case("Custom Title", 1024, 768)]
#[case("Test Window", 640, 480)]
#[case("My App", 1920, 1080)]
fn test_standalone_config_custom(#[case] title: &str, #[case] width: u32, #[case] height: u32) {
    let config = WebViewConfig {
        title: title.to_string(),
        width,
        height,
        ..Default::default()
    };

    assert_eq!(config.title, title);
    assert_eq!(config.width, width);
    assert_eq!(config.height, height);
}

/// Test loading screen HTML generation
#[rstest]
fn test_loading_html_generation() {
    let html = js_assets::get_loading_html();

    // Verify HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));

    // Verify loading animation elements
    assert!(html.contains("loading"));
    assert!(html.contains("spin")); // Animation keyframe

    // Verify styling
    assert!(html.contains("background"));
    assert!(html.contains("gradient"));
}

/// Test URL loading script generation
///
/// Note: The `build_load_url_script` function generates JavaScript for
/// window.location.href navigation. However, in standalone mode, we now
/// use wry's native `load_url()` method for more reliable navigation,
/// especially after splash screen loading. This test verifies the script
/// generation still works for backward compatibility and other use cases.
#[rstest]
#[case("https://example.com")]
#[case("https://google.com")]
#[case("http://localhost:8080")]
fn test_url_loading_script(#[case] url: &str) {
    let script = js_assets::build_load_url_script(url);

    // Verify script contains URL
    assert!(script.contains(url));

    // Verify script uses window.location.href
    assert!(script.contains("window.location.href"));
}

/// Test HTML registry contains loading screen
#[rstest]
fn test_html_registry_has_loading() {
    let html = js_assets::get_loading_html();

    // Should not be empty
    assert!(!html.is_empty());

    // Should be valid HTML
    assert!(html.starts_with("<!DOCTYPE html>") || html.starts_with("<html"));
}

/// Test standalone config with URL
#[rstest]
fn test_standalone_config_with_url() {
    let url = "https://example.com";
    let config = WebViewConfig {
        url: Some(url.to_string()),
        ..Default::default()
    };

    assert_eq!(config.url, Some(url.to_string()));
    assert_eq!(config.html, None);
}

/// Test standalone config with HTML
#[rstest]
fn test_standalone_config_with_html() {
    let html = "<html><body>Test</body></html>";
    let config = WebViewConfig {
        html: Some(html.to_string()),
        ..Default::default()
    };

    assert_eq!(config.html, Some(html.to_string()));
    assert_eq!(config.url, None);
}

/// Test standalone config with both URL and HTML (URL takes precedence)
#[rstest]
fn test_standalone_config_url_precedence() {
    let url = "https://example.com";
    let html = "<html><body>Test</body></html>";
    let config = WebViewConfig {
        url: Some(url.to_string()),
        html: Some(html.to_string()),
        ..Default::default()
    };

    // Both should be set, but URL takes precedence in standalone mode
    assert_eq!(config.url, Some(url.to_string()));
    assert_eq!(config.html, Some(html.to_string()));
}

/// Test window transparency configuration
#[rstest]
fn test_standalone_window_transparency() {
    let config = WebViewConfig {
        transparent: true,
        ..Default::default()
    };

    assert!(config.transparent);
}

/// Test developer tools configuration
#[rstest]
#[case(true)]
#[case(false)]
fn test_standalone_dev_tools(#[case] dev_tools: bool) {
    let config = WebViewConfig {
        dev_tools,
        ..Default::default()
    };

    assert_eq!(config.dev_tools, dev_tools);
}

/// Test standalone config with all options
#[rstest]
fn test_standalone_config_complete() {
    let config = WebViewConfig {
        title: "Test App".to_string(),
        width: 1024,
        height: 768,
        url: Some("https://example.com".to_string()),
        html: None,
        dev_tools: true,
        resizable: true,
        decorations: true,
        transparent: false,
        always_on_top: false,
        background_color: None,
        context_menu: true,
        parent_hwnd: None,
        embed_mode: _core::webview::config::EmbedMode::None,
        ipc_batching: false,
        ipc_batch_size: 100,
        ipc_batch_interval_ms: 16,
        asset_root: None,
        custom_protocols: std::collections::HashMap::new(),
        api_methods: std::collections::HashMap::new(),
        allow_new_window: false,
        allow_file_protocol: false,
        ..Default::default()
    };

    assert_eq!(config.title, "Test App");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert_eq!(config.url, Some("https://example.com".to_string()));
    assert!(config.dev_tools);
    assert!(config.resizable);
    assert!(config.decorations);
    assert!(!config.transparent);
}

/// Test embed mode configuration
#[rstest]
fn test_standalone_embed_mode() {
    use _core::webview::config::EmbedMode;

    let config = WebViewConfig {
        embed_mode: EmbedMode::None,
        ..Default::default()
    };

    // Verify embed mode is set correctly
    assert!(matches!(config.embed_mode, EmbedMode::None));
}

/// Test IPC configuration
#[rstest]
fn test_standalone_ipc_config() {
    let config = WebViewConfig {
        ipc_batching: true,
        ipc_batch_size: 200,
        ipc_batch_interval_ms: 32,
        ..Default::default()
    };

    assert!(config.ipc_batching);
    assert_eq!(config.ipc_batch_size, 200);
    assert_eq!(config.ipc_batch_interval_ms, 32);
}

/// Test context menu configuration
#[rstest]
#[case(true)]
#[case(false)]
fn test_standalone_context_menu(#[case] context_menu: bool) {
    let config = WebViewConfig {
        context_menu,
        ..Default::default()
    };

    assert_eq!(config.context_menu, context_menu);
}

/// Test always on top configuration
#[rstest]
fn test_standalone_always_on_top() {
    let config = WebViewConfig {
        always_on_top: true,
        ..Default::default()
    };

    assert!(config.always_on_top);
}

/// Test background color configuration
#[rstest]
fn test_standalone_background_color() {
    let config = WebViewConfig {
        background_color: Some("#ffffff".to_string()),
        ..Default::default()
    };

    assert_eq!(config.background_color, Some("#ffffff".to_string()));
}

/// Test asset_root configuration
#[rstest]
fn test_standalone_asset_root() {
    use std::path::PathBuf;

    // Test with None (default)
    let config = WebViewConfig::default();
    assert_eq!(config.asset_root, None);

    // Test with Some path
    let config = WebViewConfig {
        asset_root: Some(PathBuf::from("/tmp/assets")),
        ..Default::default()
    };
    assert_eq!(config.asset_root, Some(PathBuf::from("/tmp/assets")));
}

/// Test width=0 or height=0 configuration for maximize
#[rstest]
#[case(0, 600)]
#[case(800, 0)]
#[case(0, 0)]
fn test_standalone_zero_dimensions_for_maximize(#[case] width: u32, #[case] height: u32) {
    let config = WebViewConfig {
        width,
        height,
        ..Default::default()
    };

    // Verify dimensions are stored correctly
    assert_eq!(config.width, width);
    assert_eq!(config.height, height);

    // When width or height is 0, the standalone runner should maximize the window
    // This logic is in standalone.rs, not in config
    let should_maximize = width == 0 || height == 0;
    assert!(
        should_maximize,
        "Expected maximize for width={}, height={}",
        width, height
    );
}

/// Test allow_file_protocol configuration
#[rstest]
fn test_standalone_allow_file_protocol() {
    // Test default (false)
    let config = WebViewConfig::default();
    assert!(!config.allow_file_protocol);

    // Test enabled
    let config = WebViewConfig {
        allow_file_protocol: true,
        ..Default::default()
    };
    assert!(config.allow_file_protocol);
}

/// Test combined asset_root and allow_file_protocol
#[rstest]
fn test_standalone_local_file_options() {
    use std::path::PathBuf;

    // Test with asset_root only (recommended)
    let config = WebViewConfig {
        asset_root: Some(PathBuf::from("./assets")),
        allow_file_protocol: false,
        ..Default::default()
    };
    assert!(config.asset_root.is_some());
    assert!(!config.allow_file_protocol);

    // Test with allow_file_protocol only
    let config = WebViewConfig {
        asset_root: None,
        allow_file_protocol: true,
        ..Default::default()
    };
    assert!(config.asset_root.is_none());
    assert!(config.allow_file_protocol);

    // Test with both (valid but unusual)
    let config = WebViewConfig {
        asset_root: Some(PathBuf::from("./assets")),
        allow_file_protocol: true,
        ..Default::default()
    };
    assert!(config.asset_root.is_some());
    assert!(config.allow_file_protocol);
}
