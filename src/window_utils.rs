//! Window utilities - Python bindings
//!
//! This module provides Python bindings for window utilities.
//! Core implementations are in auroraview-core.

use auroraview_core::window as core_window;
use pyo3::prelude::*;

/// Result of window search - Python wrapper
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

impl From<core_window::WindowInfo> for WindowInfo {
    fn from(info: core_window::WindowInfo) -> Self {
        WindowInfo {
            hwnd: info.hwnd,
            title: info.title,
            pid: info.pid,
            process_name: info.process_name,
            process_path: info.process_path,
        }
    }
}

/// Get the foreground window (currently active window)
#[pyfunction]
pub fn get_foreground_window() -> PyResult<Option<WindowInfo>> {
    Ok(core_window::get_foreground_window().map(|w| w.into()))
}

/// Find windows by title (partial match, case-insensitive)
#[pyfunction]
pub fn find_windows_by_title(title_pattern: &str) -> PyResult<Vec<WindowInfo>> {
    Ok(core_window::find_windows_by_title(title_pattern)
        .into_iter()
        .map(|w| w.into())
        .collect())
}

/// Find window by exact title match
#[pyfunction]
pub fn find_window_by_exact_title(title: &str) -> PyResult<Option<WindowInfo>> {
    Ok(core_window::find_window_by_exact_title(title).map(|w| w.into()))
}

/// Get all visible windows
#[pyfunction]
pub fn get_all_windows() -> PyResult<Vec<WindowInfo>> {
    Ok(core_window::get_all_windows()
        .into_iter()
        .map(|w| w.into())
        .collect())
}

/// Send close message to a window by HWND (Windows only)
#[pyfunction]
pub fn close_window_by_hwnd(hwnd: u64) -> PyResult<bool> {
    Ok(core_window::close_window_by_hwnd(hwnd))
}

/// Force destroy a window by HWND (Windows only)
#[pyfunction]
pub fn destroy_window_by_hwnd(hwnd: u64) -> PyResult<bool> {
    Ok(core_window::destroy_window_by_hwnd(hwnd))
}

/// Fix WebView2 child windows to prevent dragging (Qt6 compatibility)
///
/// WebView2 creates multiple child windows (Chrome_WidgetWin_0, etc.) that may
/// not inherit proper WS_CHILD styles. This function recursively fixes all child
/// windows to ensure they cannot be dragged independently.
///
/// This is especially important for Qt6 where createWindowContainer behavior
/// differs from Qt5.
///
/// Args:
///     hwnd: The WebView window handle (HWND)
///
/// Returns:
///     True if successful, False otherwise (non-Windows platforms)
#[pyfunction]
pub fn fix_webview2_child_windows(hwnd: u64) -> PyResult<bool> {
    #[cfg(target_os = "windows")]
    {
        crate::webview::backend::native::NativeBackend::fix_webview2_child_windows(hwnd as isize);
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
        Ok(false)
    }
}

/// Register window utilities functions with Python module
pub fn register_window_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_foreground_window, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows_by_title, m)?)?;
    m.add_function(wrap_pyfunction!(find_window_by_exact_title, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_windows, m)?)?;
    m.add_function(wrap_pyfunction!(close_window_by_hwnd, m)?)?;
    m.add_function(wrap_pyfunction!(destroy_window_by_hwnd, m)?)?;
    m.add_function(wrap_pyfunction!(fix_webview2_child_windows, m)?)?;
    m.add_class::<WindowInfo>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
