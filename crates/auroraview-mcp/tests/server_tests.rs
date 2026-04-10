use auroraview_mcp::{
    McpRunner, McpServerConfig, WebViewConfig, WebViewRegistry,
};
use rstest::rstest;

// --- Registry tests ---

#[rstest]
fn test_registry_register_and_list() {
    let reg = WebViewRegistry::new();
    assert!(reg.is_empty());

    let config = WebViewConfig::default();
    let id = reg.register(&config);
    assert_eq!(reg.len(), 1);

    let list = reg.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, id);
    assert_eq!(list[0].title, "AuroraView");
    assert_eq!(list[0].width, 800);
    assert_eq!(list[0].height, 600);
}

#[rstest]
fn test_registry_register_custom_config() {
    let reg = WebViewRegistry::new();
    let config = WebViewConfig {
        title: Some("My Tool".to_string()),
        url: Some("https://example.com".to_string()),
        width: Some(1280),
        height: Some(720),
        visible: Some(true),
        html: None,
        debug: Some(false),
    };
    let id = reg.register(&config);
    let info = reg.get(&id).expect("should find registered WebView");
    assert_eq!(info.title, "My Tool");
    assert_eq!(info.url, "https://example.com");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
}

#[rstest]
fn test_registry_remove() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig::default());
    assert_eq!(reg.len(), 1);

    let removed = reg.remove(&id);
    assert!(removed.is_some());
    assert!(reg.is_empty());
}

#[rstest]
fn test_registry_remove_nonexistent() {
    let reg = WebViewRegistry::new();
    let fake = "nonexistent".parse::<auroraview_mcp::WebViewId>().unwrap();
    assert!(reg.remove(&fake).is_none());
}

#[rstest]
fn test_registry_update_url() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig::default());

    let updated = reg.update_url(&id, "https://auroraview.dev");
    assert!(updated);

    let info = reg.get(&id).unwrap();
    assert_eq!(info.url, "https://auroraview.dev");
}

#[rstest]
fn test_registry_update_url_nonexistent() {
    use auroraview_mcp::WebViewId;
    use std::str::FromStr;
    let reg = WebViewRegistry::new();
    let fake = WebViewId::from_str("no-such-id").unwrap();
    let updated = reg.update_url(&fake, "https://example.com");
    assert!(!updated);
}

#[rstest]
fn test_registry_multiple_webviews() {
    let reg = WebViewRegistry::new();
    let id1 = reg.register(&WebViewConfig::default());
    let id2 = reg.register(&WebViewConfig::default());
    let id3 = reg.register(&WebViewConfig::default());

    assert_eq!(reg.len(), 3);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
}

// --- Config tests ---

#[rstest]
fn test_server_config_default() {
    let cfg = McpServerConfig::default();
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 7890);
    assert_eq!(cfg.service_name, "auroraview-mcp");
    assert!(cfg.enable_mdns);
}

#[rstest]
fn test_server_config_custom() {
    let cfg = McpServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        service_name: "my-mcp".to_string(),
        enable_mdns: false,
    };
    assert_eq!(cfg.port, 8080);
    assert!(!cfg.enable_mdns);
}

// --- Runner tests ---

#[rstest]
fn test_runner_new() {
    let config = McpServerConfig {
        enable_mdns: false, // disable mDNS in tests
        ..Default::default()
    };
    let runner = McpRunner::new(config);
    assert_eq!(runner.config().port, 7890);
}

#[tokio::test]
async fn test_runner_start_stop() {
    let config = McpServerConfig {
        port: 17890,
        enable_mdns: false,
        ..Default::default()
    };
    let runner = McpRunner::new(config);
    assert!(!runner.is_running().await);

    runner.start().await.expect("start failed");
    assert!(runner.is_running().await);

    runner.stop().await;
    assert!(!runner.is_running().await);
}

#[tokio::test]
async fn test_runner_double_start_returns_error() {
    use auroraview_mcp::McpError;
    let config = McpServerConfig {
        port: 17891,
        enable_mdns: false,
        ..Default::default()
    };
    let runner = McpRunner::new(config);
    runner.start().await.expect("first start failed");

    let result = runner.start().await;
    assert!(matches!(result, Err(McpError::AlreadyRunning(17891))));

    runner.stop().await;
}

// --- WebViewId tests ---

#[rstest]
fn test_webview_id_new_unique() {
    use auroraview_mcp::WebViewId;
    let a = WebViewId::new();
    let b = WebViewId::new();
    assert_ne!(a, b);
}

#[rstest]
fn test_webview_id_from_str() {
    use auroraview_mcp::WebViewId;
    let id: WebViewId = "my-id".parse().unwrap();
    assert_eq!(id.to_string(), "my-id");
}

#[rstest]
fn test_webview_id_display() {
    use auroraview_mcp::WebViewId;
    let id: WebViewId = "test-123".parse().unwrap();
    assert_eq!(format!("{id}"), "test-123");
}

// --- WebViewConfig tests ---

#[rstest]
fn test_webview_config_default() {
    let cfg = WebViewConfig::default();
    assert_eq!(cfg.title, Some("AuroraView".to_string()));
    assert_eq!(cfg.width, Some(800));
    assert_eq!(cfg.height, Some(600));
    assert_eq!(cfg.visible, Some(true));
    assert_eq!(cfg.debug, Some(false));
}

// --- Server tool smoke tests ---

#[rstest]
fn test_server_has_registry() {
    use auroraview_mcp::AuroraViewMcpServer;
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    assert!(server.registry().is_empty());
}

#[rstest]
fn test_server_registry_operations() {
    use auroraview_mcp::AuroraViewMcpServer;
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    let config = WebViewConfig {
        title: Some("DCC Tool".to_string()),
        url: Some("https://dcc.tool".to_string()),
        ..Default::default()
    };
    let id = server.registry().register(&config);
    let list = server.registry().list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, id);
    assert_eq!(list[0].title, "DCC Tool");
}
