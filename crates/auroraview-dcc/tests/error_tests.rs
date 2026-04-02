//! Tests for DccError variants, Display, From, and Send+Sync

use auroraview_dcc::{DccError, Result};
use rstest::rstest;

// ============================================================================
// Display tests
// ============================================================================

#[rstest]
fn display_webview_creation() {
    let e = DccError::WebViewCreation("controller init failed".to_string());
    assert_eq!(
        e.to_string(),
        "WebView creation failed: controller init failed"
    );
}

#[rstest]
fn display_invalid_parent() {
    let e = DccError::InvalidParent;
    assert_eq!(e.to_string(), "Invalid parent HWND");
}

#[rstest]
fn display_window_not_found() {
    let e = DccError::WindowNotFound("win-42".to_string());
    assert_eq!(e.to_string(), "Window not found: win-42");
}

#[rstest]
fn display_unsupported_dcc() {
    let e = DccError::UnsupportedDcc("Katana".to_string());
    assert_eq!(e.to_string(), "DCC not supported: Katana");
}

#[rstest]
fn display_threading() {
    let e = DccError::Threading("STA required".to_string());
    assert_eq!(e.to_string(), "Threading error: STA required");
}

#[rstest]
fn display_io() {
    let io_err = std::io::Error::other("pipe broken");
    let e = DccError::Io(io_err);
    let s = e.to_string();
    assert!(s.contains("IO error"));
}

// ============================================================================
// Debug
// ============================================================================

#[rstest]
fn debug_invalid_parent() {
    let e = DccError::InvalidParent;
    let s = format!("{:?}", e);
    assert!(s.contains("InvalidParent"));
}

#[rstest]
fn debug_webview_creation() {
    let e = DccError::WebViewCreation("fail".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("WebViewCreation"));
    assert!(s.contains("fail"));
}

// ============================================================================
// From conversions
// ============================================================================

#[rstest]
fn from_io_error() {
    let io_err = std::io::Error::other("file not found");
    let e: DccError = io_err.into();
    assert!(e.to_string().contains("IO error"));
}

// ============================================================================
// Error source chain
// ============================================================================

#[rstest]
fn io_variant_has_source() {
    use std::error::Error;
    let io_err = std::io::Error::other("cause");
    let e: DccError = io_err.into();
    assert!(e.source().is_some());
}

#[rstest]
fn threading_variant_no_source() {
    use std::error::Error;
    let e = DccError::Threading("x".to_string());
    assert!(e.source().is_none());
}

#[rstest]
fn window_not_found_no_source() {
    use std::error::Error;
    let e = DccError::WindowNotFound("w".to_string());
    assert!(e.source().is_none());
}

// ============================================================================
// Result type alias
// ============================================================================

#[rstest]
fn result_ok() {
    let r: Result<u32> = Ok(1);
    assert!(r.is_ok());
}

#[rstest]
fn result_err() {
    let r: Result<u32> = Err(DccError::InvalidParent);
    assert!(r.is_err());
}

// ============================================================================
// Send + Sync
// ============================================================================

fn assert_send_sync<T: Send + Sync>() {}

#[rstest]
fn dcc_error_is_send_sync() {
    assert_send_sync::<DccError>();
}

// ============================================================================
// Parametrized: string variants contain their message
// ============================================================================

#[rstest]
#[case(DccError::WebViewCreation("wv".to_string()), "wv")]
#[case(DccError::WindowNotFound("wn".to_string()), "wn")]
#[case(DccError::UnsupportedDcc("ud".to_string()), "ud")]
#[case(DccError::Threading("th".to_string()), "th")]
fn string_variant_message_in_display(#[case] e: DccError, #[case] fragment: &str) {
    assert!(e.to_string().contains(fragment));
}

// ============================================================================
// Parametrized: known DCC types unsupported message
// ============================================================================

#[rstest]
#[case("Katana")]
#[case("Substance Painter")]
#[case("Cinema 4D")]
fn unsupported_dcc_display(#[case] dcc: &str) {
    let e = DccError::UnsupportedDcc(dcc.to_string());
    assert!(e.to_string().contains(dcc));
}
