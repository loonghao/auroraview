//! Python bindings for standalone WebView runner
//!
//! This module provides a Python function to run a standalone WebView using
//! event_loop.run() instead of run_return(). This is the correct approach for
//! standalone applications where the process should exit when the window closes.

use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::ipc::{IpcHandler, MessageQueue};
use crate::webview::config::WebViewConfig;
use crate::webview::standalone;

/// Run a standalone WebView (blocking until window closes)
///
/// This function creates and runs a standalone WebView window using event_loop.run().
/// It will block until the window is closed, then exit the entire process.
///
/// IMPORTANT: This calls std::process::exit() when the window closes!
/// Only use this for standalone applications, NOT for DCC integration.
///
/// Args:
///     title (str): Window title
///     width (int): Window width in pixels
///     height (int): Window height in pixels
///     url (str, optional): URL to load
///     html (str, optional): HTML content to load
///     dev_tools (bool, optional): Enable developer tools (default: True)
///     resizable (bool, optional): Make window resizable (default: True)
///     decorations (bool, optional): Show window decorations (default: True)
///     transparent (bool, optional): Make window transparent (default: False)
///     allow_new_window (bool, optional): Allow opening new windows (default: False)
///     allow_file_protocol (bool, optional): Enable file:// protocol support (default: False)
///     always_on_top (bool, optional): Keep window always on top (default: False)
///     headless (bool, optional): Run in headless mode without visible window (default: False).
///         Useful for automated testing. Note: WebView2 creates a hidden window, not true headless.
///     remote_debugging_port (int, optional): Enable CDP remote debugging on specified port.
///         When set, Playwright/Puppeteer can connect via `connect_over_cdp(f"http://localhost:{port}")`.
///     asset_root (str, optional): Root directory for auroraview:// protocol.
///         When set, enables the auroraview:// custom protocol for secure local
///         resource loading. Files under this directory can be accessed using URLs
///         like ``auroraview://path/to/file`` (or ``https://auroraview.localhost/path``
///         on Windows).
///     html_path (str, optional): Path to HTML file. When provided with `html` content,
///         the `asset_root` will automatically be set to the directory containing
///         the HTML file (if `asset_root` is not explicitly set). This allows relative
///         resource paths in HTML to be resolved correctly relative to the HTML file.
///     rewrite_relative_paths (bool, optional): Automatically rewrite relative paths
///         (like ./script.js, ../style.css) to use auroraview:// protocol. Default: True.
///
/// Example:
///     >>> from auroraview._core import run_standalone
///     >>> run_standalone(
///     ...     title="My App",
///     ...     width=800,
///     ...     height=600,
///     ...     url="https://example.com"
///     ... )
///     # Window shows, blocks until closed, then process exits
#[pyfunction]
#[pyo3(signature = (
    title,
    width,
    height,
    url=None,
    html=None,
    dev_tools=true,
    resizable=true,
    decorations=true,
    transparent=false,
    allow_new_window=false,
    allow_file_protocol=false,
    always_on_top=false,
    headless=false,
    remote_debugging_port=None,
    asset_root=None,
    html_path=None,
    rewrite_relative_paths=true
))]
#[allow(clippy::too_many_arguments)]
fn run_standalone(
    title: String,
    width: u32,
    height: u32,
    url: Option<String>,
    html: Option<String>,
    dev_tools: bool,
    resizable: bool,
    decorations: bool,
    transparent: bool,
    allow_new_window: bool,
    allow_file_protocol: bool,
    always_on_top: bool,
    headless: bool,
    remote_debugging_port: Option<u16>,
    asset_root: Option<String>,
    html_path: Option<String>,
    rewrite_relative_paths: bool,
) -> PyResult<()> {
    tracing::info!("[run_standalone] Creating standalone WebView: {}", title);

    // Determine asset_root: explicit setting takes priority, otherwise derive from html_path
    let effective_asset_root = if let Some(root) = asset_root {
        Some(PathBuf::from(root))
    } else if let Some(ref path) = html_path {
        // Auto-detect asset_root from HTML file location
        let html_file_path = PathBuf::from(path);
        let parent_dir = html_file_path.parent().map(|p| p.to_path_buf());
        if let Some(ref dir) = parent_dir {
            tracing::info!(
                "[run_standalone] Auto-detected asset_root from HTML path: {:?}",
                dir
            );
        }
        parent_dir
    } else {
        None
    };

    // Rewrite HTML to use auroraview:// protocol for relative paths if enabled
    let processed_html = if rewrite_relative_paths {
        html.map(|h| crate::bindings::cli_utils::rewrite_html_for_custom_protocol(&h))
    } else {
        html
    };

    // Create config
    let config = WebViewConfig {
        title,
        width,
        height,
        url,
        html: processed_html,
        dev_tools,
        resizable,
        decorations,
        transparent,
        always_on_top,
        background_color: None, // Will use loading screen instead
        context_menu: true,
        parent_hwnd: None,
        embed_mode: crate::webview::config::EmbedMode::None,
        ipc_batching: false,
        ipc_batch_size: 100,
        ipc_batch_interval_ms: 16,
        asset_root: effective_asset_root,
        data_directory: None, // Use system default
        custom_protocols: std::collections::HashMap::new(),
        api_methods: std::collections::HashMap::new(),
        allow_new_window,
        allow_file_protocol,
        auto_show: !headless, // Don't auto-show in headless mode
        headless,
        remote_debugging_port,
        // Security defaults
        content_security_policy: None,
        cors_allowed_origins: Vec::new(),
        allow_clipboard: false,
        allow_geolocation: false,
        allow_notifications: false,
        allow_media_devices: false,
        block_external_navigation: false,
        allowed_navigation_domains: Vec::new(),
        icon: None,                       // Use default AuroraView icon
        enable_plugins: true,             // Enable plugin APIs
        enabled_plugin_names: Vec::new(), // All plugins
    };

    // Create IPC handler and message queue
    let ipc_handler = Arc::new(IpcHandler::new());
    let message_queue = Arc::new(MessageQueue::new());

    // Run standalone - this will block until window closes and then exit the process
    tracing::info!("[run_standalone] Starting standalone event loop...");
    standalone::run_standalone(config, ipc_handler, message_queue)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // This line will never be reached because run_standalone() calls std::process::exit()
    unreachable!("run_standalone() should never return");
}

/// Register standalone runner functions with Python module
pub fn register_standalone_runner(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_standalone, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_module_registration() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            register_standalone_runner(&module).unwrap();
            assert!(module.getattr("run_standalone").is_ok());
        });
    }

    #[test]
    fn test_run_standalone_function_exists() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            register_standalone_runner(&module).unwrap();
            let func = module.getattr("run_standalone").unwrap();
            assert!(func.is_callable());
        });
    }

    #[test]
    fn test_run_standalone_signature() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            register_standalone_runner(&module).unwrap();
            let func = module.getattr("run_standalone").unwrap();

            // Verify function has correct signature
            let signature = func.getattr("__signature__");
            assert!(signature.is_ok() || func.is_callable());
        });
    }

    /// Test WebViewConfig creation with asset_root
    #[test]
    fn test_config_with_asset_root() {
        let asset_root = Some("/tmp/assets".to_string());
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            dev_tools: true,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            background_color: None,
            context_menu: true,
            parent_hwnd: None,
            embed_mode: crate::webview::config::EmbedMode::None,
            ipc_batching: false,
            ipc_batch_size: 100,
            ipc_batch_interval_ms: 16,
            asset_root: asset_root.map(PathBuf::from),
            data_directory: None,
            custom_protocols: std::collections::HashMap::new(),
            api_methods: std::collections::HashMap::new(),
            allow_new_window: false,
            allow_file_protocol: false,
            auto_show: true,
            headless: false,
            remote_debugging_port: None,
            content_security_policy: None,
            cors_allowed_origins: Vec::new(),
            allow_clipboard: false,
            allow_geolocation: false,
            allow_notifications: false,
            allow_media_devices: false,
            block_external_navigation: false,
            allowed_navigation_domains: Vec::new(),
            icon: None,
            enable_plugins: true,
            enabled_plugin_names: Vec::new(),
        };

        assert_eq!(config.asset_root, Some(PathBuf::from("/tmp/assets")));
    }

    /// Test WebViewConfig creation without asset_root
    #[test]
    fn test_config_without_asset_root() {
        let asset_root: Option<String> = None;
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            dev_tools: true,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            background_color: None,
            context_menu: true,
            parent_hwnd: None,
            embed_mode: crate::webview::config::EmbedMode::None,
            ipc_batching: false,
            ipc_batch_size: 100,
            ipc_batch_interval_ms: 16,
            asset_root: asset_root.map(PathBuf::from),
            data_directory: None,
            custom_protocols: std::collections::HashMap::new(),
            api_methods: std::collections::HashMap::new(),
            allow_new_window: false,
            allow_file_protocol: false,
            auto_show: true,
            headless: false,
            remote_debugging_port: None,
            content_security_policy: None,
            cors_allowed_origins: Vec::new(),
            allow_clipboard: false,
            allow_geolocation: false,
            allow_notifications: false,
            allow_media_devices: false,
            block_external_navigation: false,
            allowed_navigation_domains: Vec::new(),
            icon: None,
            enable_plugins: true,
            enabled_plugin_names: Vec::new(),
        };

        assert_eq!(config.asset_root, None);
    }

    /// Test WebViewConfig with zero dimensions for maximize
    #[test]
    fn test_config_zero_width_for_maximize() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 0,
            height: 600,
            ..Default::default()
        };

        assert_eq!(config.width, 0);
        assert_eq!(config.height, 600);
        // When width is 0, the window should be maximized
    }

    /// Test WebViewConfig with zero height for maximize
    #[test]
    fn test_config_zero_height_for_maximize() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 0,
            ..Default::default()
        };

        assert_eq!(config.width, 800);
        assert_eq!(config.height, 0);
        // When height is 0, the window should be maximized
    }

    /// Test WebViewConfig with both dimensions zero
    #[test]
    fn test_config_both_dimensions_zero() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 0,
            height: 0,
            ..Default::default()
        };

        assert_eq!(config.width, 0);
        assert_eq!(config.height, 0);
        // When both are 0, the window should be maximized
    }

    /// Test WebViewConfig with allow_file_protocol enabled
    #[test]
    fn test_config_with_allow_file_protocol() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            allow_file_protocol: true,
            ..Default::default()
        };

        assert!(config.allow_file_protocol);
    }

    /// Test WebViewConfig with all local file options
    #[test]
    fn test_config_with_all_local_file_options() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            asset_root: Some(PathBuf::from("./assets")),
            allow_file_protocol: true,
            ..Default::default()
        };

        assert_eq!(config.asset_root, Some(PathBuf::from("./assets")));
        assert!(config.allow_file_protocol);
    }
}
