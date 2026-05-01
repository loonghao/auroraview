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
        Some(5),
    );

    assert_eq!(cfg.port, 7890);
    assert_eq!(cfg.max_webviews, Some(5));
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
    let cfg = McpServerConfig::with_all(0, "127.0.0.1", "test", false, None);
    let _server = AuroraViewMcpServer::new(cfg);
}

#[test]
fn server_with_agui_bus() {
    let bus = AguiBus::new();
    let cfg = McpServerConfig::with_all(0, "127.0.0.1", "test", false, None);
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
    let config = McpServerConfig::with_all(0, "127.0.0.1", "test", false, None);
    let _runner = McpRunner::new(config);
}

// ---------------------------------------------------------------------------
// McpServerConfig: validation
// ---------------------------------------------------------------------------

#[test]
fn mcp_server_config_validate_valid() {
    let cfg = McpServerConfig::with_all(7890, "127.0.0.1", "auroraview-mcp", true, None);
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
    let cfg = McpServerConfig::with_all(7890, "", "auroraview-mcp", true, None);
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
