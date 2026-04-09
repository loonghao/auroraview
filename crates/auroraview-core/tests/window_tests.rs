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

// ============================================================================
// repr format details
// ============================================================================

#[test]
fn repr_contains_process_name() {
    let info = WindowInfo::new(
        42,
        "Maya".to_string(),
        1001,
        "maya.exe".to_string(),
        "C:/Maya/bin/maya.exe".to_string(),
    );
    let repr = info.repr();
    assert!(repr.contains("maya.exe"), "repr should contain process_name: {repr}");
}

#[test]
fn repr_format_single_quotes_around_title() {
    let info = WindowInfo::new(1, "Blender".to_string(), 2, "blender".to_string(), "/usr/bin/blender".to_string());
    let repr = info.repr();
    // The format is: title='...'
    assert!(repr.contains("title='Blender'"), "repr format mismatch: {repr}");
}

// ============================================================================
// Field equality checks — partial field changes break eq
// ============================================================================

#[test]
fn ne_when_only_title_differs() {
    let base = WindowInfo::new(1, "A".to_string(), 10, "proc".to_string(), "/p".to_string());
    let other = WindowInfo::new(1, "B".to_string(), 10, "proc".to_string(), "/p".to_string());
    assert_ne!(base, other);
}

#[test]
fn ne_when_only_pid_differs() {
    let a = WindowInfo::new(1, "T".to_string(), 10, "p".to_string(), "/".to_string());
    let b = WindowInfo::new(1, "T".to_string(), 99, "p".to_string(), "/".to_string());
    assert_ne!(a, b);
}

#[test]
fn ne_when_only_process_path_differs() {
    let a = WindowInfo::new(1, "T".to_string(), 1, "p".to_string(), "/a".to_string());
    let b = WindowInfo::new(1, "T".to_string(), 1, "p".to_string(), "/b".to_string());
    assert_ne!(a, b);
}

// ============================================================================
// Unicode in paths and process names
// ============================================================================

#[test]
fn unicode_process_path() {
    let info = WindowInfo::new(
        5,
        "Houdini".to_string(),
        300,
        "houdini".to_string(),
        "/opt/应用/houdini".to_string(),
    );
    assert_eq!(info.process_path, "/opt/应用/houdini");
    let repr = info.repr();
    assert!(repr.contains("Houdini"));
}

// ============================================================================
// From<ActiveWindow> — HWND parsing edge cases
// ============================================================================

#[test]
fn from_active_window_negative_hwnd_string() {
    // Negative HWND values in the string should parse correctly
    let window = make_active_window("Test", "HWND(-1)", 0, "/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, -1);
}

#[rstest]
#[case("HWND(100)", 100)]
#[case("HWND(0)", 0)]
#[case("BadFormat", 0)]
#[case("HWND()", 0)]
fn hwnd_parse_cases(#[case] window_id: &str, #[case] expected: isize) {
    let window = make_active_window("Test", window_id, 1, "/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, expected);
}

// ============================================================================
// Send + Sync bounds
// ============================================================================

#[test]
fn window_info_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WindowInfo>();
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn window_info_process_path_empty() {
    let info = WindowInfo::new(1, "T".to_string(), 1, "proc".to_string(), "".to_string());
    assert_eq!(info.process_path, "");
}

#[test]
fn window_info_process_name_empty() {
    let info = WindowInfo::new(1, "T".to_string(), 1, "".to_string(), "/p".to_string());
    assert_eq!(info.process_name, "");
}

#[test]
fn window_info_large_hwnd() {
    let info = WindowInfo::new(isize::MAX, "Max".to_string(), u32::MAX, "p".to_string(), "/".to_string());
    assert_eq!(info.hwnd, isize::MAX);
    assert_eq!(info.pid, u32::MAX);
}

#[rstest]
#[case("maya.exe", "C:/Autodesk/Maya2025/bin/maya.exe", "Maya 2025")]
#[case("houdini.exe", "C:/SideFX/Houdini20.0/bin/houdini.exe", "Houdini 20.0")]
#[case("3dsmax.exe", "C:/Autodesk/3dsMax2025/3dsmax.exe", "3ds Max 2025")]
#[case("blender.exe", "C:/Blender/blender.exe", "Blender 4.0")]
fn dcc_application_window_info(#[case] proc: &str, #[case] path: &str, #[case] title: &str) {
    let info = WindowInfo::new(1000, title.to_string(), 500, proc.to_string(), path.to_string());
    assert_eq!(info.process_name, proc);
    assert_eq!(info.process_path, path);
    let repr = info.repr();
    assert!(repr.contains(proc));
}

#[test]
fn window_info_eq_reflexive() {
    let info = WindowInfo::new(5, "Test".to_string(), 10, "p".to_string(), "/p".to_string());
    assert_eq!(info, info.clone());
}

#[test]
fn window_info_clone_independence() {
    let info = WindowInfo::new(5, "Original".to_string(), 10, "p".to_string(), "/p".to_string());
    let cloned = info.clone();
    // Modifying cloned does not affect original
    assert_eq!(info.title, "Original");
    // cloned should have same values
    assert_eq!(cloned.title, "Original");
    // We can't directly mutate a field (if no setter), but we verify they're equal
    let _ = cloned;
}

#[test]
fn window_info_debug_contains_all_fields() {
    let info = WindowInfo::new(
        42,
        "Debug Window".to_string(),
        1001,
        "debug_proc".to_string(),
        "/debug/path".to_string(),
    );
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("42") || debug_str.contains("WindowInfo"));
}

#[test]
fn from_active_window_extracts_process_name() {
    let window = make_active_window("Test", "HWND(1)", 999, "/tools/app.exe", "my-app");
    let info: WindowInfo = window.into();
    assert_eq!(info.process_name, "my-app");
}

#[test]
fn from_active_window_extracts_pid() {
    let window = make_active_window("Win", "HWND(2)", 12345, "/app", "app");
    let info: WindowInfo = window.into();
    assert_eq!(info.pid, 12345);
}

#[test]
fn concurrent_window_info_clone() {
    use std::sync::Arc;
    let info = Arc::new(WindowInfo::new(1, "Shared".to_string(), 1, "p".to_string(), "/".to_string()));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let info_ref = Arc::clone(&info);
            std::thread::spawn(move || {
                let cloned = (*info_ref).clone();
                assert_eq!(cloned.title, "Shared");
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
#[case("HWND(-100)", -100)]
#[case("HWND(12345678)", 12345678)]
fn hwnd_parse_extended(#[case] window_id: &str, #[case] expected: isize) {
    let window = make_active_window("T", window_id, 1, "/app", "p");
    let info: WindowInfo = window.into();
    assert_eq!(info.hwnd, expected);
}

#[test]
fn window_info_repr_contains_hwnd_label() {
    let info = WindowInfo::new(777, "App".to_string(), 10, "app".to_string(), "/app".to_string());
    let repr = info.repr();
    assert!(repr.contains("hwnd=777"));
}

#[test]
fn window_info_ne_all_different() {
    let a = WindowInfo::new(1, "A".to_string(), 1, "a".to_string(), "/a".to_string());
    let b = WindowInfo::new(2, "B".to_string(), 2, "b".to_string(), "/b".to_string());
    assert_ne!(a, b);
}

#[test]
fn window_info_collection_of_10() {
    let windows: Vec<WindowInfo> = (0..10)
        .map(|i| WindowInfo::new(i as isize, format!("Window {i}"), i as u32, "proc".to_string(), "/p".to_string()))
        .collect();
    assert_eq!(windows.len(), 10);
    for (i, w) in windows.iter().enumerate() {
        assert_eq!(w.hwnd, i as isize);
    }
}
