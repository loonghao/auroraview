//! Python bindings for WebView cleanup functions
//!
//! Provides Python API for cleaning up stale WebView user data directories
//! and getting cleanup statistics.
//!
//! Supports all platforms: Windows, macOS, Linux

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Clean up stale WebView user data directories
///
/// Scans the WebView data directory for directories belonging to
/// terminated processes and removes them.
///
/// Platform directories:
/// - Windows: `%LOCALAPPDATA%\AuroraView\WebView2\`
/// - macOS: `~/Library/Application Support/AuroraView/WebView/`
/// - Linux: `~/.local/share/auroraview/webview/`
///
/// Returns:
///     int: Number of directories cleaned up
///
/// Example:
///     >>> import auroraview
///     >>> cleaned = auroraview.cleanup_webview_dirs()
///     >>> print(f"Cleaned {cleaned} stale directories")
#[pyfunction]
pub fn cleanup_webview_dirs() -> PyResult<usize> {
    auroraview_core::cleanup::cleanup_stale_webview_dirs().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to cleanup WebView directories: {}",
            e
        ))
    })
}

/// Get cleanup statistics for WebView directories
///
/// Returns information about the current state of WebView user data directories
/// without performing any cleanup.
///
/// Returns:
///     dict: Dictionary containing:
///         - total_dirs: Total number of process directories found
///         - alive_dirs: Number of directories with alive processes
///         - stale_dirs: Number of directories with dead processes (can be cleaned)
///         - stale_size_mb: Approximate size of stale directories in megabytes
///
/// Example:
///     >>> import auroraview
///     >>> stats = auroraview.get_cleanup_stats()
///     >>> print(f"Found {stats['stale_dirs']} stale dirs ({stats['stale_size_mb']:.1f} MB)")
#[pyfunction]
pub fn get_cleanup_stats() -> Py<PyDict> {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        let stats = auroraview_core::cleanup::get_cleanup_stats();

        dict.set_item("total_dirs", stats.total_dirs).unwrap();
        dict.set_item("alive_dirs", stats.alive_dirs).unwrap();
        dict.set_item("stale_dirs", stats.stale_dirs).unwrap();
        dict.set_item(
            "stale_size_mb",
            stats.stale_size_bytes as f64 / (1024.0 * 1024.0),
        )
        .unwrap();

        dict.into()
    })
}

/// Get the WebView data directory path
///
/// Returns:
///     Optional[str]: Path to the WebView data directory, or None if not supported
///
/// Example:
///     >>> import auroraview
///     >>> path = auroraview.get_webview_data_dir()
///     >>> print(f"WebView data directory: {path}")
#[pyfunction]
pub fn get_webview_data_dir() -> Option<String> {
    auroraview_core::cleanup::get_webview_base_dir().map(|p| p.to_string_lossy().to_string())
}

/// Register cleanup functions in the Python module
pub fn register_cleanup_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cleanup_webview_dirs, m)?)?;
    m.add_function(wrap_pyfunction!(get_cleanup_stats, m)?)?;
    m.add_function(wrap_pyfunction!(get_webview_data_dir, m)?)?;
    Ok(())
}
