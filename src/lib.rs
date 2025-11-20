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
pub mod service_discovery;
mod utils;
pub mod webview;
pub mod window_utils;

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

// Comprehensive module import tests
#[cfg(test)]
mod tests {
    //! Comprehensive module import tests for AuroraView
    //!
    //! These tests ensure all modules can be compiled and their public APIs are accessible.
    //! This provides broad coverage of the codebase without requiring runtime dependencies.

    /// Test that core webview modules compile and export expected symbols
    #[test]
    fn test_webview_module_imports() {
        // Test webview module structure
        use crate::webview;

        // Verify main types are accessible
        let _: Option<webview::AuroraView> = None;
        let _: Option<webview::WebViewConfig> = None;
        let _: Option<webview::WebViewBuilder> = None;
    }

    /// Test IPC module imports
    #[test]
    fn test_ipc_module_imports() {
        use crate::ipc;

        // Verify IPC types are accessible
        let _: Option<ipc::IpcHandler> = None;
        let _: Option<ipc::MessageQueue> = None;
        let _: Option<ipc::IpcMetrics> = None;
    }

    /// Test service discovery module imports
    #[test]
    fn test_service_discovery_imports() {
        use crate::service_discovery;

        // Verify service discovery types are accessible
        let _: Option<service_discovery::PortAllocator> = None;
    }

    /// Test window utilities module imports
    #[test]
    fn test_window_utils_imports() {
        use crate::window_utils;

        // Verify window utils functions are accessible (compile-time check)
        let _ = window_utils::get_all_windows;
    }

    /// Test platform-specific modules compile
    #[test]
    fn test_platform_module_imports() {
        #[cfg(target_os = "windows")]
        {
            // Verify Windows platform module compiles
            #[allow(unused_imports)]
            use crate::platform::windows;
            // Module exists and compiles
            let _: Option<()> = None;
        }
    }

    /// Test utils module imports
    #[test]
    fn test_utils_module_imports() {
        use crate::utils;

        // Verify utils functions are accessible
        let _ = utils::init_logging;
    }

    /// Test webview submodules
    #[test]
    fn test_webview_submodules() {
        // Test backend module (public)
        use crate::webview::backend;
        let _: Option<backend::BackendType> = None;

        // Test js_assets module (public)
        use crate::webview::js_assets;
        // Verify constants are accessible
        let _: &str = js_assets::EVENT_BRIDGE;
    }

    /// Test IPC submodules
    #[test]
    fn test_ipc_submodules() {
        // Test handler module
        use crate::ipc::handler;
        let _: Option<handler::IpcHandler> = None;

        // Test message queue module
        use crate::ipc::message_queue;
        let _: Option<message_queue::MessageQueue> = None;

        // Test metrics module
        use crate::ipc::metrics;
        let _: Option<metrics::IpcMetrics> = None;

        // Test dead letter queue module
        use crate::ipc::dead_letter_queue;
        let _: Option<dead_letter_queue::DeadLetterQueue> = None;

        // Test backend module
        use crate::ipc::backend;
        let _: Option<backend::IpcMessage> = None;

        // Test threaded module
        use crate::ipc::threaded;
        let _: Option<threaded::ThreadedBackend> = None;
    }

    /// Test that Python bindings module compiles (when feature is enabled)
    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_python_bindings_imports() {
        use crate::bindings;

        // Verify bindings modules compile
        let _ = bindings::ipc::register_json_functions;
        let _ = bindings::service_discovery::register_service_discovery;
        let _ = bindings::ipc_metrics::register_ipc_metrics;
    }

    /// Test WebView2 bindings (Windows + feature flag)
    #[cfg(all(
        target_os = "windows",
        feature = "win-webview2",
        feature = "python-bindings"
    ))]
    #[test]
    fn test_webview2_bindings_imports() {
        use crate::bindings::webview2;

        // Verify WebView2 bindings compile
        let _ = webview2::register_webview2_api;
    }

    /// Test that all public re-exports are accessible
    #[test]
    fn test_public_api_exports() {
        // Test top-level exports from lib.rs
        let _: Option<crate::WebViewConfig> = None;
        let _: Option<crate::WebViewBuilder> = None;
        let _: Option<crate::AuroraView> = None;
    }

    /// Python module initialization test
    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_pymodule_init_registers_symbols() {
        use super::*;

        pyo3::Python::attach(|py| {
            let m = pyo3::types::PyModule::new(py, "auroraview_test").unwrap();
            _core(&m).expect("module init should succeed");
            assert!(m.getattr("get_all_windows").is_ok());
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }
}
