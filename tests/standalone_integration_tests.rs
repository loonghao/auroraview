//! Integration tests for standalone WebView mode
//!
//! These tests verify the standalone WebView functionality including:
//! - Configuration handling
//! - Loading screen HTML generation
//! - URL loading script generation

use auroraview_core::webview::config::WebViewConfig;
use auroraview_core::webview::js_assets;
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
    assert!(html.contains("Loading"));
    assert!(html.contains("spinner"));

    // Verify styling
    assert!(html.contains("background"));
    assert!(html.contains("gradient"));
}

/// Test URL loading script generation
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
