//! Tests for DCC error types

use auroraview_dcc::error::DccError;

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
