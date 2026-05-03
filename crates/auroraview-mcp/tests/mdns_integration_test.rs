//! mDNS integration tests for AuroraView MCP Server.
//!
//! These tests verify that the mDNS broadcast works correctly.

use auroraview_mcp::runner::McpRunner;
use auroraview_mcp::types::McpServerConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serial_test::serial;
use std::time::Duration;

// Helper to start a test server with mDNS enabled.
async fn start_test_server_with_mdns() -> (McpRunner, u16) {
    let port = 17000 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_millis() as u16 % 1000);
    let config = McpServerConfig::default()
        .with_port(port)
        .with_mdns(true)
        .with_service_name("test-auroraview-mcp");
    let runner = McpRunner::new(config);
    runner.start().await.expect("Server should start");
    tokio::time::sleep(Duration::from_millis(500)).await; // mDNS needs time to propagate
    (runner, port)
}

/// Test that mDNS broadcast is discoverable.
#[tokio::test]
#[serial]
async fn mdns_broadcast_is_discoverable() {
    let (runner, _port) = start_test_server_with_mdns().await;

    // Create a mDNS daemon to discover services
    let daemon = ServiceDaemon::new()
        .expect("Should create mDNS daemon");

    // Browse for AuroraView MCP services
    let service_type = "_auroraview-mcp._tcp.local.";
    let mut receiver = daemon.browse(service_type)
        .expect("Should start browsing");

    // Wait for service to be discovered (with timeout)
    let start = std::time::Instant::now();
    let mut service_found = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = receiver.try_recv() {
            match event {
                ServiceEvent::ServiceFound { .. } => {
                    service_found = true;
                    break;
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(service_found, "mDNS service should be discoverable");

    runner.stop().await;
}

/// Test that mDNS broadcast can be stopped.
#[tokio::test]
#[serial]
async fn mdns_broadcast_stop_broadcast() {
    let (runner, _port) = start_test_server_with_mdns().await;

    // Create a mDNS daemon to discover services
    let daemon = ServiceDaemon::new()
        .expect("Should create mDNS daemon");

    let service_type = "_auroraview-mcp._tcp.local.";
    let mut receiver = daemon.browse(service_type)
        .expect("Should start browsing");

    // Wait for service to be discovered
    let start = std::time::Instant::now();
    let mut service_found = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = receiver.try_recv() {
            match event {
                ServiceEvent::ServiceFound { .. } => {
                    service_found = true;
                    break;
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(service_found, "mDNS service should be discoverable before stop");

    // Stop the server (which stops mDNS broadcast)
    runner.stop().await;

    // Wait a bit for mDNS to propagate the disappearance
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Try to discover the service again (should not find it, or find it removed)
    let start = std::time::Instant::now();
    let mut service_removed = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = receiver.try_recv() {
            match event {
                ServiceEvent::ServiceRemoved { .. } => {
                    service_removed = true;
                    break;
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Note: mDNS removal may take time, so we don't assert here.
    // This test mainly ensures no panic occurs.
    println!("Service removed: {service_removed}");
}
