//! Protocol tests

use auroraview_core::protocol::{
    extract_protocol_path, file_url_to_auroraview, guess_mime_type, is_auroraview_url,
    local_path_to_auroraview, normalize_url, strip_protocol_type, FileResponse, AURORAVIEW_HOST,
    PROTOCOL_TYPE_FILE, PROTOCOL_TYPE_LOCAL,
};
use std::path::Path;

// ========================================================================
// URL Conversion Tests
// ========================================================================

#[test]
fn test_file_url_to_auroraview() {
    // file:// URLs should use type:file prefix
    assert_eq!(
        file_url_to_auroraview("file:///C:/path/to/file.html"),
        "https://auroraview.localhost/type:file/C:/path/to/file.html"
    );
    assert_eq!(
        file_url_to_auroraview("file:///path/to/file.html"),
        "https://auroraview.localhost/type:file/path/to/file.html"
    );
    // Windows backslashes in file:// URL
    assert_eq!(
        file_url_to_auroraview("file:///C:\\Users\\test\\file.html"),
        "https://auroraview.localhost/type:file/C:/Users/test/file.html"
    );
}

#[test]
fn test_local_path_to_auroraview() {
    // Local paths should use type:local prefix
    assert_eq!(
        local_path_to_auroraview("C:/path/to/file.html"),
        "https://auroraview.localhost/type:local/C:/path/to/file.html"
    );
    assert_eq!(
        local_path_to_auroraview("/path/to/file.html"),
        "https://auroraview.localhost/type:local/path/to/file.html"
    );
    // Windows backslashes should be normalized
    assert_eq!(
        local_path_to_auroraview("C:\\Users\\test\\file.html"),
        "https://auroraview.localhost/type:local/C:/Users/test/file.html"
    );
}

#[test]
fn test_strip_protocol_type() {
    assert_eq!(
        strip_protocol_type("type:file/C:/path/to/file.html", PROTOCOL_TYPE_FILE),
        Some("C:/path/to/file.html")
    );
    assert_eq!(
        strip_protocol_type("type:local/path/to/file.html", PROTOCOL_TYPE_LOCAL),
        Some("path/to/file.html")
    );
    // Wrong prefix
    assert_eq!(
        strip_protocol_type("type:file/path", PROTOCOL_TYPE_LOCAL),
        None
    );
    // No prefix
    assert_eq!(
        strip_protocol_type("path/to/file.html", PROTOCOL_TYPE_FILE),
        None
    );
}

#[test]
fn test_is_auroraview_url() {
    assert!(is_auroraview_url(
        "https://auroraview.localhost/type:file/C:/path"
    ));
    assert!(is_auroraview_url("https://auroraview.localhost/index.html"));
    assert!(is_auroraview_url("auroraview://localhost/index.html"));
    assert!(!is_auroraview_url("https://example.com"));
    assert!(!is_auroraview_url("file:///C:/path/to/file.html"));
}

#[test]
fn test_protocol_constants() {
    assert_eq!(AURORAVIEW_HOST, "auroraview.localhost");
    assert_eq!(PROTOCOL_TYPE_FILE, "type:file");
    assert_eq!(PROTOCOL_TYPE_LOCAL, "type:local");
}

// ========================================================================
// Legacy Tests (existing functionality)
// ========================================================================

#[test]
fn test_normalize_url() {
    assert_eq!(normalize_url("example.com"), "https://example.com");
    assert_eq!(normalize_url("https://example.com"), "https://example.com");
    assert_eq!(normalize_url("http://example.com"), "http://example.com");
    assert_eq!(normalize_url("file:///path"), "file:///path");
    assert_eq!(normalize_url(""), "");
}

#[test]
fn test_extract_protocol_path() {
    assert_eq!(
        extract_protocol_path("auroraview://localhost/index.html", "auroraview"),
        Some("index.html".to_string())
    );
    assert_eq!(
        extract_protocol_path("auroraview://localhost", "auroraview"),
        Some("index.html".to_string())
    );
    assert_eq!(
        extract_protocol_path("https://auroraview.localhost/css/style.css", "auroraview"),
        Some("css/style.css".to_string())
    );
    assert_eq!(
        extract_protocol_path("auroraview://path/to/file", "auroraview"),
        Some("path/to/file".to_string())
    );
    assert_eq!(
        extract_protocol_path("http://example.com", "auroraview"),
        None
    );
}

#[test]
fn test_guess_mime_type() {
    assert_eq!(guess_mime_type(Path::new("style.css")), "text/css");
    assert_eq!(guess_mime_type(Path::new("script.js")), "text/javascript");
    assert_eq!(guess_mime_type(Path::new("index.html")), "text/html");
    assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
}

#[test]
fn test_file_response() {
    let resp = FileResponse::ok(b"hello".to_vec(), "text/plain".to_string());
    assert_eq!(resp.status, 200);

    let resp = FileResponse::not_found();
    assert_eq!(resp.status, 404);

    let resp = FileResponse::forbidden();
    assert_eq!(resp.status, 403);
}
