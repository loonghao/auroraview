//! AuroraView - Python-facing WebView class
//!
//! This module provides the Python API for creating and managing WebView instances.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{Py, PyAny};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::config::WebViewConfig;
use super::webview_inner::WebViewInner;
use crate::bindings::webview::py_dict_to_json;
use crate::ipc::{IpcHandler, MessageQueue, WebViewMessage};

/// Python-facing WebView class
/// Supports both standalone and embedded modes (for DCC integration)
#[pyclass(name = "WebView", unsendable)]
pub struct AuroraView {
    pub(crate) inner: Rc<RefCell<Option<WebViewInner>>>,
    pub(crate) config: Rc<RefCell<WebViewConfig>>,
    pub(crate) ipc_handler: Arc<IpcHandler>,
    /// Thread-safe message queue for cross-thread communication
    pub(crate) message_queue: Arc<MessageQueue>,
    /// Event loop proxy for sending close events (standalone mode only)
    pub(crate) event_loop_proxy:
        Rc<RefCell<Option<tao::event_loop::EventLoopProxy<super::event_loop::UserEvent>>>>,
}

#[pymethods]
#[allow(clippy::useless_conversion)]
impl AuroraView {
    /// Create a new WebView instance
    ///
    /// Args:
    ///     title (str): Window title
    ///     width (int): Window width in pixels
    ///     height (int): Window height in pixels
    ///     url (str, optional): URL to load
    ///     html (str, optional): HTML content to load
    ///     dev_tools (bool, optional): Enable developer tools (default: True)
    ///     context_menu (bool, optional): Enable native context menu (default: True)
    ///     resizable (bool, optional): Make window resizable (default: True)
    ///     parent_hwnd (int, optional): Parent window handle (HWND on Windows)
    ///     parent_mode (str, optional): "child" or "owner" (Windows only)
    ///     asset_root (str, optional): Root directory for auroraview:// protocol
    ///
    /// Returns:
    ///     WebView: A new WebView instance
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (title="DCC WebView", width=800, height=600, url=None, html=None, dev_tools=true, context_menu=true, resizable=true, decorations=true, parent_hwnd=None, parent_mode=None, asset_root=None))]
    fn new(
        title: &str,
        width: u32,
        height: u32,
        url: Option<&str>,
        html: Option<&str>,
        dev_tools: bool,
        context_menu: bool,
        resizable: bool,
        decorations: bool,
        parent_hwnd: Option<u64>,
        parent_mode: Option<&str>,
        asset_root: Option<&str>,
    ) -> PyResult<Self> {
        tracing::info!("AuroraView::new() called with title: {}, dev_tools: {}, context_menu: {}, resizable: {}, decorations: {}, parent_hwnd: {:?}, parent_mode: {:?}, asset_root: {:?}",
            title, dev_tools, context_menu, resizable, decorations, parent_hwnd, parent_mode, asset_root);

        #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
        let mut config = WebViewConfig {
            title: title.to_string(),
            width,
            height,
            url: url.map(|s| s.to_string()),
            html: html.map(|s| s.to_string()),
            dev_tools,
            context_menu,
            resizable,
            decorations,
            parent_hwnd,
            asset_root: asset_root.map(std::path::PathBuf::from),
            ..Default::default()
        };

        // Map string to EmbedMode (Windows)
        #[cfg(target_os = "windows")]
        {
            use crate::webview::config::EmbedMode;

            // Log the parent_mode for debugging
            if let Some(ref mode_str) = parent_mode {
                tracing::info!("[DEBUG] parent_mode received: '{}'", mode_str);
            } else {
                tracing::info!("[DEBUG] parent_mode is None");
            }

            config.embed_mode = match parent_mode.map(|s| s.to_ascii_lowercase()) {
                Some(ref m) if m == "child" => {
                    tracing::info!("[OK] Using EmbedMode::Child");
                    EmbedMode::Child
                }
                Some(ref m) if m == "owner" => {
                    tracing::info!("[OK] Using EmbedMode::Owner (cross-thread safe)");
                    EmbedMode::Owner
                }
                _ => {
                    if parent_hwnd.is_some() {
                        tracing::warn!("[WARNING] parent_hwnd provided but parent_mode not recognized, defaulting to Child (may cause issues!)");
                        EmbedMode::Child
                    } else {
                        tracing::info!("[OK] No parent, using EmbedMode::None");
                        EmbedMode::None
                    }
                }
            };

            tracing::info!("[OK] Final embed_mode: {:?}", config.embed_mode);
        }

        Ok(AuroraView {
            inner: Rc::new(RefCell::new(None)),
            config: Rc::new(RefCell::new(config)),
            ipc_handler: Arc::new(IpcHandler::new()),
            message_queue: Arc::new(MessageQueue::new()),
            event_loop_proxy: Rc::new(RefCell::new(None)), // Will be set when creating standalone WebView
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

        // Create or re-create WebView instance
        let mut inner = self.inner.borrow_mut();
        let need_create = if let Some(existing) = inner.as_ref() {
            // If previous event loop already ran, we must recreate
            existing.event_loop.is_none()
        } else {
            true
        };

        if need_create {
            let mut webview = WebViewInner::create_standalone(
                self.config.borrow().clone(),
                self.ipc_handler.clone(),
                self.message_queue.clone(),
            )
            .map_err(|e| {
                tracing::error!("Failed to create standalone WebView: {}", e);
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;

            // Extract and store event loop proxy for close() method
            if let Some(proxy) = webview.event_loop_proxy.take() {
                *self.event_loop_proxy.borrow_mut() = Some(proxy);
                tracing::info!("[OK] Event loop proxy extracted and stored in AuroraView");
            }

            *inner = Some(webview);
        }

        // Release the borrow before running the event loop
        drop(inner);

        // Run the event loop (this will block until window is closed)
        if let Some(webview_inner) = self.inner.borrow_mut().as_mut() {
            webview_inner.run_event_loop_blocking();
        }

        // After event loop exits, drop inner so next show() recreates cleanly
        let mut inner_after = self.inner.borrow_mut();
        *inner_after = None;

        Ok(())
    }

    /// Show WebView window
    ///
    /// Behavior depends on embed_mode:
    /// - EmbedMode::None: Creates standalone window and runs event loop (blocking)
    /// - EmbedMode::Child/Owner: Creates embedded window and returns immediately (non-blocking)
    fn show(&self) -> PyResult<()> {
        use crate::webview::config::EmbedMode;

        let embed_mode = self.config.borrow().embed_mode;

        match embed_mode {
            EmbedMode::None => {
                // Standalone mode: run event loop (blocking)
                tracing::info!(" [show] Standalone mode - will run event loop (blocking)");
                self.show_window()
            }
            #[cfg(target_os = "windows")]
            EmbedMode::Child | EmbedMode::Owner => {
                // Embedded mode: create window but don't run event loop (non-blocking)
                tracing::info!(
                    "[OK] [show] Embedded mode ({:?}) - creating window without event loop",
                    embed_mode
                );
                self.show_embedded()
            }
        }
    }

    /// Show embedded WebView (non-blocking)
    ///
    /// Creates the WebView window as a child/owned window of the parent.
    /// Does NOT run the event loop - the parent window's event loop handles events.
    /// Returns immediately, allowing the parent application to continue running.
    #[cfg(target_os = "windows")]
    fn show_embedded(&self) -> PyResult<()> {
        let title = self.config.borrow().title.clone();
        let embed_mode = self.config.borrow().embed_mode;
        tracing::info!(
            "[OK] [show_embedded] Creating embedded WebView: {} (mode: {:?})",
            title,
            embed_mode
        );

        // Create or re-create WebView instance
        let mut inner = self.inner.borrow_mut();
        let need_create = if let Some(existing) = inner.as_ref() {
            // If previous event loop already ran, we must recreate
            existing.event_loop.is_none()
        } else {
            true
        };

        if need_create {
            tracing::info!("[OK] [show_embedded] Creating new embedded WebView instance...");

            let config = self.config.borrow().clone();
            let parent_hwnd = config.parent_hwnd.unwrap_or(0);
            let width = config.width;
            let height = config.height;

            let webview = WebViewInner::create_embedded(
                parent_hwnd,
                width,
                height,
                config,
                self.ipc_handler.clone(),
                self.message_queue.clone(),
            )
            .map_err(|e| {
                tracing::error!(
                    "[ERROR] [show_embedded] Failed to create embedded WebView: {}",
                    e
                );
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;
            *inner = Some(webview);
            tracing::info!("[OK] [show_embedded] Embedded WebView instance created");
        }

        // For embedded mode, we DON'T run the event loop
        // The parent window's event loop will handle our window's events
        tracing::info!("[OK] [show_embedded] Embedded WebView created successfully (non-blocking)");
        tracing::info!(" [show_embedded] IMPORTANT: Keep the Python WebView object alive!");
        tracing::info!(" [show_embedded] If the Python object is destroyed, the window will close");
        tracing::info!(
            " [show_embedded] Store it in a global variable: __main__.webview = webview"
        );

        Ok(())
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
            let webview = WebViewInner::create_embedded(
                parent_hwnd,
                width,
                height,
                self.config.borrow().clone(),
                self.ipc_handler.clone(),
                self.message_queue.clone(),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            *inner = Some(webview);
        }

        Ok(())
    }

    /// Create WebView for DCC integration (no event loop)
    ///
    /// This is the recommended method for integrating WebView into DCC applications
    /// like Maya, Houdini, Nuke, etc. It creates a WebView that uses the DCC's
    /// Qt message pump instead of creating its own event loop.
    ///
    /// Key features:
    /// - No event loop conflicts with DCC's Qt event loop
    /// - Non-blocking - DCC UI remains responsive
    /// - Requires periodic calls to `process_messages()` from Qt timer
    /// - No PySide dependency required
    ///
    /// Args:
    ///     parent_hwnd (int): Parent window handle (Windows HWND)
    ///     title (str, optional): Window title
    ///     width (int, optional): Width in pixels (default: 800)
    ///     height (int, optional): Height in pixels (default: 600)
    ///
    /// Example:
    /// ```text
    /// import hou
    /// from auroraview import WebView
    ///
    /// # Get Houdini main window HWND
    /// main_window = hou.qt.mainWindow()
    /// hwnd = int(main_window.winId())
    ///
    /// # Create WebView for DCC
    /// webview = WebView.create_for_dcc(
    ///     parent_hwnd=hwnd,
    ///     title="My Tool",
    ///     width=650,
    ///     height=500
    /// )
    ///
    /// # Load content
    /// webview.load_html("<h1>Hello from Houdini!</h1>")
    ///
    /// # Setup Qt timer to process messages
    /// from PySide2.QtCore import QTimer
    /// timer = QTimer()
    /// timer.timeout.connect(webview.process_messages)
    /// timer.start(16)  # 60 FPS
    /// ```
    #[staticmethod]
    #[pyo3(signature = (parent_hwnd, title="DCC WebView", width=800, height=600))]
    fn create_for_dcc(parent_hwnd: u64, title: &str, width: u32, height: u32) -> PyResult<Self> {
        tracing::info!(
            "[OK] [create_for_dcc] Creating WebView for DCC integration (parent_hwnd: {})",
            parent_hwnd
        );

        let config = WebViewConfig {
            title: title.to_string(),
            width,
            height,
            parent_hwnd: Some(parent_hwnd),
            ..Default::default()
        };

        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // Create WebView using DCC integration mode
        let webview = WebViewInner::create_for_dcc(
            parent_hwnd,
            config.clone(),
            ipc_handler.clone(),
            message_queue.clone(),
        )
        .map_err(|e| {
            tracing::error!("[ERROR] [create_for_dcc] Failed to create WebView: {}", e);
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })?;

        tracing::info!("[OK] [create_for_dcc] WebView created successfully");
        tracing::info!("[OK] Remember to call process_messages() periodically from Qt timer!");

        Ok(AuroraView {
            inner: Rc::new(RefCell::new(Some(webview))),
            config: Rc::new(RefCell::new(config)),
            ipc_handler,
            message_queue,
            event_loop_proxy: Rc::new(RefCell::new(None)),
        })
    }

    /// Process messages for DCC integration mode
    ///
    /// This method should be called periodically from a Qt timer to process
    /// WebView messages without running a dedicated event loop.
    ///
    /// Returns:
    ///     bool: True if the window should be closed, False otherwise
    ///
    /// Example:
    /// ```text
    /// from PySide2.QtCore import QTimer
    ///
    /// timer = QTimer()
    /// timer.timeout.connect(webview.process_messages)
    /// timer.start(16)  # 60 FPS
    /// ```
    fn process_messages(&self) -> PyResult<bool> {
        let inner = self.inner.borrow();
        if let Some(webview) = inner.as_ref() {
            let should_close = webview.process_messages();
            Ok(should_close)
        } else {
            Ok(false)
        }
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
    ///
    /// Note: This method is thread-safe. It pushes the script to a message queue
    /// that is processed by the WebView thread's event loop.
    #[allow(clippy::useless_conversion)]
    fn eval_js(&self, script: &str) -> PyResult<()> {
        tracing::info!("Queueing JavaScript execution: {}", script);

        // Push to message queue for thread-safe execution
        self.message_queue
            .push(WebViewMessage::EvalJs(script.to_string()));

        Ok(())
    }

    /// Emit an event to JavaScript
    ///
    /// Args:
    ///     event_name (str): Name of the event
    ///     data (dict): Data to send with the event
    ///
    /// Note: This method is thread-safe. It pushes the event to a message queue
    /// that is processed by the WebView thread's event loop.
    #[allow(clippy::useless_conversion)]
    fn emit(&self, event_name: &str, data: &Bound<'_, PyDict>) -> PyResult<()> {
        tracing::info!("[CLOSE] [AuroraView::emit] START - Event: {}", event_name);

        // Convert Python dict to JSON
        tracing::info!("[CLOSE] [AuroraView::emit] Converting Python dict to JSON...");
        let json_data = py_dict_to_json(data).map_err(|e| {
            tracing::error!(
                "[ERROR] [AuroraView::emit] Failed to convert dict to JSON: {}",
                e
            );
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })?;
        tracing::info!("[CLOSE] [AuroraView::emit] JSON data: {}", json_data);

        // Push to message queue for thread-safe execution
        tracing::info!("[CLOSE] [AuroraView::emit] Pushing to message queue...");
        self.message_queue.push(WebViewMessage::EmitEvent {
            event_name: event_name.to_string(),
            data: json_data,
        });
        tracing::info!("[OK] [AuroraView::emit] Message pushed to queue successfully");

        Ok(())
    }

    /// Register a Python callback for JavaScript events
    ///
    /// Args:
    ///     event_name (str): Name of the event to listen for
    ///     callback (callable): Python function to call when event occurs
    #[allow(clippy::useless_conversion)]
    fn on(&self, event_name: &str, callback: Py<PyAny>) -> PyResult<()> {
        tracing::info!("Registering callback for event: {}", event_name);

        // Store the callback in the IPC handler
        self.ipc_handler
            .register_python_callback(event_name, callback);

        tracing::debug!("Callback registered for event: {}", event_name);

        Ok(())
    }

    /// Register API methods for JavaScript access
    ///
    /// This method updates the WebView configuration to include API methods
    /// that will be automatically injected into JavaScript as wrapper functions.
    ///
    /// Args:
    ///     namespace (str): API namespace (e.g., "api")
    ///     methods (list[str]): List of method names to register
    ///
    /// Example:
    ///     >>> webview.register_api_methods("api", ["get_data", "set_data"])
    ///     >>> # JavaScript can now call: window.auroraview.api.get_data()
    #[allow(clippy::useless_conversion)]
    fn register_api_methods(&self, namespace: &str, methods: Vec<String>) -> PyResult<()> {
        tracing::info!(
            "Registering {} API methods in namespace '{}'",
            methods.len(),
            namespace
        );

        // Update config with API methods
        if let Ok(mut config) = self.config.try_borrow_mut() {
            config
                .api_methods
                .insert(namespace.to_string(), methods.clone());
            tracing::debug!("Updated config with API methods: {:?}", config.api_methods);
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to borrow config",
            ));
        }

        // Generate and execute JavaScript to register methods
        let methods_json: Vec<String> = methods
            .iter()
            .map(|m| format!("'{}'", m.replace('\'', "\\'")))
            .collect();

        let js_code = format!(
            r#"
            (function() {{
                if (window.auroraview && window.auroraview._registerApiMethods) {{
                    window.auroraview._registerApiMethods('{}', [{}]);
                }} else {{
                    console.warn('[AuroraView] Event bridge not ready, API methods will be registered on page load');
                }}
            }})();
            "#,
            namespace.replace('\'', "\\'"),
            methods_json.join(", ")
        );

        // Try to execute immediately (may fail if page not loaded)
        match self.eval_js(&js_code) {
            Ok(_) => {
                tracing::info!("Successfully registered API methods in JavaScript");
            }
            Err(e) => {
                tracing::debug!(
                    "Could not register API methods immediately (page may not be loaded): {}",
                    e
                );
                // This is OK - methods will be registered when page loads via init script
            }
        }

        Ok(())
    }

    /// Close the WebView window
    #[allow(clippy::useless_conversion)]
    fn close(&self) -> PyResult<()> {
        tracing::info!("{}", "=".repeat(80));
        tracing::info!("[CLOSE] [AuroraView::close] Close method called");

        // Try to use event loop proxy first (standalone mode)
        if let Ok(proxy_opt) = self.event_loop_proxy.try_borrow() {
            if let Some(proxy) = proxy_opt.as_ref() {
                tracing::info!(
                    "[CLOSE] [AuroraView::close] Using event loop proxy to send close event"
                );
                match proxy.send_event(crate::webview::event_loop::UserEvent::CloseWindow) {
                    Ok(_) => {
                        tracing::info!("[OK] [AuroraView::close] Close event sent successfully");
                        tracing::info!("{}", "=".repeat(80));
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::error!(
                            "[ERROR] [AuroraView::close] Failed to send close event: {:?}",
                            e
                        );
                        // Fall through to direct close method
                    }
                }
            } else {
                tracing::info!("[CLOSE] [AuroraView::close] No event loop proxy (embedded mode), using direct close");
            }
        } else {
            tracing::warn!("[WARNING] [AuroraView::close] Failed to borrow event_loop_proxy");
        }

        // Fallback: Direct close (for embedded mode or if proxy fails)
        if let Ok(mut inner_opt) = self.inner.try_borrow_mut() {
            tracing::info!(
                "[CLOSE] [AuroraView::close] Successfully borrowed inner for direct close"
            );

            if let Some(inner) = inner_opt.as_mut() {
                tracing::info!("[CLOSE] [AuroraView::close] Inner exists");

                if let Some(window) = &inner.window {
                    tracing::info!(
                        "[CLOSE] [AuroraView::close] Window exists, attempting to close..."
                    );

                    // Set window invisible first
                    window.set_visible(false);
                    tracing::info!("[OK] [AuroraView::close] Window set to invisible");

                    // For embedded windows, we need special handling to ensure proper cleanup
                    #[cfg(target_os = "windows")]
                    {
                        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                        use std::ffi::c_void;
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::UI::WindowsAndMessaging::{
                            DestroyWindow, DispatchMessageW, PeekMessageW, TranslateMessage, MSG,
                            PM_REMOVE, WM_DESTROY, WM_NCDESTROY,
                        };

                        if let Ok(window_handle) = window.window_handle() {
                            let raw_handle = window_handle.as_raw();
                            if let RawWindowHandle::Win32(handle) = raw_handle {
                                let hwnd_value = handle.hwnd.get();
                                let hwnd = HWND(hwnd_value as *mut c_void);

                                tracing::info!("[CLOSE] [AuroraView::close] HWND: {:?}", hwnd);

                                // Step 1: Destroy the window
                                tracing::info!(
                                    "[CLOSE] [AuroraView::close] Calling DestroyWindow..."
                                );
                                unsafe {
                                    let result = DestroyWindow(hwnd);
                                    if result.is_ok() {
                                        tracing::info!(
                                            "[OK] [AuroraView::close] DestroyWindow succeeded"
                                        );

                                        // Step 2: CRITICAL - Process pending messages for this window
                                        // This ensures WM_DESTROY and WM_NCDESTROY are handled
                                        tracing::info!("[CLOSE] [AuroraView::close] Processing pending window messages...");
                                        let mut msg = MSG::default();
                                        let mut processed_count = 0;
                                        let max_iterations = 100; // Prevent infinite loop

                                        // Process all pending messages for this specific window
                                        while processed_count < max_iterations
                                            && PeekMessageW(&mut msg, Some(hwnd), 0, 0, PM_REMOVE)
                                                .as_bool()
                                        {
                                            processed_count += 1;

                                            // Log important messages
                                            if msg.message == WM_DESTROY {
                                                tracing::info!(
                                                    "[CLOSE] [AuroraView::close] Processing WM_DESTROY"
                                                );
                                            } else if msg.message == WM_NCDESTROY {
                                                tracing::info!(
                                                    "[CLOSE] [AuroraView::close] Processing WM_NCDESTROY (final cleanup)"
                                                );
                                            }

                                            let _ = TranslateMessage(&msg);
                                            DispatchMessageW(&msg);
                                        }

                                        tracing::info!(
                                            "[OK] [AuroraView::close] Processed {} window messages",
                                            processed_count
                                        );

                                        // Step 3: Small delay to allow window to fully disappear
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                        tracing::info!(
                                            "[OK] [AuroraView::close] Window cleanup completed"
                                        );
                                    } else {
                                        tracing::error!(
                                            "[ERROR] [AuroraView::close] DestroyWindow failed: {:?}",
                                            result
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    tracing::warn!("[WARNING] [AuroraView::close] No window found");
                }
            } else {
                tracing::warn!("[WARNING] [AuroraView::close] Inner is None");
            }
        } else {
            tracing::error!(
                "[ERROR] [AuroraView::close] Failed to borrow inner (already borrowed?)"
            );
        }

        tracing::info!("{}", "=".repeat(80));
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

    /// Process pending window events (for embedded mode)
    ///
    /// This method processes all pending Windows messages without blocking.
    /// It should be called periodically (e.g., from a Maya timer) to keep
    /// the window responsive in embedded mode.
    ///
    /// Returns:
    ///     bool: True if the window should be closed, False otherwise
    ///
    /// Note: In Maya, create a timer to process events using scriptJob with the idle event.
    /// Call this method periodically to handle window close events.
    fn process_events(&self) -> PyResult<bool> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            Ok(inner.process_events())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "WebView not initialized. Call show() first.",
            ))
        }
    }

    /// Process only internal IPC/messages without touching the host's native event loop.
    ///
    /// This is intended for host-driven embedding scenarios (Qt, DCC, etc.)
    /// where the parent application owns the Win32/OS event loop and is
    /// responsible for pumping window messages. It ONLY drains the internal
    /// WebView message queue and respects lifecycle close requests.
    ///
    /// Returns:
    ///     bool: True if the window should close, False otherwise
    fn process_ipc_only(&self) -> PyResult<bool> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            Ok(inner.process_ipc_only())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "WebView not initialized. Call show() first.",
            ))
        }
    }

    /// Check if the window is still valid (Windows only)
    ///
    /// This method checks if the window handle is still valid.
    /// Useful for detecting when a window has been closed externally.
    ///
    /// Returns:
    ///     bool: True if the window is valid, False otherwise
    ///
    /// Example:
    ///     >>> webview = WebView(parent_hwnd=maya_hwnd, embedded=True)
    ///     >>> webview.show()
    ///     >>> # Check if window is still valid
    ///     >>> if not webview.is_window_valid():
    ///     ...     print("Window was closed")
    fn is_window_valid(&self) -> PyResult<bool> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            Ok(inner.is_window_valid())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "WebView not initialized. Call show() first.",
            ))
        }
    }

    /// Set window position
    ///
    /// Moves the window to the specified screen coordinates.
    /// This is useful for implementing custom window dragging in frameless windows.
    ///
    /// Args:
    ///     x (int): X coordinate in screen pixels
    ///     y (int): Y coordinate in screen pixels
    fn set_window_position(&self, x: i32, y: i32) -> PyResult<()> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            inner.set_window_position(x, y);
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "WebView not initialized. Call show() first.",
            ))
        }
    }

    /// Get window handle (HWND on Windows)
    ///
    /// Returns the native window handle for the WebView window.
    /// On Windows, this is the HWND value as an integer.
    ///
    /// Returns:
    ///     int: Window handle (HWND on Windows), or None if not available
    ///
    /// Example:
    ///     >>> webview = WebView(title="My Window")
    ///     >>> webview.show()
    ///     >>> hwnd = webview.get_hwnd()
    ///     >>> print(f"Window HWND: 0x{hwnd:x}")
    fn get_hwnd(&self) -> PyResult<Option<u64>> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            Ok(inner.get_hwnd())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "WebView not initialized. Call show() first.",
            ))
        }
    }

    /// Get IPC performance metrics
    ///
    /// Returns a snapshot of current IPC metrics including:
    /// - Messages sent/failed/dropped
    /// - Success rate
    /// - Average latency
    /// - Peak queue length
    ///
    /// Returns:
    ///     IpcMetrics: Snapshot of current metrics
    ///
    /// Example:
    ///     ```python
    ///     metrics = webview.get_ipc_metrics()
    ///     print(f"Messages sent: {metrics.messages_sent}")
    ///     print(f"Success rate: {metrics.success_rate}%")
    ///     print(metrics.format())  # Human-readable format
    ///     ```
    fn get_ipc_metrics(&self) -> crate::bindings::ipc_metrics::PyIpcMetrics {
        let snapshot = self.message_queue.get_metrics_snapshot();
        snapshot.into()
    }

    /// Reset IPC performance metrics
    ///
    /// Resets all IPC metrics counters to zero.
    /// Useful for measuring performance over specific time periods.
    ///
    /// Example:
    ///     ```python
    ///     webview.reset_ipc_metrics()
    ///     # ... perform operations ...
    ///     metrics = webview.get_ipc_metrics()
    ///     print(f"Operations sent: {metrics.messages_sent}")
    ///     ```
    fn reset_ipc_metrics(&self) {
        // Note: MessageQueue doesn't expose reset() yet
        // This is a placeholder for future implementation
        tracing::warn!("[IPC] reset_ipc_metrics() not yet implemented");
    }

    /// Register a custom protocol handler
    ///
    /// Args:
    ///     scheme (str): Protocol scheme (e.g., "maya", "fbx")
    ///     handler (callable): Python function that takes URI string and returns dict with:
    ///         - data (bytes): Response data
    ///         - mime_type (str): MIME type (e.g., "image/png")
    ///         - status (int): HTTP status code (e.g., 200, 404)
    ///
    /// # Example
    ///
    /// ```text
    /// # Python example
    /// def handle_fbx(uri: str) -> dict:
    ///     path = uri.replace("fbx://", "")
    ///     try:
    ///         with open(f"C:/models/{path}", "rb") as f:
    ///             return {
    ///                 "data": f.read(),
    ///                 "mime_type": "application/octet-stream",
    ///                 "status": 200
    ///             }
    ///     except FileNotFoundError:
    ///         return {
    ///             "data": b"Not Found",
    ///             "mime_type": "text/plain",
    ///             "status": 404
    ///         }
    ///
    /// webview.register_protocol("fbx", handle_fbx)
    /// ```
    fn register_protocol(&self, scheme: &str, handler: Py<PyAny>) -> PyResult<()> {
        tracing::info!("[Protocol] Registering custom protocol: {}", scheme);

        // Wrap Python callback in Rust closure
        let callback = Arc::new(move |uri: &str| -> Option<(Vec<u8>, String, u16)> {
            Python::attach(|py| {
                // Call Python handler
                match handler.call1(py, (uri,)) {
                    Ok(result) => {
                        // Extract dict fields
                        #[allow(deprecated)]
                        let dict = result.downcast_bound::<pyo3::types::PyDict>(py).ok()?;

                        let data = dict
                            .get_item("data")
                            .ok()
                            .flatten()
                            .and_then(|v| v.extract::<Vec<u8>>().ok())?;
                        let mime_type = dict
                            .get_item("mime_type")
                            .ok()
                            .flatten()
                            .and_then(|v| v.extract::<String>().ok())?;
                        let status = dict
                            .get_item("status")
                            .ok()
                            .flatten()
                            .and_then(|v| v.extract::<u16>().ok())?;

                        Some((data, mime_type, status))
                    }
                    Err(e) => {
                        tracing::error!("[Protocol] Python handler error: {}", e);
                        None
                    }
                }
            })
        });

        // Add to config
        self.config
            .borrow_mut()
            .custom_protocols
            .insert(scheme.to_string(), callback);

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

/// Implement Drop to track when AuroraView is destroyed
impl Drop for AuroraView {
    fn drop(&mut self) {
        let title = self.config.borrow().title.clone();
        tracing::warn!(
            "[CLOSE] [AuroraView::drop] WebView '{}' is being destroyed!",
            title
        );
        tracing::warn!("[CLOSE] [AuroraView::drop] This will close the WebView window");

        // The inner WebViewInner will be dropped automatically
        // which will close the window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aurora_view_config_creation() {
        Python::initialize();
        Python::attach(|_py| {
            let result = AuroraView::new(
                "Test WebView",
                800,
                600,
                None,
                None,
                true,
                true,
                true,
                true,
                None,
                None,
                None,
            );
            assert!(result.is_ok());
            let webview = result.unwrap();
            let config = webview.config.borrow();
            assert_eq!(config.title, "Test WebView");
            assert_eq!(config.width, 800);
            assert_eq!(config.height, 600);
        });
    }

    #[test]
    fn test_aurora_view_with_url() {
        Python::initialize();
        Python::attach(|_py| {
            let result = AuroraView::new(
                "URL Test",
                1024,
                768,
                Some("https://example.com"),
                None,
                false,
                false,
                false,
                false,
                None,
                None,
                None,
            );
            assert!(result.is_ok());
            let config = result.unwrap().config.borrow().clone();
            assert_eq!(config.url, Some("https://example.com".to_string()));
        });
    }

    #[test]
    fn test_load_url_stores_in_config() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "Test", 800, 600, None, None, true, true, true, true, None, None, None,
            )
            .unwrap();
            let result = webview.load_url("https://test.com");
            assert!(result.is_ok());
            let config = webview.config.borrow();
            assert_eq!(config.url, Some("https://test.com".to_string()));
        });
    }

    #[test]
    fn test_eval_js_queues_message() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "Test", 800, 600, None, None, true, true, true, true, None, None, None,
            )
            .unwrap();
            let result = webview.eval_js("console.log('test')");
            assert!(result.is_ok());
            assert!(!webview.message_queue.is_empty());
        });
    }

    #[test]
    fn test_emit_queues_event_message() {
        Python::initialize();
        Python::attach(|py| {
            let webview = AuroraView::new(
                "Test", 800, 600, None, None, true, true, true, true, None, None, None,
            )
            .unwrap();
            let dict = PyDict::new(py);
            dict.set_item("key", "value").unwrap();
            let result = webview.emit("test_event", &dict);
            assert!(result.is_ok());
            assert!(!webview.message_queue.is_empty());
        });
    }

    #[test]
    fn test_repr() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "My WebView",
                1024,
                768,
                None,
                None,
                true,
                true,
                true,
                true,
                None,
                None,
                None,
            )
            .unwrap();
            let repr = webview.__repr__();
            assert!(repr.contains("My WebView"));
            assert!(repr.contains("1024"));
            assert!(repr.contains("768"));
        });
    }

    #[test]
    fn test_message_queue_initialization() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "Test", 800, 600, None, None, true, true, true, true, None, None, None,
            )
            .unwrap();
            assert!(webview.message_queue.is_empty());
            assert_eq!(webview.message_queue.len(), 0);
        });
    }

    #[test]
    fn test_load_html_clears_url() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "Test",
                800,
                600,
                Some("https://example.com"),
                None,
                true,
                true,
                true,
                true,
                None,
                None,
                None,
            )
            .unwrap();
            webview.load_html("<h1>HTML</h1>").unwrap();
            let config = webview.config.borrow();
            assert_eq!(config.html, Some("<h1>HTML</h1>".to_string()));
            assert_eq!(config.url, None);
        });
    }

    #[test]
    fn test_load_url_clears_html() {
        Python::initialize();
        Python::attach(|_py| {
            let webview = AuroraView::new(
                "Test",
                800,
                600,
                None,
                Some("<h1>HTML</h1>"),
                true,
                true,
                true,
                true,
                None,
                None,
                None,
            )
            .unwrap();
            webview.load_url("https://example.com").unwrap();
            let config = webview.config.borrow();
            assert_eq!(config.url, Some("https://example.com".to_string()));
            assert_eq!(config.html, None);
        });
    }
}
