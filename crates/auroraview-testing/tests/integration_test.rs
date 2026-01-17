//! Integration tests for auroraview-testing
//!
//! These tests require a running WebView/browser with CDP enabled.
//! Run with: cargo test -p auroraview-testing --test integration_test -- --ignored

use auroraview_testing::{Inspector, InspectorConfig};
use rstest::*;
use std::time::Duration;

/// Test endpoint - override with AURORAVIEW_TEST_CDP_ENDPOINT env var
fn get_test_endpoint() -> String {
    std::env::var("AURORAVIEW_TEST_CDP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9222".to_string())
}

/// Check if CDP endpoint is available
async fn is_cdp_available() -> bool {
    let endpoint = get_test_endpoint();
    let targets_url = format!("{}/json", endpoint.trim_end_matches('/'));
    reqwest::get(&targets_url).await.is_ok()
}

/// Test connecting to CDP endpoint
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_connect_to_cdp() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let result = Inspector::connect(&endpoint).await;

    assert!(result.is_ok(), "Failed to connect: {:?}", result.err());

    let inspector = result.unwrap();
    assert!(inspector.is_connected());

    // Cleanup
    inspector.close().await.ok();
}

/// Test getting page snapshot
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_snapshot() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    let snapshot = inspector.snapshot().await;
    assert!(snapshot.is_ok(), "Failed to get snapshot: {:?}", snapshot.err());

    let snap = snapshot.unwrap();
    assert!(!snap.url.is_empty(), "URL should not be empty");
    assert!(snap.viewport.0 > 0, "Viewport width should be positive");
    assert!(snap.viewport.1 > 0, "Viewport height should be positive");

    // Snapshot should have some refs if page has interactive elements
    println!("Snapshot: {} refs, URL: {}", snap.ref_count(), snap.url);
    println!("{}", snap);

    inspector.close().await.ok();
}

/// Test taking screenshot
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_screenshot() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    let screenshot = inspector.screenshot().await;
    assert!(screenshot.is_ok(), "Failed to take screenshot: {:?}", screenshot.err());

    let bytes = screenshot.unwrap();
    assert!(!bytes.is_empty(), "Screenshot should not be empty");
    // Check PNG magic bytes
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "Should be PNG format");

    println!("Screenshot size: {} bytes", bytes.len());

    inspector.close().await.ok();
}

/// Test navigation
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_navigation() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Navigate to about:blank
    let result = inspector.goto("about:blank").await;
    assert!(result.is_ok(), "Failed to navigate: {:?}", result.err());

    let action = result.unwrap();
    assert!(action.success, "Navigation should succeed");

    // Verify URL changed
    let url = inspector.url().await.expect("Failed to get URL");
    assert!(url.contains("blank"), "Should be at about:blank");

    inspector.close().await.ok();
}

/// Test JavaScript evaluation
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_eval() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Simple evaluation
    let result = inspector.eval("1 + 1").await;
    assert!(result.is_ok(), "Failed to eval: {:?}", result.err());

    let value = result.unwrap();
    assert_eq!(value["value"].as_i64(), Some(2), "1 + 1 should equal 2");

    // String evaluation
    let result = inspector.eval("'hello ' + 'world'").await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap()["value"].as_str(),
        Some("hello world")
    );

    inspector.close().await.ok();
}

/// Test wait conditions
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_wait() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Wait for network idle (should pass quickly on about:blank)
    let result = inspector.wait("idle", Some(Duration::from_secs(5))).await;
    assert!(result.is_ok(), "Wait failed: {:?}", result.err());
    assert!(result.unwrap(), "Should be idle");

    // Wait for DOM content loaded
    let result = inspector.wait("loaded", Some(Duration::from_secs(5))).await;
    assert!(result.is_ok());
    assert!(result.unwrap(), "Should be loaded");

    inspector.close().await.ok();
}

/// Test keyboard input
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_press_key() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Press Tab key (should not error even without focused element)
    let result = inspector.press("Tab").await;
    assert!(result.is_ok(), "Press key failed: {:?}", result.err());
    assert!(result.unwrap().success);

    // Press Escape
    let result = inspector.press("Escape").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);

    inspector.close().await.ok();
}

/// Test scroll
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_scroll() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    use auroraview_testing::snapshot::ScrollDirection;

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Scroll down
    let result = inspector.scroll(ScrollDirection::Down, 100).await;
    assert!(result.is_ok(), "Scroll failed: {:?}", result.err());
    assert!(result.unwrap().success);

    // Scroll up
    let result = inspector.scroll(ScrollDirection::Up, 100).await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);

    inspector.close().await.ok();
}

/// Test custom config
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_connect_with_config() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let config = InspectorConfig {
        timeout: Duration::from_secs(60),
        capture_screenshots: false,
        detect_changes: false,
    };

    let result = Inspector::connect_with_config(&endpoint, config).await;
    assert!(result.is_ok(), "Failed to connect with config: {:?}", result.err());

    let inspector = result.unwrap();
    assert!(inspector.is_connected());

    inspector.close().await.ok();
}

/// Test page properties
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_page_properties() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    // Get URL
    let url = inspector.url().await;
    assert!(url.is_ok(), "Failed to get URL: {:?}", url.err());
    println!("URL: {}", url.unwrap());

    // Get title
    let title = inspector.title().await;
    assert!(title.is_ok(), "Failed to get title: {:?}", title.err());
    println!("Title: {}", title.unwrap());

    inspector.close().await.ok();
}

/// Test Snapshot to_text and to_json
#[rstest]
#[tokio::test]
#[ignore = "Requires running CDP endpoint"]
async fn test_snapshot_formats() {
    if !is_cdp_available().await {
        eprintln!("CDP endpoint not available, skipping test");
        return;
    }

    let endpoint = get_test_endpoint();
    let inspector = Inspector::connect(&endpoint).await.expect("Failed to connect");

    let snapshot = inspector.snapshot().await.expect("Failed to get snapshot");

    // Test to_text format
    let text = snapshot.to_text();
    assert!(!text.is_empty());
    assert!(text.contains("Page:"));
    assert!(text.contains("Viewport:"));
    println!("Text format:\n{}", text);

    // Test to_json format
    let json = snapshot.to_json();
    assert!(!json.is_empty());
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert!(parsed.get("title").is_some());
    assert!(parsed.get("url").is_some());
    assert!(parsed.get("viewport").is_some());
    println!("JSON format:\n{}", json);

    inspector.close().await.ok();
}
