//! Python bindings for CLI utility functions
//!
//! This module exposes Rust CLI utilities to Python, allowing the Python CLI
//! to leverage high-performance Rust implementations while maintaining uvx compatibility.
//!
//! The core logic lives in `auroraview-core::cli`, this module only provides
//! PyO3 bindings.

use pyo3::prelude::*;

// Re-export core functions for Rust users
pub use auroraview_core::cli::{
    normalize_url as core_normalize_url, rewrite_html_for_custom_protocol,
};

/// Normalize URL by adding https:// prefix if missing
///
/// # Arguments
/// * `url_str` - URL string to normalize
///
/// # Returns
/// Normalized URL with proper scheme
///
/// # Examples
/// ```python
/// from auroraview import normalize_url
///
/// url = normalize_url("example.com")
/// assert url == "https://example.com"
///
/// url = normalize_url("http://example.com")
/// assert url == "http://example.com"
/// ```
#[pyfunction]
fn normalize_url(url_str: &str) -> PyResult<String> {
    core_normalize_url(url_str).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Rewrite HTML to use auroraview:// protocol for relative paths
///
/// This function rewrites HTML content to use the custom auroraview:// protocol
/// for relative resource paths (CSS, JS, images), enabling proper loading of
/// local assets through the custom protocol handler.
///
/// # Arguments
/// * `html` - HTML content to rewrite
///
/// # Returns
/// Rewritten HTML with auroraview:// protocol for relative paths
///
/// # Examples
/// ```python
/// from auroraview import rewrite_html_for_custom_protocol
///
/// html = '<link href="style.css" rel="stylesheet">'
/// rewritten = rewrite_html_for_custom_protocol(html)
/// assert 'href="auroraview://style.css"' in rewritten
/// ```
#[pyfunction(name = "rewrite_html_for_custom_protocol")]
fn py_rewrite_html_for_custom_protocol(html: &str) -> String {
    rewrite_html_for_custom_protocol(html)
}

/// Register CLI utility functions with Python module
pub fn register_cli_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize_url, m)?)?;
    m.add_function(wrap_pyfunction!(py_rewrite_html_for_custom_protocol, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_normalize_url_without_scheme() {
        Python::attach(|_py| {
            let result = normalize_url("example.com").unwrap();
            assert_eq!(result, "https://example.com/");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_normalize_url_with_http() {
        Python::attach(|_py| {
            let result = normalize_url("http://example.com").unwrap();
            assert_eq!(result, "http://example.com/");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_normalize_url_with_https() {
        Python::attach(|_py| {
            let result = normalize_url("https://example.com/path").unwrap();
            assert_eq!(result, "https://example.com/path");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_normalize_url_with_port() {
        Python::attach(|_py| {
            let result = normalize_url("localhost:8080").unwrap();
            assert_eq!(result, "https://localhost:8080/");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_normalize_url_invalid() {
        Python::attach(|_py| {
            let result = normalize_url("://invalid");
            assert!(result.is_err());
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_rewrite_html_preserves_anchor_links() {
        let html = "<a href=\"#section\">Link</a>";
        let result = rewrite_html_for_custom_protocol(html);
        assert!(result.contains("href=\"#section\""));
    }

    #[test]
    fn test_rewrite_html_empty_input() {
        let html = "";
        let result = rewrite_html_for_custom_protocol(html);
        assert_eq!(result, "");
    }

    #[test]
    fn test_register_cli_utils_module() {
        Python::attach(|py| {
            let m = pyo3::types::PyModule::new(py, "cli_test").unwrap();
            register_cli_utils(&m).expect("register should succeed");
            assert!(m.getattr("normalize_url").is_ok());
            // Function is registered with name="rewrite_html_for_custom_protocol"
            assert!(m.getattr("rewrite_html_for_custom_protocol").is_ok());
        });
    }
}
