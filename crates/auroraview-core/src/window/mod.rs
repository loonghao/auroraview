//! Window utilities core types
//!
//! This module provides core window information structures and utilities
//! that can be used across different platforms and bindings.
//!
//! ## Design Philosophy
//!
//! This implementation uses `active-win-pos-rs` to get the active window, which is
//! perfect for DCC integration because:
//!
//! 1. **DCC scripts run in the foreground** - When you execute a script in Blender/Maya/Houdini,
//!    that DCC window is almost always the active window
//! 2. **Cross-platform** - Works on Windows, macOS, and Linux
//! 3. **Simple and reliable** - No complex window enumeration needed

use active_win_pos_rs::{get_active_window, ActiveWindow};

/// Window information structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    /// Window handle (HWND on Windows, window ID on Linux, etc.)
    pub hwnd: isize,

    /// Window title
    pub title: String,

    /// Process ID
    pub pid: u32,

    /// Process name
    pub process_name: String,

    /// Process path
    pub process_path: String,
}

impl WindowInfo {
    /// Create a new WindowInfo
    pub fn new(
        hwnd: isize,
        title: String,
        pid: u32,
        process_name: String,
        process_path: String,
    ) -> Self {
        Self {
            hwnd,
            title,
            pid,
            process_name,
            process_path,
        }
    }

    /// Get a formatted representation of the window info
    pub fn repr(&self) -> String {
        format!(
            "WindowInfo(hwnd={}, title='{}', pid={}, process='{}')",
            self.hwnd, self.title, self.pid, self.process_name
        )
    }
}

/// Convert from active-win-pos-rs ActiveWindow to our WindowInfo
impl From<ActiveWindow> for WindowInfo {
    fn from(window: ActiveWindow) -> Self {
        // Parse window_id string (e.g., "HWND(9700584)") to extract the numeric ID
        let hwnd = window
            .window_id
            .trim_start_matches("HWND(")
            .trim_end_matches(")")
            .parse::<isize>()
            .unwrap_or(0);

        WindowInfo {
            hwnd,
            title: window.title,
            pid: window.process_id as u32,
            process_name: window.app_name,
            process_path: window.process_path.to_string_lossy().to_string(),
        }
    }
}

/// Get the foreground window (currently active window)
pub fn get_foreground_window() -> Option<WindowInfo> {
    get_active_window().ok().map(|w| w.into())
}

/// Find windows by title (partial match, case-insensitive)
///
/// For DCC integration, this checks if the active window matches the pattern.
pub fn find_windows_by_title(title_pattern: &str) -> Vec<WindowInfo> {
    let pattern = title_pattern.to_lowercase();

    match get_active_window() {
        Ok(window) => {
            if window.title.to_lowercase().contains(&pattern) {
                vec![window.into()]
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

/// Find window by exact title match
pub fn find_window_by_exact_title(title: &str) -> Option<WindowInfo> {
    find_windows_by_title(title)
        .into_iter()
        .find(|w| w.title == title)
}

/// Get all visible windows
///
/// For DCC integration, this returns the active window.
pub fn get_all_windows() -> Vec<WindowInfo> {
    get_active_window()
        .ok()
        .map(|w| vec![w.into()])
        .unwrap_or_default()
}

/// Send close message to a window by HWND (Windows only)
#[cfg(target_os = "windows")]
pub fn close_window_by_hwnd(hwnd: u64) -> bool {
    use std::ffi::c_void;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

    let hwnd_ptr = HWND(hwnd as *mut c_void);

    unsafe {
        let result = PostMessageW(Some(hwnd_ptr), WM_CLOSE, WPARAM(0), LPARAM(0));
        if result.is_ok() {
            tracing::info!(
                "[OK] [close_window_by_hwnd] Sent WM_CLOSE to HWND: 0x{:x}",
                hwnd
            );
            true
        } else {
            tracing::error!(
                "[ERROR] [close_window_by_hwnd] Failed to send WM_CLOSE to HWND: 0x{:x}",
                hwnd
            );
            false
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn close_window_by_hwnd(_hwnd: u64) -> bool {
    tracing::warn!("[WARNING] [close_window_by_hwnd] Not supported on non-Windows platforms");
    false
}

/// Force destroy a window by HWND (Windows only)
#[cfg(target_os = "windows")]
pub fn destroy_window_by_hwnd(hwnd: u64) -> bool {
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;

    let hwnd_ptr = HWND(hwnd as *mut c_void);

    unsafe {
        let result = DestroyWindow(hwnd_ptr);
        if result.is_ok() {
            tracing::info!(
                "[OK] [destroy_window_by_hwnd] Destroyed window HWND: 0x{:x}",
                hwnd
            );
            true
        } else {
            tracing::error!(
                "[ERROR] [destroy_window_by_hwnd] Failed to destroy window HWND: 0x{:x}",
                hwnd
            );
            false
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn destroy_window_by_hwnd(_hwnd: u64) -> bool {
    tracing::warn!("[WARNING] [destroy_window_by_hwnd] Not supported on non-Windows platforms");
    false
}

#[cfg(test)]
mod tests {
    use super::*;
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
            position: active_win_pos_rs::WindowPosition {
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
            position: active_win_pos_rs::WindowPosition {
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
            position: active_win_pos_rs::WindowPosition {
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
}
