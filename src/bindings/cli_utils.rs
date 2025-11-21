//! Python bindings for CLI utility functions
//!
//! This module exposes Rust CLI utilities to Python, allowing the Python CLI
//! to leverage high-performance Rust implementations while maintaining uvx compatibility.

use pyo3::prelude::*;

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
    use url::Url;

    // If it already has a scheme, validate and return
    if url_str.contains("://") {
        let url = Url::parse(url_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid URL: {}", e)))?;
        return Ok(url.to_string());
    }

    // Add https:// prefix for URLs without scheme
    let with_scheme = format!("https://{}", url_str);
    let url = Url::parse(&with_scheme)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid URL: {}", e)))?;
    Ok(url.to_string())
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
#[pyfunction]
fn rewrite_html_for_custom_protocol(html: &str) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Helper function to check if a path is relative
    fn is_relative_path(path: &str) -> bool {
        !path.starts_with("http://")
            && !path.starts_with("https://")
            && !path.starts_with("data:")
            && !path.starts_with("//")
            && !path.starts_with("auroraview://")
    }

    // Rewrite link href
    let link_re = Regex::new(r#"<link\s+([^>]*)href="([^"]+)""#).unwrap();
    result = link_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<link {}href="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite script src
    let script_re = Regex::new(r#"<script\s+([^>]*)src="([^"]+)""#).unwrap();
    result = script_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<script {}src="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite img src
    let img_re = Regex::new(r#"<img\s+([^>]*)src="([^"]+)""#).unwrap();
    result = img_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<img {}src="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite CSS url()
    let css_url_re = Regex::new(r#"url\(["']?([^"':)]+)["']?\)"#).unwrap();
    result = css_url_re
        .replace_all(&result, |caps: &regex::Captures| {
            let path = &caps[1];
            if is_relative_path(path) {
                format!(r#"url("auroraview://{}")"#, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    result
}

/// Register CLI utility functions with Python module
pub fn register_cli_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize_url, m)?)?;
    m.add_function(wrap_pyfunction!(rewrite_html_for_custom_protocol, m)?)?;
    Ok(())
}
