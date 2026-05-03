//! Integration tests for AuroraView MCP Server.
//!
//! These tests verify the HTTP transport and MCP protocol integration.

use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;
use serial_test::serial;

/// Parse SSE response format: split by "\n\n", find "data: {...}" lines.
fn parse_sse_response(sse_text: &str) -> serde_json::Value {
    // SSE format: messages separated by "\n\n"
    for line in sse_text.lines() {
        let line = line.trim();
        if line.starts_with("data: ") {
            let json_str = line.strip_prefix("data: ").unwrap().trim();
            if !json_str.is_empty() && (json_str.starts_with('{') || json_str.starts_with('[')) {
                return serde_json::from_str(json_str)
                    .expect("Should parse SSE data as JSON");
            }
        }
    }
    panic!("No valid JSON found in SSE response: {sse_text}");
}

/// Test that `McpRunner` can be created with default config.
#[test]
fn runner_creates_with_defaults() {
    let config = McpServerConfig::default();
    let runner = McpRunner::new(config);
    assert_eq!(runner.config().port, 7890);
    assert!(runner.config().enable_mdns); // default is true
}

/// Test that `McpRunner` can be created with custom port.
#[test]
fn runner_creates_with_custom_port() {
    let config = McpServerConfig::default().with_port(9000);
    let runner = McpRunner::new(config);
    assert_eq!(runner.config().port, 9000);
}

/// Test that `McpRunner::start()` and `stop()` work without panicking.
///
/// **Note**: This test only verifies that the server can start and stop
/// without errors. It does not test actual HTTP requests.
#[tokio::test]
async fn runner_start_and_stop() {
    let config = McpServerConfig::default().with_port(12345); // Use a specific port
    let runner = McpRunner::new(config);

    // Start the server (should not panic)
    let result = runner.start().await;
    assert!(
        result.is_ok(),
        "Server should start without error: {:?}",
        result
    );

    // Give the server a moment to initialize
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check if server is running
    let is_running = runner.is_running().await;
    assert!(is_running, "Server should be running after start()");

    // Stop the server (should not panic)
    runner.stop().await;

    // Give the server a moment to shut down
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check if server is stopped
    let is_running = runner.is_running().await;
    assert!(!is_running, "Server should not be running after stop()");
}

/// Test that starting a server on the same port fails.
#[tokio::test]
async fn runner_start_twice_fails() {
    let config = McpServerConfig::default().with_port(12346);
    let runner = McpRunner::new(config);

    // First start should succeed
    let result1 = runner.start().await;
    assert!(result1.is_ok(), "First start should succeed");

    // Second start should fail (already running)
    let result2 = runner.start().await;
    assert!(result2.is_err(), "Second start should fail");

    // Clean up
    runner.stop().await;
}

// ---------------------------------------------------------------------------
// MCP Protocol Integration Tests
// ---------------------------------------------------------------------------

/// Helper to create a JSON-RPC request body.
fn jsonrpc_request(id: u64, method: &str, params: Option<serde_json::Value>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
    });
    if let Some(p) = params {
        body["params"] = p;
    }
    body
}

/// Helper to start a test server on a unique port and return the port.
async fn start_test_server() -> (McpRunner, u16) {
    // Use a different port for each test to avoid AddrInUse errors.
    // We use timestamp-based port selection to minimize collisions.
    let port = 15000 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_millis() as u16 % 1000);
    let config = McpServerConfig::default()
        .with_port(port)
        .with_mdns(false); // Disable mDNS for tests
    let runner = McpRunner::new(config);
    runner.start().await.expect("Server should start");
    // Give the server time to bind
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    (runner, port)
}

#[tokio::test]
#[serial]
async fn mcp_initialize_returns_ok() {
    let (runner, port) = start_test_server().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/mcp");

    let body = jsonrpc_request(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "0.1.0"}
        })),
    );

    let resp = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .expect("Should send request");

    let status = resp.status();
    let headers = resp.headers().clone();
    let text = resp.text().await.expect("Should read response");
    
    println!("Response status: {status}");
    println!("Response headers: {headers:#?}");
    println!("Response body: {text}");
    
    assert!(
        status.is_success(),
        "Initialize should return 2xx, got {status}: {text}"
    );

    // Parse SSE format response using helper function
    let json = parse_sse_response(&text);
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);
    assert!(
        json["result"]["capabilities"].is_object(),
        "Should have capabilities, got: {text}"
    );

    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn mcp_list_tools_returns_tools() {
    let (runner, port) = start_test_server().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/mcp");

    // First, initialize the session (required by MCP protocol)
    let init_body = jsonrpc_request(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "0.1.0"}
        })),
    );
    let init_resp = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("Should send initialize request");
    
    // Extract session ID from initialize response
    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .expect("Should have mcp-session-id header")
        .to_str()
        .unwrap()
        .to_string();

    // Now list tools (with session ID)
    let list_body = jsonrpc_request(2, "tools/list", None);
    let resp = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .header("Mcp-Session-Id", &session_id)
        .json(&list_body)
        .send()
        .await
        .expect("Should send request");

    let status = resp.status();
    let text = resp.text().await.expect("Should read response");
    
    assert!(status.is_success(), "tools/list should return 2xx, got {status}: {text}");

    // Parse SSE format response
    let json = parse_sse_response(&text);
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 2);
    assert!(json["result"]["tools"].is_array(), "Should have tools array, got: {text}");

    let tools = json["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    // Verify expected tools are present
    assert!(
        tool_names.contains(&"screenshot"),
        "Should have screenshot tool"
    );
    assert!(
        tool_names.contains(&"eval_js"),
        "Should have eval_js tool"
    );
    assert!(
        tool_names.contains(&"load_url"),
        "Should have load_url tool"
    );
    assert!(
        tool_names.contains(&"send_event"),
        "Should have send_event tool"
    );

    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn mcp_call_tool_without_cdp_returns_error() {
    let (runner, port) = start_test_server().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/mcp");

    // Initialize and save response to extract session ID
    let init_body = jsonrpc_request(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "0.1.0"}
        })),
    );
    let init_resp = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("Should send initialize request");
    
    // Extract session ID from initialize response
    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .expect("Should have mcp-session-id header")
        .to_str()
        .unwrap()
        .to_string();

    // Call a tool (will fail because no CDP endpoint is available)
    let call_body = jsonrpc_request(
        2,
        "tools/call",
        Some(serde_json::json!({
            "name": "screenshot",
            "arguments": {"format": "png"}
        })),
    );
    let resp = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .header("Mcp-Session-Id", &session_id)
        .json(&call_body)
        .send()
        .await
        .expect("Should send request");

    let status = resp.status();
    let text = resp.text().await.expect("Should read response");
    
    assert!(status.is_success(), "tools/call should return 2xx, got {status}: {text}");

    // Parse SSE format response
    let json = parse_sse_response(&text);
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 2);
    // The tool call should return a JSON-RPC error (CDP not available)
    assert!(
        json.get("error").is_some(),
        "Tool call should return error when CDP is not available, got: {text}"
    );
    assert_eq!(
        json["error"]["code"],
        -32603,
        "Should return internal error code"
    );

    runner.stop().await;
}
