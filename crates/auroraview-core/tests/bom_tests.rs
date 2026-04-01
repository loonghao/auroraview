//! BOM (Browser Object Model) tests

use auroraview_core::bom::{
    handle_window_command_default, js, BomError, BomResult, PhysicalPosition, PhysicalSize,
    WindowControlApi,
};

// ---------------------------------------------------------------------------
// PhysicalSize
// ---------------------------------------------------------------------------

#[test]
fn test_physical_size() {
    let size = PhysicalSize::new(800, 600);
    assert_eq!(size.width, 800);
    assert_eq!(size.height, 600);
}

#[test]
fn test_physical_size_default() {
    let size = PhysicalSize::default();
    assert_eq!(size.width, 0);
    assert_eq!(size.height, 0);
}

#[test]
fn test_physical_size_clone_copy() {
    let size = PhysicalSize::new(1920, 1080);
    let size2 = size; // Copy
    let size3 = size; // also Copy (same as clone for Copy types)
    assert_eq!(size, size2);
    assert_eq!(size, size3);
}

#[test]
fn test_physical_size_eq() {
    let a = PhysicalSize::new(100, 200);
    let b = PhysicalSize::new(100, 200);
    let c = PhysicalSize::new(100, 201);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_physical_size_debug() {
    let size = PhysicalSize::new(640, 480);
    let s = format!("{:?}", size);
    assert!(s.contains("640"));
    assert!(s.contains("480"));
}

// ---------------------------------------------------------------------------
// PhysicalPosition
// ---------------------------------------------------------------------------

#[test]
fn test_physical_position() {
    let pos = PhysicalPosition::new(100, 200);
    assert_eq!(pos.x, 100);
    assert_eq!(pos.y, 200);
}

#[test]
fn test_physical_position_default() {
    let pos = PhysicalPosition::default();
    assert_eq!(pos.x, 0);
    assert_eq!(pos.y, 0);
}

#[test]
fn test_physical_position_negative() {
    let pos = PhysicalPosition::new(-100, -200);
    assert_eq!(pos.x, -100);
    assert_eq!(pos.y, -200);
}

#[test]
fn test_physical_position_clone_copy() {
    let pos = PhysicalPosition::new(50, 75);
    let pos2 = pos; // Copy
    let pos3 = pos; // also Copy
    assert_eq!(pos, pos2);
    assert_eq!(pos, pos3);
}

#[test]
fn test_physical_position_eq() {
    let a = PhysicalPosition::new(10, 20);
    let b = PhysicalPosition::new(10, 20);
    let c = PhysicalPosition::new(10, 21);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_physical_position_debug() {
    let pos = PhysicalPosition::new(-5, 10);
    let s = format!("{:?}", pos);
    assert!(s.contains("-5"));
    assert!(s.contains("10"));
}

// ---------------------------------------------------------------------------
// BomError
// ---------------------------------------------------------------------------

#[test]
fn test_bom_error_display() {
    let err = BomError::WebViewUnavailable;
    assert_eq!(err.to_string(), "WebView not available or locked");

    let err = BomError::JsExecutionFailed("syntax error".to_string());
    assert!(err.to_string().contains("syntax error"));
}

#[test]
fn test_bom_error_window_unavailable() {
    let err = BomError::WindowUnavailable;
    assert_eq!(err.to_string(), "Window not available");
}

#[test]
fn test_bom_error_platform_not_supported() {
    let err = BomError::PlatformNotSupported;
    assert!(err.to_string().contains("Platform not supported"));
}

#[test]
fn test_bom_error_operation_failed() {
    let err = BomError::OperationFailed("timeout".to_string());
    assert!(err.to_string().contains("timeout"));
    assert!(err.to_string().contains("Operation failed"));
}

#[test]
fn test_bom_error_debug() {
    let err = BomError::WebViewUnavailable;
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("WebViewUnavailable"));
}

#[test]
fn test_bom_error_clone() {
    let err = BomError::JsExecutionFailed("test".to_string());
    let cloned = err.clone();
    assert_eq!(err.to_string(), cloned.to_string());
}

// ---------------------------------------------------------------------------
// js module constants
// ---------------------------------------------------------------------------

#[test]
fn test_js_templates() {
    assert!(js::GO_BACK.contains("history.back()"));
    assert!(js::GO_FORWARD.contains("history.forward()"));
    assert!(js::STOP.contains("window.stop()"));
    assert!(js::RELOAD.contains("location.reload()"));

    let zoom_script = js::set_zoom(1.5);
    assert!(zoom_script.contains("1.5"));
}

#[test]
fn test_js_can_go_back() {
    assert!(js::CAN_GO_BACK.contains("history.length"));
}

#[test]
fn test_js_can_go_forward() {
    assert!(!js::CAN_GO_FORWARD.is_empty());
}

#[test]
fn test_js_is_loading() {
    assert!(js::IS_LOADING.contains("readyState"));
}

#[test]
fn test_js_get_load_progress() {
    assert!(js::GET_LOAD_PROGRESS.contains("readyState"));
    assert!(js::GET_LOAD_PROGRESS.contains("100"));
}

#[test]
fn test_js_get_current_url() {
    assert!(js::GET_CURRENT_URL.contains("location.href"));
}

#[test]
fn test_js_clear_all_browsing_data() {
    assert!(js::CLEAR_ALL_BROWSING_DATA.contains("clearAllBrowsingData"));
}

#[test]
fn test_js_clear_local_storage() {
    assert!(js::CLEAR_LOCAL_STORAGE.contains("clearLocalStorage"));
}

#[test]
fn test_js_clear_session_storage() {
    assert!(js::CLEAR_SESSION_STORAGE.contains("clearSessionStorage"));
}

#[test]
fn test_js_clear_indexed_db() {
    assert!(js::CLEAR_INDEXED_DB.contains("clearIndexedDB"));
}

#[test]
fn test_js_clear_cookies() {
    assert!(js::CLEAR_COOKIES.contains("clearCookies"));
}

#[test]
fn test_js_get_zoom() {
    assert!(js::GET_ZOOM.contains("1.0"));
}

#[test]
fn test_js_zoom_in() {
    assert!(js::ZOOM_IN.contains("zoomIn"));
}

#[test]
fn test_js_zoom_out() {
    assert!(js::ZOOM_OUT.contains("zoomOut"));
}

#[test]
fn test_js_reset_zoom() {
    assert!(js::RESET_ZOOM.contains("resetZoom"));
}

#[test]
fn test_js_set_zoom_values() {
    let s1 = js::set_zoom(1.0);
    assert!(s1.contains("1"));
    let s2 = js::set_zoom(2.0);
    assert!(s2.contains("2"));
    let s3 = js::set_zoom(0.5);
    assert!(s3.contains("0.5"));
}

// ---------------------------------------------------------------------------
// handle_window_command_default
// ---------------------------------------------------------------------------

/// Minimal WindowControlApi for testing handle_window_command_default
struct MockWindow {
    maximized: bool,
    minimized: bool,
    fullscreen: bool,
    visible: bool,
    always_on_top: bool,
}

impl MockWindow {
    fn new() -> Self {
        Self {
            maximized: false,
            minimized: false,
            fullscreen: false,
            visible: true,
            always_on_top: false,
        }
    }
}

impl WindowControlApi for MockWindow {
    fn minimize(&self) -> BomResult<()> {
        Ok(())
    }
    fn maximize(&self) -> BomResult<()> {
        Ok(())
    }
    fn unmaximize(&self) -> BomResult<()> {
        Ok(())
    }
    fn toggle_maximize(&self) -> BomResult<()> {
        Ok(())
    }
    fn is_maximized(&self) -> BomResult<bool> {
        Ok(self.maximized)
    }
    fn is_minimized(&self) -> BomResult<bool> {
        Ok(self.minimized)
    }
    fn set_fullscreen(&self, _fullscreen: bool) -> BomResult<()> {
        Ok(())
    }
    fn is_fullscreen(&self) -> BomResult<bool> {
        Ok(self.fullscreen)
    }
    fn set_visible(&self, _visible: bool) -> BomResult<()> {
        Ok(())
    }
    fn is_visible(&self) -> BomResult<bool> {
        Ok(self.visible)
    }
    fn is_focused(&self) -> BomResult<bool> {
        Ok(true)
    }
    fn set_focus(&self) -> BomResult<()> {
        Ok(())
    }
    fn set_title(&self, _title: &str) -> BomResult<()> {
        Ok(())
    }
    fn title(&self) -> BomResult<String> {
        Ok("Test".to_string())
    }
    fn set_size(&self, _w: u32, _h: u32) -> BomResult<()> {
        Ok(())
    }
    fn inner_size(&self) -> BomResult<PhysicalSize> {
        Ok(PhysicalSize::new(800, 600))
    }
    fn outer_size(&self) -> BomResult<PhysicalSize> {
        Ok(PhysicalSize::new(820, 640))
    }
    fn position(&self) -> BomResult<PhysicalPosition> {
        Ok(PhysicalPosition::new(100, 200))
    }
    fn set_position(&self, _x: i32, _y: i32) -> BomResult<()> {
        Ok(())
    }
    fn center(&self) -> BomResult<()> {
        Ok(())
    }
    fn set_decorations(&self, _decorations: bool) -> BomResult<()> {
        Ok(())
    }
    fn set_resizable(&self, _resizable: bool) -> BomResult<()> {
        Ok(())
    }
    fn set_min_size(&self, _w: u32, _h: u32) -> BomResult<()> {
        Ok(())
    }
    fn set_max_size(&self, _w: u32, _h: u32) -> BomResult<()> {
        Ok(())
    }
    fn set_always_on_top(&self, _always_on_top: bool) -> BomResult<()> {
        Ok(())
    }
    fn is_always_on_top(&self) -> BomResult<bool> {
        Ok(self.always_on_top)
    }
}

#[test]
fn test_window_command_close() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "close", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_minimize() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "minimize", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_maximize() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "maximize", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_unmaximize() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "unmaximize", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_restore_alias() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "restore", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_toggle_maximize() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "toggle_maximize", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_is_maximized() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "is_maximized", &serde_json::Value::Null).unwrap();
    assert_eq!(result["result"], false);
}

#[test]
fn test_window_command_is_minimized() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "is_minimized", &serde_json::Value::Null).unwrap();
    assert_eq!(result["result"], false);
}

#[test]
fn test_window_command_set_fullscreen() {
    let w = MockWindow::new();
    let params = serde_json::json!({"fullscreen": true});
    let result = handle_window_command_default(&w, "set_fullscreen", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_is_fullscreen() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "is_fullscreen", &serde_json::Value::Null).unwrap();
    assert_eq!(result["result"], false);
}

#[test]
fn test_window_command_show() {
    let w = MockWindow::new();
    let params = serde_json::json!({"visible": true});
    let result = handle_window_command_default(&w, "show", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_hide() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "hide", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_is_visible() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "is_visible", &serde_json::Value::Null).unwrap();
    assert_eq!(result["result"], true);
}

#[test]
fn test_window_command_set_title() {
    let w = MockWindow::new();
    let params = serde_json::json!({"title": "New Title"});
    let result = handle_window_command_default(&w, "set_title", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_get_size() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "get_size", &serde_json::Value::Null).unwrap();
    assert_eq!(result["width"], 800);
    assert_eq!(result["height"], 600);
}

#[test]
fn test_window_command_inner_size_alias() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "inner_size", &serde_json::Value::Null).unwrap();
    assert_eq!(result["width"], 800);
}

#[test]
fn test_window_command_get_position() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "get_position", &serde_json::Value::Null).unwrap();
    assert_eq!(result["x"], 100);
    assert_eq!(result["y"], 200);
}

#[test]
fn test_window_command_position_alias() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "position", &serde_json::Value::Null).unwrap();
    assert_eq!(result["x"], 100);
}

#[test]
fn test_window_command_set_position() {
    let w = MockWindow::new();
    let params = serde_json::json!({"x": 50, "y": 75});
    let result = handle_window_command_default(&w, "set_position", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_set_size() {
    let w = MockWindow::new();
    let params = serde_json::json!({"width": 1280, "height": 720});
    let result = handle_window_command_default(&w, "set_size", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_center() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "center", &serde_json::Value::Null).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_set_always_on_top() {
    let w = MockWindow::new();
    let params = serde_json::json!({"always_on_top": true});
    let result =
        handle_window_command_default(&w, "set_always_on_top", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_is_always_on_top() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "is_always_on_top", &serde_json::Value::Null).unwrap();
    assert_eq!(result["result"], false);
}

#[test]
fn test_window_command_set_decorations() {
    let w = MockWindow::new();
    let params = serde_json::json!({"decorations": false});
    let result =
        handle_window_command_default(&w, "set_decorations", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_set_resizable() {
    let w = MockWindow::new();
    let params = serde_json::json!({"resizable": true});
    let result =
        handle_window_command_default(&w, "set_resizable", &params).unwrap();
    assert_eq!(result["ok"], true);
}

#[test]
fn test_window_command_unknown_returns_error() {
    let w = MockWindow::new();
    let result =
        handle_window_command_default(&w, "nonexistent_command", &serde_json::Value::Null);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown window command"));
}
