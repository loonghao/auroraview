//! Tests for DCC error types

use auroraview_dcc::error::DccError;
use rstest::rstest;

// ============================================================================
// DccError variant tests
// ============================================================================

#[test]
fn webview_creation_error_message() {
    let e = DccError::WebViewCreation("Failed to create WebView2".to_string());
    let msg = e.to_string();
    assert!(msg.contains("WebView creation failed"));
    assert!(msg.contains("Failed to create WebView2"));
}

#[test]
fn invalid_parent_error_message() {
    let e = DccError::InvalidParent;
    assert!(e.to_string().contains("Invalid parent HWND"));
}

#[test]
fn window_not_found_error_message() {
    let e = DccError::WindowNotFound("maya_main_window".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Window not found"));
    assert!(msg.contains("maya_main_window"));
}

#[test]
fn unsupported_dcc_error_message() {
    let e = DccError::UnsupportedDcc("Cinema4D".to_string());
    let msg = e.to_string();
    assert!(msg.contains("DCC not supported"));
    assert!(msg.contains("Cinema4D"));
}

#[test]
fn threading_error_message() {
    let e = DccError::Threading("deadlock detected".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Threading error"));
    assert!(msg.contains("deadlock detected"));
}

#[test]
fn io_error_from_std_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let dcc_err = DccError::from(io_err);
    let msg = dcc_err.to_string();
    assert!(msg.contains("IO error"));
}

#[test]
fn result_type_ok_variant() {
    let result: auroraview_dcc::error::Result<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn result_type_err_variant() {
    let result: auroraview_dcc::error::Result<i32> =
        Err(DccError::InvalidParent);
    assert!(result.is_err());
}

#[test]
fn dcc_error_debug_output() {
    let e = DccError::WebViewCreation("test".to_string());
    let debug_str = format!("{:?}", e);
    assert!(debug_str.contains("WebViewCreation"));
}

#[test]
fn window_not_found_empty_name() {
    let e = DccError::WindowNotFound(String::new());
    let msg = e.to_string();
    assert!(msg.contains("Window not found"));
}

#[test]
fn unsupported_dcc_empty_name() {
    let e = DccError::UnsupportedDcc(String::new());
    let msg = e.to_string();
    assert!(msg.contains("DCC not supported"));
}

#[test]
fn threading_error_unicode_message() {
    let e = DccError::Threading("线程安全违规".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Threading error"));
    assert!(msg.contains("线程安全违规"));
}

#[test]
fn io_error_permission_denied() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let dcc_err = DccError::from(io_err);
    let msg = dcc_err.to_string();
    assert!(msg.contains("IO error"));
    assert!(msg.contains("access denied"));
}

// ============================================================================
// Send + Sync bounds
// ============================================================================

#[test]
fn dcc_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DccError>();
}

// ============================================================================
// Debug output contains all variant names
// ============================================================================

#[rstest]
#[case(DccError::WebViewCreation("x".into()), "WebViewCreation")]
#[case(DccError::WindowNotFound("x".into()), "WindowNotFound")]
#[case(DccError::UnsupportedDcc("x".into()), "UnsupportedDcc")]
#[case(DccError::Threading("x".into()), "Threading")]
fn debug_contains_variant(#[case] err: DccError, #[case] variant: &str) {
    let debug = format!("{:?}", err);
    assert!(debug.contains(variant), "Debug should contain '{}': {}", variant, debug);
}

// ============================================================================
// Display messages contain the inner string
// ============================================================================

#[rstest]
#[case(DccError::WebViewCreation("my-detail".into()), "my-detail")]
#[case(DccError::WindowNotFound("blender_main".into()), "blender_main")]
#[case(DccError::UnsupportedDcc("Nuke15".into()), "Nuke15")]
#[case(DccError::Threading("race condition".into()), "race condition")]
fn display_contains_inner_string(#[case] err: DccError, #[case] expected: &str) {
    assert!(
        err.to_string().contains(expected),
        "Display should contain '{}': {}",
        expected, err
    );
}

// ============================================================================
// IoError source chain
// ============================================================================

#[test]
fn io_error_has_source() {
    use std::error::Error;
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
    let dcc_err = DccError::from(io_err);
    assert!(dcc_err.source().is_some());
}

#[test]
fn string_variants_have_no_source() {
    use std::error::Error;
    let err = DccError::WindowNotFound("w".into());
    assert!(err.source().is_none());
}

// ============================================================================
// InvalidParent has no source
// ============================================================================

#[test]
fn invalid_parent_has_no_source() {
    use std::error::Error;
    let err = DccError::InvalidParent;
    assert!(err.source().is_none());
}

// ============================================================================
// Concurrent error creation
// ============================================================================

#[test]
fn concurrent_dcc_error_creation() {
    use std::sync::{Arc, Mutex};
    let results: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let results = Arc::clone(&results);
            std::thread::spawn(move || {
                let err = DccError::WindowNotFound(format!("window_{}", i));
                results.lock().unwrap().push(err.to_string());
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let collected = results.lock().unwrap();
    assert_eq!(collected.len(), 8);
}

// ============================================================================
// Long string payloads
// ============================================================================

#[test]
fn long_payload_preserved() {
    let long = "A".repeat(500);
    let e = DccError::WebViewCreation(long.clone());
    assert!(e.to_string().contains(&long));
}

// ============================================================================
// IoError different kinds
// ============================================================================

#[rstest]
#[case(std::io::ErrorKind::TimedOut, "timed out")]
#[case(std::io::ErrorKind::WouldBlock, "would block")]
fn io_error_various_kinds(#[case] kind: std::io::ErrorKind, #[case] msg: &str) {
    let io_err = std::io::Error::new(kind, msg);
    let dcc_err: DccError = io_err.into();
    assert!(dcc_err.to_string().contains(msg));
}

