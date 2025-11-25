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
    handle_auroraview_protocol, handle_custom_protocol, is_windows_absolute_path_without_colon,
    normalize_windows_path_without_colon, parse_protocol_path,
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

// ============================================================================
// Path Parsing Tests
// ============================================================================

#[rstest]
fn test_is_windows_absolute_path_without_colon() {
    // Should detect Windows paths without colon
    assert!(is_windows_absolute_path_without_colon("c/users/test"));
    assert!(is_windows_absolute_path_without_colon("C/Users/test"));
    assert!(is_windows_absolute_path_without_colon(
        "d/projects/file.txt"
    ));

    // Should not detect these as Windows paths without colon
    assert!(!is_windows_absolute_path_without_colon("C:/users/test"));
    assert!(!is_windows_absolute_path_without_colon("/path/to/file"));
    assert!(!is_windows_absolute_path_without_colon("relative/path"));
    assert!(!is_windows_absolute_path_without_colon("file.txt"));

    // Edge cases: short paths
    assert!(!is_windows_absolute_path_without_colon(""));
    assert!(!is_windows_absolute_path_without_colon("c"));
    assert!(!is_windows_absolute_path_without_colon("/"));
}

#[rstest]
fn test_normalize_windows_path_without_colon() {
    assert_eq!(
        normalize_windows_path_without_colon("c/users/test"),
        "C:/users/test"
    );
    assert_eq!(
        normalize_windows_path_without_colon("d/projects/file.txt"),
        "D:/projects/file.txt"
    );

    // Edge cases: short paths should return as-is
    assert_eq!(normalize_windows_path_without_colon(""), "");
    assert_eq!(normalize_windows_path_without_colon("c"), "c");
}

#[rstest]
fn test_parse_protocol_path_relative() {
    use std::path::PathBuf;

    let asset_root = PathBuf::from("/tmp/assets");

    // Relative paths should be joined with asset_root
    let result = parse_protocol_path("css/style.css", &asset_root);
    assert_eq!(result, PathBuf::from("/tmp/assets/css/style.css"));

    let result = parse_protocol_path("images/logo.png", &asset_root);
    assert_eq!(result, PathBuf::from("/tmp/assets/images/logo.png"));
}

#[rstest]
fn test_parse_protocol_path_windows_without_colon() {
    use std::path::PathBuf;

    let asset_root = PathBuf::from("/tmp/assets");

    // Test that Windows paths without colon are detected and normalized
    // This specifically tests the branch at lines 290-292
    let result = parse_protocol_path("c/users/test/file.txt", &asset_root);
    // On Windows, this should be normalized to C:/users/test/file.txt
    // On Unix, this should still work as the function handles it
    #[cfg(windows)]
    assert_eq!(result, PathBuf::from("C:/users/test/file.txt"));
    #[cfg(unix)]
    assert_eq!(result, PathBuf::from("C:/users/test/file.txt"));

    // Test with uppercase drive letter
    let result = parse_protocol_path("D/Projects/app.exe", &asset_root);
    #[cfg(windows)]
    assert_eq!(result, PathBuf::from("D:/Projects/app.exe"));
    #[cfg(unix)]
    assert_eq!(result, PathBuf::from("D:/Projects/app.exe"));
}

#[rstest]
#[cfg(unix)]
fn test_parse_protocol_path_absolute_unix() {
    use std::path::PathBuf;

    let asset_root = PathBuf::from("/tmp/assets");

    // Absolute Unix paths should be used directly
    let result = parse_protocol_path("/usr/share/file.txt", &asset_root);
    assert_eq!(result, PathBuf::from("/usr/share/file.txt"));
}

#[rstest]
#[cfg(windows)]
fn test_parse_protocol_path_absolute_windows() {
    use std::path::PathBuf;

    let asset_root = PathBuf::from("C:\\temp\\assets");

    // Standard Windows absolute path
    let result = parse_protocol_path("C:/Users/test/file.txt", &asset_root);
    assert_eq!(result, PathBuf::from("C:/Users/test/file.txt"));

    // Windows path without colon (from URL) - lowercase drive letter
    let result = parse_protocol_path("c/users/test/file.txt", &asset_root);
    assert_eq!(result, PathBuf::from("C:/users/test/file.txt"));

    // Windows path without colon (from URL) - uppercase drive letter
    let result = parse_protocol_path("D/Projects/app/data.json", &asset_root);
    assert_eq!(result, PathBuf::from("D:/Projects/app/data.json"));

    // Windows path without colon - different drives
    let result = parse_protocol_path("e/temp/file.txt", &asset_root);
    assert_eq!(result, PathBuf::from("E:/temp/file.txt"));
}

#[rstest]
fn test_parse_protocol_path_with_dots() {
    use std::path::PathBuf;

    let asset_root = PathBuf::from("/tmp/assets");

    // Paths with .. should be cleaned
    let result = parse_protocol_path("css/../images/logo.png", &asset_root);
    assert_eq!(result, PathBuf::from("/tmp/assets/images/logo.png"));
}

#[rstest]
fn test_auroraview_protocol_with_windows_absolute_path() {
    // This test simulates the real-world scenario from the bug report
    // URL: auroraview://c/users/username/projects/myapp/0.1.0/assets/images/logo.gif

    let temp_dir = TempDir::new().unwrap();
    let asset_root = temp_dir.path();

    // Create a test file at an absolute Windows-style path
    // Note: In real scenario, this would be on C: drive, but for testing we use temp dir
    let test_file = temp_dir.path().join("test.gif");
    fs::write(&test_file, b"GIF89a").unwrap();

    // Simulate the URL path format (without drive colon)
    let path_str = test_file.to_str().unwrap();

    // On Windows, convert C:\path\to\file to c/path/to/file format
    #[cfg(windows)]
    let url_path = {
        let normalized = path_str.replace('\\', "/");
        // Remove the colon after drive letter to simulate URL format
        if normalized.len() > 2 && normalized.chars().nth(1) == Some(':') {
            let drive = normalized.chars().next().unwrap().to_ascii_lowercase();
            format!("{}{}", drive, &normalized[2..])
        } else {
            normalized
        }
    };

    #[cfg(unix)]
    let url_path = path_str.to_string();

    // Test that parse_protocol_path correctly handles this
    let parsed = parse_protocol_path(&url_path, asset_root);

    // The parsed path should be able to read the file
    assert!(
        parsed.exists(),
        "Parsed path {:?} should exist (from URL path: {})",
        parsed,
        url_path
    );
}
