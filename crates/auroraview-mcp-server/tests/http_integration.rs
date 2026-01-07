//! HTTP Server integration tests.
//!
//! These tests verify the HTTP server can start, respond to health checks,
//! and handle MCP requests properly.

use std::sync::Arc;
use std::time::Duration;

use auroraview_mcp_server::{HttpServer, HttpServerConfig, IpcClient};

/// Test that HTTP server can start and stop.
#[tokio::test]
async fn test_http_server_start_stop() {
    // Create a mock IPC client (won't actually connect)
    let ipc_client = Arc::new(IpcClient::new("test_channel", "test_token"));

    let config = HttpServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Auto-assign port
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
        heartbeat_interval: 30,
    };

    let server = HttpServer::new(config, ipc_client);

    // Start server
    let port = server.start().await.expect("Failed to start server");
    assert!(port > 0, "Port should be assigned");
    assert!(server.is_running(), "Server should be running");
    assert_eq!(server.port(), port, "Port should match");

    // Stop server
    server.stop().await;
    assert!(!server.is_running(), "Server should not be running");
    assert_eq!(server.port(), 0, "Port should be reset");
}

/// Test that HTTP server can handle multiple start calls gracefully.
#[tokio::test]
async fn test_http_server_idempotent_start() {
    let ipc_client = Arc::new(IpcClient::new("test_channel", "test_token"));

    let config = HttpServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
        heartbeat_interval: 30,
    };

    let server = HttpServer::new(config, ipc_client);

    // Start twice
    let port1 = server.start().await.expect("First start failed");
    let port2 = server.start().await.expect("Second start failed");

    // Should return the same port
    assert_eq!(port1, port2, "Same port should be returned");

    server.stop().await;
}

/// Test health endpoint returns correct response.
#[tokio::test]
async fn test_health_endpoint() {
    let ipc_client = Arc::new(IpcClient::new("test_channel", "test_token"));

    let config = HttpServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
        heartbeat_interval: 30,
    };

    let server = HttpServer::new(config, ipc_client);
    let port = server.start().await.expect("Failed to start server");

    // Give server time to fully start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Make health check request
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Health request failed");

    assert!(response.status().is_success(), "Health check should succeed");

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["transport"], "streamable-http");

    server.stop().await;
}

/// Test MCP endpoint is accessible (basic connectivity).
#[tokio::test]
async fn test_mcp_endpoint_accessible() {
    let ipc_client = Arc::new(IpcClient::new("test_channel", "test_token"));

    let config = HttpServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
        heartbeat_interval: 30,
    };

    let server = HttpServer::new(config, ipc_client);
    let port = server.start().await.expect("Failed to start server");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // The MCP endpoint should respond (even if with an error for malformed requests)
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .body("{}")
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("MCP request failed");

    // Should get a response (not necessarily success, but the endpoint exists)
    assert!(response.status().as_u16() > 0, "Should get some response");

    server.stop().await;
}

