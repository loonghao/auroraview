//! Tests for `python_bindings` module — `PyMcpServer` and `PyMcpConfig`.
//!
//! PyMcpServer is designed for synchronous Python/DCC environments.
//! All tests use plain `#[test]` (not `#[tokio::test]`) to match real usage.
//! The server creates its own tokio Runtime internally, just as it would
//! when called from Python via PyO3.

use auroraview_mcp::{
    AguiEvent, McpServerConfig, PyMcpServer,
    python_bindings::PyMcpConfig,
};
use rstest::rstest;
use std::sync::{Arc, Barrier};
use std::thread;

// ---------------------------------------------------------------------------
// PyMcpConfig tests
// ---------------------------------------------------------------------------

#[test]
fn config_default_values() {
    let cfg = PyMcpConfig::default();
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 7890);
    assert_eq!(cfg.service_name, "auroraview-mcp");
    assert!(cfg.enable_mdns);
}

#[test]
fn config_custom_values() {
    let cfg = PyMcpConfig::new("0.0.0.0".into(), 9000, "my-mcp".into(), false);
    assert_eq!(cfg.host, "0.0.0.0");
    assert_eq!(cfg.port, 9000);
    assert_eq!(cfg.service_name, "my-mcp");
    assert!(!cfg.enable_mdns);
}

#[test]
fn config_into_mcp_server_config() {
    let py_cfg = PyMcpConfig::new("localhost".into(), 8080, "test-svc".into(), false);
    let cfg: McpServerConfig = py_cfg.into();
    assert_eq!(cfg.host, "localhost");
    assert_eq!(cfg.port, 8080);
    assert_eq!(cfg.service_name, "test-svc");
    assert!(!cfg.enable_mdns);
}

#[rstest]
#[case(7890)]
#[case(8080)]
#[case(9999)]
fn config_round_trip_port(#[case] port: u16) {
    let py_cfg = PyMcpConfig::new("127.0.0.1".into(), port, "svc".into(), false);
    let cfg: McpServerConfig = py_cfg.into();
    assert_eq!(cfg.port, port);
}

#[test]
fn config_enable_mdns_false() {
    let py_cfg = PyMcpConfig::new("127.0.0.1".into(), 7890, "svc".into(), false);
    let cfg: McpServerConfig = py_cfg.into();
    assert!(!cfg.enable_mdns);
}

// ---------------------------------------------------------------------------
// PyMcpServer construction tests
// ---------------------------------------------------------------------------

#[test]
fn server_new_with_port() {
    let server = PyMcpServer::new(find_free_port());
    assert!(!server.is_running());
}

#[test]
fn server_port_stored() {
    let port = find_free_port();
    let server = PyMcpServer::new(port);
    assert_eq!(server.port(), port);
}

#[test]
fn server_host_default() {
    let server = PyMcpServer::new(find_free_port());
    assert_eq!(server.host(), "127.0.0.1");
}

#[test]
fn server_from_config_host() {
    let config = McpServerConfig {
        host: "0.0.0.0".into(),
        port: find_free_port(),
        service_name: "av-mcp-test".into(),
        enable_mdns: false,
        max_webviews: None,
    };
    let server = PyMcpServer::from_config(config);
    assert_eq!(server.host(), "0.0.0.0");
}

#[test]
fn server_mcp_url_format() {
    let port = find_free_port();
    let server = PyMcpServer::new(port);
    assert_eq!(server.mcp_url(), format!("http://127.0.0.1:{port}/mcp"));
}

#[test]
fn server_agui_url_format() {
    let port = find_free_port();
    let server = PyMcpServer::new(port);
    assert_eq!(
        server.agui_url(),
        format!("http://127.0.0.1:{port}/agui/events")
    );
}

#[test]
fn server_not_running_initially() {
    let server = PyMcpServer::new(find_free_port());
    assert!(!server.is_running());
}

// ---------------------------------------------------------------------------
// PyMcpServer start/stop lifecycle
// ---------------------------------------------------------------------------

#[test]
fn server_start_stop() {
    let server = PyMcpServer::new(find_free_port());
    assert!(!server.is_running());
    server.start().expect("start should succeed");
    assert!(server.is_running());
    server.stop().expect("stop should succeed");
    assert!(!server.is_running());
}

#[test]
fn server_double_start_returns_error() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("first start ok");
    let result = server.start();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("already running"), "error was: {err}");
    server.stop().ok();
}

#[test]
fn server_stop_not_running_is_noop() {
    let server = PyMcpServer::new(find_free_port());
    server.stop().expect("stop when not running is noop");
}

#[test]
fn server_stop_twice_is_noop() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server.stop().expect("stop 1");
    server.stop().expect("stop 2 noop");
}

#[test]
fn server_restart_after_stop() {
    let port = find_free_port();
    let server = PyMcpServer::new(port);
    server.start().expect("start 1");
    server.stop().expect("stop 1");
    // Brief delay to let OS release port
    thread::sleep(std::time::Duration::from_millis(100));
    server.start().expect("start 2 after stop");
    server.stop().ok();
}

// ---------------------------------------------------------------------------
// emit_event before start — should error
// ---------------------------------------------------------------------------

#[test]
fn emit_before_start_run_started_errors() {
    let server = PyMcpServer::new(find_free_port());
    let result = server.emit_run_started("run-1", "thread-1");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not running"));
}

#[test]
fn emit_before_start_tool_call_errors() {
    let server = PyMcpServer::new(find_free_port());
    let result = server.emit_tool_call_start("run-1", "call-1", "screenshot");
    assert!(result.is_err());
}

#[test]
fn emit_before_start_custom_errors() {
    let server = PyMcpServer::new(find_free_port());
    let result = server.emit_custom("run-1", "ev", serde_json::json!({}));
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// emit_event after start — should succeed
// ---------------------------------------------------------------------------

#[test]
fn emit_run_started_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server
        .emit_run_started("run-42", "thread-42")
        .expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_run_finished_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server
        .emit_run_finished("run-1", "thread-1")
        .expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_tool_call_start_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server
        .emit_tool_call_start("run-1", "call-1", "screenshot")
        .expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_tool_call_end_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server
        .emit_tool_call_end("run-1", "call-1")
        .expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_custom_event_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    server
        .emit_custom("run-1", "my-event", serde_json::json!({"key": "value"}))
        .expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_arbitrary_agui_event_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    let event = AguiEvent::StateSnapshot {
        run_id: "run-99".into(),
        snapshot: serde_json::json!({"progress": 50}),
    };
    server.emit_event(event).expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_state_delta_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    let event = AguiEvent::StateDelta {
        run_id: "run-1".into(),
        delta: vec![serde_json::json!({"op": "replace", "path": "/key", "value": "v"})],
    };
    server.emit_event(event).expect("emit ok");
    server.stop().ok();
}

#[test]
fn emit_text_message_start_after_start() {
    let server = PyMcpServer::new(find_free_port());
    server.start().expect("start ok");
    let event = AguiEvent::TextMessageStart {
        run_id: "run-1".into(),
        message_id: "msg-1".into(),
        role: "assistant".into(),
    };
    server.emit_event(event).expect("emit ok");
    server.stop().ok();
}

// ---------------------------------------------------------------------------
// Drop impl — auto-stop on drop
// ---------------------------------------------------------------------------

#[test]
fn server_drop_stops_server() {
    let port = find_free_port();
    {
        let server = PyMcpServer::new(port);
        server.start().expect("start ok");
        assert!(server.is_running());
        // server is dropped here
    }
    thread::sleep(std::time::Duration::from_millis(150));
    // Should be able to bind the port again
    let server2 = PyMcpServer::new(port);
    server2.start().expect("second server on same port after drop");
    server2.stop().ok();
}

// ---------------------------------------------------------------------------
// Thread safety
// ---------------------------------------------------------------------------

#[test]
fn server_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PyMcpServer>();
}

#[test]
fn server_shared_across_threads() {
    let server = Arc::new(PyMcpServer::new(find_free_port()));
    let barrier = Arc::new(Barrier::new(3));

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let server = Arc::clone(&server);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let _ = server.is_running();
            })
        })
        .collect();

    barrier.wait();
    for h in handles {
        h.join().expect("thread ok");
    }
}

#[test]
fn concurrent_stop_calls_safe() {
    let server = Arc::new(PyMcpServer::new(find_free_port()));
    server.start().expect("start ok");

    let s1 = Arc::clone(&server);
    let s2 = Arc::clone(&server);

    let h1 = thread::spawn(move || s1.stop());
    let h2 = thread::spawn(move || s2.stop());

    h1.join().unwrap().ok();
    h2.join().unwrap().ok();
}

// ---------------------------------------------------------------------------
// Multiple servers on different ports
// ---------------------------------------------------------------------------

#[test]
fn multiple_servers_on_different_ports() {
    let s1 = PyMcpServer::new(find_free_port());
    let s2 = PyMcpServer::new(find_free_port());
    s1.start().expect("s1 start");
    s2.start().expect("s2 start");
    assert!(s1.is_running());
    assert!(s2.is_running());
    s1.stop().ok();
    s2.stop().ok();
}

// ---------------------------------------------------------------------------
// URL helpers with custom host
// ---------------------------------------------------------------------------

#[test]
fn custom_host_mcp_url() {
    let config = McpServerConfig {
        host: "192.168.1.100".into(),
        port: 7890,
        service_name: "av".into(),
        enable_mdns: false,
        max_webviews: None,
    };
    let server = PyMcpServer::from_config(config);
    assert_eq!(server.mcp_url(), "http://192.168.1.100:7890/mcp");
}

#[test]
fn custom_host_agui_url() {
    let config = McpServerConfig {
        host: "10.0.0.1".into(),
        port: 8080,
        service_name: "av".into(),
        enable_mdns: false,
        max_webviews: None,
    };
    let server = PyMcpServer::from_config(config);
    assert_eq!(server.agui_url(), "http://10.0.0.1:8080/agui/events");
}

// ---------------------------------------------------------------------------
// Config debug format
// ---------------------------------------------------------------------------

#[test]
fn config_debug_format_contains_port() {
    let cfg: McpServerConfig = PyMcpConfig::default().into();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("7890"));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn find_free_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.local_addr().expect("addr").port()
}
