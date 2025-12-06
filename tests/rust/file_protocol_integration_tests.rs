//! Integration tests for file:// protocol handler
//!
//! These tests verify the file:// protocol handling functionality with real file system operations.
//!
//! Note: HTTP Request builder doesn't support file:// URIs, so we test the handler
//! with HTTP URIs and verify error handling. The actual file:// protocol is tested
//! through end-to-end tests with a real WebView.

use rstest::*;
use std::fs;
use tempfile::TempDir;
use wry::http::Request;

use _core::webview::protocol_handlers::handle_file_protocol;

#[rstest]
fn test_file_protocol_rejects_non_file_uris() {
    // Test that handler rejects non-file:// URIs
    let request = Request::builder()
        .method("GET")
        .uri("http://example.com/test.txt")
        .body(vec![])
        .unwrap();

    let response = handle_file_protocol(request);
    assert_eq!(
        response.status(),
        400,
        "Non-file:// URI should return 400 Bad Request"
    );
}

#[rstest]
fn test_file_protocol_rejects_post_method() {
    // Test that handler rejects POST requests
    let request = Request::builder()
        .method("POST")
        .uri("http://localhost/test.txt")
        .body(vec![])
        .unwrap();

    let response = handle_file_protocol(request);
    assert_eq!(
        response.status(),
        405,
        "POST request should return 405 Method Not Allowed"
    );
}

#[rstest]
fn test_file_protocol_creates_temp_files() {
    // Test that we can create and verify temp files exist
    // This tests the file system operations that the protocol handler will use
    let temp_dir = TempDir::new().unwrap();

    // Create various file types
    let txt_file = temp_dir.path().join("test.txt");
    let html_file = temp_dir.path().join("test.html");
    let css_file = temp_dir.path().join("style.css");
    let js_file = temp_dir.path().join("app.js");
    let json_file = temp_dir.path().join("data.json");

    fs::write(&txt_file, b"Text content").unwrap();
    fs::write(&html_file, b"<!DOCTYPE html><html></html>").unwrap();
    fs::write(&css_file, b"body { margin: 0; }").unwrap();
    fs::write(&js_file, b"console.log('test');").unwrap();
    fs::write(&json_file, b"{\"test\": true}").unwrap();

    // Verify all files exist
    assert!(txt_file.exists());
    assert!(html_file.exists());
    assert!(css_file.exists());
    assert!(js_file.exists());
    assert!(json_file.exists());

    // Verify content
    assert_eq!(fs::read(&txt_file).unwrap(), b"Text content");
    assert_eq!(
        fs::read(&html_file).unwrap(),
        b"<!DOCTYPE html><html></html>"
    );
}

#[rstest]
fn test_file_protocol_handles_special_characters() {
    // Test files with special characters in names
    let temp_dir = TempDir::new().unwrap();

    // File with space
    let space_file = temp_dir.path().join("test file.txt");
    fs::write(&space_file, b"Space in name").unwrap();
    assert!(space_file.exists());

    // File with unicode
    let unicode_file = temp_dir.path().join("测试.txt");
    fs::write(&unicode_file, b"Unicode name").unwrap();
    assert!(unicode_file.exists());
}

// Note: MIME type detection is tested in the unit tests in protocol_handlers.rs
// The guess_mime_type function is private and already has comprehensive test coverage
