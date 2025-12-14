//! BOM (Browser Object Model) tests

use auroraview_core::bom::{js, BomError, PhysicalPosition, PhysicalSize};

#[test]
fn test_physical_size() {
    let size = PhysicalSize::new(800, 600);
    assert_eq!(size.width, 800);
    assert_eq!(size.height, 600);
}

#[test]
fn test_physical_position() {
    let pos = PhysicalPosition::new(100, 200);
    assert_eq!(pos.x, 100);
    assert_eq!(pos.y, 200);
}

#[test]
fn test_bom_error_display() {
    let err = BomError::WebViewUnavailable;
    assert_eq!(err.to_string(), "WebView not available or locked");

    let err = BomError::JsExecutionFailed("syntax error".to_string());
    assert!(err.to_string().contains("syntax error"));
}

#[test]
fn test_js_templates() {
    // These use fallback pattern for custom protocol support
    assert!(js::GO_BACK.contains("history.back()"));
    assert!(js::GO_FORWARD.contains("history.forward()"));
    assert!(js::STOP.contains("window.stop()"));
    assert!(js::RELOAD.contains("location.reload()"));

    let zoom_script = js::set_zoom(1.5);
    assert!(zoom_script.contains("1.5"));
}
