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

// ============================================================================
// Windows-only: Com variant
// ============================================================================

#[rstest]
#[cfg(target_os = "windows")]
fn com_variant_display() {
    let e = DccError::Com("HRESULT 0x80004001".to_string());
    let s = e.to_string();
    assert!(s.contains("HRESULT"));
    assert!(s.contains("COM"));
}

#[rstest]
#[cfg(target_os = "windows")]
fn com_variant_debug() {
    let e = DccError::Com("E_NOTIMPL".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("Com"));
    assert!(s.contains("E_NOTIMPL"));
}

#[rstest]
#[cfg(target_os = "windows")]
#[case("0x80070057")]
#[case("E_INVALIDARG")]
#[case("E_POINTER")]
fn com_variant_message_in_display(#[case] msg: &str) {
    let e = DccError::Com(msg.to_string());
    assert!(e.to_string().contains(msg));
}

// ============================================================================
// Error as Box<dyn Error> / std::error::Error
// ============================================================================

#[rstest]
fn webview_creation_no_source() {
    use std::error::Error;
    let e = DccError::WebViewCreation("x".to_string());
    assert!(e.source().is_none());
}

#[rstest]
fn invalid_parent_no_source() {
    use std::error::Error;
    let e = DccError::InvalidParent;
    assert!(e.source().is_none());
}

#[rstest]
fn unsupported_dcc_no_source() {
    use std::error::Error;
    let e = DccError::UnsupportedDcc("Maya".to_string());
    assert!(e.source().is_none());
}

#[rstest]
fn error_as_box_dyn_error() {
    let e: Box<dyn std::error::Error + Send + Sync> = Box::new(DccError::InvalidParent);
    assert!(e.to_string().contains("Invalid parent"));
}

#[rstest]
fn error_in_result_chain() {
    let result: std::result::Result<(), DccError> =
        Err(DccError::WebViewCreation("fail".to_string()));
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("fail"));
}

// ============================================================================
// Display prefix correctness
// ============================================================================

#[rstest]
fn webview_creation_display_prefix() {
    let e = DccError::WebViewCreation("x".to_string());
    assert!(e.to_string().starts_with("WebView creation failed:"));
}

#[rstest]
fn window_not_found_display_prefix() {
    let e = DccError::WindowNotFound("w".to_string());
    assert!(e.to_string().starts_with("Window not found:"));
}

#[rstest]
fn unsupported_dcc_display_prefix() {
    let e = DccError::UnsupportedDcc("D".to_string());
    assert!(e.to_string().starts_with("DCC not supported:"));
}

#[rstest]
fn threading_display_prefix() {
    let e = DccError::Threading("t".to_string());
    assert!(e.to_string().starts_with("Threading error:"));
}

// ============================================================================
// Additional parametrized: WebViewCreation messages
// ============================================================================

#[rstest]
#[case("controller init failed")]
#[case("ICoreWebView2Environment::CreateCoreWebView2Controller failed")]
#[case("unsupported runtime")]
fn webview_creation_messages(#[case] msg: &str) {
    let e = DccError::WebViewCreation(msg.to_string());
    assert!(e.to_string().contains(msg));
}

#[rstest]
#[case("win-1")]
#[case("maya-panel-42")]
#[case("dcc://houdini/scene")]
fn window_not_found_ids(#[case] id: &str) {
    let e = DccError::WindowNotFound(id.to_string());
    assert!(e.to_string().contains(id));
}

#[rstest]
#[case("deadlock detected")]
#[case("STA thread required")]
#[case("COM apartment mismatch")]
fn threading_messages(#[case] msg: &str) {
    let e = DccError::Threading(msg.to_string());
    assert!(e.to_string().contains(msg));
}

// ============================================================================
// Concurrent error construction (no panic)
// ============================================================================

#[rstest]
fn concurrent_error_construction_no_panic() {
    use std::sync::Arc;
    let errors: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let errs = Arc::clone(&errors);
            std::thread::spawn(move || {
                let e = if i % 3 == 0 {
                    DccError::InvalidParent
                } else if i % 3 == 1 {
                    DccError::WebViewCreation(format!("thread {}", i))
                } else {
                    DccError::Threading(format!("t{}", i))
                };
                let mut guard = errs.lock().unwrap();
                guard.push(e.to_string());
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }

    let guard = errors.lock().unwrap();
    assert_eq!(guard.len(), 8);
}

// ============================================================================
// DccError in Arc/shared context
// ============================================================================

#[rstest]
fn dcc_error_in_arc() {
    use std::sync::Arc;
    let e = Arc::new(DccError::InvalidParent);
    assert!(e.to_string().contains("Invalid parent"));
}

#[rstest]
fn result_ok_value() {
    let r: Result<String> = Ok("hello".to_string());
    assert_eq!(r.unwrap(), "hello");
}

#[rstest]
fn result_err_value() {
    let r: Result<()> = Err(DccError::UnsupportedDcc("Nuke".to_string()));
    let msg = r.unwrap_err().to_string();
    assert!(msg.contains("Nuke"));
}

#[rstest]
fn result_map_err() {
    let r: Result<()> = Err(DccError::Threading("sta".to_string()));
    let mapped = r.map_err(|e| format!("wrapped: {}", e));
    assert!(mapped.unwrap_err().contains("sta"));
}
