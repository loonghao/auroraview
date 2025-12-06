//! Integration tests for WebView configuration
//!
//! These tests verify the WebViewConfig and WebViewBuilder functionality.

#[cfg(target_os = "windows")]
use _core::webview::config::EmbedMode;
use _core::webview::config::{WebViewBuilder, WebViewConfig};
use rstest::*;
use std::path::PathBuf;
use std::sync::Arc;

/// Fixture: Create a default config
#[fixture]
fn default_config() -> WebViewConfig {
    WebViewConfig::default()
}

/// Fixture: Create a builder
#[fixture]
fn builder() -> WebViewBuilder {
    WebViewBuilder::new()
}

#[rstest]
fn test_default_config_values(default_config: WebViewConfig) {
    assert_eq!(default_config.title, "AuroraView");
    assert_eq!(default_config.width, 800);
    assert_eq!(default_config.height, 600);
    assert!(default_config.url.is_none());
    assert!(default_config.html.is_none());
    assert!(default_config.dev_tools);
    assert!(default_config.context_menu);
    assert!(default_config.resizable);
    assert!(default_config.decorations);
    assert!(!default_config.always_on_top);
    assert!(!default_config.transparent);
    assert!(default_config.ipc_batching);
    assert_eq!(default_config.ipc_batch_size, 10);
    assert_eq!(default_config.ipc_batch_interval_ms, 16);
    assert!(default_config.asset_root.is_none());
    assert_eq!(default_config.custom_protocols.len(), 0);
}

#[rstest]
fn test_builder_title(builder: WebViewBuilder) {
    let config = builder.title("Test Window").build();
    assert_eq!(config.title, "Test Window");
}

#[rstest]
#[case(1024, 768)]
#[case(1920, 1080)]
#[case(800, 600)]
fn test_builder_size(builder: WebViewBuilder, #[case] width: u32, #[case] height: u32) {
    let config = builder.size(width, height).build();
    assert_eq!(config.width, width);
    assert_eq!(config.height, height);
}

#[rstest]
fn test_builder_url(builder: WebViewBuilder) {
    let config = builder.url("https://example.com").build();
    assert_eq!(config.url.as_deref(), Some("https://example.com"));
}

#[rstest]
fn test_builder_html(builder: WebViewBuilder) {
    let html = "<h1>Hello World</h1>";
    let config = builder.html(html).build();
    assert_eq!(config.html.as_deref(), Some(html));
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_dev_tools(builder: WebViewBuilder, #[case] enabled: bool) {
    let config = builder.dev_tools(enabled).build();
    assert_eq!(config.dev_tools, enabled);
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_context_menu(builder: WebViewBuilder, #[case] enabled: bool) {
    let config = builder.context_menu(enabled).build();
    assert_eq!(config.context_menu, enabled);
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_resizable(builder: WebViewBuilder, #[case] resizable: bool) {
    let config = builder.resizable(resizable).build();
    assert_eq!(config.resizable, resizable);
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_decorations(builder: WebViewBuilder, #[case] decorations: bool) {
    let config = builder.decorations(decorations).build();
    assert_eq!(config.decorations, decorations);
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_always_on_top(builder: WebViewBuilder, #[case] always_on_top: bool) {
    let config = builder.always_on_top(always_on_top).build();
    assert_eq!(config.always_on_top, always_on_top);
}

#[rstest]
#[case(true)]
#[case(false)]
fn test_builder_transparent(builder: WebViewBuilder, #[case] transparent: bool) {
    let config = builder.transparent(transparent).build();
    assert_eq!(config.transparent, transparent);
}

#[rstest]
fn test_builder_asset_root(builder: WebViewBuilder) {
    let path = PathBuf::from("/tmp/assets");
    let config = builder.asset_root(path.clone()).build();
    assert_eq!(config.asset_root, Some(path));
}

#[rstest]
fn test_builder_register_protocol(builder: WebViewBuilder) {
    let handler = Arc::new(|uri: &str| {
        if uri.starts_with("test://") {
            Some((b"test data".to_vec(), "text/plain".to_string(), 200))
        } else {
            None
        }
    });

    let config = builder.register_protocol("test", handler).build();
    assert_eq!(config.custom_protocols.len(), 1);
    assert!(config.custom_protocols.contains_key("test"));
}

#[rstest]
fn test_builder_chain_multiple_settings(builder: WebViewBuilder) {
    let config = builder
        .title("Chained Test")
        .size(1280, 720)
        .url("https://test.com")
        .dev_tools(false)
        .resizable(false)
        .build();

    assert_eq!(config.title, "Chained Test");
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 720);
    assert_eq!(config.url.as_deref(), Some("https://test.com"));
    assert!(!config.dev_tools);
    assert!(!config.resizable);
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_embed_mode_windows() {
    let config = WebViewConfig::default();
    assert_eq!(config.embed_mode, EmbedMode::None);
}

#[rstest]
fn test_builder_ipc_batching(builder: WebViewBuilder) {
    let config = builder.build();
    assert!(config.ipc_batching);
    assert_eq!(config.ipc_batch_size, 10);
    assert_eq!(config.ipc_batch_interval_ms, 16);
}

#[rstest]
fn test_builder_default_values(builder: WebViewBuilder) {
    let config = builder.build();
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.dev_tools);
    assert!(config.context_menu);
    assert!(config.resizable);
    assert!(config.decorations);
}

#[rstest]
fn test_builder_multiple_protocols(builder: WebViewBuilder) {
    let handler1 = Arc::new(|uri: &str| {
        if uri.starts_with("custom1://") {
            Some((b"data1".to_vec(), "text/plain".to_string(), 200))
        } else {
            None
        }
    });

    let handler2 = Arc::new(|uri: &str| {
        if uri.starts_with("custom2://") {
            Some((b"data2".to_vec(), "text/html".to_string(), 200))
        } else {
            None
        }
    });

    let config = builder
        .register_protocol("custom1", handler1)
        .register_protocol("custom2", handler2)
        .build();

    assert_eq!(config.custom_protocols.len(), 2);
    assert!(config.custom_protocols.contains_key("custom1"));
    assert!(config.custom_protocols.contains_key("custom2"));
}
