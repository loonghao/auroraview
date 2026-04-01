//! Protocol tests

use std::path::Path;

use auroraview_core::protocol::{
    extract_protocol_path, file_url_to_auroraview, guess_mime_type, is_auroraview_url,
    local_path_to_auroraview, normalize_url, strip_protocol_type, FileResponse, MemoryAssets,
    StartupError, AURORAVIEW_HOST, PROTOCOL_TYPE_FILE, PROTOCOL_TYPE_LOCAL,
};

// ============================================================================
// URL Conversion Tests
// ============================================================================

#[test]
fn test_file_url_to_auroraview() {
    assert_eq!(
        file_url_to_auroraview("file:///C:/path/to/file.html"),
        "https://auroraview.localhost/type:file/C:/path/to/file.html"
    );
    assert_eq!(
        file_url_to_auroraview("file:///path/to/file.html"),
        "https://auroraview.localhost/type:file/path/to/file.html"
    );
    assert_eq!(
        file_url_to_auroraview("file:///C:\\Users\\test\\file.html"),
        "https://auroraview.localhost/type:file/C:/Users/test/file.html"
    );
}

#[test]
fn test_local_path_to_auroraview() {
    assert_eq!(
        local_path_to_auroraview("C:/path/to/file.html"),
        "https://auroraview.localhost/type:local/C:/path/to/file.html"
    );
    assert_eq!(
        local_path_to_auroraview("/path/to/file.html"),
        "https://auroraview.localhost/type:local/path/to/file.html"
    );
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
    assert_eq!(
        strip_protocol_type("type:file/path", PROTOCOL_TYPE_LOCAL),
        None
    );
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

// ============================================================================
// Legacy Tests (existing functionality)
// ============================================================================

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

// ============================================================================
// New Tests
// ============================================================================

// ---- FileResponse ----

#[test]
fn test_file_response_internal_error() {
    let resp = FileResponse::internal_error("Something went wrong");
    assert_eq!(resp.status, 500);
    assert_eq!(resp.mime_type, "text/plain");
    assert!(std::str::from_utf8(&resp.data).unwrap().contains("Something went wrong"));
}

#[test]
fn test_file_response_ok_content() {
    let data = b"<html><body>Hello</body></html>".to_vec();
    let resp = FileResponse::ok(data.clone(), "text/html".to_string());
    assert_eq!(resp.status, 200);
    assert_eq!(resp.mime_type, "text/html");
    assert_eq!(&*resp.data, &data[..]);
}

#[test]
fn test_file_response_not_found_mime() {
    let resp = FileResponse::not_found();
    assert_eq!(resp.mime_type, "text/plain");
}

#[test]
fn test_file_response_forbidden_mime() {
    let resp = FileResponse::forbidden();
    assert_eq!(resp.mime_type, "text/plain");
}

// ---- MIME type coverage ----

#[test]
fn test_guess_mime_type_more_types() {
    assert_eq!(guess_mime_type(Path::new("image.jpg")), "image/jpeg");
    assert_eq!(guess_mime_type(Path::new("image.jpeg")), "image/jpeg");
    assert_eq!(guess_mime_type(Path::new("image.gif")), "image/gif");
    assert_eq!(guess_mime_type(Path::new("image.svg")), "image/svg+xml");
    assert_eq!(guess_mime_type(Path::new("image.webp")), "image/webp");
    assert_eq!(guess_mime_type(Path::new("font.woff")), "application/font-woff");
    // woff2 mime varies by mime_guess version; accept either standard value
    let woff2_mime = guess_mime_type(Path::new("font.woff2"));
    assert!(
        woff2_mime == "font/woff2" || woff2_mime == "application/font-woff2",
        "unexpected woff2 mime: {}",
        woff2_mime
    );
    assert_eq!(guess_mime_type(Path::new("data.json")), "application/json");
    // Unknown extension falls back to octet-stream
    assert_eq!(
        guess_mime_type(Path::new("unknown.xyz123")),
        "application/octet-stream"
    );
}

// ---- MemoryAssets ----

#[test]
fn test_memory_assets_new_empty() {
    let assets = MemoryAssets::new();
    assert!(assets.is_empty());
    assert_eq!(assets.len(), 0);
}

#[test]
fn test_memory_assets_insert_and_retrieve() {
    let mut assets = MemoryAssets::new();
    assets.insert("index.html".to_string(), b"<html/>".to_vec());

    assert!(!assets.is_empty());
    assert_eq!(assets.len(), 1);

    let resp = assets.handle_request("index.html");
    assert_eq!(resp.status, 200);
    assert_eq!(&*resp.data, b"<html/>");
    assert!(resp.mime_type.contains("text/html"));
}

#[test]
fn test_memory_assets_default_to_index_html() {
    let mut assets = MemoryAssets::new();
    assets.insert("index.html".to_string(), b"root page".to_vec());

    // Empty path should serve index.html
    let resp = assets.handle_request("");
    assert_eq!(resp.status, 200);
    assert_eq!(&*resp.data, b"root page");

    // Slash-only path also serves index.html
    let resp2 = assets.handle_request("/");
    assert_eq!(resp2.status, 200);
}

#[test]
fn test_memory_assets_not_found() {
    let assets = MemoryAssets::new();
    let resp = assets.handle_request("nonexistent.html");
    assert_eq!(resp.status, 404);
}

#[test]
fn test_memory_assets_from_map() {
    let mut map = std::collections::HashMap::new();
    map.insert("a.js".to_string(), b"var a=1;".to_vec());
    map.insert("b.css".to_string(), b"body{}".to_vec());

    let assets = MemoryAssets::from_map(map);
    assert_eq!(assets.len(), 2);

    let resp = assets.handle_request("a.js");
    assert_eq!(resp.status, 200);
}

#[test]
fn test_memory_assets_from_vec() {
    let vec = vec![
        ("x.html".to_string(), b"<x/>".to_vec()),
        ("y.html".to_string(), b"<y/>".to_vec()),
    ];
    let assets = MemoryAssets::from_vec(vec);
    assert_eq!(assets.len(), 2);

    let resp = assets.handle_request("y.html");
    assert_eq!(resp.status, 200);
    assert_eq!(&*resp.data, b"<y/>");
}

#[test]
fn test_memory_assets_with_loading_html() {
    let assets = MemoryAssets::new().with_loading_html("<h1>Loading...</h1>".to_string());
    let resp = assets.handle_request("__loading__");
    assert_eq!(resp.status, 200);
    assert!(std::str::from_utf8(&resp.data).unwrap().contains("Loading"));
    assert!(resp.mime_type.contains("text/html"));
}

#[test]
fn test_memory_assets_loading_not_set() {
    let assets = MemoryAssets::new();
    // No loading HTML set — should return 404
    let resp = assets.handle_request("__loading__");
    assert_eq!(resp.status, 404);
}

#[test]
fn test_memory_assets_startup_error() {
    let mut assets = MemoryAssets::new();
    assets.set_startup_error(StartupError {
        message: "Python import failed".to_string(),
        python_output: Some("Traceback...".to_string()),
        entry_point: Some("main:run".to_string()),
    });

    let resp = assets.handle_request("__startup_error__");
    assert_eq!(resp.status, 500);
    assert!(resp.mime_type.contains("text/html"));
    let body = std::str::from_utf8(&resp.data).unwrap();
    assert!(body.contains("Python import failed"));
}

#[test]
fn test_memory_assets_clear_startup_error() {
    let mut assets = MemoryAssets::new();
    assets.set_startup_error(StartupError {
        message: "err".to_string(),
        python_output: None,
        entry_point: None,
    });

    assets.clear_startup_error();
    let resp = assets.handle_request("__startup_error__");
    // After clear, __startup_error__ should not be served
    assert_ne!(resp.status, 500);
}

#[test]
fn test_memory_assets_list_paths() {
    let mut assets = MemoryAssets::new();
    assets.insert("p1.html".to_string(), vec![]);
    assets.insert("p2.js".to_string(), vec![]);

    let paths = assets.list_paths();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.as_str() == "p1.html"));
    assert!(paths.iter().any(|p| p.as_str() == "p2.js"));
}

#[test]
fn test_memory_assets_default() {
    let assets = MemoryAssets::default();
    assert!(assets.is_empty());
}

// ---- MemoryAssets frontend/ prefix fallback ----

#[test]
fn test_memory_assets_frontend_prefix_fallback() {
    let mut assets = MemoryAssets::new();
    assets.insert("frontend/index.html".to_string(), b"frontend page".to_vec());

    // Requesting "index.html" should find "frontend/index.html"
    let resp = assets.handle_request("index.html");
    assert_eq!(resp.status, 200);
    assert_eq!(&*resp.data, b"frontend page");
}

// ---- normalize_url edge cases ----

#[test]
fn test_normalize_url_with_whitespace() {
    assert_eq!(normalize_url("  example.com  "), "https://example.com");
}

#[test]
fn test_normalize_url_auroraview_protocol() {
    let url = "auroraview://localhost/index.html";
    // Has "://" so should be kept as-is
    assert_eq!(normalize_url(url), url);
}

// ---- is_auroraview_url edge cases ----

#[test]
fn test_is_auroraview_url_http_localhost() {
    // http://localhost is NOT an auroraview URL
    assert!(!is_auroraview_url("http://localhost:3000"));
}

#[test]
fn test_is_auroraview_url_type_local() {
    assert!(is_auroraview_url(
        "https://auroraview.localhost/type:local/dist/index.html"
    ));
}

// ---- file_url_to_auroraview edge cases ----

#[test]
fn test_file_url_to_auroraview_without_triple_slash() {
    // file:// (without extra slash for UNC paths)
    let result = file_url_to_auroraview("file://server/share/file.html");
    assert!(result.contains("auroraview.localhost"));
    assert!(result.contains("type:file"));
}
