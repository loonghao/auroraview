//! Python bindings for WebView2 warmup/preheat functionality
//!
//! This module exposes the WebView2 warmup API to Python, allowing
//! DCC applications to pre-initialize WebView2 for faster window creation.

use pyo3::prelude::*;

#[cfg(target_os = "windows")]
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use crate::platform::windows::warmup;

/// Start WebView2 warmup in background thread
///
/// This function initiates background pre-initialization of WebView2:
/// 1. Initializes COM in STA mode
/// 2. Creates WebView2 Environment (triggers runtime discovery)
/// 3. Pre-allocates shared user data folder
///
/// Call this early in your application startup (e.g., when importing auroraview).
///
/// Args:
///     user_data_folder: Optional path for shared user data folder.
///         If None, uses system default (%LOCALAPPDATA%\AuroraView\WebView2)
///
/// Example:
///     >>> import auroraview
///     >>> auroraview.start_warmup()  # Start background warmup
///     >>> # ... do other initialization ...
///     >>> webview = auroraview.WebView.create(...)  # Fast creation!
#[pyfunction]
#[pyo3(signature = (user_data_folder=None))]
pub fn start_warmup(user_data_folder: Option<String>) {
    #[cfg(target_os = "windows")]
    {
        let path = user_data_folder.map(PathBuf::from);
        warmup::start_warmup(path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = user_data_folder;
        tracing::debug!("[warmup] Warmup is Windows-only, skipping");
    }
}

/// Synchronous warmup - blocks until WebView2 is ready
///
/// Use this if you need to ensure WebView2 is fully initialized before
/// creating windows. For most cases, prefer `start_warmup()` for
/// non-blocking background initialization.
///
/// Args:
///     user_data_folder: Optional path for shared user data folder
///     timeout_ms: Maximum time to wait in milliseconds (default: 30000)
///
/// Returns:
///     True if warmup succeeded, False if failed or timed out
///
/// Example:
///     >>> import auroraview
///     >>> if auroraview.warmup_sync():
///     ...     print("WebView2 ready!")
#[pyfunction]
#[pyo3(signature = (user_data_folder=None, timeout_ms=None))]
pub fn warmup_sync(user_data_folder: Option<String>, timeout_ms: Option<u64>) -> bool {
    #[cfg(target_os = "windows")]
    {
        let path = user_data_folder.map(PathBuf::from);
        warmup::warmup_sync(path, timeout_ms).is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (user_data_folder, timeout_ms);
        true // Always "ready" on non-Windows
    }
}

/// Check if warmup is complete
///
/// Returns:
///     True if warmup has completed (successfully or with error)
#[pyfunction]
pub fn is_warmup_complete() -> bool {
    #[cfg(target_os = "windows")]
    {
        warmup::is_warmup_complete()
    }

    #[cfg(not(target_os = "windows"))]
    {
        true // Always "complete" on non-Windows
    }
}

/// Get warmup progress percentage (0-100)
///
/// Returns:
///     int: Progress percentage (0-100)
#[pyfunction]
pub fn get_warmup_progress() -> u8 {
    #[cfg(target_os = "windows")]
    {
        warmup::get_warmup_progress()
    }

    #[cfg(not(target_os = "windows"))]
    {
        100 // Always "complete" on non-Windows
    }
}

/// Get warmup stage description
///
/// Returns:
///     str: Human-readable description of current warmup stage
#[pyfunction]
pub fn get_warmup_stage() -> String {
    #[cfg(target_os = "windows")]
    {
        warmup::get_warmup_stage_description()
    }

    #[cfg(not(target_os = "windows"))]
    {
        "Ready".to_string()
    }
}

/// Get warmup status information
///
/// Returns:
///     dict with keys:
///         - initiated: bool - Whether warmup has been started
///         - complete: bool - Whether warmup has finished
///         - progress: int - Progress percentage (0-100)
///         - stage: str - Current stage description
///         - duration_ms: Optional[int] - Warmup duration in milliseconds
///         - error: Optional[str] - Error message if failed
///         - user_data_folder: Optional[str] - Path to shared user data folder
#[pyfunction]
pub fn get_warmup_status() -> pyo3::Py<pyo3::types::PyDict> {
    Python::attach(|py| {
        let dict = pyo3::types::PyDict::new(py);

        #[cfg(target_os = "windows")]
        {
            let status = warmup::get_warmup_status();
            dict.set_item("initiated", status.initiated).unwrap();
            dict.set_item("complete", status.complete).unwrap();
            dict.set_item("progress", status.stage.progress()).unwrap();
            dict.set_item("stage", status.stage.description()).unwrap();
            dict.set_item("duration_ms", status.duration_ms).unwrap();
            dict.set_item("error", status.error).unwrap();
            dict.set_item(
                "user_data_folder",
                status
                    .user_data_folder
                    .map(|p| p.to_string_lossy().to_string()),
            )
            .unwrap();
        }

        #[cfg(not(target_os = "windows"))]
        {
            dict.set_item("initiated", false).unwrap();
            dict.set_item("complete", true).unwrap();
            dict.set_item("progress", 100u8).unwrap();
            dict.set_item("stage", "Ready").unwrap();
            dict.set_item("duration_ms", Option::<u64>::None).unwrap();
            dict.set_item("error", Option::<String>::None).unwrap();
            dict.set_item("user_data_folder", Option::<String>::None)
                .unwrap();
        }

        dict.into()
    })
}

/// Get shared user data folder path
///
/// Returns:
///     Optional path to the shared user data folder, or None if not set
#[pyfunction]
pub fn get_shared_user_data_folder() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        warmup::get_shared_user_data_folder().map(|p| p.to_string_lossy().to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Register warmup functions in the Python module
pub fn register_warmup_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_warmup, m)?)?;
    m.add_function(wrap_pyfunction!(warmup_sync, m)?)?;
    m.add_function(wrap_pyfunction!(is_warmup_complete, m)?)?;
    m.add_function(wrap_pyfunction!(get_warmup_progress, m)?)?;
    m.add_function(wrap_pyfunction!(get_warmup_stage, m)?)?;
    m.add_function(wrap_pyfunction!(get_warmup_status, m)?)?;
    m.add_function(wrap_pyfunction!(get_shared_user_data_folder, m)?)?;
    Ok(())
}
