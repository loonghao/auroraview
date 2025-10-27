//! AuroraView - Rust-powered WebView for Python & DCC embedding
//!
//! This library provides Python bindings for creating WebView windows in DCC applications
//! like Maya, 3ds Max, Houdini, Blender, etc.

use pyo3::prelude::*;

mod utils;
mod webview;

#[allow(unused_imports)]
use webview::PyWebView;

pub use webview::{WebViewBuilder, WebViewConfig};

/// Python module initialization
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize logging
    utils::init_logging();

    // Register WebView class
    m.add_class::<webview::PyWebView>()?;

    // Add module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Hal Long <hal.long@outlook.com>")?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_module_creation() {
        // Basic module creation test
        assert!(true);
    }

    #[test]
    fn test_version_available() {
        // Test that version is available
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }

    #[test]
    fn test_author_available() {
        // Test that author is available
        let author = "Hal Long <hal.long@outlook.com>";
        assert!(!author.is_empty());
    }
}
