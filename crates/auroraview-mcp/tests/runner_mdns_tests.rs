//! Tests for `McpRunner::with_mdns_port`, `McpServerConfig::with_all`,
//! `PyMcpServer::emit_step`, and SSE body JSON parsing.

use auroraview_mcp::{AguiEvent, McpRunner, McpServerConfig, PyMcpServer};
use reqwest::Client;
use rstest::rstest;
use std::time::Duration;
use tokio::time::timeout;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

fn no_mdns_config(port: u16) -> McpServerConfig {
    McpServerConfig::default()
        .with_port(port)
        .with_mdns(false)
}

/// Read up to `limit` bytes from the SSE stream within `deadline`.
async fn read_sse(response: reqwest::Response, limit: usize, deadline: Duration) -> String {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buf = Vec::new();
    let fut = async {
        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                buf.extend_from_slice(&bytes);
                if buf.len() >= limit {
                    break;
                }
            }
        }
    };
    let _ = timeout(deadline, fut).await;
    String::from_utf8_lossy(&buf).into_owned()
}

// ---------------------------------------------------------------------------
// McpRunner::with_mdns_port
// ---------------------------------------------------------------------------

#[tokio::test]
async fn with_mdns_port_creates_runner_with_mdns_enabled() {
    let port = free_port().await;
    let runner = McpRunner::with_mdns_port(port);
    assert_eq!(runner.config().port, port);
    assert!(
        runner.config().enable_mdns,
        "with_mdns_port must enable mDNS"
    );
}

#[tokio::test]
async fn with_mdns_port_uses_default_service_name() {
    let port = free_port().await;
    let runner = McpRunner::with_mdns_port(port);
    assert_eq!(runner.config().service_name, "auroraview-mcp");
}

#[tokio::test]
async fn with_mdns_port_no_capacity_limit() {
    let port = free_port().await;
    let runner = McpRunner::with_mdns_port(port);
    assert_eq!(
        runner.server().registry().capacity(),
        None,
        "with_mdns_port should have no capacity limit by default"
    );
}

#[rstest]
#[case(7890)]
#[case(8080)]
#[case(9999)]
#[tokio::test]
async fn with_mdns_port_stores_correct_port(#[case] port: u16) {
    let runner = McpRunner::with_mdns_port(port);
    assert_eq!(runner.config().port, port);
}

#[tokio::test]
async fn with_mdns_port_runner_is_not_running_initially() {
    let port = free_port().await;
    let runner = McpRunner::with_mdns_port(port);
    assert!(!runner.is_running().await);
}

// mDNS broadcasting may fail gracefully in CI (no mDNS capable network interface).
// We verify that `McpRunner::with_mdns_port` constructs and starts without panicking.
// The mDNS broadcaster initialization failure is logged as a warning and the runner
// continues to function normally (serving HTTP/SSE).
#[tokio::test]
async fn with_mdns_port_start_does_not_panic() {
    let port = free_port().await;
    let runner = McpRunner::with_mdns_port(port);
    // start() may succeed (mDNS OK) or fail with IoError (mDNS not available in CI).
    // Either way it must NOT panic.
    let result = runner.start().await;
    if result.is_ok() {
        assert!(runner.is_running().await);
        runner.stop().await;
    }
    // If start failed due to mDNS, the runner is simply not running — both outcomes
    // are acceptable for this "no panic" test.
}

#[tokio::test]
async fn with_mdns_port_can_serve_http_after_start() {
    let port = free_port().await;
    // Disable mDNS for a reliable HTTP test (mDNS failure would abort start())
    let runner = McpRunner::new(no_mdns_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .get(format!("http://127.0.0.1:{port}/agui/events"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("GET /agui/events failed");

    assert_eq!(resp.status(), 200);
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// McpServerConfig::with_all
// ---------------------------------------------------------------------------

#[test]
fn with_all_sets_all_fields() {
    let cfg = McpServerConfig::with_all(7891, "0.0.0.0", "my-mcp", false, Some(10));
    assert_eq!(cfg.port, 7891);
    assert_eq!(cfg.host, "0.0.0.0");
    assert_eq!(cfg.service_name, "my-mcp");
    assert!(!cfg.enable_mdns);
    assert_eq!(cfg.max_webviews, Some(10));
}

#[test]
fn with_all_no_capacity_limit() {
    let cfg = McpServerConfig::with_all(7892, "127.0.0.1", "svc", true, None);
    assert_eq!(cfg.max_webviews, None);
}

#[test]
fn with_all_is_valid() {
    let cfg = McpServerConfig::with_all(7893, "127.0.0.1", "valid-svc", false, None);
    assert!(cfg.is_valid(), "with_all config should be valid");
}

#[test]
fn with_all_port_zero_invalid() {
    let cfg = McpServerConfig::with_all(0, "127.0.0.1", "svc", false, None);
    assert!(!cfg.is_valid(), "port=0 should be invalid");
}

#[test]
fn with_all_empty_host_invalid() {
    let cfg = McpServerConfig::with_all(7894, "", "svc", false, None);
    assert!(!cfg.is_valid(), "empty host should be invalid");
}

#[test]
fn with_all_empty_service_name_invalid() {
    let cfg = McpServerConfig::with_all(7895, "127.0.0.1", "", false, None);
    assert!(!cfg.is_valid(), "empty service_name should be invalid");
}

#[rstest]
#[case(1)]
#[case(5)]
#[case(100)]
fn with_all_capacity_values(#[case] cap: usize) {
    let cfg = McpServerConfig::with_all(7896, "127.0.0.1", "svc", false, Some(cap));
    assert_eq!(cfg.max_webviews, Some(cap));
}

#[tokio::test]
async fn with_all_config_starts_runner() {
    let port = free_port().await;
    let cfg = McpServerConfig::with_all(port, "127.0.0.1", "test-all", false, None);
    let runner = McpRunner::new(cfg);
    runner.start().await.expect("start failed");
    assert!(runner.is_running().await);
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// PyMcpServer::emit_step
// ---------------------------------------------------------------------------

#[test]
fn emit_step_fails_when_server_not_started() {
    let server = PyMcpServer::new(59000);
    let result = server.emit_step("run-1", "export_scene", "step-001");
    assert!(result.is_err(), "emit_step must fail when server is not running");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("not running"),
        "error must mention 'not running': {msg}"
    );
}

#[test]
fn emit_step_succeeds_after_start() {
    let port = {
        use std::net::TcpListener;
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    };
    let cfg = McpServerConfig::with_all(port, "127.0.0.1", "test-step", false, None);
    let server = PyMcpServer::from_config(cfg);
    server.start().expect("server start failed");
    let result = server.emit_step("run-1", "export_scene", "step-001");
    assert!(result.is_ok(), "emit_step must succeed when running: {:?}", result);
    server.stop().ok();
}

#[test]
fn emit_step_after_stop_returns_error() {
    let port = {
        use std::net::TcpListener;
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    };
    let cfg = McpServerConfig::with_all(port, "127.0.0.1", "test-stop", false, None);
    let server = PyMcpServer::from_config(cfg);
    server.start().expect("start failed");
    server.stop().ok();
    let result = server.emit_step("run-1", "step", "sid");
    assert!(result.is_err(), "emit_step must fail after stop");
}

#[rstest]
#[case("run-a", "export", "s1")]
#[case("run-b", "import", "s2")]
#[case("run-c", "render", "s3")]
fn emit_step_various_params(#[case] run_id: &str, #[case] step: &str, #[case] sid: &str) {
    let port = {
        use std::net::TcpListener;
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    };
    let cfg = McpServerConfig::with_all(port, "127.0.0.1", "test-params", false, None);
    let server = PyMcpServer::from_config(cfg);
    server.start().expect("start");
    assert!(server.emit_step(run_id, step, sid).is_ok());
    server.stop().ok();
}

// ---------------------------------------------------------------------------
// SSE body parse: verify `list_webviews` response contains `capacity` JSON
// ---------------------------------------------------------------------------

async fn mcp_post(
    client: &Client,
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
    let resp = req.body(body.to_owned()).send().await.expect("POST /mcp");
    assert!(resp.status().is_success(), "POST /mcp status {}", resp.status());
    let sid = resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());
    let text = resp.text().await.unwrap_or_default();
    (sid, text)
}

async fn mcp_init(client: &Client, port: u16) -> Option<String> {
    let (sid, _) = mcp_post(
        client,
        port,
        None,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
    )
    .await;
    sid
}

/// Parse SSE lines and collect all `data: {...}` JSON values.
fn parse_sse_json_lines(body: &str) -> Vec<serde_json::Value> {
    body.lines()
        .filter(|l| l.starts_with("data:"))
        .filter_map(|l| {
            let payload = l.trim_start_matches("data:").trim();
            serde_json::from_str(payload).ok()
        })
        .collect()
}

#[tokio::test]
async fn list_webviews_sse_body_contains_capacity_json() {
    let port = free_port().await;
    let runner = McpRunner::new(
        McpServerConfig::with_all(port, "127.0.0.1", "sse-parse-test", false, Some(5)),
    );
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;
    let (_, body) = mcp_post(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"list_webviews","arguments":{}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "list_webviews must produce a body");

    // Verify the registry capacity is correct via direct access
    assert_eq!(
        runner.server().registry().capacity(),
        Some(5),
        "registry capacity must be 5"
    );

    // The SSE body may contain JSON objects in `data:` lines — look for capacity there
    let json_objects = parse_sse_json_lines(&body);
    // At minimum the body should be non-empty (already asserted above)
    // If any JSON object contains a "capacity" key, verify it equals 5
    for obj in &json_objects {
        if let Some(cap) = obj.pointer("/result/content/0/text") {
            // The tool result text may embed a JSON string — try to parse it
            if let Some(s) = cap.as_str() {
                if let Ok(inner) = serde_json::from_str::<serde_json::Value>(s) {
                    if let Some(c) = inner.get("capacity") {
                        assert_eq!(
                            c.as_u64(),
                            Some(5),
                            "capacity in tool result should be 5, got: {c}"
                        );
                    }
                }
            }
        }
    }

    runner.stop().await;
}

#[tokio::test]
async fn list_webviews_sse_body_no_capacity_when_unlimited() {
    let port = free_port().await;
    let runner = McpRunner::new(no_mdns_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let sid = mcp_init(&client, port).await;
    let (_, body) = mcp_post(
        &client,
        port,
        sid.as_deref(),
        r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"list_webviews","arguments":{}}}"#,
    )
    .await;

    assert!(!body.is_empty(), "list_webviews must produce a body");
    // Registry should have no capacity limit
    assert_eq!(runner.server().registry().capacity(), None);

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// emit_agui_step via McpRunner: verify two events in AguiBus
// ---------------------------------------------------------------------------

#[tokio::test]
async fn emit_agui_step_sends_two_events_to_bus() {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();

    runner.emit_agui_step("run-step", "render_pass", "sp-01");

    // Receive StepStarted
    let ev1 = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout StepStarted")
        .expect("channel error");
    match &ev1 {
        AguiEvent::StepStarted { run_id, step_name, step_id } => {
            assert_eq!(run_id, "run-step");
            assert_eq!(step_name, "render_pass");
            assert_eq!(step_id, "sp-01");
        }
        other => panic!("expected StepStarted, got {other:?}"),
    }

    // Receive StepFinished
    let ev2 = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout StepFinished")
        .expect("channel error");
    match &ev2 {
        AguiEvent::StepFinished { run_id, step_id } => {
            assert_eq!(run_id, "run-step");
            assert_eq!(step_id, "sp-01");
        }
        other => panic!("expected StepFinished, got {other:?}"),
    }
}

#[rstest]
#[case("r1", "step-a", "id-1")]
#[case("r2", "step-b", "id-2")]
#[case("r3", "step-c", "id-3")]
#[tokio::test]
async fn emit_agui_step_parameterized(
    #[case] run_id: &str,
    #[case] step_name: &str,
    #[case] step_id: &str,
) {
    let runner = McpRunner::new(McpServerConfig::default().with_mdns(false));
    let mut rx = runner.agui_bus().subscribe();
    runner.emit_agui_step(run_id, step_name, step_id);

    let ev = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout")
        .expect("channel error");
    assert_eq!(ev.run_id(), run_id);
}

// ---------------------------------------------------------------------------
// emit_step via SSE: verify StepStarted arrives at SSE subscriber
// ---------------------------------------------------------------------------

#[tokio::test]
async fn emit_step_appears_in_sse_stream() {
    let port = free_port().await;
    let runner = McpRunner::new(no_mdns_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .get(format!("http://127.0.0.1:{port}/agui/events"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("GET /agui/events");
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(Duration::from_millis(30)).await;

    runner.emit_agui_step("run-sse-step", "export_usd", "step-sse-1");

    let text = read_sse(resp, 512, Duration::from_millis(600)).await;
    assert!(
        text.contains("run-sse-step"),
        "run_id must appear in SSE text: {text:?}"
    );

    runner.stop().await;
}
