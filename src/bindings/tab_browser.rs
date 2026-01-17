//! Python bindings for multi-tab browser (TabManager)
//!
//! This module exposes the `TabManager` to Python, allowing users to create
//! multi-tab browser applications following Microsoft WebView2Browser architecture.
//!
//! ## Architecture Reference
//!
//! Based on the official Microsoft Edge WebView2Browser sample:
//! - Repository: <https://github.com/MicrosoftEdge/WebView2Browser>
//! - Single UI thread with shared `CoreWebView2Environment`
//! - All tabs managed by show/hide (not create/destroy)
//!
//! ## Usage Example
//!
//! ```python
//! from auroraview._core import run_browser
//!
//! # Simple usage - opens a single-tab browser
//! run_browser(
//!     title="My Browser",
//!     width=1280,
//!     height=900,
//!     home_url="https://www.google.com",
//! )
//!
//! # With multiple initial tabs
//! run_browser(
//!     title="Multi-Tab Browser",
//!     initial_urls=["https://google.com", "https://github.com"],
//!     debug=True,
//! )
//! ```

use pyo3::prelude::*;

use crate::webview::tab_manager::{TabManager, TabManagerConfig};

/// Run a multi-tab browser (blocking until window closes)
///
/// This function creates and runs a multi-tab browser window using the
/// TabManager architecture based on Microsoft WebView2Browser.
///
/// Key features:
/// - Single event loop for all WebViews (correct WebView2 threading model)
/// - Shared CoreWebView2Environment (automatic with wry)
/// - Tab management (create, close, switch tabs)
/// - Navigation controls (back, forward, reload)
/// - URL bar with search support
///
/// Args:
///     title (str): Window title
///     width (int): Window width in pixels (default: 1280)
///     height (int): Window height in pixels (default: 900)
///     home_url (str): Home page URL (default: https://www.google.com)
///     debug (bool): Enable DevTools (default: False)
///     initial_urls (list[str]): URLs to open as initial tabs
///
/// Example:
///     >>> from auroraview._core import run_browser
///     >>> run_browser(
///     ...     title="My Browser",
///     ...     width=1280,
///     ...     height=900,
///     ...     home_url="https://github.com",
///     ...     debug=True,
///     ... )
///
/// See Also:
///     - Microsoft WebView2Browser: <https://github.com/MicrosoftEdge/WebView2Browser>
///     - AuroraView documentation: multi-tab-browser.md
#[pyfunction]
#[pyo3(signature = (
    title="AuroraView Browser",
    width=1280,
    height=900,
    home_url="https://www.google.com",
    debug=false,
    initial_urls=None
))]
fn run_browser(
    title: &str,
    width: u32,
    height: u32,
    home_url: &str,
    debug: bool,
    initial_urls: Option<Vec<String>>,
) -> PyResult<()> {
    tracing::info!(
        "[run_browser] Starting multi-tab browser: {} ({}x{})",
        title,
        width,
        height
    );

    // Build configuration
    let mut config = TabManagerConfig::default()
        .with_title(title)
        .with_size(width, height)
        .with_home_url(home_url)
        .with_debug(debug);

    if let Some(urls) = initial_urls {
        config = config.with_initial_urls(urls);
    }

    // Create and run the tab manager
    // Note: TabManager contains wry::WebView which is !Send, so we cannot use
    // py.allow_threads() here. The browser runs on the main thread.
    // For Python callbacks during browser execution, use IPC/message passing.
    let mut manager = TabManager::new(config);
    manager.run();

    tracing::info!("[run_browser] Browser closed");
    Ok(())
}

/// Alias for run_browser (deprecated, use run_browser instead)
#[pyfunction]
#[pyo3(signature = (
    title="AuroraView Browser",
    width=1280,
    height=900,
    home_url="https://www.google.com",
    debug=false,
    initial_urls=None
))]
fn run_tab_browser(
    title: &str,
    width: u32,
    height: u32,
    home_url: &str,
    debug: bool,
    initial_urls: Option<Vec<String>>,
) -> PyResult<()> {
    run_browser(title, width, height, home_url, debug, initial_urls)
}

/// Register tab browser functions with Python module
pub fn register_tab_browser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_browser, m)?)?;
    m.add_function(wrap_pyfunction!(run_tab_browser, m)?)?; // Backward compatibility
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registration() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            register_tab_browser(&module).unwrap();
            assert!(module.getattr("run_browser").is_ok());
            assert!(module.getattr("run_tab_browser").is_ok()); // Backward compat
        });
    }

    #[test]
    fn test_run_browser_function_exists() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            register_tab_browser(&module).unwrap();
            let func = module.getattr("run_browser").unwrap();
            assert!(func.is_callable());
        });
    }
}
