//! Window module tests

use active_win_pos_rs::{ActiveWindow, WindowPosition};
use auroraview_core::window::WindowInfo;
use rstest::rstest;
use std::path::PathBuf;

// ============================================================================
// WindowInfo::new
// ============================================================================

#[test]
fn window_info_creation() {
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
fn window_info_clone() {
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
fn window_info_repr() {
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
fn window_info_repr_format() {
    let info = WindowInfo::new(0, "".to_string(), 0, "".to_string(), "".to_string());
    let repr = info.repr();
    assert!(repr.contains("WindowInfo"));
    assert!(repr.contains("hwnd=0"));
    assert!(repr.contains("pid=0"));
}

#[test]
fn window_info_unicode_title() {
    let info = WindowInfo::new(
        1,
        "窗口标题 🪟".to_string(),
        100,
        "proc".to_string(),
        "/proc".to_string(),
    );
    assert_eq!(info.title, "窗口标题 🪟");
}

#[test]
fn window_info_zero_hwnd() {
    let info = WindowInfo::new(0, "Zero".to_string(), 0, "zero".to_string(), "".to_string());
    assert_eq!(info.hwnd, 0);
    assert_eq!(info.pid, 0);
}

#[test]
fn window_info_negative_hwnd() {
    let info = WindowInfo::new(-1, "Neg".to_string(), 1, "proc".to_string(), "".to_string());
    assert_eq!(info.hwnd, -1);
}

#[rstest]
#[case(1, "Window 1", 100)]
#[case(99999, "Large HWND", 9999)]
#[case(0, "", 0)]
fn window_info_various(#[case] hwnd: isize, #[case] title: &str, #[case] pid: u32) {
    let info = WindowInfo::new(hwnd, title.to_string(), pid, "proc".to_string(), "".to_string());
    assert_eq!(info.hwnd, hwnd);
    assert_eq!(info.title, title);
    assert_eq!(info.pid, pid);
}

// ============================================================================
// From<ActiveWindow>
// ============================================================================

fn make_active_window(title: &str, window_id: &str, pid: u64, path: &str, app: &str) -> ActiveWindow {
    ActiveWindow {
        title: title.to_string(),
        window_id: window_id.to_string(),
        process_id: pid,
        process_path: PathBuf::from(path),
        app_name: app.to_string(),
        position: WindowPosition {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
    }
}

#[test]
fn from_active_window_valid_hwnd() {
    let window = make_active_window("Test Window", "HWND(12345)", 1234, "C:/test/app.exe", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.title, "Test Window");
    assert_eq!(info.hwnd, 12345);
    assert_eq!(info.pid, 1234);
    assert_eq!(info.process_name, "app");
    assert_eq!(info.process_path, "C:/test/app.exe");
}

#[test]
fn from_active_window_invalid_hwnd() {
    let window = make_active_window("Test", "InvalidHWND", 1234, "/test/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, 0);
}

#[test]
fn from_active_window_empty_hwnd() {
    let window = make_active_window("Test", "HWND()", 1234, "/test/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, 0);
}

#[test]
fn from_active_window_zero_hwnd() {
    let window = make_active_window("Test", "HWND(0)", 0, "/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, 0);
}

#[test]
fn from_active_window_large_hwnd() {
    let window = make_active_window("Big", "HWND(999999)", 1, "/app", "proc");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, 999999);
}

#[test]
fn window_info_debug() {
    let info = WindowInfo::new(
        12345,
        "Test Window".to_string(),
        1234,
        "app".to_string(),
        "C:/test/app.exe".to_string(),
    );
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("WindowInfo"));
    assert!(debug_str.contains("12345"));
    assert!(debug_str.contains("Test Window"));
}

#[test]
fn window_info_eq() {
    let a = WindowInfo::new(1, "A".to_string(), 10, "a".to_string(), "/a".to_string());
    let b = WindowInfo::new(1, "A".to_string(), 10, "a".to_string(), "/a".to_string());
    assert_eq!(a, b);
}

#[test]
fn window_info_ne() {
    let a = WindowInfo::new(1, "A".to_string(), 10, "a".to_string(), "/a".to_string());
    let b = WindowInfo::new(2, "B".to_string(), 20, "b".to_string(), "/b".to_string());
    assert_ne!(a, b);
}

// ============================================================================
// Concurrent WindowInfo creation
// ============================================================================

#[test]
fn concurrent_window_info_creation() {
    let handles: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                WindowInfo::new(
                    i,
                    format!("Window {}", i),
                    i as u32 * 100,
                    format!("proc_{}", i),
                    format!("/proc/{}", i),
                )
            })
        })
        .collect();

    let infos: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(infos.len(), 8);
    for (i, info) in infos.iter().enumerate() {
        assert_eq!(info.hwnd, i as isize);
    }
}
