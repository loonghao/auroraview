//! Python bindings for static assets
//!
//! This module exposes JavaScript and HTML assets to Python for testing
//! and advanced use cases where scripts need to be injected manually.

use pyo3::prelude::*;

use crate::webview::js_assets;

/// Get the Midscene AI testing bridge JavaScript code.
///
/// This script provides browser-side utilities for AI-powered UI testing:
/// - DOM analysis and element location
/// - Screenshot capture
/// - Element interaction helpers
/// - Page state inspection
///
/// The bridge is accessible via `window.__midscene_bridge__` or
/// `window.auroraview.midscene` when the event bridge is loaded.
///
/// Returns:
///     JavaScript code as a string.
///
/// Example:
///     ```python
///     from auroraview._core import get_midscene_bridge_js
///
///     script = get_midscene_bridge_js()
///     await page.evaluate(script)
///     # Now window.__midscene_bridge__ is available
///     ```
#[pyfunction]
pub fn get_midscene_bridge_js() -> String {
    js_assets::midscene_bridge()
}

/// Get the event bridge JavaScript code.
///
/// This is the core AuroraView bridge that provides:
/// - `window.auroraview.call()` - Call Python methods
/// - `window.auroraview.on()` - Subscribe to events
/// - `window.auroraview.send_event()` - Send events to Python
///
/// Returns:
///     JavaScript code as a string.
#[pyfunction]
pub fn get_event_bridge_js() -> String {
    js_assets::event_bridge()
}

/// Get the test callback JavaScript code.
///
/// This script provides callback mechanism for AuroraTest framework
/// to receive JavaScript evaluation results asynchronously.
///
/// Returns:
///     JavaScript code as a string.
#[pyfunction]
pub fn get_test_callback_js() -> String {
    js_assets::test_callback()
}

/// Get the context menu disable JavaScript code.
///
/// This script disables the browser's default context menu.
///
/// Returns:
///     JavaScript code as a string.
#[pyfunction]
pub fn get_context_menu_js() -> String {
    js_assets::context_menu()
}

/// Register asset functions to the Python module.
pub fn register_assets_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_midscene_bridge_js, m)?)?;
    m.add_function(wrap_pyfunction!(get_event_bridge_js, m)?)?;
    m.add_function(wrap_pyfunction!(get_test_callback_js, m)?)?;
    m.add_function(wrap_pyfunction!(get_context_menu_js, m)?)?;
    Ok(())
}
