//! Integration tests for AuroraView MCP Server.
//!
//! These tests verify the HTTP transport and MCP protocol integration.

use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;

/// Helper: create a config with a random-ish port.
fn test_config(port: u16) -> McpServerConfig {
    McpServerConfig::default().with_port(port).with_mdns(false) // disable mDNS for tests
}

/// Helper: send MCP initialize request.
async fn mcp_initialize(base_url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });
    client
        .post(format!("{base_url}/mcp"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
}

#[tokio::test]
#[ignore = "TODO: fix MCP initialize request format for rmcp 1.5"]
async fn mcp_initialize_returns_ok() {
    let config = test_config(12450);
    let runner = McpRunner::new(config);
    let base_url = "http://127.0.0.1:12450".to_string();

    // Start server
    runner.start().await.expect("Server should start");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Send initialize request
    let resp = mcp_initialize(&base_url)
        .await
        .expect("Request should succeed");
    assert_eq!(resp.status(), 200);

    // Check response body
    let body: serde_json::Value = resp.json().await.expect("Should parse JSON");
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    assert!(body["result"].is_object(), "Should have result object");

    // Clean up
    runner.stop().await;
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
