//! AuroraView - Rust-powered WebView for Python & DCC embedding
//!
//! This library provides Python bindings for creating WebView windows in DCC applications
//! like Maya, 3ds Max, Houdini, Blender, etc.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
mod bindings;
mod ipc;
mod platform;
mod service_discovery;
mod utils;
mod webview;
mod window_utils;

#[allow(unused_imports)]
use webview::AuroraView;

pub use webview::{WebViewBuilder, WebViewConfig};

/// Python module initialization
#[cfg(feature = "python-bindings")]
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize logging
    utils::init_logging();

    // IMPORTANT: Allow calling Python from non-Python threads (e.g., Wry IPC thread)
    // This is required so Python callbacks can be invoked safely from Rust-created threads.
    // See PyO3 docs: Python::initialize must be called in extension modules
    // when you'll use Python from threads not created by Python.
    pyo3::Python::initialize();

    // Register WebView class
    m.add_class::<webview::AuroraView>()?;

    // Register window utilities
    window_utils::register_window_utils(m)?;

    // Register high-performance JSON functions (orjson-equivalent, no Python deps)
    bindings::ipc::register_json_functions(m)?;

    // Register service discovery module
    bindings::service_discovery::register_service_discovery(m)?;

    // Register IPC metrics class
    bindings::ipc_metrics::register_ipc_metrics(m)?;

    // Windows-only: register minimal WebView2 embedded API (feature-gated)
    #[cfg(all(target_os = "windows", feature = "win-webview2"))]
    bindings::webview2::register_webview2_api(m)?;

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

#[cfg(all(test, feature = "python-bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_pymodule_init_registers_symbols() {
        pyo3::Python::attach(|py| {
            let m = pyo3::types::PyModule::new(py, "auroraview_test").unwrap();
            _core(&m).expect("module init should succeed");
            assert!(m.getattr("get_all_windows").is_ok());
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }
}
