//! Python bindings for WindowManager
//!
//! Exposes window management functions to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::webview::window_manager::WindowManager;

/// Get all window labels
#[pyfunction]
pub fn get_all_window_labels() -> Vec<String> {
    WindowManager::global().get_all_labels()
}

/// Get window count
#[pyfunction]
pub fn get_window_count() -> usize {
    WindowManager::global().window_count()
}

/// Check if window exists
#[pyfunction]
pub fn has_window(label: &str) -> bool {
    WindowManager::global().has_window(label)
}

/// Get main window label
#[pyfunction]
pub fn get_main_window_label() -> Option<String> {
    WindowManager::global().get_main_window_label()
}

/// Get window info by label
#[pyfunction]
pub fn get_window_info(py: Python<'_>, label: &str) -> Option<Py<PyDict>> {
    let info = WindowManager::global().get_window(label)?;
    let dict = PyDict::new(py);
    dict.set_item("label", &info.label).ok()?;
    dict.set_item("title", &info.title).ok()?;
    dict.set_item("is_main", info.is_main).ok()?;
    dict.set_item("url", &info.url).ok()?;
    dict.set_item("width", info.width).ok()?;
    dict.set_item("height", info.height).ok()?;
    dict.set_item("visible", info.visible).ok()?;
    dict.set_item("focused", info.focused).ok()?;
    dict.set_item("parent_label", &info.parent_label).ok()?;
    Some(dict.into())
}

/// Get all windows info
#[pyfunction]
pub fn get_all_windows_info(py: Python<'_>) -> Vec<Py<PyDict>> {
    WindowManager::global()
        .get_all_windows()
        .into_iter()
        .filter_map(|info| {
            let dict = PyDict::new(py);
            dict.set_item("label", &info.label).ok()?;
            dict.set_item("title", &info.title).ok()?;
            dict.set_item("is_main", info.is_main).ok()?;
            dict.set_item("url", &info.url).ok()?;
            dict.set_item("width", info.width).ok()?;
            dict.set_item("height", info.height).ok()?;
            dict.set_item("visible", info.visible).ok()?;
            dict.set_item("focused", info.focused).ok()?;
            dict.set_item("parent_label", &info.parent_label).ok()?;
            Some(dict.into())
        })
        .collect()
}

/// Emit event to a specific window
#[pyfunction]
pub fn emit_to_window(label: &str, event: &str, data: &str) -> bool {
    let data: serde_json::Value = serde_json::from_str(data).unwrap_or(serde_json::Value::Null);
    WindowManager::global().emit_to(label, event, data)
}

/// Emit event to all windows
#[pyfunction]
pub fn emit_to_all_windows(event: &str, data: &str) {
    let data: serde_json::Value = serde_json::from_str(data).unwrap_or(serde_json::Value::Null);
    WindowManager::global().emit_all(event, data);
}

/// Emit event to all windows except one
#[pyfunction]
pub fn emit_to_other_windows(except_label: &str, event: &str, data: &str) {
    let data: serde_json::Value = serde_json::from_str(data).unwrap_or(serde_json::Value::Null);
    WindowManager::global().emit_others(except_label, event, data);
}

/// Register window manager functions with Python module
pub fn register_window_manager_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_all_window_labels, m)?)?;
    m.add_function(wrap_pyfunction!(get_window_count, m)?)?;
    m.add_function(wrap_pyfunction!(has_window, m)?)?;
    m.add_function(wrap_pyfunction!(get_main_window_label, m)?)?;
    m.add_function(wrap_pyfunction!(get_window_info, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_windows_info, m)?)?;
    m.add_function(wrap_pyfunction!(emit_to_window, m)?)?;
    m.add_function(wrap_pyfunction!(emit_to_all_windows, m)?)?;
    m.add_function(wrap_pyfunction!(emit_to_other_windows, m)?)?;
    Ok(())
}
