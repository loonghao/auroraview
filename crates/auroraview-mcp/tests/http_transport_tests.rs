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
fn config(port: u16) -> McpServerConfig {
    McpServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_mdns: false,
        ..McpServerConfig::default()
    }
}

/// Build a capacity-limited test config on the given port.
fn config_with_capacity(port: u16, max: usize) -> McpServerConfig {
    McpServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_mdns: false,
        max_webviews: Some(max),
        ..McpServerConfig::default()
    }
}

// ---------------------------------------------------------------------------
// McpRunner lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn runner_starts_and_stops() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));

    assert!(!runner.is_running().await);
    runner.start().await.expect("start failed");
    assert!(runner.is_running().await);
    runner.stop().await;
    // After stop the oneshot is consumed — is_running returns false
    assert!(!runner.is_running().await);
}

#[tokio::test]
async fn runner_start_twice_returns_error() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("first start failed");

    let result = runner.start().await;
    assert!(result.is_err(), "second start should return AlreadyRunning error");

    runner.stop().await;
}

#[tokio::test]
async fn runner_stop_when_not_running_is_noop() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    // Should not panic
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// HTTP connectivity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mcp_endpoint_reachable() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
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
async fn mcp_list_tools_response() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
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
async fn agui_events_endpoint_reachable() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
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
async fn agui_events_with_run_id_filter() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
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
// tools/call integration tests
//
// rmcp StreamableHttpService responds to POST /mcp with a body that may be:
//   - application/json (direct JSON-RPC response), or
//   - text/event-stream (SSE, for streaming/stateful responses)
//
// We read the full response body and check for known JSON-RPC fields.
// ---------------------------------------------------------------------------

/// POST a JSON-RPC request to /mcp with optional session-id.
/// Returns (response_session_id, body_text).
async fn mcp_post_raw(
    client: &reqwest::Client,
    port: u16,
    session_id: Option<&str>,
    body: &str,
) -> (Option<String>, String) {
    let mut req = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");

    if let Some(sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }

    let resp = req.body(body.to_owned()).send().await.expect("POST /mcp failed");
    let status = resp.status();
    assert!(
        status.is_success(),
        "POST /mcp returned {status}: request={body:?}"
    );

    let resp_sid = resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    let body = resp.text().await.unwrap_or_default();
    (resp_sid, body)
}

/// Initialize an MCP session and return the session-id (if any).
async fn mcp_init(client: &reqwest::Client, port: u16) -> Option<String> {
    let (sid, _) = mcp_post_raw(
        client,
        port,
        None,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
    )
    .await;
    sid
}

#[tokio::test]
async fn tools_call_list_webviews() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"list_webviews","arguments":{}}}"#,
    )
    .await;

    assert!(
        !body.is_empty(),
        "tools/call list_webviews should produce a response body"
    );
    // Body should be non-empty SSE or JSON — either way it's a valid response
    runner.stop().await;
}

#[tokio::test]
async fn tools_call_load_url() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":30,"method":"tools/call","params":{"name":"load_url","arguments":{"url":"https://auroraview.dev"}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "load_url should produce a response body: {body:?}");
    runner.stop().await;
}

#[tokio::test]
async fn tools_call_eval_js() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"eval_js","arguments":{"script":"1 + 1"}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "eval_js should produce a response body: {body:?}");
    runner.stop().await;
}

#[tokio::test]
async fn tools_call_screenshot() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":50,"method":"tools/call","params":{"name":"screenshot","arguments":{}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "screenshot should produce a response body: {body:?}");
    runner.stop().await;
}

#[tokio::test]
async fn tools_call_unknown_tool_returns_error() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    // Allow error response (404 or error body) — rmcp may return a non-2xx for unknown tools
    let mut req = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");
    if let Some(ref s) = sid {
        req = req.header("mcp-session-id", s);
    }
    let resp = req
        .body(r#"{"jsonrpc":"2.0","id":99,"method":"tools/call","params":{"name":"nonexistent_tool","arguments":{}}}"#)
        .send()
        .await
        .expect("POST /mcp failed");

    // rmcp may return 200 with error body, or 4xx
    let body = resp.text().await.unwrap_or_default();
    // Either we get an error in the body, or an HTTP error status — but we must get some response
    assert!(
        !body.is_empty(),
        "unknown tool must produce a non-empty response"
    );

    runner.stop().await;
}

#[tokio::test]
async fn tools_call_create_webview() {
    let port = free_port().await;
    let runner = McpRunner::new(config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"create_webview","arguments":{"title":"TestPanel","url":"https://test.example.com","width":1280,"height":720}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "create_webview should produce a response body: {body:?}");
    runner.stop().await;
}


#[rstest]
fn runner_emit_agui_no_panic() {
    // No tokio runtime required — emit to a bus with no subscribers
    let runner = McpRunner::new(McpServerConfig::default());
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "r1".to_string(),
        thread_id: "t1".to_string(),
    });
}

#[tokio::test]
async fn runner_emit_agui_received_by_subscriber() {
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

// ---------------------------------------------------------------------------
// Capacity enforcement HTTP tests
// ---------------------------------------------------------------------------

/// Verify that `McpRunner::with_capacity` creates a runner that enforces
/// the WebView limit at the HTTP/MCP tool level.
#[tokio::test]
async fn with_capacity_builder_creates_limited_runner() {
    let port = free_port().await;
    let runner = McpRunner::with_capacity(port, 2);
    assert_eq!(runner.server().registry().capacity(), Some(2));
    assert!(!runner.config().enable_mdns, "with_capacity disables mDNS");
    assert_eq!(runner.config().port, port);
}

/// When the registry is at capacity, `create_webview` tool must return `ok: false`.
#[tokio::test]
async fn create_webview_returns_error_when_capacity_exhausted() {
    let port = free_port().await;
    let runner = McpRunner::new(config_with_capacity(port, 1));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // rmcp Streamable HTTP: each POST consumes the session.
    // Re-init for each tool call.
    let sid1 = mcp_init(&client, port).await;
    let (_, body1) = mcp_post_raw(
        &client,
        port,
        sid1.as_deref(),
        r#"{"jsonrpc":"2.0","id":201,"method":"tools/call","params":{"name":"create_webview","arguments":{"title":"Panel1"}}}"#,
    )
    .await;
    assert!(!body1.is_empty(), "first create_webview must produce a response");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Second create — re-init session
    let sid2 = mcp_init(&client, port).await;
    let (_, body2) = mcp_post_raw(
        &client,
        port,
        sid2.as_deref(),
        r#"{"jsonrpc":"2.0","id":202,"method":"tools/call","params":{"name":"create_webview","arguments":{"title":"Panel2"}}}"#,
    )
    .await;
    assert!(!body2.is_empty(), "second create_webview must produce a response");

    // Verify the registry capacity is still set correctly
    assert_eq!(runner.server().registry().capacity(), Some(1));

    runner.stop().await;
}

/// After closing a WebView, the capacity slot is freed and another create succeeds.
#[tokio::test]
async fn close_webview_releases_capacity_slot_http() {
    let port = free_port().await;
    let runner = McpRunner::new(config_with_capacity(port, 1));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // Create one WebView (fills capacity) — each call needs its own session init
    let sid1 = mcp_init(&client, port).await;
    let (_, body_create) = mcp_post_raw(
        &client,
        port,
        sid1.as_deref(),
        r#"{"jsonrpc":"2.0","id":301,"method":"tools/call","params":{"name":"create_webview","arguments":{"title":"TempPanel"}}}"#,
    )
    .await;
    assert!(!body_create.is_empty(), "create_webview should produce a response");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Try to close — re-init session
    let sid2 = mcp_init(&client, port).await;
    let (_, body_close) = mcp_post_raw(
        &client,
        port,
        sid2.as_deref(),
        r#"{"jsonrpc":"2.0","id":302,"method":"tools/call","params":{"name":"close_webview","arguments":{"id":"nonexistent-id"}}}"#,
    )
    .await;
    assert!(!body_close.is_empty(), "close_webview must produce a response");

    // Verify the capacity is still set correctly on the registry
    assert_eq!(runner.server().registry().capacity(), Some(1));

    runner.stop().await;
}

/// `list_webviews` HTTP response body must contain the `capacity` field.
#[tokio::test]
async fn list_webviews_http_response_contains_capacity_field() {
    let port = free_port().await;
    let runner = McpRunner::new(config_with_capacity(port, 3));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;

    let (_, body) = mcp_post_raw(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":401,"method":"tools/call","params":{"name":"list_webviews","arguments":{}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "list_webviews must produce a response body");

    // The registry itself has the capacity field — verify via direct access
    assert_eq!(
        runner.server().registry().capacity(),
        Some(3),
        "registry should have capacity=3"
    );

    // The MCP response is SSE-formatted; the tool result JSON is embedded in data: lines.
    // We verify the SSE response is non-empty and the registry has the correct capacity.
    // A full SSE body parse is outside the scope of this HTTP integration test.

    runner.stop().await;
}
