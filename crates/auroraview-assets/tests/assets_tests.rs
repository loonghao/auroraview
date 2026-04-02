//! Tests for auroraview-assets public API.
//!
//! These tests cover Page enum, AssetError, MIME type detection, and the
//! non-embed functions that do not require a built frontend/dist directory.

use auroraview_assets::{AssetError, Page, asset_exists, get_asset, get_mime_type, list_assets};
use rstest::rstest;

// ---------------------------------------------------------------------------
// Page enum
// ---------------------------------------------------------------------------

#[test]
fn test_page_html_paths() {
    assert_eq!(Page::Loading.html_path(), "loading/index.html");
    assert_eq!(Page::Error.html_path(), "error/index.html");
    assert_eq!(Page::Browser.html_path(), "browser/index.html");
    assert_eq!(
        Page::BrowserController.html_path(),
        "browser-controller/index.html"
    );
}

#[test]
fn test_page_all_returns_four() {
    assert_eq!(Page::all().len(), 4);
}

#[test]
fn test_page_all_contains_all_variants() {
    let all = Page::all();
    assert!(all.contains(&Page::Loading));
    assert!(all.contains(&Page::Error));
    assert!(all.contains(&Page::Browser));
    assert!(all.contains(&Page::BrowserController));
}

#[test]
fn test_page_clone() {
    let p = Page::Loading;
    let q = p;
    assert_eq!(p, q);
}

#[test]
fn test_page_debug() {
    let debug = format!("{:?}", Page::Error);
    assert!(debug.contains("Error"), "{debug}");
}

#[test]
fn test_page_eq() {
    assert_eq!(Page::Browser, Page::Browser);
    assert_ne!(Page::Browser, Page::Error);
}

#[test]
fn test_page_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Page::Loading);
    set.insert(Page::Error);
    set.insert(Page::Browser);
    set.insert(Page::BrowserController);
    assert_eq!(set.len(), 4);
}

// ---------------------------------------------------------------------------
// AssetError
// ---------------------------------------------------------------------------

#[test]
fn test_asset_error_not_found_display() {
    let err = AssetError::NotFound("missing.js".to_string());
    assert!(err.to_string().contains("missing.js"));
    assert!(err.to_string().contains("not found") || err.to_string().contains("Asset"));
}

#[test]
fn test_asset_error_invalid_utf8_display() {
    let err = AssetError::InvalidUtf8("bad.bin".to_string());
    assert!(err.to_string().contains("bad.bin"));
}

#[test]
fn test_asset_error_debug() {
    let err = AssetError::NotFound("x.html".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("NotFound") || debug.contains("x.html"), "{debug}");
}

#[test]
fn test_asset_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AssetError>();
}

// ---------------------------------------------------------------------------
// MIME type detection
// ---------------------------------------------------------------------------

#[rstest]
#[case("index.html", "text/html")]
#[case("app.js", "text/javascript")]
#[case("styles.css", "text/css")]
#[case("logo.svg", "image/svg+xml")]
#[case("favicon.ico", "image/x-icon")]
#[case("font.woff2", "font/woff2")]
#[case("data.json", "application/json")]
fn test_get_mime_type(#[case] path: &str, #[case] expected: &str) {
    assert_eq!(get_mime_type(path), expected, "MIME mismatch for {path}");
}

#[test]
fn test_get_mime_type_unknown_returns_octet_stream() {
    // .avunk is not a registered MIME type
    let mime = get_mime_type("file.avunk");
    assert_eq!(mime, "application/octet-stream");
}

#[test]
fn test_get_mime_type_no_extension() {
    // A file with no extension returns octet-stream fallback
    let mime = get_mime_type("Makefile");
    assert!(!mime.is_empty());
}

#[test]
fn test_get_mime_type_uppercase_extension() {
    // mime_guess is case-insensitive on most platforms
    let mime = get_mime_type("IMAGE.PNG");
    assert!(mime.contains("image") || mime.contains("octet"), "{mime}");
}

// ---------------------------------------------------------------------------
// get_asset / asset_exists (no real files needed – they return None/false)
// ---------------------------------------------------------------------------

#[test]
fn test_get_asset_missing_returns_none() {
    let result = get_asset("nonexistent/path/file.html");
    assert!(result.is_none());
}

#[test]
fn test_asset_exists_missing_returns_false() {
    assert!(!asset_exists("nonexistent/path.html"));
}

// list_assets returns a Vec (may be empty if dist not built, but must not panic)
#[test]
fn test_list_assets_returns_vec() {
    let assets = list_assets();
    // Just check it's a Vec, not that it has specific contents (requires dist build)
    let _ = assets;
}

// ---------------------------------------------------------------------------
// Page html_path coverage via parametrised test
// ---------------------------------------------------------------------------

#[rstest]
#[case(Page::Loading, "loading/index.html")]
#[case(Page::Error, "error/index.html")]
#[case(Page::Browser, "browser/index.html")]
#[case(Page::BrowserController, "browser-controller/index.html")]
fn test_page_html_path_parametrised(#[case] page: Page, #[case] expected: &str) {
    assert_eq!(page.html_path(), expected);
}
