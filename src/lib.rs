//! AuroraView - Rust-powered WebView for Python & DCC embedding
//!
//! This library provides Python bindings for creating WebView windows in DCC applications
//! like Maya, 3ds Max, Houdini, Blender, etc.

use pyo3::prelude::*;

mod ipc;
mod metrics;
mod platform;
mod service_discovery;
mod utils;
mod webview;
mod window_utils;

#[cfg(all(target_os = "windows", feature = "win-webview2"))]
mod win_webview2_api;

#[allow(unused_imports)]
use webview::AuroraView;

pub use webview::{WebViewBuilder, WebViewConfig};

/// Python module initialization
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize logging
    utils::init_logging();

    // IMPORTANT: Allow calling Python from non-Python threads (e.g., Wry IPC thread)
    // This is required so Python callbacks can be invoked safely from Rust-created threads.
    // See PyO3 docs: prepare_freethreaded_python must be called in extension modules
    // when you'll use Python from threads not created by Python.
    pyo3::prepare_freethreaded_python();

    // Register WebView class
    m.add_class::<webview::AuroraView>()?;

    // Register window utilities
    window_utils::register_window_utils(m)?;

    // Register high-performance JSON functions (orjson-equivalent, no Python deps)
    ipc::json_bindings::register_json_functions(m)?;

    // Register service discovery module
    service_discovery::python_bindings::register_service_discovery(m)?;

    // Windows-only: register minimal WebView2 embedded API (feature-gated)
    #[cfg(all(target_os = "windows", feature = "win-webview2"))]
    {
        use pyo3::wrap_pyfunction;
        m.add_function(wrap_pyfunction!(
            win_webview2_api::win_webview2_create_embedded,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(
            win_webview2_api::win_webview2_set_bounds,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(
            win_webview2_api::win_webview2_navigate,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(win_webview2_api::win_webview2_eval, m)?)?;
        m.add_function(wrap_pyfunction!(
            win_webview2_api::win_webview2_post_message,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(win_webview2_api::win_webview2_dispose, m)?)?;
        m.add_function(wrap_pyfunction!(
            win_webview2_api::win_webview2_on_message,
            m
        )?)?;
    }

    // Add module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Hal Long <hal.long@outlook.com>")?;

    Ok(())
}

// Tests are disabled because they require Python runtime and GUI environment
// Run integration tests in Maya/Houdini/Blender instead
//
// Note: Even empty test modules require Python DLL to be present
// Use `cargo build` to verify compilation instead of `cargo test`

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pymodule_init_registers_symbols() {
        pyo3::Python::with_gil(|py| {
            let m = pyo3::types::PyModule::new(py, "auroraview_test").unwrap();
            _core(&m).expect("module init should succeed");
            assert!(m.getattr("get_all_windows").is_ok());
        });
    }
}
