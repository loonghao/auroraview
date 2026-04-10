/// Integration tests for the real HTTP transport layer.
///
/// These tests bind to an ephemeral port, make actual HTTP requests
/// to the MCP server, and verify the SSE/JSON responses.
use auroraview_mcp::{AguiEvent, McpRunner, McpServerConfig};
use rstest::rstest;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find an available port by binding to :0 and returning the assigned port.
async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

/// Build a test config on the given port.
fn test_config(port: u16) -> McpServerConfig {
    McpServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_mdns: false,
        ..McpServerConfig::default()
    }
}

// ---------------------------------------------------------------------------
// McpRunner lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_runner_starts_and_stops() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));

    assert!(!runner.is_running().await);
    runner.start().await.expect("start failed");
    assert!(runner.is_running().await);
    runner.stop().await;
    // After stop the oneshot is consumed — is_running returns false
    assert!(!runner.is_running().await);
}

#[tokio::test]
async fn test_runner_start_twice_returns_error() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("first start failed");

    let result = runner.start().await;
    assert!(result.is_err(), "second start should return AlreadyRunning error");

    runner.stop().await;
}

#[tokio::test]
async fn test_runner_stop_when_not_running_is_noop() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    // Should not panic
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// HTTP connectivity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_mcp_endpoint_reachable() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");

    // Give tokio a moment to bind
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // MCP initialize call — should return 200 SSE
    let response = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#)
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        response.status(),
        200,
        "MCP initialize should return 200"
    );

    runner.stop().await;
}

#[tokio::test]
async fn test_mcp_list_tools_response() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // Initialize first to establish a session
    let init_resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(init_resp.status(), 200);

    // If a session ID was returned, use it to call tools/list
    if let Some(sid) = init_resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
    {
        let tools_resp = client
            .post(format!("http://127.0.0.1:{port}/mcp"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &sid)
            .body(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#)
            .send()
            .await
            .unwrap();

        assert_eq!(
            tools_resp.status(),
            200,
            "tools/list should return 200"
        );
        // The SSE body contains the JSON-RPC response — verify it's non-empty
        let body = tools_resp.text().await.unwrap();
        assert!(!body.is_empty(), "tools/list body should not be empty");
    }

    runner.stop().await;
}

#[tokio::test]
async fn test_agui_events_endpoint_reachable() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    // GET /agui/events should return 200 (SSE stream, will timeout reading but headers are 200)
    let response = client
        .get(format!("http://127.0.0.1:{port}/agui/events"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        response.status(),
        200,
        "AG-UI SSE endpoint should return 200"
    );
    let ct = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/event-stream"),
        "AG-UI endpoint content-type should be text/event-stream, got: {ct}"
    );

    runner.stop().await;
}

#[tokio::test]
async fn test_agui_events_with_run_id_filter() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    let response = client
        .get(format!("http://127.0.0.1:{port}/agui/events?run_id=my-run-123"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200, "Filtered AG-UI SSE should return 200");

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// emit_agui
// ---------------------------------------------------------------------------

#[rstest]
fn test_runner_emit_agui_no_panic() {
    // No tokio runtime required — emit to a bus with no subscribers
    let runner = McpRunner::new(McpServerConfig::default());
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "r1".to_string(),
        thread_id: "t1".to_string(),
    });
}

#[tokio::test]
async fn test_runner_emit_agui_received_by_subscriber() {
    let runner = McpRunner::new(McpServerConfig::default());
    let mut rx = runner.agui_bus().subscribe();

    let sent = AguiEvent::StepStarted {
        run_id: "run-xyz".to_string(),
        step_name: "screenshot".to_string(),
        step_id: "step-1".to_string(),
    };
    runner.emit_agui(sent.clone());

    let received = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout")
        .expect("channel error");

    assert_eq!(received.run_id(), "run-xyz");
}
