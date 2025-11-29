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
pub fn rewrite_html_for_custom_protocol(html: &str) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Helper function to check if a path is relative (should be rewritten)
    fn is_relative_path(path: &str) -> bool {
        !path.starts_with("http://")
            && !path.starts_with("https://")
            && !path.starts_with("data:")
            && !path.starts_with("//")
            && !path.starts_with("auroraview://")
            && !path.starts_with('#') // Anchor links
    }

    // Helper function to normalize relative path
    // Strips leading "./" prefix for cleaner URLs
    // Keeps "../" as the protocol handler will resolve it
    fn normalize_path(path: &str) -> &str {
        path.strip_prefix("./").unwrap_or(path)
    }

    // Rewrite link href
    let link_re = Regex::new(r#"<link\s+([^>]*)href="([^"]+)""#).unwrap();
    result = link_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                let normalized = normalize_path(path);
                format!(r#"<link {}href="auroraview://{}""#, attrs, normalized)
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
                let normalized = normalize_path(path);
                format!(r#"<script {}src="auroraview://{}""#, attrs, normalized)
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
                let normalized = normalize_path(path);
                format!(r#"<img {}src="auroraview://{}""#, attrs, normalized)
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
                let normalized = normalize_path(path);
                format!(r#"url("auroraview://{}")"#, normalized)
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

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_rewrite_relative_paths() {
        let html = r#"
        <html>
            <head>
                <link rel="stylesheet" href="./style.css">
                <link rel="stylesheet" href="styles/main.css">
            </head>
            <body>
                <script src="./script.js"></script>
                <script src="js/app.js"></script>
                <img src="./logo.png">
                <img src="images/icon.png">
            </body>
        </html>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        // Check that relative paths are rewritten (./xxx -> xxx)
        assert!(result.contains(r#"href="auroraview://style.css""#));
        assert!(result.contains(r#"href="auroraview://styles/main.css""#));
        assert!(result.contains(r#"src="auroraview://script.js""#));
        assert!(result.contains(r#"src="auroraview://js/app.js""#));
        assert!(result.contains(r#"src="auroraview://logo.png""#));
        assert!(result.contains(r#"src="auroraview://images/icon.png""#));
    }

    #[test]
    fn test_preserve_absolute_urls() {
        let html = r#"
        <html>
            <head>
                <link rel="stylesheet" href="https://cdn.example.com/style.css">
                <link rel="stylesheet" href="http://example.com/style.css">
            </head>
            <body>
                <script src="https://cdn.example.com/script.js"></script>
                <img src="data:image/png;base64,ABC123">
                <img src="//cdn.example.com/image.png">
            </body>
        </html>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        // Check that absolute URLs are preserved
        assert!(result.contains(r#"href="https://cdn.example.com/style.css""#));
        assert!(result.contains(r#"href="http://example.com/style.css""#));
        assert!(result.contains(r#"src="https://cdn.example.com/script.js""#));
        assert!(result.contains(r#"src="data:image/png;base64,ABC123""#));
        assert!(result.contains(r#"src="//cdn.example.com/image.png""#));
    }

    #[test]
    fn test_preserve_existing_auroraview_protocol() {
        let html = r#"
        <html>
            <head>
                <link rel="stylesheet" href="auroraview://style.css">
            </head>
            <body>
                <script src="auroraview://script.js"></script>
            </body>
        </html>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        // Check that auroraview:// URLs are preserved (not double-rewritten)
        assert!(result.contains(r#"href="auroraview://style.css""#));
        assert!(result.contains(r#"src="auroraview://script.js""#));
        // Make sure there's no double protocol
        assert!(!result.contains("auroraview://auroraview://"));
    }

    #[test]
    fn test_rewrite_parent_directory_paths() {
        let html = r#"
        <html>
            <head>
                <link rel="stylesheet" href="../assets/style.css">
            </head>
            <body>
                <script src="../js/app.js"></script>
                <img src="../../images/logo.png">
            </body>
        </html>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        // Check that parent directory paths are rewritten
        assert!(result.contains(r#"href="auroraview://../assets/style.css""#));
        assert!(result.contains(r#"src="auroraview://../js/app.js""#));
        assert!(result.contains(r#"src="auroraview://../../images/logo.png""#));
    }

    #[test]
    fn test_rewrite_css_url() {
        let html = r#"
        <style>
            body {
                background: url('./images/bg.png');
            }
            .icon {
                background-image: url(icons/icon.svg);
            }
        </style>
        "#;

        let result = rewrite_html_for_custom_protocol(html);

        // Check that CSS url() references are rewritten
        assert!(result.contains(r#"url("auroraview://images/bg.png")"#));
        assert!(result.contains(r#"url("auroraview://icons/icon.svg")"#));
    }

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
            // Invalid URL should return error
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
        // Anchor links should be preserved (not rewritten)
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
            assert!(m.getattr("rewrite_html_for_custom_protocol").is_ok());
        });
    }
}
