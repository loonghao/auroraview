//! AuroraView - Rust-powered WebView for Python & DCC embedding
//!
//! This library provides Python bindings for creating WebView windows in DCC applications
//! like Maya, 3ds Max, Houdini, Blender, etc.

use pyo3::prelude::*;

mod ipc;
mod metrics;
mod utils;
mod webview;

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
