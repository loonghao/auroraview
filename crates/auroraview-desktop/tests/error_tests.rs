//! Tests for DesktopError type

use std::io;

use auroraview_desktop::error::{DesktopError, Result};
use rstest::rstest;

// ============================================================================
// DesktopError — variant Display messages
// ============================================================================

#[rstest]
fn window_creation_display() {
    let e = DesktopError::WindowCreation("HWND failed".into());
    assert_eq!(e.to_string(), "Window creation failed: HWND failed");
}

#[rstest]
fn webview_creation_display() {
    let e = DesktopError::WebViewCreation("WebView2 not installed".into());
    assert_eq!(
        e.to_string(),
        "WebView creation failed: WebView2 not installed"
    );
}

#[rstest]
fn window_not_found_display() {
    let e = DesktopError::WindowNotFound("window_42".into());
    assert_eq!(e.to_string(), "Window not found: window_42");
}

#[rstest]
fn event_loop_display() {
    let e = DesktopError::EventLoop("loop exited unexpectedly".into());
    assert_eq!(e.to_string(), "Event loop error: loop exited unexpectedly");
}

#[rstest]
fn tray_display() {
    let e = DesktopError::Tray("icon missing".into());
    assert_eq!(e.to_string(), "Tray error: icon missing");
}

#[rstest]
fn io_error_from() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let e = DesktopError::from(io_err);
    assert!(matches!(e, DesktopError::Io(_)));
    assert!(e.to_string().contains("file not found"));
}

#[rstest]
fn io_error_into() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let e: DesktopError = io_err.into();
    assert!(e.to_string().contains("access denied"));
}

// ============================================================================
// std::error::Error trait
// ============================================================================

#[rstest]
fn implements_std_error() {
    let e: &dyn std::error::Error = &DesktopError::WindowCreation("x".into());
    assert!(e.source().is_none());
}

#[rstest]
fn io_error_source() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let e = DesktopError::Io(io_err);
    let e_ref: &dyn std::error::Error = &e;
    assert!(e_ref.source().is_some());
}

// ============================================================================
// Send + Sync
// ============================================================================

fn assert_send_sync<T: Send + Sync>() {}

#[rstest]
fn desktop_error_is_send_sync() {
    assert_send_sync::<DesktopError>();
}

// ============================================================================
// Result type alias
// ============================================================================

#[rstest]
fn result_ok() {
    let val: i32 = 99;
    let r: Result<i32> = Ok(val);
    assert!(r.is_ok());
}

#[rstest]
fn result_err() {
    let r: Result<i32> = Err(DesktopError::WindowNotFound("w1".into()));
    assert!(r.is_err());
}

// ============================================================================
// All variants coverage
// ============================================================================

#[rstest]
fn all_variants_have_non_empty_display() {
    let io_err = io::Error::other("io");
    let variants: Vec<DesktopError> = vec![
        DesktopError::WindowCreation("a".into()),
        DesktopError::WebViewCreation("b".into()),
        DesktopError::WindowNotFound("c".into()),
        DesktopError::EventLoop("d".into()),
        DesktopError::Tray("e".into()),
        DesktopError::Io(io_err),
    ];
    assert_eq!(variants.len(), 6);
    for v in &variants {
        assert!(!v.to_string().is_empty());
    }
}

// ============================================================================
// Additional tests
// ============================================================================

#[rstest]
fn window_creation_contains_msg() {
    let msg = "CreateWindowEx returned NULL";
    let e = DesktopError::WindowCreation(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn webview_creation_contains_msg() {
    let msg = "CoreWebView2Environment creation failed";
    let e = DesktopError::WebViewCreation(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn window_not_found_contains_id() {
    let id = "maya_panel_001";
    let e = DesktopError::WindowNotFound(id.into());
    assert!(e.to_string().contains(id));
}

#[rstest]
fn event_loop_contains_msg() {
    let msg = "winit event loop terminated";
    let e = DesktopError::EventLoop(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn tray_contains_msg() {
    let msg = "failed to create notification icon";
    let e = DesktopError::Tray(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn window_creation_debug_contains_variant() {
    let e = DesktopError::WindowCreation("x".into());
    let s = format!("{:?}", e);
    assert!(s.contains("WindowCreation"));
}

#[rstest]
fn webview_creation_debug_contains_variant() {
    let e = DesktopError::WebViewCreation("y".into());
    let s = format!("{:?}", e);
    assert!(s.contains("WebViewCreation"));
}

#[rstest]
fn window_not_found_debug_contains_variant() {
    let e = DesktopError::WindowNotFound("z".into());
    let s = format!("{:?}", e);
    assert!(s.contains("WindowNotFound"));
}

#[rstest]
fn event_loop_debug_contains_variant() {
    let e = DesktopError::EventLoop("ev".into());
    let s = format!("{:?}", e);
    assert!(s.contains("EventLoop"));
}

#[rstest]
fn tray_debug_contains_variant() {
    let e = DesktopError::Tray("tr".into());
    let s = format!("{:?}", e);
    assert!(s.contains("Tray"));
}

#[rstest]
fn window_creation_unicode_msg() {
    let msg = "ウィンドウ作成失敗: HWNDがNULL";
    let e = DesktopError::WindowCreation(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn tray_unicode_msg() {
    let msg = "トレイアイコンが見つかりません";
    let e = DesktopError::Tray(msg.into());
    assert!(e.to_string().contains(msg));
}

#[rstest]
fn window_creation_long_msg() {
    let msg = "x".repeat(512);
    let e = DesktopError::WindowCreation(msg.clone());
    assert!(e.to_string().contains(&msg));
}

#[rstest]
fn io_various_kinds() {
    let kinds = [
        io::ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::TimedOut,
        io::ErrorKind::ConnectionRefused,
    ];
    for kind in &kinds {
        let io_err = io::Error::new(*kind, "detail");
        let e: DesktopError = io_err.into();
        assert!(matches!(e, DesktopError::Io(_)));
        assert!(!e.to_string().is_empty());
    }
}

#[rstest]
fn result_error_contains_window_not_found() {
    let r: Result<()> = Err(DesktopError::WindowNotFound("panel_x".into()));
    assert!(matches!(r, Err(DesktopError::WindowNotFound(name)) if name == "panel_x"));
}


#[rstest]
fn result_ok_unit() {
    let r: Result<()> = Ok(());
    assert!(r.is_ok());
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[rstest]
fn all_variants_have_unique_display() {
    let io_err1 = io::Error::other("io1");
    let messages: Vec<String> = vec![
        DesktopError::WindowCreation("x".into()).to_string(),
        DesktopError::WebViewCreation("x".into()).to_string(),
        DesktopError::WindowNotFound("x".into()).to_string(),
        DesktopError::EventLoop("x".into()).to_string(),
        DesktopError::Tray("x".into()).to_string(),
        DesktopError::Io(io_err1).to_string(),
    ];
    // All display strings should be non-empty and different from each other
    for (i, m) in messages.iter().enumerate() {
        assert!(!m.is_empty(), "Message {i} should not be empty");
        for (j, n) in messages.iter().enumerate() {
            if i != j {
                assert_ne!(m, n, "Messages {i} and {j} should differ");
            }
        }
    }
}

#[rstest]
fn window_creation_empty_msg() {
    let e = DesktopError::WindowCreation("".into());
    assert!(e.to_string().contains("Window creation failed"));
}

#[rstest]
fn webview_creation_empty_msg() {
    let e = DesktopError::WebViewCreation("".into());
    assert!(e.to_string().contains("WebView creation failed"));
}

#[rstest]
fn window_not_found_empty_id() {
    let e = DesktopError::WindowNotFound("".into());
    assert!(e.to_string().contains("Window not found"));
}

#[rstest]
fn event_loop_empty_msg() {
    let e = DesktopError::EventLoop("".into());
    assert!(e.to_string().contains("Event loop error"));
}

#[rstest]
fn tray_empty_msg() {
    let e = DesktopError::Tray("".into());
    assert!(e.to_string().contains("Tray error"));
}

#[rstest]
fn io_error_not_found_kind() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let e: DesktopError = io_err.into();
    assert!(e.to_string().contains("file missing"));
}

#[rstest]
fn io_error_source_is_some() {
    let io_err = io::Error::new(io::ErrorKind::Other, "chain");
    let e = DesktopError::Io(io_err);
    let e_ref: &dyn std::error::Error = &e;
    assert!(e_ref.source().is_some());
}

#[rstest]
fn all_non_io_variants_source_is_none() {
    let variants: Vec<DesktopError> = vec![
        DesktopError::WindowCreation("x".into()),
        DesktopError::WebViewCreation("x".into()),
        DesktopError::WindowNotFound("x".into()),
        DesktopError::EventLoop("x".into()),
        DesktopError::Tray("x".into()),
    ];
    for e in &variants {
        let e_ref: &dyn std::error::Error = e;
        assert!(e_ref.source().is_none(), "{:?} should have no source", e);
    }
}

#[rstest]
fn result_err_window_creation() {
    let r: Result<i32> = Err(DesktopError::WindowCreation("hwnd null".into()));
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("hwnd null"));
}

#[rstest]
fn result_err_webview_creation() {
    let r: Result<i32> = Err(DesktopError::WebViewCreation("env failed".into()));
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("env failed"));
}

#[rstest]
fn all_variants_debug_not_empty() {
    let io_err = io::Error::other("io-debug");
    let variants: Vec<DesktopError> = vec![
        DesktopError::WindowCreation("x".into()),
        DesktopError::WebViewCreation("x".into()),
        DesktopError::WindowNotFound("x".into()),
        DesktopError::EventLoop("x".into()),
        DesktopError::Tray("x".into()),
        DesktopError::Io(io_err),
    ];
    for e in &variants {
        assert!(!format!("{e:?}").is_empty());
    }
}

#[rstest]
fn desktop_error_display_prefix_correctness() {
    // Verify each variant has its expected prefix
    assert!(DesktopError::WindowCreation("x".into()).to_string().starts_with("Window creation failed"));
    assert!(DesktopError::WebViewCreation("x".into()).to_string().starts_with("WebView creation failed"));
    assert!(DesktopError::WindowNotFound("x".into()).to_string().starts_with("Window not found"));
    assert!(DesktopError::EventLoop("x".into()).to_string().starts_with("Event loop error"));
    assert!(DesktopError::Tray("x".into()).to_string().starts_with("Tray error"));
}
