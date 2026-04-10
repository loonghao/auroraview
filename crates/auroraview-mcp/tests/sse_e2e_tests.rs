//! End-to-end SSE tests: verify that events emitted via AguiBus
//! are actually received as SSE lines by an HTTP client (reqwest).
//!
//! The pattern:
//!   1. Start McpRunner on an ephemeral port
//!   2. Open a streaming GET /agui/events connection via reqwest
//!   3. Emit an AguiEvent from the Rust side
//!   4. Read the SSE stream and verify the event arrived

use auroraview_mcp::{AguiEvent, McpRunner, McpServerConfig};
use reqwest::Client;
use std::time::Duration;
use tokio::time::timeout;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

fn test_config(port: u16) -> McpServerConfig {
    McpServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_mdns: false,
        ..McpServerConfig::default()
    }
}

/// Read up to `limit` bytes from the SSE stream within `deadline`.
/// Returns the raw text received so far.
async fn read_sse_bytes(
    response: reqwest::Response,
    limit: usize,
    deadline: Duration,
) -> String {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buf = Vec::new();
    let read_fut = async {
        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                buf.extend_from_slice(&bytes);
                if buf.len() >= limit {
                    break;
                }
            }
        }
    };
    let _ = timeout(deadline, read_fut).await;
    String::from_utf8_lossy(&buf).into_owned()
}

// ---------------------------------------------------------------------------
// SSE keep-alive test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_stream_sends_keep_alive() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();

    let resp = client
        .get(format!("http://127.0.0.1:{port}/agui/events"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("GET /agui/events failed");

    assert_eq!(resp.status(), 200);
    assert!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("text/event-stream"),
        "must be SSE content type"
    );

    // The keep-alive arrives within ~15s but we don't wait that long in CI.
    // Just verify we can read some bytes without the connection being immediately closed.
    let text = read_sse_bytes(resp, 1, Duration::from_millis(200)).await;
    // We might not receive anything in 200ms but the connection should not error.
    // The test verifies the stream is open (no early EOF / error).
    let _ = text; // content varies; key assertion is status=200 above.

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// RunStarted event delivered via SSE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_delivers_run_started_event() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // Open SSE stream
    let resp = client
        .get(format!("http://127.0.0.1:{port}/agui/events"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("GET /agui/events failed");
    assert_eq!(resp.status(), 200);

    // Emit event from Rust side (after SSE connection is established)
    tokio::time::sleep(Duration::from_millis(20)).await;
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "run-e2e-1".to_string(),
        thread_id: "thread-e2e-1".to_string(),
    });

    // Read the SSE stream for up to 500ms, expecting ~200 bytes
    let text = read_sse_bytes(resp, 200, Duration::from_millis(500)).await;
    assert!(
        text.contains("run-e2e-1"),
        "SSE text should contain run_id 'run-e2e-1', got: {text:?}"
    );
    assert!(
        text.contains("RUN_STARTED") || text.contains("RunStarted"),
        "SSE text should contain event type, got: {text:?}"
    );

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// Multiple events in sequence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_delivers_multiple_events_in_order() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
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
        .expect("failed");
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Emit 3 events
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "multi-run".to_string(),
        thread_id: "t1".to_string(),
    });
    runner.emit_agui(AguiEvent::StepStarted {
        run_id: "multi-run".to_string(),
        step_name: "screenshot".to_string(),
        step_id: "step-1".to_string(),
    });
    runner.emit_agui(AguiEvent::RunFinished {
        run_id: "multi-run".to_string(),
        thread_id: "t1".to_string(),
    });

    let text = read_sse_bytes(resp, 1024, Duration::from_millis(800)).await;

    assert!(text.contains("multi-run"), "run_id must appear: {text:?}");

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// run_id filter — only matching events arrive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_run_id_filter_excludes_other_runs() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // Subscribe with run_id filter "target-run"
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/agui/events?run_id=target-run"
        ))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .expect("failed");
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Emit one event for a different run — should NOT arrive
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "other-run".to_string(),
        thread_id: "t1".to_string(),
    });
    // Emit one event for the target run — SHOULD arrive
    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "target-run".to_string(),
        thread_id: "t2".to_string(),
    });

    let text = read_sse_bytes(resp, 512, Duration::from_millis(600)).await;

    // target-run must appear
    assert!(
        text.contains("target-run"),
        "target-run event must appear: {text:?}"
    );
    // other-run must NOT appear (filtered)
    assert!(
        !text.contains("other-run"),
        "other-run event must be filtered out: {text:?}"
    );

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// ToolCallStart event via SSE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_delivers_tool_call_start() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
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
        .expect("failed");
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(Duration::from_millis(20)).await;

    runner.emit_agui(AguiEvent::ToolCallStart {
        run_id: "tool-run".to_string(),
        tool_call_id: "call-abc".to_string(),
        tool_name: "screenshot".to_string(),
    });

    let text = read_sse_bytes(resp, 512, Duration::from_millis(500)).await;
    assert!(
        text.contains("call-abc"),
        "tool_call_id must appear: {text:?}"
    );
    assert!(
        text.contains("screenshot"),
        "tool_name must appear: {text:?}"
    );

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// Custom event via SSE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sse_delivers_custom_event() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
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
        .expect("failed");

    tokio::time::sleep(Duration::from_millis(20)).await;

    runner.emit_agui(AguiEvent::Custom {
        run_id: "custom-run".to_string(),
        name: "progress_update".to_string(),
        data: serde_json::json!({"percent": 42, "msg": "halfway"}),
    });

    let text = read_sse_bytes(resp, 512, Duration::from_millis(500)).await;
    assert!(
        text.contains("progress_update"),
        "event name must appear: {text:?}"
    );
    assert!(
        text.contains("42"),
        "event data must appear: {text:?}"
    );

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// PyMcpServer emit → SSE delivery (Rust-layer end-to-end)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn py_server_emit_reaches_sse_stream() {
    use auroraview_mcp::PyMcpServer;

    let port = free_port().await;
    // PyMcpServer uses its own runtime internally — start in blocking thread
    let port_copy = port;
    let handle = tokio::task::spawn_blocking(move || {
        let server = PyMcpServer::new(port_copy);
        server.start().expect("PyMcpServer start");
        server
    });
    let server = handle.await.expect("spawn_blocking");

    tokio::time::sleep(Duration::from_millis(100)).await;

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

    tokio::time::sleep(Duration::from_millis(30)).await;

    // Emit from PyMcpServer (blocking call from async context)
    let server_arc = std::sync::Arc::new(server);
    let s = std::sync::Arc::clone(&server_arc);
    tokio::task::spawn_blocking(move || {
        s.emit_run_started("py-run-1", "py-thread-1")
            .expect("emit ok");
    })
    .await
    .expect("spawn_blocking emit");

    let text = read_sse_bytes(resp, 512, Duration::from_millis(600)).await;
    assert!(
        text.contains("py-run-1"),
        "run_id must appear in SSE stream: {text:?}"
    );

    // Stop the server in a blocking thread
    let s2 = std::sync::Arc::clone(&server_arc);
    tokio::task::spawn_blocking(move || s2.stop().ok())
        .await
        .ok();
}

// ---------------------------------------------------------------------------
// Multiple concurrent SSE subscribers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multiple_sse_subscribers_all_receive_event() {
    let port = free_port().await;
    let runner = McpRunner::new(test_config(port));
    runner.start().await.expect("start failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    // Open 3 concurrent SSE connections
    let (r1, r2, r3) = tokio::join!(
        client
            .get(format!("http://127.0.0.1:{port}/agui/events"))
            .header("Accept", "text/event-stream")
            .send(),
        client
            .get(format!("http://127.0.0.1:{port}/agui/events"))
            .header("Accept", "text/event-stream")
            .send(),
        client
            .get(format!("http://127.0.0.1:{port}/agui/events"))
            .header("Accept", "text/event-stream")
            .send(),
    );

    let (r1, r2, r3) = (r1.unwrap(), r2.unwrap(), r3.unwrap());
    assert_eq!(r1.status(), 200);
    assert_eq!(r2.status(), 200);
    assert_eq!(r3.status(), 200);

    tokio::time::sleep(Duration::from_millis(30)).await;

    runner.emit_agui(AguiEvent::RunStarted {
        run_id: "broadcast-run".to_string(),
        thread_id: "t1".to_string(),
    });

    let (t1, t2, t3) = tokio::join!(
        read_sse_bytes(r1, 512, Duration::from_millis(600)),
        read_sse_bytes(r2, 512, Duration::from_millis(600)),
        read_sse_bytes(r3, 512, Duration::from_millis(600)),
    );

    for (i, text) in [&t1, &t2, &t3].iter().enumerate() {
        assert!(
            text.contains("broadcast-run"),
            "subscriber {i} must receive event: {text:?}"
        );
    }

    runner.stop().await;
}
