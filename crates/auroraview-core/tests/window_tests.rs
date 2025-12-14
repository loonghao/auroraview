//! Window module tests

use active_win_pos_rs::{ActiveWindow, WindowPosition};
use auroraview_core::window::WindowInfo;
use std::path::PathBuf;

#[test]
fn test_window_info_creation() {
    let info = WindowInfo::new(
        12345,
        "Test Window".to_string(),
        1234,
        "app".to_string(),
        "/path/to/app".to_string(),
    );

    assert_eq!(info.hwnd, 12345);
    assert_eq!(info.title, "Test Window");
    assert_eq!(info.pid, 1234);
    assert_eq!(info.process_name, "app");
    assert_eq!(info.process_path, "/path/to/app");
}

#[test]
fn test_window_info_clone() {
    let info = WindowInfo::new(
        12345,
        "Test Window".to_string(),
        1234,
        "app".to_string(),
        "/path/to/app".to_string(),
    );

    let cloned = info.clone();
    assert_eq!(info, cloned);
}

#[test]
fn test_window_info_repr() {
    let info = WindowInfo::new(
        12345,
        "Test Window".to_string(),
        1234,
        "app".to_string(),
        "C:/test/app.exe".to_string(),
    );
    let repr = info.repr();

    assert!(repr.contains("WindowInfo"));
    assert!(repr.contains("hwnd=12345"));
    assert!(repr.contains("Test Window"));
    assert!(repr.contains("pid=1234"));
    assert!(repr.contains("app"));
}

#[test]
fn test_window_info_from_active_window() {
    let window = ActiveWindow {
        title: "Test Window".to_string(),
        window_id: "HWND(12345)".to_string(),
        process_id: 1234,
        process_path: PathBuf::from("C:/test/app.exe"),
        app_name: "app".to_string(),
        position: WindowPosition {
            x: 100.0,
            y: 200.0,
            width: 800.0,
            height: 600.0,
        },
    };

    let window_info: WindowInfo = window.into();

    assert_eq!(window_info.title, "Test Window");
    assert_eq!(window_info.hwnd, 12345);
    assert_eq!(window_info.pid, 1234);
    assert_eq!(window_info.process_name, "app");
    assert_eq!(window_info.process_path, "C:/test/app.exe");
}

#[test]
fn test_window_info_from_invalid_hwnd() {
    let window = ActiveWindow {
        title: "Test".to_string(),
        window_id: "InvalidHWND".to_string(),
        process_id: 1234,
        process_path: PathBuf::from("/test/app"),
        app_name: "app".to_string(),
        position: WindowPosition {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
    };

    let window_info: WindowInfo = window.into();
    // Invalid HWND should parse to 0
    assert_eq!(window_info.hwnd, 0);
}

#[test]
fn test_window_info_from_empty_hwnd() {
    let window = ActiveWindow {
        title: "Test".to_string(),
        window_id: "HWND()".to_string(),
        process_id: 1234,
        process_path: PathBuf::from("/test/app"),
        app_name: "app".to_string(),
        position: WindowPosition {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
    };

    let window_info: WindowInfo = window.into();
    // Empty HWND should parse to 0
    assert_eq!(window_info.hwnd, 0);
}

#[test]
fn test_window_info_debug() {
    let window = WindowInfo::new(
        12345,
        "Test Window".to_string(),
        1234,
        "app".to_string(),
        "C:/test/app.exe".to_string(),
    );

    let debug_str = format!("{:?}", window);
    assert!(debug_str.contains("WindowInfo"));
    assert!(debug_str.contains("12345"));
    assert!(debug_str.contains("Test Window"));
}
