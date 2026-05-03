//! Integration tests for AuroraView MCP Server.
//!
//! These tests verify the HTTP transport and MCP protocol integration.

use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;

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
