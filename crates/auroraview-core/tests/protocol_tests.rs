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
fn file_url_to_auroraview_converts_paths() {
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
fn local_path_to_auroraview_converts_paths() {
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
fn strip_protocol_type_extracts_expected_prefix() {
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
fn is_auroraview_url_detects_supported_urls() {
    assert!(is_auroraview_url(
        "https://auroraview.localhost/type:file/C:/path"
    ));
    assert!(is_auroraview_url("https://auroraview.localhost/index.html"));
    assert!(is_auroraview_url("auroraview://localhost/index.html"));
    assert!(!is_auroraview_url("https://example.com"));
    assert!(!is_auroraview_url("file:///C:/path/to/file.html"));
}

#[test]
fn protocol_constants() {
    assert_eq!(AURORAVIEW_HOST, "auroraview.localhost");
    assert_eq!(PROTOCOL_TYPE_FILE, "type:file");
    assert_eq!(PROTOCOL_TYPE_LOCAL, "type:local");
}

// ============================================================================
// Legacy Tests (existing functionality)
// ============================================================================

#[test]
fn normalize_url_handles_common_inputs() {
    assert_eq!(normalize_url("example.com"), "https://example.com");
    assert_eq!(normalize_url("https://example.com"), "https://example.com");
    assert_eq!(normalize_url("http://example.com"), "http://example.com");
    assert_eq!(normalize_url("file:///path"), "file:///path");
    assert_eq!(normalize_url(""), "");
}

#[test]
fn extract_protocol_path_parses_supported_forms() {
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
fn guess_mime_type_for_common_assets() {
    assert_eq!(guess_mime_type(Path::new("style.css")), "text/css");
    assert_eq!(guess_mime_type(Path::new("script.js")), "text/javascript");
    assert_eq!(guess_mime_type(Path::new("index.html")), "text/html");
    assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
}

#[test]
fn file_response() {
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
fn file_response_internal_error() {
    let resp = FileResponse::internal_error("Something went wrong");
    assert_eq!(resp.status, 500);
    assert_eq!(resp.mime_type, "text/plain");
    assert!(std::str::from_utf8(&resp.data)
        .unwrap()
        .contains("Something went wrong"));
}

#[test]
fn file_response_ok_content() {
    let data = b"<html><body>Hello</body></html>".to_vec();
    let resp = FileResponse::ok(data.clone(), "text/html".to_string());
    assert_eq!(resp.status, 200);
    assert_eq!(resp.mime_type, "text/html");
    assert_eq!(&*resp.data, &data[..]);
}

#[test]
fn file_response_not_found_mime() {
    let resp = FileResponse::not_found();
    assert_eq!(resp.mime_type, "text/plain");
}

#[test]
fn file_response_forbidden_mime() {
    let resp = FileResponse::forbidden();
    assert_eq!(resp.mime_type, "text/plain");
}

// ---- MIME type coverage ----

#[test]
fn guess_mime_type_more_types() {
    assert_eq!(guess_mime_type(Path::new("image.jpg")), "image/jpeg");
    assert_eq!(guess_mime_type(Path::new("image.jpeg")), "image/jpeg");
    assert_eq!(guess_mime_type(Path::new("image.gif")), "image/gif");
    assert_eq!(guess_mime_type(Path::new("image.svg")), "image/svg+xml");
    assert_eq!(guess_mime_type(Path::new("image.webp")), "image/webp");
    assert_eq!(
        guess_mime_type(Path::new("font.woff")),
        "application/font-woff"
    );
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
fn memory_assets_new_empty() {
    let assets = MemoryAssets::new();
    assert!(assets.is_empty());
    assert_eq!(assets.len(), 0);
}

#[test]
fn memory_assets_insert_and_retrieve() {
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
fn memory_assets_default_to_index_html() {
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
fn memory_assets_not_found() {
    let assets = MemoryAssets::new();
    let resp = assets.handle_request("nonexistent.html");
    assert_eq!(resp.status, 404);
}

#[test]
fn memory_assets_from_map() {
    let mut map = std::collections::HashMap::new();
    map.insert("a.js".to_string(), b"var a=1;".to_vec());
    map.insert("b.css".to_string(), b"body{}".to_vec());

    let assets = MemoryAssets::from_map(map);
    assert_eq!(assets.len(), 2);

    let resp = assets.handle_request("a.js");
    assert_eq!(resp.status, 200);
}

#[test]
fn memory_assets_from_vec() {
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
fn memory_assets_with_loading_html() {
    let assets = MemoryAssets::new().with_loading_html("<h1>Loading...</h1>".to_string());
    let resp = assets.handle_request("__loading__");
    assert_eq!(resp.status, 200);
    assert!(std::str::from_utf8(&resp.data).unwrap().contains("Loading"));
    assert!(resp.mime_type.contains("text/html"));
}

#[test]
fn memory_assets_loading_not_set() {
    let assets = MemoryAssets::new();
    // No loading HTML set — should return 404
    let resp = assets.handle_request("__loading__");
    assert_eq!(resp.status, 404);
}

#[test]
fn memory_assets_startup_error() {
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
fn memory_assets_clear_startup_error() {
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
fn memory_assets_list_paths() {
    let mut assets = MemoryAssets::new();
    assets.insert("p1.html".to_string(), vec![]);
    assets.insert("p2.js".to_string(), vec![]);

    let paths = assets.list_paths();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.as_str() == "p1.html"));
    assert!(paths.iter().any(|p| p.as_str() == "p2.js"));
}

#[test]
fn memory_assets_default() {
    let assets = MemoryAssets::default();
    assert!(assets.is_empty());
}

// ---- MemoryAssets frontend/ prefix fallback ----

#[test]
fn memory_assets_frontend_prefix_fallback() {
    let mut assets = MemoryAssets::new();
    assets.insert("frontend/index.html".to_string(), b"frontend page".to_vec());

    // Requesting "index.html" should find "frontend/index.html"
    let resp = assets.handle_request("index.html");
    assert_eq!(resp.status, 200);
    assert_eq!(&*resp.data, b"frontend page");
}

// ---- normalize_url edge cases ----

#[test]
fn normalize_url_with_whitespace() {
    assert_eq!(normalize_url("  example.com  "), "https://example.com");
}

#[test]
fn normalize_url_auroraview_protocol() {
    let url = "auroraview://localhost/index.html";
    // Has "://" so should be kept as-is
    assert_eq!(normalize_url(url), url);
}

// ---- is_auroraview_url edge cases ----

#[test]
fn is_auroraview_url_http_localhost() {
    // http://localhost is NOT an auroraview URL
    assert!(!is_auroraview_url("http://localhost:3000"));
}

#[test]
fn is_auroraview_url_type_local() {
    assert!(is_auroraview_url(
        "https://auroraview.localhost/type:local/dist/index.html"
    ));
}

// ---- file_url_to_auroraview edge cases ----

#[test]
fn file_url_to_auroraview_without_triple_slash() {
    // file:// (without extra slash for UNC paths)
    let result = file_url_to_auroraview("file://server/share/file.html");
    assert!(result.contains("auroraview.localhost"));
    assert!(result.contains("type:file"));
}

// ---- normalize_url additional edge cases ----

#[test]
fn normalize_url_data_uri_gets_https_prefix() {
    // normalize_url only checks for "://" — "data:" has no "://" so gets https:// prefix
    let data_uri = "data:text/html,<h1>test</h1>";
    let result = normalize_url(data_uri);
    assert_eq!(result, format!("https://{}", data_uri));
}

#[test]
fn normalize_url_javascript_scheme_gets_https_prefix() {
    // "javascript:" has no "://" so gets https:// prefix
    let js_uri = "javascript:void(0)";
    let result = normalize_url(js_uri);
    assert_eq!(result, format!("https://{}", js_uri));
}

#[test]
fn normalize_url_about_blank_gets_https_prefix() {
    // "about:" has no "://" so gets https:// prefix
    let about = "about:blank";
    let result = normalize_url(about);
    assert_eq!(result, format!("https://{}", about));
}

#[test]
fn normalize_url_uppercase_http() {
    // URLs already containing "://" should pass through unchanged
    let url = "HTTP://example.com/page";
    let result = normalize_url(url);
    // Has "://" so returned as-is (trimmed)
    assert_eq!(result, url);
}

// ---- file_url_to_auroraview: percent-encoded path ----

#[test]
fn file_url_to_auroraview_percent_encoded_spaces() {
    let result = file_url_to_auroraview("file:///C:/My%20Documents/file.html");
    assert!(result.contains("auroraview.localhost"));
    assert!(result.contains("type:file"));
}

// ---- local_path_to_auroraview: UNC / edge cases ----

#[test]
fn local_path_to_auroraview_trailing_slash() {
    // Trailing slash should still produce a valid URL
    let result = local_path_to_auroraview("C:/path/to/dir/");
    assert!(result.contains("auroraview.localhost"));
    assert!(result.contains("type:local"));
}

#[test]
fn local_path_to_auroraview_deep_path() {
    let result = local_path_to_auroraview("C:/a/b/c/d/e/f/index.html");
    assert_eq!(
        result,
        "https://auroraview.localhost/type:local/C:/a/b/c/d/e/f/index.html"
    );
}

// ---- strip_protocol_type: exhaustive ----

#[test]
fn strip_protocol_type_empty_path_after_prefix() {
    // "type:file/" with nothing after → empty string
    let result = strip_protocol_type("type:file/", PROTOCOL_TYPE_FILE);
    assert_eq!(result, Some(""));
}

#[test]
fn strip_protocol_type_wrong_prefix_partial_match() {
    // "type:filee/path" is not a valid file prefix
    let result = strip_protocol_type("type:filee/path", PROTOCOL_TYPE_FILE);
    assert_eq!(result, None);
}

// ---- is_auroraview_url: more variants ----

#[test]
fn is_auroraview_url_with_query_and_fragment() {
    assert!(is_auroraview_url(
        "https://auroraview.localhost/index.html?v=1#section"
    ));
}

#[test]
fn is_auroraview_url_empty_string() {
    assert!(!is_auroraview_url(""));
}

#[test]
fn is_auroraview_url_localhost_no_scheme() {
    // Contains "auroraview.localhost" string → returns true even without scheme
    assert!(is_auroraview_url("auroraview.localhost/index.html"));
}

// ---- MemoryAssets concurrent access ----

#[test]
fn memory_assets_concurrent_insert_and_read() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    // Wrap MemoryAssets in RwLock for concurrent test
    let assets = Arc::new(RwLock::new(MemoryAssets::new()));

    // Pre-populate
    {
        let mut w = assets.write().unwrap();
        w.insert("shared.html".to_string(), b"<shared/>".to_vec());
    }

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let a = assets.clone();
            thread::spawn(move || {
                if i % 2 == 0 {
                    let r = a.read().unwrap();
                    let resp = r.handle_request("shared.html");
                    assert_eq!(resp.status, 200);
                } else {
                    let mut w = a.write().unwrap();
                    w.insert(format!("file{i}.js"), format!("var x{i}=1;").into_bytes());
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// ---- MemoryAssets mime-type edge cases ----

#[test]
fn memory_assets_binary_file_mime() {
    let mut assets = MemoryAssets::new();
    assets.insert("img.png".to_string(), vec![0x89, 0x50, 0x4E, 0x47]);
    let resp = assets.handle_request("img.png");
    assert_eq!(resp.status, 200);
    assert!(resp.mime_type.contains("image/png"));
}

#[test]
fn memory_assets_woff2_mime() {
    let mut assets = MemoryAssets::new();
    assets.insert("font.woff2".to_string(), b"fake_woff2".to_vec());
    let resp = assets.handle_request("font.woff2");
    assert_eq!(resp.status, 200);
    let mt = &resp.mime_type;
    assert!(
        mt.contains("woff2") || mt.contains("font"),
        "unexpected mime: {}",
        mt
    );
}

#[test]
fn memory_assets_unknown_ext_octet_stream() {
    let mut assets = MemoryAssets::new();
    assets.insert("data.bin99".to_string(), b"binary".to_vec());
    let resp = assets.handle_request("data.bin99");
    assert_eq!(resp.status, 200);
    assert_eq!(resp.mime_type, "application/octet-stream");
}

// ---- extract_protocol_path: additional patterns ----

#[test]
fn extract_protocol_path_trailing_slash() {
    // "https://auroraview.localhost/" → strip_prefix returns empty string ""
    // (only "auroraview://localhost" without trailing slash maps to "index.html")
    let result = extract_protocol_path("https://auroraview.localhost/", "auroraview");
    assert_eq!(result, Some("".to_string()));
}

#[test]
fn extract_protocol_path_nested_segments() {
    let result = extract_protocol_path(
        "https://auroraview.localhost/static/icons/arrow.svg",
        "auroraview",
    );
    assert_eq!(result, Some("static/icons/arrow.svg".to_string()));
}

#[test]
fn extract_protocol_path_query_string_included() {
    // Query strings are part of the path string
    let result = extract_protocol_path("https://auroraview.localhost/api/data?v=1", "auroraview");
    // Should return the path portion (implementation specific, but not None)
    assert!(result.is_some());
}

// ---- FileResponse: edge byte content ----

#[test]
fn file_response_empty_body() {
    let resp = FileResponse::ok(vec![], "text/html".to_string());
    assert_eq!(resp.status, 200);
    assert!(resp.data.is_empty());
}

#[test]
fn file_response_large_body() {
    let data = vec![b'A'; 1024 * 1024]; // 1 MiB
    let resp = FileResponse::ok(data.clone(), "application/octet-stream".to_string());
    assert_eq!(resp.status, 200);
    assert_eq!(resp.data.len(), 1024 * 1024);
}

// ---- StartupError: python_output and entry_point variation ----

#[test]
fn memory_assets_startup_error_with_all_fields() {
    let mut assets = MemoryAssets::new();
    assets.set_startup_error(StartupError {
        message: "Module not found".to_string(),
        python_output: Some("ModuleNotFoundError: No module named 'foo'".to_string()),
        entry_point: Some("app.main:run".to_string()),
    });

    let resp = assets.handle_request("__startup_error__");
    assert_eq!(resp.status, 500);
    let body = std::str::from_utf8(&resp.data).unwrap();
    assert!(body.contains("Module not found"));
}

#[test]
fn memory_assets_startup_error_no_python_output() {
    let mut assets = MemoryAssets::new();
    assets.set_startup_error(StartupError {
        message: "Init failed".to_string(),
        python_output: None,
        entry_point: None,
    });

    let resp = assets.handle_request("__startup_error__");
    assert_eq!(resp.status, 500);
    let body = std::str::from_utf8(&resp.data).unwrap();
    assert!(body.contains("Init failed"));
}

// ---- MemoryAssets list_paths after operations ----

#[test]
fn memory_assets_list_paths_after_overwrite() {
    let mut assets = MemoryAssets::new();
    assets.insert("file.txt".to_string(), b"v1".to_vec());
    assets.insert("file.txt".to_string(), b"v2".to_vec()); // overwrite

    let paths = assets.list_paths();
    // Should still only have one entry for this key
    let count = paths.iter().filter(|p| p.as_str() == "file.txt").count();
    assert_eq!(count, 1);
}

// ---- guess_mime_type: more types ----

#[test]
fn guess_mime_type_ico() {
    let mt = guess_mime_type(Path::new("favicon.ico"));
    // Accepts both common values
    assert!(
        mt.contains("ico") || mt.contains("image/x-icon") || mt.contains("image/vnd"),
        "unexpected ico mime: {}",
        mt
    );
}

#[test]
fn guess_mime_type_mp4() {
    let mt = guess_mime_type(Path::new("video.mp4"));
    assert!(mt.contains("video"), "unexpected mp4 mime: {}", mt);
}

#[test]
fn guess_mime_type_wasm() {
    let mt = guess_mime_type(Path::new("module.wasm"));
    assert!(
        mt.contains("wasm") || mt.contains("octet-stream"),
        "unexpected wasm mime: {}",
        mt
    );
}
