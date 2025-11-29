//! Window utilities for getting window handles across different platforms
//!
//! This module provides cross-platform utilities for finding and working with
//! window handles, particularly useful for DCC (Digital Content Creation) applications.
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
//!
//! For `find_windows_by_title()`, we simply check if the active window matches the pattern.
//! This is sufficient for DCC use cases where you're looking for "the Blender window" and
//! you're running the script from within Blender.

use active_win_pos_rs::{get_active_window, ActiveWindow};
use pyo3::prelude::*;

/// Result of window search
#[pyclass]
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window handle (HWND on Windows, window ID on Linux, etc.)
    #[pyo3(get)]
    pub hwnd: isize,

    /// Window title
    #[pyo3(get)]
    pub title: String,

    /// Process ID
    #[pyo3(get)]
    pub pid: u32,

    /// Process name
    #[pyo3(get)]
    pub process_name: String,

    /// Process path
    #[pyo3(get)]
    pub process_path: String,
}

#[pymethods]
impl WindowInfo {
    fn __repr__(&self) -> String {
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
///
/// Returns:
///     WindowInfo or None if no foreground window
///
/// Example:
///     >>> from auroraview import get_foreground_window
///     >>> window = get_foreground_window()
///     >>> if window:
///     ...     print(f"Active window: {window.title} (HWND: {window.hwnd})")
#[pyfunction]
pub fn get_foreground_window() -> PyResult<Option<WindowInfo>> {
    match get_active_window() {
        Ok(window) => Ok(Some(window.into())),
        Err(_) => Ok(None),
    }
}

/// Find windows by title (partial match, case-insensitive)
///
/// For DCC integration, this checks if the active window matches the pattern.
/// This is perfect for DCC use cases because when you run a script in Blender/Maya/Houdini,
/// that DCC window is the active window.
///
/// Args:
///     title_pattern: String to search for in window titles
///
/// Returns:
///     List of WindowInfo objects matching the pattern (0 or 1 element)
///
/// Example:
///     >>> from auroraview import find_windows_by_title
///     >>> # Find Blender window (when running from within Blender)
///     >>> blender_windows = find_windows_by_title("Blender")
///     >>> if blender_windows:
///     ...     print(f"Found: {blender_windows[0].title}")
#[pyfunction]
pub fn find_windows_by_title(title_pattern: &str) -> PyResult<Vec<WindowInfo>> {
    let pattern = title_pattern.to_lowercase();

    match get_active_window() {
        Ok(window) => {
            if window.title.to_lowercase().contains(&pattern) {
                Ok(vec![window.into()])
            } else {
                Ok(Vec::new())
            }
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Find window by exact title match
///
/// Args:
///     title: Exact window title to search for
///
/// Returns:
///     WindowInfo or None if not found
///
/// Example:
///     >>> from auroraview import find_window_by_exact_title
///     >>> window = find_window_by_exact_title("Blender")
///     >>> if window:
///     ...     print(f"Found: {window.hwnd}")
#[pyfunction]
pub fn find_window_by_exact_title(title: &str) -> PyResult<Option<WindowInfo>> {
    let windows = find_windows_by_title(title)?;
    Ok(windows.into_iter().find(|w| w.title == title))
}

/// Get all visible windows
///
/// For DCC integration, this returns the active window (which is typically the DCC window).
/// This is sufficient for DCC use cases where you're running the script from within the DCC.
///
/// Returns:
///     List containing the active window
///
/// Example:
///     >>> from auroraview import get_all_windows
///     >>> windows = get_all_windows()
///     >>> if windows:
///     ...     print(f"Active window: {windows[0].title}")
#[pyfunction]
pub fn get_all_windows() -> PyResult<Vec<WindowInfo>> {
    match get_active_window() {
        Ok(window) => Ok(vec![window.into()]),
        Err(_) => Ok(Vec::new()),
    }
}

/// Send close message to a window by HWND (Windows only)
///
/// Sends WM_CLOSE message to the specified window handle.
/// This is a graceful close that allows the window to clean up.
///
/// Args:
///     hwnd (int): Window handle (HWND on Windows)
///
/// Returns:
///     bool: True if message was sent successfully, False otherwise
///
/// Example:
///     >>> from auroraview import close_window_by_hwnd
///     >>> hwnd = 0x12345678
///     >>> if close_window_by_hwnd(hwnd):
///     ...     print("Close message sent")
#[pyfunction]
pub fn close_window_by_hwnd(_hwnd: u64) -> PyResult<bool> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

        let hwnd_ptr = HWND(_hwnd as *mut c_void);

        unsafe {
            let result = PostMessageW(Some(hwnd_ptr), WM_CLOSE, WPARAM(0), LPARAM(0));
            if result.is_ok() {
                tracing::info!(
                    "[OK] [close_window_by_hwnd] Sent WM_CLOSE to HWND: 0x{:x}",
                    _hwnd
                );
                Ok(true)
            } else {
                tracing::error!(
                    "[ERROR] [close_window_by_hwnd] Failed to send WM_CLOSE to HWND: 0x{:x}",
                    _hwnd
                );
                Ok(false)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        tracing::warn!("[WARNING] [close_window_by_hwnd] Not supported on non-Windows platforms");
        Ok(false)
    }
}

/// Force destroy a window by HWND (Windows only)
///
/// Directly destroys the window without waiting for cleanup.
/// This is more aggressive than WM_CLOSE and should be used as a last resort.
///
/// WARNING: This bypasses normal cleanup and may cause resource leaks.
/// Use close_window_by_hwnd() first, and only use this if that fails.
///
/// Args:
///     hwnd (int): Window handle (HWND on Windows)
///
/// Returns:
///     bool: True if window was destroyed successfully, False otherwise
///
/// Example:
///     >>> from auroraview import destroy_window_by_hwnd
///     >>> hwnd = 0x12345678
///     >>> if destroy_window_by_hwnd(hwnd):
///     ...     print("Window destroyed")
#[pyfunction]
pub fn destroy_window_by_hwnd(_hwnd: u64) -> PyResult<bool> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;

        let hwnd_ptr = HWND(_hwnd as *mut c_void);

        unsafe {
            let result = DestroyWindow(hwnd_ptr);
            if result.is_ok() {
                tracing::info!(
                    "[OK] [destroy_window_by_hwnd] Destroyed window HWND: 0x{:x}",
                    _hwnd
                );
                Ok(true)
            } else {
                tracing::error!(
                    "[ERROR] [destroy_window_by_hwnd] Failed to destroy window HWND: 0x{:x}",
                    _hwnd
                );
                Ok(false)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        tracing::warn!("[WARNING] [destroy_window_by_hwnd] Not supported on non-Windows platforms");
        Ok(false)
    }
}

// ============================================================================
// Python module registration
// ============================================================================

/// Register window utilities functions with Python module
pub fn register_window_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_foreground_window, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows_by_title, m)?)?;
    m.add_function(wrap_pyfunction!(find_window_by_exact_title, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_windows, m)?)?;
    m.add_function(wrap_pyfunction!(close_window_by_hwnd, m)?)?;
    m.add_function(wrap_pyfunction!(destroy_window_by_hwnd, m)?)?;
    m.add_class::<WindowInfo>()?;
    Ok(())
}

// Note: Integration tests have been moved to tests/window_utils_integration_tests.rs
// This includes tests for:
// - get_foreground_window()
// - get_all_windows()
// - find_windows_by_title()

#[cfg(test)]
mod tests {
    use super::*;
    use active_win_pos_rs::ActiveWindow;
    use rstest::*;
    use std::path::PathBuf;

    #[fixture]
    fn sample_active_window() -> ActiveWindow {
        ActiveWindow {
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
        }
    }

    #[rstest]
    fn test_window_info_from_active_window(sample_active_window: ActiveWindow) {
        let window_info: WindowInfo = sample_active_window.into();

        assert_eq!(window_info.title, "Test Window");
        assert_eq!(window_info.hwnd, 12345);
        assert_eq!(window_info.pid, 1234);
        assert_eq!(window_info.process_name, "app");
        assert_eq!(window_info.process_path, "C:/test/app.exe");
    }

    #[rstest]
    fn test_window_info_repr(sample_active_window: ActiveWindow) {
        let window_info: WindowInfo = sample_active_window.into();
        let repr = window_info.__repr__();

        assert!(repr.contains("WindowInfo"));
        assert!(repr.contains("hwnd=12345"));
        assert!(repr.contains("Test Window"));
        assert!(repr.contains("pid=1234"));
        assert!(repr.contains("app"));
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
    fn test_register_window_utils() {
        pyo3::Python::attach(|py| {
            let m = pyo3::types::PyModule::new(py, "window_test").unwrap();
            register_window_utils(&m).expect("register should succeed");
            assert!(m.getattr("get_foreground_window").is_ok());
            assert!(m.getattr("find_windows_by_title").is_ok());
            assert!(m.getattr("find_window_by_exact_title").is_ok());
            assert!(m.getattr("get_all_windows").is_ok());
            assert!(m.getattr("close_window_by_hwnd").is_ok());
            assert!(m.getattr("destroy_window_by_hwnd").is_ok());
            assert!(m.getattr("WindowInfo").is_ok());
        });
    }

    #[test]
    fn test_window_info_clone() {
        let window = WindowInfo {
            hwnd: 12345,
            title: "Test Window".to_string(),
            pid: 1234,
            process_name: "app".to_string(),
            process_path: "C:/test/app.exe".to_string(),
        };

        let cloned = window.clone();
        assert_eq!(window.hwnd, cloned.hwnd);
        assert_eq!(window.title, cloned.title);
        assert_eq!(window.pid, cloned.pid);
        assert_eq!(window.process_name, cloned.process_name);
        assert_eq!(window.process_path, cloned.process_path);
    }

    #[test]
    fn test_window_info_debug() {
        let window = WindowInfo {
            hwnd: 12345,
            title: "Test Window".to_string(),
            pid: 1234,
            process_name: "app".to_string(),
            process_path: "C:/test/app.exe".to_string(),
        };

        let debug_str = format!("{:?}", window);
        assert!(debug_str.contains("WindowInfo"));
        assert!(debug_str.contains("12345"));
        assert!(debug_str.contains("Test Window"));
    }
}
