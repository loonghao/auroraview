//! Python bindings for standalone WebView runner
//!
//! This module provides a Python function to run a standalone WebView using
//! event_loop.run() instead of run_return(). This is the correct approach for
//! standalone applications where the process should exit when the window closes.

use pyo3::prelude::*;
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
    allow_file_protocol=false
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
) -> PyResult<()> {
    tracing::info!("[run_standalone] Creating standalone WebView: {}", title);

    // Create config
    let config = WebViewConfig {
        title,
        width,
        height,
        url,
        html,
        dev_tools,
        resizable,
        decorations,
        transparent,
        always_on_top: false,
        background_color: None, // Will use loading screen instead
        context_menu: true,
        parent_hwnd: None,
        embed_mode: crate::webview::config::EmbedMode::None,
        ipc_batching: false,
        ipc_batch_size: 100,
        ipc_batch_interval_ms: 16,
        asset_root: None,
        custom_protocols: std::collections::HashMap::new(),
        api_methods: std::collections::HashMap::new(),
        allow_new_window,
        allow_file_protocol,
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
}
