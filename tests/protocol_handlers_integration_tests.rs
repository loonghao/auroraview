//! Integration tests for protocol handlers
//!
//! These tests verify the complete protocol handling functionality with file system operations.

use rstest::*;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use wry::http::Request;

// Import the protocol handler functions
// Note: These need to be public in the source file
use auroraview_core::webview::protocol_handlers::{
    handle_auroraview_protocol, handle_custom_protocol,
};

#[rstest]
fn test_handle_auroraview_protocol_security() {
    // Create temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let asset_root = temp_dir.path();

    // Create a file inside asset_root
    let safe_file = asset_root.join("safe.txt");
    fs::write(&safe_file, b"Safe content").unwrap();

    // Create a file outside asset_root
    let outside_dir = TempDir::new().unwrap();
    let unsafe_file = outside_dir.path().join("unsafe.txt");
    fs::write(&unsafe_file, b"Unsafe content").unwrap();

    // Test 1: Valid request within asset_root
    let request = Request::builder()
        .method("GET")
        .uri("auroraview://safe.txt")
        .body(vec![])
        .unwrap();

    let response = handle_auroraview_protocol(asset_root, request);
    assert_eq!(
        response.status(),
        200,
        "Valid file request should return 200"
    );

    // Test 2: Directory traversal attempt (should be blocked)
    let request = Request::builder()
        .method("GET")
        .uri("auroraview://../../../etc/passwd")
        .body(vec![])
        .unwrap();

    let response = handle_auroraview_protocol(asset_root, request);
    // Should return 403 Forbidden or 404 Not Found
    assert!(
        response.status() == 403 || response.status() == 404,
        "Directory traversal should be blocked with 403 or 404, got {}",
        response.status()
    );

    // Test 3: Non-GET request
    let request = Request::builder()
        .method("POST")
        .uri("auroraview://safe.txt")
        .body(vec![])
        .unwrap();

    let response = handle_auroraview_protocol(asset_root, request);
    assert_eq!(
        response.status(),
        405,
        "POST request should return 405 Method Not Allowed"
    );
}

#[rstest]
fn test_handle_custom_protocol() {
    // Create a simple callback
    // Note: The URI passed to callback is the full URI string from request.uri().to_string()
    let callback = Arc::new(|uri: &str| -> Option<(Vec<u8>, String, u16)> {
        if uri == "test://hello.txt" || uri == "test://hello.txt/" {
            Some((b"Hello, World!".to_vec(), "text/plain".to_string(), 200))
        } else {
            None
        }
    });

    // Test 1: Successful request
    let request = Request::builder()
        .uri("test://hello.txt")
        .body(vec![])
        .unwrap();

    let response = handle_custom_protocol(&*callback, request);
    assert_eq!(
        response.status(),
        200,
        "Valid custom protocol request should return 200"
    );
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "text/plain",
        "Content-Type should be text/plain"
    );

    // Test 2: Not found
    let request = Request::builder()
        .uri("test://notfound.txt")
        .body(vec![])
        .unwrap();

    let response = handle_custom_protocol(&*callback, request);
    assert_eq!(response.status(), 404, "Unknown resource should return 404");
}

#[rstest]
fn test_auroraview_protocol_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let asset_root = temp_dir.path();

    // Create subdirectory structure
    let subdir = asset_root.join("assets").join("images");
    fs::create_dir_all(&subdir).unwrap();
    let image_file = subdir.join("logo.png");
    fs::write(&image_file, b"PNG data").unwrap();

    // Test accessing file in subdirectory
    let request = Request::builder()
        .method("GET")
        .uri("auroraview://assets/images/logo.png")
        .body(vec![])
        .unwrap();

    let response = handle_auroraview_protocol(asset_root, request);
    assert_eq!(
        response.status(),
        200,
        "Subdirectory file access should succeed"
    );
}

#[rstest]
fn test_custom_protocol_with_various_responses() {
    let callback = Arc::new(|uri: &str| -> Option<(Vec<u8>, String, u16)> {
        // Match based on URI path/content, not exact string
        if uri.contains("ok") {
            Some((b"OK".to_vec(), "text/plain".to_string(), 200))
        } else if uri.contains("redirect") {
            Some((b"".to_vec(), "text/plain".to_string(), 302))
        } else if uri.contains("error") {
            Some((b"Error".to_vec(), "text/plain".to_string(), 500))
        } else {
            None
        }
    });

    // Test different status codes
    let test_cases = vec![
        ("test://ok", 200),
        ("test://redirect", 302),
        ("test://error", 500),
        ("test://notfound", 404),
    ];

    for (uri, expected_status) in test_cases {
        let request = Request::builder().uri(uri).body(vec![]).unwrap();
        let response = handle_custom_protocol(&*callback, request);
        assert_eq!(
            response.status(),
            expected_status,
            "URI {} should return status {}",
            uri,
            expected_status
        );
    }
}
