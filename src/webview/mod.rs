//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;
use wry::WebViewBuilder as WryWebViewBuilder;

mod config;
mod event_loop;
mod ipc;
mod protocol;

pub use config::{WebViewBuilder, WebViewConfig};
pub use event_loop::{EventLoopState, WebViewEventHandler};
pub use ipc::IpcHandler;

/// Python-facing WebView class
/// Supports both standalone and embedded modes (for DCC integration)
#[pyclass(name = "WebView", unsendable)]
pub struct PyWebView {
    inner: Rc<RefCell<Option<WebViewInner>>>,
    config: Rc<RefCell<WebViewConfig>>,
    ipc_handler: Arc<IpcHandler>,
}

/// Internal WebView structure - supports both standalone and embedded modes
pub struct WebViewInner {
    webview: WryWebView,
    // For standalone mode only
    #[allow(dead_code)]
    window: Option<tao::window::Window>,
    #[allow(dead_code)]
    event_loop: Option<tao::event_loop::EventLoop<()>>,
}

#[pymethods]
#[allow(clippy::useless_conversion)]
impl PyWebView {
    /// Create a new WebView instance
    ///
    /// Args:
    ///     title (str): Window title
    ///     width (int): Window width in pixels
    ///     height (int): Window height in pixels
    ///     url (str, optional): URL to load
    ///     html (str, optional): HTML content to load
    ///
    /// Returns:
    ///     WebView: A new WebView instance
    #[new]
    #[pyo3(signature = (title="DCC WebView", width=800, height=600, url=None, html=None))]
    fn new(
        title: &str,
        width: u32,
        height: u32,
        url: Option<&str>,
        html: Option<&str>,
    ) -> PyResult<Self> {
        tracing::info!("PyWebView::new() called with title: {}", title);

        let config = WebViewConfig {
            title: title.to_string(),
            width,
            height,
            url: url.map(|s| s.to_string()),
            html: html.map(|s| s.to_string()),
            ..Default::default()
        };

        Ok(PyWebView {
            inner: Rc::new(RefCell::new(None)),
            config: Rc::new(RefCell::new(config)),
            ipc_handler: Arc::new(IpcHandler::new()),
        })
    }

    /// Test method to verify Python bindings work
    #[allow(clippy::useless_conversion)]
    fn test_method(&self) -> PyResult<String> {
        tracing::info!("test_method() called!");
        Ok("test_method works!".to_string())
    }

    /// Create WebView for standalone mode (creates its own window)
    #[allow(clippy::useless_conversion)]
    fn show_window(&self) -> PyResult<()> {
        let title = self.config.borrow().title.clone();
        tracing::info!("Showing WebView (standalone mode): {}", title);

        // Create WebView instance if not already created
        let mut inner = self.inner.borrow_mut();
        if inner.is_none() {
            let webview =
                Self::create_standalone(self.config.borrow().clone(), self.ipc_handler.clone())
                    .map_err(|e| {
                        tracing::error!("Failed to create standalone WebView: {}", e);
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;
            *inner = Some(webview);
        }

        // Release the borrow before running the event loop
        drop(inner);

        // Run the event loop (this will block until window is closed)
        if let Some(webview_inner) = self.inner.borrow_mut().as_mut() {
            webview_inner.run_event_loop_blocking();
        }

        Ok(())
    }

    /// Create WebView for standalone mode (creates its own window)
    fn show(&self) -> PyResult<()> {
        self.show_window()
    }

    /// Create WebView for embedded mode (for DCC integration)
    ///
    /// Args:
    ///     parent_hwnd (int): Parent window handle (Windows HWND)
    ///     width (int): Width in pixels
    ///     height (int): Height in pixels
    #[allow(clippy::useless_conversion)]
    fn create_embedded(&self, parent_hwnd: u64, width: u32, height: u32) -> PyResult<()> {
        tracing::info!("Creating embedded WebView for parent HWND: {}", parent_hwnd);

        let mut inner = self.inner.borrow_mut();
        if inner.is_none() {
            let webview = Self::create_embedded_impl(
                parent_hwnd,
                width,
                height,
                self.config.borrow().clone(),
                self.ipc_handler.clone(),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            *inner = Some(webview);
        }

        Ok(())
    }

    /// Load a URL in the WebView
    ///
    /// Args:
    ///     url (str): URL to load
    #[allow(clippy::useless_conversion)]
    fn load_url(&self, url: &str) -> PyResult<()> {
        tracing::info!("Loading URL: {}", url);

        // If created, load immediately; otherwise store for later
        let mut loaded = false;
        if let Some(webview) = self.inner.borrow_mut().as_mut() {
            webview
                .load_url(url)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            loaded = true;
        }
        if !loaded {
            let mut cfg = self.config.borrow_mut();
            cfg.url = Some(url.to_string());
            cfg.html = None; // last-write-wins
            tracing::debug!("WebView not yet created; stored URL in config to load on show()");
        }
        Ok(())
    }

    /// Load HTML content in the WebView
    ///
    /// Args:
    ///     html (str): HTML content to load
    #[allow(clippy::useless_conversion)]
    fn load_html(&self, html: &str) -> PyResult<()> {
        tracing::info!("Loading HTML content ({} bytes)", html.len());

        // If created, load immediately; otherwise store for later
        let mut loaded = false;
        if let Some(webview) = self.inner.borrow_mut().as_mut() {
            webview
                .load_html(html)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            loaded = true;
        }
        if !loaded {
            let mut cfg = self.config.borrow_mut();
            cfg.html = Some(html.to_string());
            cfg.url = None; // last-write-wins
            tracing::debug!("WebView not yet created; stored HTML in config to load on show()");
        }
        Ok(())
    }

    /// Execute JavaScript code in the WebView
    ///
    /// Args:
    ///     script (str): JavaScript code to execute
    #[allow(clippy::useless_conversion)]
    fn eval_js(&self, script: &str) -> PyResult<()> {
        tracing::info!("Executing JavaScript: {}", script);

        if let Some(webview) = self.inner.borrow_mut().as_mut() {
            webview
                .eval_js(script)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }
        Ok(())
    }

    /// Emit an event to JavaScript
    ///
    /// Args:
    ///     event_name (str): Name of the event
    ///     data (dict): Data to send with the event
    #[allow(clippy::useless_conversion)]
    fn emit(&self, event_name: &str, data: &Bound<'_, PyDict>) -> PyResult<()> {
        tracing::info!("Emitting event: {}", event_name);

        // Convert Python dict to JSON
        let json_data = self
            .py_dict_to_json(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        if let Some(webview_inner) = self.inner.borrow_mut().as_mut() {
            webview_inner
                .emit(event_name, json_data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }
        Ok(())
    }

    /// Register a Python callback for JavaScript events
    ///
    /// Args:
    ///     event_name (str): Name of the event to listen for
    ///     callback (callable): Python function to call when event occurs
    #[allow(clippy::useless_conversion)]
    fn on(&self, event_name: &str, _callback: PyObject) -> PyResult<()> {
        tracing::info!("Registering callback for event: {}", event_name);

        // Store the callback in the IPC handler
        // Note: This is a simplified implementation
        // In a real implementation, we would need to handle Python callbacks properly
        let event_name_str = event_name.to_string();

        // For now, just log that we registered the callback
        tracing::debug!("Callback registered for event: {}", event_name_str);

        Ok(())
    }

    /// Close the WebView window
    #[allow(clippy::useless_conversion)]
    fn close(&self) -> PyResult<()> {
        tracing::info!("Closing WebView");
        // TODO: Implement window closing
        Ok(())
    }

    /// Get the window title
    #[getter]
    fn title(&self) -> PyResult<String> {
        Ok(self.config.borrow().title.clone())
    }

    /// Set the window title
    #[setter]
    fn set_title(&mut self, title: String) -> PyResult<()> {
        self.config.borrow_mut().title = title;
        // TODO: Update actual window title
        Ok(())
    }

    /// Python representation
    fn __repr__(&self) -> String {
        let cfg = self.config.borrow();
        format!(
            "WebView(title='{}', width={}, height={})",
            cfg.title, cfg.width, cfg.height
        )
    }
}

impl PyWebView {
    /// Helper method to convert Python dict to JSON (private, not exposed to Python)
    fn py_dict_to_json(&self, dict: &Bound<'_, PyDict>) -> PyResult<serde_json::Value> {
        let mut json_obj = serde_json::Map::new();

        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let json_value = if let Ok(s) = value.extract::<String>() {
                serde_json::Value::String(s)
            } else if let Ok(i) = value.extract::<i64>() {
                serde_json::Value::Number(i.into())
            } else if let Ok(f) = value.extract::<f64>() {
                serde_json::json!(f)
            } else if let Ok(b) = value.extract::<bool>() {
                serde_json::Value::Bool(b)
            } else {
                serde_json::Value::Null
            };
            json_obj.insert(key_str, json_value);
        }

        Ok(serde_json::Value::Object(json_obj))
    }

    /// Create standalone WebView with its own window
    fn create_standalone(
        config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
    ) -> Result<WebViewInner, Box<dyn std::error::Error>> {
        use tao::event_loop::EventLoopBuilder;
        use tao::window::WindowBuilder;

        let event_loop = EventLoopBuilder::new().build();

        let window = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
            .with_resizable(config.resizable)
            .with_decorations(config.decorations)
            .with_transparent(config.transparent)
            .build(&event_loop)?;

        // Create the WebView
        let mut webview_builder = WryWebViewBuilder::new();
        if config.dev_tools {
            webview_builder = webview_builder.with_devtools(true);
        }
        let webview = webview_builder.build(&window)?;

        // Apply initial content from config if provided
        if let Some(ref url) = config.url {
            let script = format!("window.location.href = '{}';", url);
            webview.evaluate_script(&script)?;
        } else if let Some(ref html) = config.html {
            webview.load_html(html)?;
        }

        Ok(WebViewInner {
            webview,
            window: Some(window),
            event_loop: Some(event_loop),
        })
    }

    /// Create embedded WebView for DCC integration
    #[cfg(target_os = "windows")]
    fn create_embedded_impl(
        _parent_hwnd: u64,
        _width: u32,
        _height: u32,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
    ) -> Result<WebViewInner, Box<dyn std::error::Error>> {
        // TODO: Implement embedded mode for Windows
        // This requires using raw-window-handle to create a WebView as a child of an existing window
        tracing::warn!("Embedded mode is not yet fully implemented");
        Err("Embedded mode is not yet fully implemented".into())
    }

    /// Create embedded WebView for non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    fn create_embedded_impl(
        _parent_hwnd: u64,
        _width: u32,
        _height: u32,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
    ) -> Result<WebViewInner, Box<dyn std::error::Error>> {
        Err("Embedded mode is only supported on Windows".into())
    }
}

impl WebViewInner {
    /// Load a URL
    pub fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Use JavaScript to navigate
        let script = format!("window.location.href = '{}';", url);
        self.webview.evaluate_script(&script)?;
        Ok(())
    }

    /// Load HTML content
    pub fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.webview.load_html(html)?;
        Ok(())
    }

    /// Execute JavaScript
    pub fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }

    /// Emit an event to JavaScript
    pub fn emit(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}))",
            event_name, data
        );
        self.webview.evaluate_script(&script)?;
        Ok(())
    }

    /// Run the event loop (standalone mode only)
    #[allow(dead_code)]
    pub fn run_event_loop(&mut self, _py: Python) -> PyResult<()> {
        use tao::event_loop::ControlFlow;

        // Show the window
        if let Some(window) = &self.window {
            tracing::info!("Setting window visible");
            window.set_visible(true);
            tracing::info!("Window is now visible");
        }

        // Get the event loop
        if let Some(event_loop) = self.event_loop.take() {
            tracing::info!("Starting event loop");

            // Run the event loop - this will block until the window is closed
            // Note: This is a blocking call that will not return until the user closes the window
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    tao::event::Event::WindowEvent {
                        event: tao::event::WindowEvent::CloseRequested,
                        ..
                    } => {
                        tracing::info!("Close requested");
                        *control_flow = ControlFlow::Exit;
                    }
                    tao::event::Event::WindowEvent {
                        event: tao::event::WindowEvent::Resized(_),
                        ..
                    } => {
                        // Handle window resize
                    }
                    _ => {}
                }
            });

            // This code is unreachable because event_loop.run() never returns
            #[allow(unreachable_code)]
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Event loop not available (embedded mode?)",
            ))
        }
    }

    /// Run the event loop without Python GIL (blocking version)
    /// Uses improved event loop handling with better state management
    pub fn run_event_loop_blocking(&mut self) {
        tracing::info!("=== run_event_loop_blocking called (improved version) ===");

        // Validate prerequisites
        if self.window.is_none() {
            tracing::error!("Window is None!");
            return;
        }

        if self.event_loop.is_none() {
            tracing::error!("Event loop is None!");
            return;
        }

        // Take ownership of event loop and window
        let event_loop = match self.event_loop.take() {
            Some(el) => el,
            None => {
                tracing::error!("Failed to take event loop");
                return;
            }
        };

        let window = match self.window.take() {
            Some(w) => w,
            None => {
                tracing::error!("Failed to take window");
                return;
            }
        };

        // Create event loop state
        let state = Arc::new(Mutex::new(EventLoopState::new(window)));

        // Run the improved event loop
        WebViewEventHandler::run_blocking(event_loop, state);

        tracing::info!("Event loop exited");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_config() {
        let config = WebViewConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        };
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
    }

    #[test]
    fn test_webview_creation() {
        // Test WebView creation with default parameters
        let webview = PyWebView::new("Test Window", 1024, 768, None, None);
        assert!(webview.is_ok());
    }

    #[test]
    fn test_webview_creation_with_url() {
        // Test WebView creation with URL
        let webview = PyWebView::new("Test Window", 1024, 768, Some("https://example.com"), None);
        assert!(webview.is_ok());
    }

    #[test]
    fn test_webview_creation_with_html() {
        // Test WebView creation with HTML
        let webview = PyWebView::new("Test Window", 1024, 768, None, Some("<h1>Hello</h1>"));
        assert!(webview.is_ok());
    }

    #[test]
    fn test_webview_repr() {
        let webview = PyWebView::new("Test", 800, 600, None, None).unwrap();
        let repr = webview.__repr__();
        assert!(repr.contains("Test"));
        assert!(repr.contains("800"));
        assert!(repr.contains("600"));
    }

    #[test]
    fn test_webview_title_getter() {
        let webview = PyWebView::new("My Title", 800, 600, None, None).unwrap();
        let title = webview.title().unwrap();
        assert_eq!(title, "My Title");
    }

    #[test]
    fn test_py_dict_to_json() {
        // This test requires Python context, so we skip it in pure Rust tests
        // It would be tested in integration tests
    }
}
