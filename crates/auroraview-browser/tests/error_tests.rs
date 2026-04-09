//! Tests for BrowserError variants, Display, Debug, From conversions, Send+Sync bounds

use std::io;

use auroraview_browser::BrowserError;
use rstest::rstest;

// ---------------------------------------------------------------------------
// Display formatting
// ---------------------------------------------------------------------------

#[rstest]
fn tab_not_found_display() {
    let err = BrowserError::TabNotFound("tab-1".to_string());
    assert_eq!(err.to_string(), "Tab not found: tab-1");
}

#[rstest]
fn bookmark_not_found_display() {
    let err = BrowserError::BookmarkNotFound("bm-42".to_string());
    assert_eq!(err.to_string(), "Bookmark not found: bm-42");
}

#[rstest]
fn extension_not_found_display() {
    let err = BrowserError::ExtensionNotFound("ext-id".to_string());
    assert_eq!(err.to_string(), "Extension not found: ext-id");
}

#[rstest]
fn webview_creation_display() {
    let err = BrowserError::WebViewCreation("init failed".to_string());
    assert_eq!(err.to_string(), "WebView creation failed: init failed");
}

#[rstest]
fn window_creation_display() {
    let err = BrowserError::WindowCreation("no HWND".to_string());
    assert_eq!(err.to_string(), "Window creation failed: no HWND");
}

#[rstest]
fn navigation_display() {
    let err = BrowserError::Navigation("404".to_string());
    assert_eq!(err.to_string(), "Navigation failed: 404");
}

#[rstest]
fn invalid_url_display() {
    let err = BrowserError::InvalidUrl("not-a-url".to_string());
    assert_eq!(err.to_string(), "Invalid URL: not-a-url");
}

#[rstest]
fn extension_error_display() {
    let err = BrowserError::Extension("disabled".to_string());
    assert_eq!(err.to_string(), "Extension error: disabled");
}

// ---------------------------------------------------------------------------
// Debug output contains variant name
// ---------------------------------------------------------------------------

#[rstest]
fn debug_contains_variant_names() {
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

#[rstest]
fn from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let browser_err: BrowserError = io_err.into();
    assert!(browser_err.to_string().contains("file missing"));
}

#[rstest]
fn from_io_error_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let browser_err: BrowserError = BrowserError::from(io_err);
    let msg = browser_err.to_string();
    assert!(msg.contains("access denied"));
}

// ---------------------------------------------------------------------------
// From<serde_json::Error>
// ---------------------------------------------------------------------------

#[rstest]
fn from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let browser_err: BrowserError = json_err.into();
    let msg = browser_err.to_string();
    assert!(msg.contains("Serialization") || msg.contains("expected"), "{msg}");
}

// ---------------------------------------------------------------------------
// Result alias
// ---------------------------------------------------------------------------

#[rstest]
fn result_alias_ok() {
    use auroraview_browser::Result;
    let r: Result<u32> = Ok(42);
    assert!(matches!(r, Ok(42)));
}


#[rstest]
fn result_alias_err() {
    use auroraview_browser::Result;
    let r: Result<u32> = Err(BrowserError::TabNotFound("t".into()));
    assert!(r.is_err());
}

// ---------------------------------------------------------------------------
// Send + Sync bounds
// ---------------------------------------------------------------------------

#[rstest]
fn send_sync_bounds() {
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
fn display_contains_inner(#[case] err: BrowserError, #[case] expected: &str) {
    assert!(
        err.to_string().contains(expected),
        "Display should contain '{expected}', got: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// Error source chain (io/serde variants have a source)
// ---------------------------------------------------------------------------

#[rstest]
fn io_error_has_source() {
    use std::error::Error;
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let browser_err: BrowserError = io_err.into();
    assert!(browser_err.source().is_some());
}

#[rstest]
fn string_variants_have_no_source() {
    use std::error::Error;
    let err = BrowserError::TabNotFound("t".into());
    assert!(err.source().is_none());
}

// ---------------------------------------------------------------------------
// Multiple errors distinguish by message
// ---------------------------------------------------------------------------

#[rstest]
fn errors_distinguish_by_message() {
    let e1 = BrowserError::TabNotFound("tab-1".into());
    let e2 = BrowserError::TabNotFound("tab-2".into());
    assert_ne!(e1.to_string(), e2.to_string());
}

#[rstest]
fn different_variants_have_different_messages() {
    let e1 = BrowserError::TabNotFound("x".into());
    let e2 = BrowserError::BookmarkNotFound("x".into());
    assert_ne!(e1.to_string(), e2.to_string());
}

// ---------------------------------------------------------------------------
// Empty string payloads are handled gracefully
// ---------------------------------------------------------------------------

#[rstest]
fn empty_string_payload() {
    let err = BrowserError::TabNotFound(String::new());
    assert_eq!(err.to_string(), "Tab not found: ");
}

#[rstest]
fn long_string_payload() {
    let long = "x".repeat(1000);
    let err = BrowserError::Navigation(long.clone());
    assert!(err.to_string().contains(&long));
}

// ---------------------------------------------------------------------------
// Unicode and special-character payloads
// ---------------------------------------------------------------------------

#[rstest]
fn unicode_payload_chinese() {
    let err = BrowserError::TabNotFound("标签页-中文".to_string());
    assert!(err.to_string().contains("标签页-中文"));
}

#[rstest]
fn unicode_payload_emoji() {
    let err = BrowserError::Navigation("🚀 rocket navigation".to_string());
    assert!(err.to_string().contains("🚀"));
}

#[rstest]
fn special_chars_payload() {
    let payload = r#"<script>alert("xss")</script>"#;
    let err = BrowserError::InvalidUrl(payload.to_string());
    assert!(err.to_string().contains(payload));
}

#[rstest]
fn newline_in_payload() {
    let err = BrowserError::Extension("line1\nline2".to_string());
    let s = err.to_string();
    assert!(s.contains("line1"));
    assert!(s.contains("line2"));
}

// ---------------------------------------------------------------------------
// IoError — various ErrorKind values
// ---------------------------------------------------------------------------

#[rstest]
#[case(std::io::ErrorKind::TimedOut, "timed out")]
#[case(std::io::ErrorKind::ConnectionRefused, "connection refused")]
#[case(std::io::ErrorKind::WouldBlock, "would block")]
fn from_io_various_kinds(#[case] kind: std::io::ErrorKind, #[case] msg: &str) {
    let io_err = io::Error::new(kind, msg);
    let browser_err: BrowserError = io_err.into();
    assert!(
        browser_err.to_string().contains(msg),
        "Expected '{}' in: {}",
        msg,
        browser_err
    );
}

// ---------------------------------------------------------------------------
// All string variants produce non-empty Display output
// ---------------------------------------------------------------------------

#[rstest]
#[case(BrowserError::TabNotFound("t".into()))]
#[case(BrowserError::BookmarkNotFound("b".into()))]
#[case(BrowserError::ExtensionNotFound("e".into()))]
#[case(BrowserError::WebViewCreation("w".into()))]
#[case(BrowserError::WindowCreation("win".into()))]
#[case(BrowserError::Navigation("nav".into()))]
#[case(BrowserError::InvalidUrl("url".into()))]
#[case(BrowserError::Extension("ext".into()))]
fn all_variants_non_empty_display(#[case] err: BrowserError) {
    let s = err.to_string();
    assert!(!s.is_empty(), "Display output should not be empty: {:?}", err);
}

// ---------------------------------------------------------------------------
// Concurrent error construction
// ---------------------------------------------------------------------------

#[rstest]
fn concurrent_error_creation() {
    use std::sync::{Arc, Mutex};
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let errors = Arc::clone(&errors);
            std::thread::spawn(move || {
                let err = BrowserError::TabNotFound(format!("tab-{}", i));
                errors.lock().unwrap().push(err.to_string());
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let collected = errors.lock().unwrap();
    assert_eq!(collected.len(), 10);
}

// ---------------------------------------------------------------------------
// serde_json error contains "expected" or similar diagnostic
// ---------------------------------------------------------------------------

#[rstest]
fn serde_json_error_display_not_empty() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad json}").unwrap_err();
    let browser_err: BrowserError = json_err.into();
    assert!(!browser_err.to_string().is_empty());
}

// ---------------------------------------------------------------------------
// IoError source is preserved through From conversion
// ---------------------------------------------------------------------------

#[rstest]
fn io_error_source_preserved() {
    use std::error::Error;
    let inner = io::Error::new(io::ErrorKind::InvalidData, "bad data");
    let browser_err: BrowserError = inner.into();
    assert!(browser_err.source().is_some());
}

// ---------------------------------------------------------------------------
// Different variant messages have different prefixes
// ---------------------------------------------------------------------------

#[rstest]
fn variant_messages_distinct_prefixes() {
    let pairs: &[(&str, BrowserError)] = &[
        ("Tab not found", BrowserError::TabNotFound("x".into())),
        ("Bookmark not found", BrowserError::BookmarkNotFound("x".into())),
        ("Extension not found", BrowserError::ExtensionNotFound("x".into())),
        ("WebView creation failed", BrowserError::WebViewCreation("x".into())),
        ("Window creation failed", BrowserError::WindowCreation("x".into())),
        ("Navigation failed", BrowserError::Navigation("x".into())),
        ("Invalid URL", BrowserError::InvalidUrl("x".into())),
        ("Extension error", BrowserError::Extension("x".into())),
    ];
    for (prefix, err) in pairs {
        assert!(
            err.to_string().starts_with(prefix),
            "Expected prefix '{}', got: {}",
            prefix,
            err
        );
    }
}
