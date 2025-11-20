//! Integration tests for HTTP Discovery Service
//!
//! These tests verify the complete HTTP discovery service functionality including
//! server lifecycle, endpoint responses, and error handling.

use auroraview_core::service_discovery::http_discovery::{DiscoveryResponse, HttpDiscovery};
use rstest::*;

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_start_stop_and_request() {
    let mut server = HttpDiscovery::new(0, 9101); // Use port 0 for OS-assigned port
    assert!(
        !server.is_running(),
        "Server should not be running initially"
    );

    server.start().await.expect("server should start");
    assert!(server.is_running(), "Server should be running after start");

    // Get the actual bound port from the server
    let bound_port = server.port;
    assert!(bound_port > 0, "Port should be set after start");

    // Query the discovery endpoint using reqwest
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/discover", bound_port))
        .send()
        .await
        .expect("GET /discover should succeed");

    assert!(
        resp.status().is_success(),
        "Discovery endpoint should return success"
    );

    let json: DiscoveryResponse = resp.json().await.unwrap();
    assert_eq!(json.service, "AuroraView Bridge");
    assert_eq!(json.port, 9101);
    assert_eq!(json.protocol, "websocket");

    // Stop server
    server.stop().expect("server should stop");
    assert!(
        !server.is_running(),
        "Server should not be running after stop"
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_double_start_and_double_stop_are_safe() {
    let mut server = HttpDiscovery::new(0, 9102); // 0 → OS-assign port

    // First start
    server.start().await.expect("first start ok");
    assert!(
        server.is_running(),
        "Server should be running after first start"
    );

    // Second start should be a no-op and Ok
    server.start().await.expect("second start ok");
    assert!(
        server.is_running(),
        "Server should still be running after second start"
    );

    // First stop
    server.stop().expect("first stop ok");
    assert!(
        !server.is_running(),
        "Server should not be running after first stop"
    );

    // Second stop should not panic
    server.stop().expect("second stop ok");
    assert!(
        !server.is_running(),
        "Server should still not be running after second stop"
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_unknown_path_returns_404() {
    // Use port 0 for OS-assigned port to avoid flakiness
    let mut server = HttpDiscovery::new(0, 9101);
    server.start().await.expect("server start");

    let bound_port = server.port;
    assert!(bound_port > 0, "Port should be set after start");

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/unknown", bound_port))
        .send()
        .await
        .expect("GET should succeed");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::NOT_FOUND,
        "Unknown path should return 404"
    );

    server.stop().expect("server stop");
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_stop_without_start_is_ok() {
    let mut server = HttpDiscovery::new(0, 9101);

    // Calling stop on a non-started server should be a no-op and Ok
    server.stop().expect("stop ok");
    assert!(
        !server.is_running(),
        "Server should not be running after stop without start"
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_drop_stops_server() {
    let mut server = HttpDiscovery::new(0, 9101);
    server.start().await.expect("start ok");
    assert!(server.is_running(), "Server should be running after start");

    drop(server); // should invoke Drop::drop -> stop()

    // Allow a short moment for graceful shutdown
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_http_discovery_concurrent_requests() {
    let mut server = HttpDiscovery::new(0, 9103);
    server.start().await.expect("server start");

    let bound_port = server.port;
    let client = reqwest::Client::new();

    // Send multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..5 {
        let client_clone = client.clone();
        let port = bound_port;
        let handle = tokio::spawn(async move {
            client_clone
                .get(format!("http://127.0.0.1:{}/discover", port))
                .send()
                .await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let resp = handle.await.unwrap().expect("Request should succeed");
        assert!(resp.status().is_success(), "All requests should succeed");
    }

    server.stop().expect("server stop");
}
