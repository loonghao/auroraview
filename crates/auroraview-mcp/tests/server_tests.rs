use auroraview_mcp::{
    AguiBus, AguiEvent, AuroraViewMcpServer, McpRunner, McpServerConfig, WebViewConfig,
    WebViewRegistry,
};
// WebViewInfo is used in serde round-trip tests via auroraview_mcp::WebViewInfo (inline import)
use rstest::rstest;

// --- Registry tests ---

#[rstest]
fn registry_register_and_list() {
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
fn registry_register_custom_config() {
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
fn registry_remove() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig::default());
    assert_eq!(reg.len(), 1);

    let removed = reg.remove(&id);
    assert!(removed.is_some());
    assert!(reg.is_empty());
}

#[rstest]
fn registry_remove_nonexistent() {
    let reg = WebViewRegistry::new();
    let fake = "nonexistent".parse::<auroraview_mcp::WebViewId>().unwrap();
    assert!(reg.remove(&fake).is_none());
}

#[rstest]
fn registry_update_url() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig::default());

    let updated = reg.update_url(&id, "https://auroraview.dev");
    assert!(updated);

    let info = reg.get(&id).unwrap();
    assert_eq!(info.url, "https://auroraview.dev");
}

#[rstest]
fn registry_update_url_nonexistent() {
    use auroraview_mcp::WebViewId;
    use std::str::FromStr;
    let reg = WebViewRegistry::new();
    let fake = WebViewId::from_str("no-such-id").unwrap();
    let updated = reg.update_url(&fake, "https://example.com");
    assert!(!updated);
}

#[rstest]
fn registry_multiple_webviews() {
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
fn server_config_default() {
    let cfg = McpServerConfig::default();
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 7890);
    assert_eq!(cfg.service_name, "auroraview-mcp");
    assert!(cfg.enable_mdns);
}

#[rstest]
fn server_config_custom() {
    let cfg = McpServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        service_name: "my-mcp".to_string(),
        enable_mdns: false,
        max_webviews: None,
    };
    assert_eq!(cfg.port, 8080);
    assert!(!cfg.enable_mdns);
}

// --- Runner tests ---

#[rstest]
fn runner_new() {
    let config = McpServerConfig {
        enable_mdns: false, // disable mDNS in tests
        ..Default::default()
    };
    let runner = McpRunner::new(config);
    assert_eq!(runner.config().port, 7890);
}

#[tokio::test]
async fn runner_start_stop() {
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
async fn runner_double_start_returns_error() {
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
fn webview_id_new_unique() {
    use auroraview_mcp::WebViewId;
    let a = WebViewId::new();
    let b = WebViewId::new();
    assert_ne!(a, b);
}

#[rstest]
fn webview_id_from_str() {
    use auroraview_mcp::WebViewId;
    let id: WebViewId = "my-id".parse().unwrap();
    assert_eq!(id.to_string(), "my-id");
}

#[rstest]
fn webview_id_display() {
    use auroraview_mcp::WebViewId;
    let id: WebViewId = "test-123".parse().unwrap();
    assert_eq!(format!("{id}"), "test-123");
}

// --- WebViewConfig tests ---

#[rstest]
fn webview_config_default() {
    let cfg = WebViewConfig::default();
    assert_eq!(cfg.title, Some("AuroraView".to_string()));
    assert_eq!(cfg.width, Some(800));
    assert_eq!(cfg.height, Some(600));
    assert_eq!(cfg.visible, Some(true));
    assert_eq!(cfg.debug, Some(false));
}

// --- Server tool smoke tests ---

#[rstest]
fn server_has_registry() {
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    assert!(server.registry().is_empty());
}

#[rstest]
fn server_registry_operations() {
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

// --- AguiBus integration with server ---

#[rstest]
fn server_without_agui_bus() {
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    assert!(server.agui_bus().is_none());
}

#[rstest]
fn server_with_agui_bus_some() {
    let bus = AguiBus::new();
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    })
    .with_agui_bus(bus);
    assert!(server.agui_bus().is_some());
}

#[rstest]
fn server_agui_bus_emit_received() {
    let bus = AguiBus::new();
    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    })
    .with_agui_bus(bus.clone());

    let mut rx = bus.subscribe();
    server.agui_bus().unwrap().emit(AguiEvent::RunStarted {
        run_id: "r1".to_string(),
        thread_id: "t1".to_string(),
    });
    let ev = rx.try_recv().expect("should receive event");
    assert_eq!(ev.run_id(), "r1");
}

#[rstest]
fn server_agui_bus_clone_shares_channel() {
    let bus1 = AguiBus::new();
    let bus2 = bus1.clone();

    let server = AuroraViewMcpServer::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    })
    .with_agui_bus(bus1);

    let mut rx = bus2.subscribe();
    server.agui_bus().unwrap().emit(AguiEvent::RunFinished {
        run_id: "done".to_string(),
        thread_id: "t".to_string(),
    });
    let ev = rx.try_recv().expect("event not received");
    assert_eq!(ev.run_id(), "done");
}

// --- WebViewInfo fields ---

#[rstest]
fn webview_info_default_visible() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig {
        visible: Some(true),
        ..Default::default()
    });
    let info = reg.get(&id).unwrap();
    assert!(info.visible);
}

#[rstest]
fn webview_info_invisible() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig {
        visible: Some(false),
        ..Default::default()
    });
    let info = reg.get(&id).unwrap();
    assert!(!info.visible);
}

#[rstest]
fn webview_info_hwnd_default_zero() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig::default());
    let info = reg.get(&id).unwrap();
    assert_eq!(info.hwnd, 0);
}

#[rstest]
fn webview_info_url_empty_by_default() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig {
        url: None,
        ..Default::default()
    });
    let info = reg.get(&id).unwrap();
    assert_eq!(info.url, "");
}

#[rstest]
fn webview_info_custom_dimensions() {
    let reg = WebViewRegistry::new();
    let id = reg.register(&WebViewConfig {
        width: Some(1920),
        height: Some(1080),
        ..Default::default()
    });
    let info = reg.get(&id).unwrap();
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
}

// --- WebViewInfo serde round-trip ---

#[rstest]
fn webview_info_serde_round_trip() {
    use auroraview_mcp::{WebViewId, WebViewInfo};
    let info = WebViewInfo {
        id: WebViewId("abc-123".to_string()),
        title: "DCC Panel".to_string(),
        url: "https://maya.tool/panel".to_string(),
        visible: true,
        width: 1920,
        height: 1080,
        hwnd: 99999,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    let back: WebViewInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.id.0, "abc-123");
    assert_eq!(back.title, "DCC Panel");
    assert_eq!(back.url, "https://maya.tool/panel");
    assert!(back.visible);
    assert_eq!(back.width, 1920);
    assert_eq!(back.height, 1080);
    assert_eq!(back.hwnd, 99999);
}

#[rstest]
fn webview_info_json_field_names() {
    use auroraview_mcp::{WebViewId, WebViewInfo};
    let info = WebViewInfo {
        id: WebViewId("field-test".to_string()),
        title: "T".to_string(),
        url: "http://x".to_string(),
        visible: false,
        width: 100,
        height: 200,
        hwnd: 0,
    };
    let v: serde_json::Value = serde_json::to_value(&info).unwrap();
    assert!(v.get("id").is_some(), "field 'id' missing");
    assert!(v.get("title").is_some(), "field 'title' missing");
    assert!(v.get("url").is_some(), "field 'url' missing");
    assert!(v.get("visible").is_some(), "field 'visible' missing");
    assert!(v.get("width").is_some(), "field 'width' missing");
    assert!(v.get("height").is_some(), "field 'height' missing");
    assert!(v.get("hwnd").is_some(), "field 'hwnd' missing");
}

#[rstest]
fn webview_info_deserialize_from_literal() {
    use auroraview_mcp::WebViewInfo;
    let json = r#"{"id":"lit-id","title":"Lit","url":"file:///x.html","visible":true,"width":800,"height":600,"hwnd":0}"#;
    let info: WebViewInfo = serde_json::from_str(json).expect("deserialize");
    assert_eq!(info.id.0, "lit-id");
    assert_eq!(info.url, "file:///x.html");
}

#[rstest]
fn webview_config_serde_round_trip() {
    use auroraview_mcp::WebViewConfig;
    let cfg = WebViewConfig {
        title: Some("Round Trip".to_string()),
        url: Some("https://rt.test".to_string()),
        html: None,
        width: Some(1280),
        height: Some(720),
        visible: Some(false),
        debug: Some(true),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: WebViewConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, Some("Round Trip".to_string()));
    assert_eq!(back.width, Some(1280));
    assert_eq!(back.debug, Some(true));
    assert!(back.html.is_none());
}

// --- McpServerConfig validation ---

#[rstest]
fn config_validate_default_is_ok() {
    let cfg = McpServerConfig::default();
    assert!(cfg.validate().is_ok());
    assert!(cfg.is_valid());
}

#[rstest]
fn config_validate_port_zero_is_err() {
    let cfg = McpServerConfig {
        port: 0,
        ..McpServerConfig::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("port"), "error should mention port: {err}");
}

#[rstest]
fn config_validate_empty_host_is_err() {
    let cfg = McpServerConfig {
        host: "".to_string(),
        ..McpServerConfig::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("host"), "error should mention host: {err}");
}

#[rstest]
fn config_validate_whitespace_host_is_err() {
    let cfg = McpServerConfig {
        host: "   ".to_string(),
        ..McpServerConfig::default()
    };
    assert!(cfg.validate().is_err());
}

#[rstest]
fn config_validate_empty_service_name_is_err() {
    let cfg = McpServerConfig {
        service_name: "".to_string(),
        ..McpServerConfig::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("service_name"), "error should mention service_name: {err}");
}

#[rstest]
fn config_validate_port_65535_is_ok() {
    let cfg = McpServerConfig {
        port: 65535,
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    assert!(cfg.validate().is_ok());
}

#[rstest]
fn config_validate_port_1_is_ok() {
    let cfg = McpServerConfig {
        port: 1,
        enable_mdns: false,
        ..McpServerConfig::default()
    };
    assert!(cfg.validate().is_ok());
}

#[rstest]
fn config_is_valid_returns_false_for_bad_config() {
    let cfg = McpServerConfig {
        port: 0,
        ..McpServerConfig::default()
    };
    assert!(!cfg.is_valid());
}

// --- McpRunner server() accessor ---

#[tokio::test]
async fn runner_server_registry_accessible() {
    let runner = McpRunner::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    let server = runner.server();
    assert!(server.registry().is_empty());
    assert_eq!(server.config().port, 7890);
}

#[tokio::test]
async fn runner_server_has_agui_bus() {
    let runner = McpRunner::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    // McpRunner wires AguiBus into server
    assert!(runner.server().agui_bus().is_some());
}

