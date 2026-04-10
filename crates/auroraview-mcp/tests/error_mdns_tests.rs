/// Tests for McpError variants and mDNS broadcaster.
use auroraview_mcp::{McpError, McpServerConfig, McpRunner};
use rstest::rstest;

// ---------------------------------------------------------------------------
// McpError display / variant tests
// ---------------------------------------------------------------------------

#[rstest]
fn error_webview_not_found_display() {
    let e = McpError::WebViewNotFound("wv-123".to_string());
    assert_eq!(e.to_string(), "WebView not found: wv-123");
}

#[rstest]
fn error_tool_execution_display() {
    let e = McpError::ToolExecution("eval failed".to_string());
    assert_eq!(e.to_string(), "Tool execution failed: eval failed");
}

#[rstest]
fn error_server_not_running_display() {
    let e = McpError::ServerNotRunning;
    assert_eq!(e.to_string(), "Server not running");
}

#[rstest]
fn error_already_running_display() {
    let e = McpError::AlreadyRunning(7890);
    assert_eq!(e.to_string(), "Server already running on port 7890");
}

#[rstest]
fn error_mdns_broadcast_display() {
    let e = McpError::MdnsBroadcast("socket bind failed".to_string());
    assert_eq!(e.to_string(), "mDNS broadcast error: socket bind failed");
}

#[rstest]
fn error_serialization_from_serde() {
    let bad_json = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let e: McpError = bad_json.into();
    let s = e.to_string();
    assert!(s.starts_with("Serialization error:"), "got: {s}");
}

#[rstest]
fn error_io_from_std_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let e: McpError = io_err.into();
    let s = e.to_string();
    assert!(s.contains("IO error:"), "got: {s}");
}

#[rstest]
fn error_other_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("something unexpected");
    let e: McpError = anyhow_err.into();
    let s = e.to_string();
    assert!(s.contains("Anyhow error:"), "got: {s}");
}

#[rstest]
fn error_already_running_preserves_port() {
    let e = McpError::AlreadyRunning(9999);
    if let McpError::AlreadyRunning(port) = e {
        assert_eq!(port, 9999);
    } else {
        panic!("Expected AlreadyRunning");
    }
}

#[rstest]
fn error_webview_not_found_preserves_id() {
    let e = McpError::WebViewNotFound("abc-xyz".to_string());
    if let McpError::WebViewNotFound(id) = e {
        assert_eq!(id, "abc-xyz");
    } else {
        panic!("Expected WebViewNotFound");
    }
}

// ---------------------------------------------------------------------------
// McpServerConfig: builder-like usage and edge cases
// ---------------------------------------------------------------------------

#[rstest]
fn config_default_values() {
    let c = McpServerConfig::default();
    assert_eq!(c.host, "127.0.0.1");
    assert_eq!(c.port, 7890);
    assert_eq!(c.service_name, "auroraview-mcp");
    assert!(c.enable_mdns);
}

#[rstest]
fn config_clone_is_independent() {
    let c1 = McpServerConfig::default();
    let mut c2 = c1.clone();
    c2.port = 9999;
    assert_eq!(c1.port, 7890);
    assert_eq!(c2.port, 9999);
}

#[rstest]
fn config_custom_host_port() {
    let c = McpServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        service_name: "test-svc".to_string(),
        enable_mdns: false,
    };
    assert_eq!(c.host, "0.0.0.0");
    assert_eq!(c.port, 8080);
    assert_eq!(c.service_name, "test-svc");
    assert!(!c.enable_mdns);
}

#[rstest]
fn config_serialize_deserialize() {
    let c = McpServerConfig {
        host: "192.168.1.1".to_string(),
        port: 1234,
        service_name: "my-mcp".to_string(),
        enable_mdns: true,
    };
    let json = serde_json::to_string(&c).unwrap();
    let restored: McpServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.host, c.host);
    assert_eq!(restored.port, c.port);
    assert_eq!(restored.service_name, c.service_name);
    assert_eq!(restored.enable_mdns, c.enable_mdns);
}

// ---------------------------------------------------------------------------
// McpRunner: mdns disabled lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_runner_mdns_disabled_starts_cleanly() {
    let port = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    let runner = McpRunner::new(McpServerConfig {
        port,
        enable_mdns: false,
        ..Default::default()
    });
    runner.start().await.expect("start should succeed without mDNS");
    assert!(runner.is_running().await);
    runner.stop().await;
    assert!(!runner.is_running().await);
}

#[tokio::test]
async fn test_runner_stop_twice_no_panic() {
    let port = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    let runner = McpRunner::new(McpServerConfig {
        port,
        enable_mdns: false,
        ..Default::default()
    });
    runner.start().await.unwrap();
    runner.stop().await;
    // Second stop should be a no-op
    runner.stop().await;
}

#[tokio::test]
async fn test_runner_restart_after_stop() {
    let port = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    // Use separate runners to avoid port conflict after first stop
    let runner1 = McpRunner::new(McpServerConfig {
        port,
        enable_mdns: false,
        ..Default::default()
    });
    runner1.start().await.unwrap();
    runner1.stop().await;

    // Small delay to let the OS release the port
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let runner2 = McpRunner::new(McpServerConfig {
        port,
        enable_mdns: false,
        ..Default::default()
    });
    runner2.start().await.expect("restart on same port should succeed");
    runner2.stop().await;
}

#[tokio::test]
async fn test_runner_config_accessed_after_start() {
    let port = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    let runner = McpRunner::new(McpServerConfig {
        port,
        service_name: "config-test".to_string(),
        enable_mdns: false,
        ..Default::default()
    });
    runner.start().await.unwrap();
    assert_eq!(runner.config().service_name, "config-test");
    assert_eq!(runner.config().port, port);
    runner.stop().await;
}

#[tokio::test]
async fn test_runner_server_accessor() {
    let runner = McpRunner::new(McpServerConfig {
        enable_mdns: false,
        ..Default::default()
    });
    // server() returns reference without needing the server to be started
    let server = runner.server();
    assert!(server.registry().is_empty());
    assert_eq!(server.config().port, 7890);
}

// ---------------------------------------------------------------------------
// McpError: Debug formatting
// ---------------------------------------------------------------------------

#[rstest]
fn error_debug_format_not_empty() {
    let variants: Vec<Box<dyn std::fmt::Debug>> = vec![
        Box::new(McpError::WebViewNotFound("x".to_string())),
        Box::new(McpError::ToolExecution("y".to_string())),
        Box::new(McpError::ServerNotRunning),
        Box::new(McpError::AlreadyRunning(1)),
        Box::new(McpError::MdnsBroadcast("z".to_string())),
    ];
    for e in &variants {
        assert!(!format!("{:?}", e).is_empty());
    }
}

// ---------------------------------------------------------------------------
// Result<T> type alias
// ---------------------------------------------------------------------------

#[rstest]
fn result_ok_value() {
    let r: auroraview_mcp::Result<u32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[rstest]
fn result_err_value() {
    let r: auroraview_mcp::Result<u32> = Err(McpError::ServerNotRunning);
    assert!(r.is_err());
}
