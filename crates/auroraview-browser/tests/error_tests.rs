//! Tests for BrowserError variants, Display, Debug, From conversions, Send+Sync bounds

use auroraview_browser::BrowserError;
use rstest::rstest;
use std::io;

// ---------------------------------------------------------------------------
// Display formatting
// ---------------------------------------------------------------------------

#[test]
fn test_tab_not_found_display() {
    let err = BrowserError::TabNotFound("tab-1".to_string());
    assert_eq!(err.to_string(), "Tab not found: tab-1");
}

#[test]
fn test_bookmark_not_found_display() {
    let err = BrowserError::BookmarkNotFound("bm-42".to_string());
    assert_eq!(err.to_string(), "Bookmark not found: bm-42");
}

#[test]
fn test_extension_not_found_display() {
    let err = BrowserError::ExtensionNotFound("ext-id".to_string());
    assert_eq!(err.to_string(), "Extension not found: ext-id");
}

#[test]
fn test_webview_creation_display() {
    let err = BrowserError::WebViewCreation("init failed".to_string());
    assert_eq!(err.to_string(), "WebView creation failed: init failed");
}

#[test]
fn test_window_creation_display() {
    let err = BrowserError::WindowCreation("no HWND".to_string());
    assert_eq!(err.to_string(), "Window creation failed: no HWND");
}

#[test]
fn test_navigation_display() {
    let err = BrowserError::Navigation("404".to_string());
    assert_eq!(err.to_string(), "Navigation failed: 404");
}

#[test]
fn test_invalid_url_display() {
    let err = BrowserError::InvalidUrl("not-a-url".to_string());
    assert_eq!(err.to_string(), "Invalid URL: not-a-url");
}

#[test]
fn test_extension_error_display() {
    let err = BrowserError::Extension("disabled".to_string());
    assert_eq!(err.to_string(), "Extension error: disabled");
}

// ---------------------------------------------------------------------------
// Debug output contains variant name
// ---------------------------------------------------------------------------

#[test]
fn test_debug_contains_variant_names() {
    let cases: &[(&str, BrowserError)] = &[
        ("TabNotFound", BrowserError::TabNotFound("x".into())),
        (
            "BookmarkNotFound",
            BrowserError::BookmarkNotFound("x".into()),
        ),
        (
            "ExtensionNotFound",
            BrowserError::ExtensionNotFound("x".into()),
        ),
        (
            "WebViewCreation",
            BrowserError::WebViewCreation("x".into()),
        ),
        ("WindowCreation", BrowserError::WindowCreation("x".into())),
        ("Navigation", BrowserError::Navigation("x".into())),
        ("InvalidUrl", BrowserError::InvalidUrl("x".into())),
        ("Extension", BrowserError::Extension("x".into())),
    ];
    for (name, err) in cases {
        let debug = format!("{err:?}");
        assert!(
            debug.contains(name),
            "Debug output should contain '{name}', got: {debug}"
        );
    }
}

// ---------------------------------------------------------------------------
// From<io::Error>
// ---------------------------------------------------------------------------

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let browser_err: BrowserError = io_err.into();
    assert!(browser_err.to_string().contains("file missing"));
}

#[test]
fn test_from_io_error_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let browser_err: BrowserError = BrowserError::from(io_err);
    let msg = browser_err.to_string();
    assert!(msg.contains("access denied"));
}

// ---------------------------------------------------------------------------
// From<serde_json::Error>
// ---------------------------------------------------------------------------

#[test]
fn test_from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let browser_err: BrowserError = json_err.into();
    let msg = browser_err.to_string();
    assert!(msg.contains("Serialization") || msg.contains("expected"), "{msg}");
}

// ---------------------------------------------------------------------------
// Result alias
// ---------------------------------------------------------------------------

#[test]
fn test_result_alias_ok() {
    use auroraview_browser::Result;
    let r: Result<u32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[test]
fn test_result_alias_err() {
    use auroraview_browser::Result;
    let r: Result<u32> = Err(BrowserError::TabNotFound("t".into()));
    assert!(r.is_err());
}

// ---------------------------------------------------------------------------
// Send + Sync bounds
// ---------------------------------------------------------------------------

#[test]
fn test_send_sync_bounds() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BrowserError>();
}

// ---------------------------------------------------------------------------
// Parametrised: Display messages contain the inner string
// ---------------------------------------------------------------------------

#[rstest]
#[case(BrowserError::TabNotFound("abc".into()), "abc")]
#[case(BrowserError::BookmarkNotFound("def".into()), "def")]
#[case(BrowserError::ExtensionNotFound("ghi".into()), "ghi")]
#[case(BrowserError::WebViewCreation("jkl".into()), "jkl")]
#[case(BrowserError::WindowCreation("mno".into()), "mno")]
#[case(BrowserError::Navigation("pqr".into()), "pqr")]
#[case(BrowserError::InvalidUrl("stu".into()), "stu")]
#[case(BrowserError::Extension("vwx".into()), "vwx")]
fn test_display_contains_inner(#[case] err: BrowserError, #[case] expected: &str) {
    assert!(
        err.to_string().contains(expected),
        "Display should contain '{expected}', got: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// Error source chain (io/serde variants have a source)
// ---------------------------------------------------------------------------

#[test]
fn test_io_error_has_source() {
    use std::error::Error;
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let browser_err: BrowserError = io_err.into();
    assert!(browser_err.source().is_some());
}

#[test]
fn test_string_variants_have_no_source() {
    use std::error::Error;
    let err = BrowserError::TabNotFound("t".into());
    assert!(err.source().is_none());
}

// ---------------------------------------------------------------------------
// Multiple errors distinguish by message
// ---------------------------------------------------------------------------

#[test]
fn test_errors_distinguish_by_message() {
    let e1 = BrowserError::TabNotFound("tab-1".into());
    let e2 = BrowserError::TabNotFound("tab-2".into());
    assert_ne!(e1.to_string(), e2.to_string());
}

#[test]
fn test_different_variants_have_different_messages() {
    let e1 = BrowserError::TabNotFound("x".into());
    let e2 = BrowserError::BookmarkNotFound("x".into());
    assert_ne!(e1.to_string(), e2.to_string());
}

// ---------------------------------------------------------------------------
// Empty string payloads are handled gracefully
// ---------------------------------------------------------------------------

#[test]
fn test_empty_string_payload() {
    let err = BrowserError::TabNotFound(String::new());
    assert_eq!(err.to_string(), "Tab not found: ");
}

#[test]
fn test_long_string_payload() {
    let long = "x".repeat(1000);
    let err = BrowserError::Navigation(long.clone());
    assert!(err.to_string().contains(&long));
}
