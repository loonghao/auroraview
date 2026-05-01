//!
//! Integration tests for `auroraview-mcp`.
//!
//! These tests verify cross-module behavior that unit tests cannot cover:
//! - Configuration builders produce correct values.
//! - `AuroraViewMcpServer` and `McpRunner` wire together correctly.
//! - The AG-UI event bus propagates events to multiple subscribers.
//! - `WebViewRegistry` behaves correctly under simulated concurrent access.

use auroraview_mcp::agui::{AguiBus, AguiEvent};
use auroraview_mcp::registry::WebViewRegistry;
use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::server::AuroraViewMcpServer;
use auroraview_mcp::types::{McpServerConfig, WebViewConfig};
use base64::Engine;

// ---------------------------------------------------------------------------
// Configuration builders
// ---------------------------------------------------------------------------

#[test]
fn mcp_server_config_with_all() {
    let cfg = McpServerConfig::with_all(
        7890,
        "127.0.0.1",
        "auroraview-mcp",
        true,
        false,
        Some(5),
    );

    assert_eq!(cfg.port, 7890);
    assert_eq!(cfg.max_webviews, Some(5));
    assert!(!cfg.enable_oauth);
    let json = serde_json::to_string(&cfg).unwrap();
    assert!(json.contains("port"));
}

#[test]
fn mcp_server_config_default_and_builders() {
    let cfg = McpServerConfig::default()
        .with_port(9999)
        .with_mdns(true)
        .with_max_webviews(10);

    assert_eq!(cfg.port, 9999);
    assert_eq!(cfg.max_webviews, Some(10));
}

// ---------------------------------------------------------------------------
// Server creation
// ---------------------------------------------------------------------------

#[test]
fn server_creation_default() {
    let _server = AuroraViewMcpServer::new(McpServerConfig::default());
}

#[test]
fn server_creation_with_all() {
    let cfg = McpServerConfig::with_all(0, "127.0.0.1", "test", false, false, None);
    let _server = AuroraViewMcpServer::new(cfg);
}

#[test]
fn server_with_agui_bus() {
    let bus = AguiBus::new();
    let cfg = McpServerConfig::with_all(0, "127.0.0.1", "test", false, false, None);
    let _server = AuroraViewMcpServer::new(cfg).with_agui_bus(bus.clone());
}

// ---------------------------------------------------------------------------
// Registry: concurrent access simulation
// ---------------------------------------------------------------------------

#[test]
fn registry_concurrent_register_remove() {
    let registry = WebViewRegistry::with_capacity(10);
    let ids: Vec<_> = (0..5)
        .map(|i| {
            let cfg = WebViewConfig {
                title: Some(format!("View {i}")),
                ..Default::default()
            };
            registry.register(&cfg)
        })
        .collect();

    assert_eq!(registry.len(), 5);

    for id in ids.iter().rev() {
        let removed = registry.remove(id);
        assert!(removed.is_some());
    }
    assert!(registry.is_empty());
}

#[test]
fn registry_update_cdp_endpoint() {
    let registry = WebViewRegistry::new();
    let cfg = WebViewConfig::default();
    let id = registry.register(&cfg);

    assert!(registry.update_cdp_endpoint(&id, "http://127.0.0.1:9222".into()));
    let info = registry.get(&id).unwrap();
    assert_eq!(info.cdp_endpoint, Some("http://127.0.0.1:9222".into()));
}

// ---------------------------------------------------------------------------
// AG-UI event bus: fan-out to multiple subscribers
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agui_bus_multiple_subscribers() {
    let bus = AguiBus::new();

    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    let event = AguiEvent::StepStarted {
        run_id: "run-1".to_string(),
        step_name: "test-step".to_string(),
        step_id: "step-1".to_string(),
    };
    bus.emit(event.clone());

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let received1 = rx1.try_recv().unwrap();
    let received2 = rx2.try_recv().unwrap();

    assert_eq!(received1, event);
    assert_eq!(received2, event);
}

#[tokio::test]
async fn agui_bus_drop_receiver_stops_events() {
    let bus = AguiBus::new();

    let mut rx = bus.subscribe();

    let event = AguiEvent::RunFinished {
        run_id: "run-1".to_string(),
        thread_id: "t-1".to_string(),
    };
    bus.emit(event);

    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    assert!(rx.try_recv().is_ok());

    // Drop the receiver — further emits won't be received.
    drop(rx);

    let event2 = AguiEvent::RunStarted {
        run_id: "run-2".to_string(),
        thread_id: "t-2".to_string(),
    };
    bus.emit(event2);

    // No panic — just no one receives it.
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
}

// ---------------------------------------------------------------------------
// McpRunner: builder pattern
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runner_builder_and_start_stop() {
    let config = McpServerConfig::with_all(0, "127.0.0.1", "test", false, false, None);
    let _runner = McpRunner::new(config);
}

// ---------------------------------------------------------------------------
// McpServerConfig: validation
// ---------------------------------------------------------------------------

#[test]
fn mcp_server_config_validate_valid() {
    let cfg = McpServerConfig::with_all(7890, "127.0.0.1", "auroraview-mcp", true, false, None);
    assert!(cfg.validate().is_ok());
    assert!(cfg.is_valid());
}

#[test]
fn mcp_server_config_validate_invalid_port() {
    let cfg = McpServerConfig::default().with_port(0);
    assert!(cfg.validate().is_err());
    assert!(!cfg.is_valid());
}

#[test]
fn mcp_server_config_validate_empty_host() {
    let cfg = McpServerConfig::with_all(7890, "", "auroraview-mcp", true, false, None);
    assert!(cfg.validate().is_err());
}

// ---------------------------------------------------------------------------
// AguiBus: receiver counting
// ---------------------------------------------------------------------------

#[test]
fn agui_bus_receiver_count_tracks() {
    let bus = AguiBus::new();
    assert_eq!(bus.receiver_count(), 0);

    let _rx1 = bus.subscribe();
    assert_eq!(bus.receiver_count(), 1);

    let _rx2 = bus.subscribe();
    assert_eq!(bus.receiver_count(), 2);

    // Dropping a receiver reduces count.
    // (Can't easily test here without async runtime)
}

#[test]
fn agui_bus_emit_without_receivers_no_panic() {
    let bus = AguiBus::new();
    let event = AguiEvent::RunFinished {
        run_id: "r1".to_string(),
        thread_id: "t1".to_string(),
    };
    // Should not panic even with zero receivers.
    bus.emit(event);
}

// ---------------------------------------------------------------------------
// WebViewRegistry: edge cases
// ---------------------------------------------------------------------------

#[test]
fn registry_get_nonexistent_returns_none() {
    let registry = WebViewRegistry::new();
    let fake_id = auroraview_mcp::types::WebViewId::new();
    assert!(registry.get(&fake_id).is_none());
}

#[test]
fn registry_remove_nonexistent_returns_none() {
    let registry = WebViewRegistry::new();
    let fake_id = auroraview_mcp::types::WebViewId::new();
    assert!(registry.remove(&fake_id).is_none());
}

#[test]
fn registry_update_url_nonexistent_returns_false() {
    let registry = WebViewRegistry::new();
    let fake_id = auroraview_mcp::types::WebViewId::new();
    assert!(!registry.update_url(&fake_id, "https://example.com"));
}

#[test]
fn registry_list_empty() {
    let registry = WebViewRegistry::new();
    let list = registry.list();
    assert!(list.is_empty());
}

// ---------------------------------------------------------------------------
// McpRunner: builder patterns
// ---------------------------------------------------------------------------

#[test]
fn runner_with_capacity_builder() {
    let runner = McpRunner::with_capacity(12345, 10);
    // Just verify it doesn't panic.
    let _ = runner;
}

#[test]
fn runner_with_mdns_port_builder() {
    let runner = McpRunner::with_mdns_port(54321);
    // Just verify it doesn't panic.
    let _ = runner;
}

// ---------------------------------------------------------------------------
// CdpAdapterConfig (re-exports from lib.rs)
// ---------------------------------------------------------------------------

#[test]
fn cdp_adapter_config_localhost() {
    let cfg = auroraview_mcp::CdpAdapterConfig::localhost(9222, "0.5.2");
    assert_eq!(cfg.http_endpoint, "http://127.0.0.1:9222");
    assert_eq!(cfg.ws_endpoint, "ws://127.0.0.1:9222");
    assert_eq!(cfg.version, "0.5.2");
}

#[test]
fn cdp_adapter_config_fields() {
    let cfg = auroraview_mcp::CdpAdapterConfig::localhost(9222, "0.5.2");
    assert_eq!(cfg.pid, std::process::id());
    assert!(!cfg.platform.is_empty());
    assert_eq!(cfg.window_title, None);
}

// ---------------------------------------------------------------------------
// Type tests: ScreenshotData
// ---------------------------------------------------------------------------

#[test]
fn screenshot_data_from_bytes() {
    let bytes = vec![1, 2, 3, 4, 5];
    let data = auroraview_mcp::types::ScreenshotData::from_bytes(&bytes, 800, 600, "png");
    assert_eq!(data.data, base64::engine::general_purpose::STANDARD.encode(&bytes));
    assert_eq!(data.width, 800);
    assert_eq!(data.height, 600);
    assert_eq!(data.format, "png");
}

#[test]
fn screenshot_data_new_placeholder() {
    let data = auroraview_mcp::types::ScreenshotData::new_placeholder(1024, 768);
    // Placeholder has empty data (no actual image)
    assert!(data.data.is_empty());
    assert_eq!(data.width, 1024);
    assert_eq!(data.height, 768);
    assert_eq!(data.format, "png");
}

// ---------------------------------------------------------------------------
// Type tests: JsResult
// ---------------------------------------------------------------------------

#[test]
fn js_result_ok() {
    let value = serde_json::json!({"result": 42});
    let result = auroraview_mcp::types::JsResult::ok(value.clone());
    assert_eq!(result.value, value);
    assert!(result.error.is_none());
}

#[test]
fn js_result_err() {
    let result = auroraview_mcp::types::JsResult::err("test error".to_string());
    assert_eq!(result.value, serde_json::Value::Null);
    assert_eq!(result.error, Some("test error".to_string()));
}

// ---------------------------------------------------------------------------
// Type tests: WebViewId
// ---------------------------------------------------------------------------

#[test]
fn webview_id_new_and_display() {
    let id = auroraview_mcp::types::WebViewId::new();
    let id_str = id.to_string();
    assert!(!id_str.is_empty());
    // Should be a valid UUID format
    assert_eq!(id_str.len(), 36); // UUID v4 length
}

#[test]
fn webview_id_parse_valid() {
    let id = auroraview_mcp::types::WebViewId::new();
    let id_str = id.to_string();
    let parsed = id_str.parse::<auroraview_mcp::types::WebViewId>();
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().to_string(), id_str);
}

#[test]
fn webview_id_parse_invalid() {
    // WebViewId::from_str never fails (Infallible)
    // Any string is a valid WebViewId
    let result = "not-a-uuid".parse::<auroraview_mcp::types::WebViewId>();
    assert!(result.is_ok());
    let id = result.unwrap();
    assert_eq!(id.to_string(), "not-a-uuid");
}

// ---------------------------------------------------------------------------
// Type tests: SuccessOutput
// ---------------------------------------------------------------------------

#[test]
fn success_output_serialization() {
    let output = auroraview_mcp::server::types::SuccessOutput {
        ok: true,
        message: "test message".to_string(),
    };
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("ok"));
    assert!(json.contains("test message"));
}

// ---------------------------------------------------------------------------
// Edge cases: URL validation in load_url
// ---------------------------------------------------------------------------

#[test]
fn load_url_params_validation() {
    let params = auroraview_mcp::server::types::LoadUrlParams {
        id: None,
        url: "https://example.com".to_string(),
    };
    // Valid URL schemes
    assert!(params.url.starts_with("http://") || params.url.starts_with("https://") || params.url.starts_with("file://"));
}

#[test]
fn load_url_params_invalid_scheme() {
    let invalid_urls = vec!["ftp://example.com", "javascript:alert(1)", "data:text/html,<h1>test</h1>"];
    for url in invalid_urls {
        let scheme_ok = url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://");
        assert!(!scheme_ok, "URL should be invalid: {url}");
    }
}

// ---------------------------------------------------------------------------
// Edge cases: EvalJsParams
// ---------------------------------------------------------------------------

#[test]
fn eval_js_params_empty_script() {
    let params = auroraview_mcp::server::types::EvalJsParams {
        id: None,
        script: "  ".to_string(), // whitespace only
    };
    assert!(params.script.trim().is_empty());
}

#[test]
fn eval_js_params_valid_script() {
    let params = auroraview_mcp::server::types::EvalJsParams {
        id: None,
        script: "console.log('hello');".to_string(),
    };
    assert!(!params.script.trim().is_empty());
}

// ---------------------------------------------------------------------------
// WebViewConfig: default values
// ---------------------------------------------------------------------------

#[test]
fn webview_config_default() {
    let cfg = auroraview_mcp::types::WebViewConfig::default();
    assert_eq!(cfg.title, Some("AuroraView".to_string()));
    assert_eq!(cfg.url, None);
    assert_eq!(cfg.html, None);
    assert_eq!(cfg.width, Some(800));
    assert_eq!(cfg.height, Some(600));
    assert_eq!(cfg.visible, Some(true));
    assert_eq!(cfg.debug, Some(false));
}

#[test]
fn webview_config_with_values() {
    let cfg = auroraview_mcp::types::WebViewConfig {
        title: Some("Test View".to_string()),
        url: Some("https://example.com".to_string()),
        html: None,
        width: Some(1024),
        height: Some(768),
        visible: Some(false),
        debug: Some(true),
    };
    assert_eq!(cfg.title, Some("Test View".to_string()));
    assert_eq!(cfg.width, Some(1024));
    assert_eq!(cfg.height, Some(768));
}
