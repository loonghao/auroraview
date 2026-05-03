//! Integration tests for AuroraView MCP Server.
//!
//! These tests verify the HTTP transport and MCP protocol integration.

use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;
use futures::StreamExt;
use serial_test::serial;
use sha2::{Sha256, Digest};
use base64_url::encode;
use urlencoding;

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

/// Helper to start a test server with OAuth enabled.
async fn start_test_server_with_oauth() -> (McpRunner, u16) {
    let port = 16000 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_millis() as u16 % 1000);
    let config = McpServerConfig::default()
        .with_port(port)
        .with_mdns(false)
        .with_oauth(true); // Enable OAuth for tests
    let runner = McpRunner::new(config);
    runner.start().await.expect("Server should start");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    (runner, port)
}

/// Helper to start a test server with mDNS enabled.
async fn start_test_server_with_mdns() -> (McpRunner, u16) {
    let port = 17000 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_millis() as u16 % 1000);
    let config = McpServerConfig::default()
        .with_port(port)
        .with_mdns(true) // Enable mDNS for tests
        .with_service_name("test-auroraview-mcp");
    let runner = McpRunner::new(config);
    runner.start().await.expect("Server should start");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await; // mDNS needs time to propagate
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

// ---------------------------------------------------------------------------
// AG-UI SSE Endpoint Tests
// ---------------------------------------------------------------------------

use auroraview_mcp::agui::AguiEvent;

/// Helper to parse SSE event from a line.
fn parse_sse_event(line: &str) -> Option<serde_json::Value> {
    let line = line.trim();
    if line.starts_with("data: ") {
        let json_str = line.strip_prefix("data: ").unwrap().trim();
        serde_json::from_str(json_str).ok()
    } else {
        None
    }
}

#[tokio::test]
#[serial]
async fn agui_events_returns_sse_stream() {
    let (runner, port) = start_test_server().await;
    let bus = runner.agui_bus().clone();

    // Emit an event in the background
    let emit_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        bus.emit(AguiEvent::RunStarted {
            run_id: "run-1".to_string(),
            thread_id: "t-1".to_string(),
        });
    });

    // Subscribe to SSE stream
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/agui/events");
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Should send request");
    assert!(resp.status().is_success(), "SSE endpoint should return 2xx");

    // Read the first SSE event (with timeout)
    let mut stream = resp.bytes_stream();
    let mut received_event = None;
    let start = tokio::time::Instant::now();
    while start.elapsed() < tokio::time::Duration::from_secs(5) {
        if let Some(chunk) = stream.next().await {
            let chunk = chunk.expect("Should read chunk");
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(event) = parse_sse_event(line) {
                    received_event = Some(event);
                    break;
                }
            }
            if received_event.is_some() {
                break;
            }
        } else {
            break;
        }
    }

    emit_handle.await.expect("Emit task should complete");
    assert!(
        received_event.is_some(),
        "Should receive at least one SSE event"
    );
    let event = received_event.unwrap();
    assert_eq!(event["type"], "RUN_STARTED");
    assert_eq!(event["run_id"], "run-1");

    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn agui_events_filters_by_run_id() {
    let (runner, port) = start_test_server().await;
    let bus = runner.agui_bus().clone();

    // Emit events for two different runs
    let emit_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        bus.emit(AguiEvent::RunStarted {
            run_id: "run-a".to_string(),
            thread_id: "t-a".to_string(),
        });
        bus.emit(AguiEvent::RunStarted {
            run_id: "run-b".to_string(),
            thread_id: "t-b".to_string(),
        });
    });

    // Subscribe to events for "run-a" only
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/agui/events?run_id=run-a");
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Should send request");
    assert!(resp.status().is_success(), "SSE endpoint should return 2xx");

    // Read the first SSE event (should be run-a)
    let mut stream = resp.bytes_stream();
    let mut received_run_ids = Vec::new();
    let start = tokio::time::Instant::now();
    while start.elapsed() < tokio::time::Duration::from_secs(5) {
        if let Some(chunk) = stream.next().await {
            let chunk = chunk.expect("Should read chunk");
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(event) = parse_sse_event(line) {
                    received_run_ids.push(event["run_id"].as_str().unwrap().to_string());
                }
            }
            if received_run_ids.len() >= 1 {
                break;
            }
        } else {
            break;
        }
    }

    emit_handle.await.expect("Emit task should complete");
    assert_eq!(
        received_run_ids.len(),
        1,
        "Should receive exactly one event for run-a"
    );
    assert_eq!(received_run_ids[0], "run-a");

    runner.stop().await;
}

// ---------------------------------------------------------------------------
// OAuth Endpoint Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn oauth_metadata_endpoint_returns_correct_metadata() {
    let (runner, port) = start_test_server_with_oauth().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/.well-known/oauth-authorization-server");
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Should send request");
    
    assert!(resp.status().is_success(), "Metadata endpoint should return 2xx");
    
    let json: serde_json::Value = resp.json().await.expect("Should parse JSON");
    assert_eq!(json["issuer"], "auroraview-mcp");
    assert!(json["authorization_endpoint"].is_string());
    assert!(json["token_endpoint"].is_string());
    assert!(json["registration_endpoint"].is_string());
    assert!(json["code_challenge_methods_supported"].is_array());
    assert!(json["scopes_supported"].is_array());
    
    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn oauth_register_endpoint_creates_client() {
    let (runner, port) = start_test_server_with_oauth().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/oauth/register");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "client_name": "test-client",
            "redirect_uris": ["http://localhost:3000/callback"],
            "scope": "mcp:tools"
        }))
        .send()
        .await
        .expect("Should send request");
    
    assert!(resp.status().is_success(), "Register endpoint should return 2xx");
    
    let json: serde_json::Value = resp.json().await.expect("Should parse JSON");
    assert!(json["client_id"].is_string());
    assert!(json["client_secret"].is_string());
    assert_eq!(json["client_name"], "test-client");
    assert!(json["redirect_uris"].is_array());
    assert_eq!(json["scope"], "mcp:tools");
    
    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn oauth_authorize_endpoint_returns_redirect() {
    let (runner, port) = start_test_server_with_oauth().await;
    let client = reqwest::Client::new();
    
    // First register a client
    let register_url = format!("http://127.0.0.1:{port}/oauth/register");
    let register_resp = client
        .post(&register_url)
        .json(&serde_json::json!({
            "client_name": "test-client",
            "redirect_uris": ["http://localhost:3000/callback"],
            "scope": "mcp:tools"
        }))
        .send()
        .await
        .expect("Should register client");
    let register_json: serde_json::Value = register_resp.json().await.unwrap();
    let client_id = register_json["client_id"].as_str().unwrap();
    let code_verifier = "test-code-verifier-which-is-long-enough-to-be-valid";
    let code_challenge = encode(&Sha256::digest(code_verifier.as_bytes()));
    
    // Now authorize
    let auth_url = format!(
        "http://127.0.0.1:{port}/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
        client_id,
        urlencoding::encode("http://localhost:3000/callback"),
        urlencoding::encode("mcp:tools"),
        code_challenge
    );
    println!("Auth URL: {auth_url}");
    let resp = client
        .get(&auth_url)
        .send()
        .await
        .expect("Should send request");
    
    println!("Auth response status: {}", resp.status());
    let headers = resp.headers().clone();
    println!("Auth response headers: {:?}", headers);
    let body = resp.text().await.expect("Should read body");
    println!("Auth response body: {}", body);
    
    assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER, "Authorize should return redirect: {body}");
    
    let location = headers.get("location").unwrap().to_str().unwrap();
    assert!(location.contains("code="), "Redirect should contain authorization code");
    
    runner.stop().await;
}

#[tokio::test]
#[serial]
async fn oauth_token_endpoint_exchanges_code_for_token() {
    let (runner, port) = start_test_server_with_oauth().await;
    let client = reqwest::Client::new();
    
    // Register client
    let register_url = format!("http://127.0.0.1:{port}/oauth/register");
    let register_resp = client
        .post(&register_url)
        .json(&serde_json::json!({
            "client_name": "test-client",
            "redirect_uris": ["http://localhost:3000/callback"],
            "scope": "mcp:tools"
        }))
        .send()
        .await
        .expect("Should register client");
    let register_json: serde_json::Value = register_resp.json().await.unwrap();
    let client_id = register_json["client_id"].as_str().unwrap();
    let code_verifier = "test-code-verifier-which-is-long-enough-to-be-valid";
    let code_challenge = encode(&Sha256::digest(code_verifier.as_bytes()));
    
    // Get authorization code
    let auth_url = format!(
        "http://127.0.0.1:{port}/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
        client_id,
        urlencoding::encode("http://localhost:3000/callback"),
        urlencoding::encode("mcp:tools"),
        code_challenge
    );
    let auth_resp = client.get(&auth_url).send().await.unwrap();
    let location = auth_resp.headers().get("location").unwrap().to_str().unwrap();
    let code = location.split("code=").nth(1).unwrap().split('&').next().unwrap();
    
    // Exchange code for token
    let token_url = format!("http://127.0.0.1:{port}/oauth/token");
    let resp = client
        .post(&token_url)
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": client_id,
            "code": code,
            "redirect_uri": "http://localhost:3000/callback",
            "code_verifier": code_verifier
        }))
        .send()
        .await
        .expect("Should send request");
    
    assert!(resp.status().is_success(), "Token endpoint should return 2xx");
    
    let json: serde_json::Value = resp.json().await.expect("Should parse JSON");
    assert!(json["access_token"].is_string());
    assert_eq!(json["token_type"], "Bearer");
    assert!(json["expires_in"].is_number());
    assert_eq!(json["scope"], "mcp:tools");
    
    runner.stop().await;
}
