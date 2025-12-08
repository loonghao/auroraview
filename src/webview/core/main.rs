//! AuroraView Core - Constructor, Lifecycle, and Properties
//!
//! This module contains the core WebView functionality:
//! - Constructor (`new`)
//! - Lifecycle methods (`show`, `close`, `run`)
//! - Factory methods (`create_for_dcc`, `create_for_dcc_async`)
//! - Property getters (`title`, `url`, `progress`, `loading`)

use pyo3::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::AuroraView;
use crate::ipc::{IpcHandler, JsCallbackManager, MessageQueue, WebViewMessage};
use crate::webview::config::WebViewConfig;
use crate::webview::webview_inner::WebViewInner;

#[pymethods]
impl AuroraView {
    /// Create a new WebView instance
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (title="DCC WebView", width=800, height=600, url=None, html=None, dev_tools=true, context_menu=true, resizable=true, decorations=true, parent_hwnd=None, parent_mode=None, asset_root=None, data_directory=None, allow_file_protocol=false, always_on_top=false, transparent=false, background_color=None, auto_show=true, ipc_batch_size=0))]
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
        data_directory: Option<&str>,
        allow_file_protocol: bool,
        always_on_top: bool,
        transparent: bool,
        background_color: Option<&str>,
        auto_show: bool,
        ipc_batch_size: usize,
    ) -> PyResult<Self> {
        tracing::info!("AuroraView::new() called with title: {}", title);

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
            data_directory: data_directory.map(std::path::PathBuf::from),
            allow_file_protocol,
            always_on_top,
            transparent,
            background_color: background_color.map(|s| s.to_string()),
            auto_show,
            ipc_batch_size,
            ..Default::default()
        };

        // Map string to EmbedMode (Windows)
        // Only "child" mode is supported (official recommended approach)
        #[cfg(target_os = "windows")]
        {
            use crate::webview::config::EmbedMode;
            config.embed_mode = match parent_mode.map(|s| s.to_ascii_lowercase()) {
                Some(ref m) if m == "child" => EmbedMode::Child,
                _ => {
                    // Default: use Child mode if parent_hwnd is provided, otherwise None
                    if parent_hwnd.is_some() {
                        EmbedMode::Child
                    } else {
                        EmbedMode::None
                    }
                }
            };
        }

        // Create IPC handler and JS callback manager
        let js_callback_manager = Arc::new(JsCallbackManager::new());
        let mut ipc_handler = IpcHandler::new();
        ipc_handler.set_js_callback_manager(js_callback_manager.clone());

        Ok(AuroraView {
            inner: Rc::new(RefCell::new(None)),
            config: Rc::new(RefCell::new(config)),
            ipc_handler: Arc::new(ipc_handler),
            message_queue: Arc::new(MessageQueue::new()),
            event_loop_proxy: Rc::new(RefCell::new(None)),
            js_callback_manager,
            on_hwnd_created: Rc::new(RefCell::new(None)),
        })
    }

    /// Set callback for when WebView2 HWND is created
    ///
    /// The callback receives the HWND (int) as argument.
    /// This is called immediately when the underlying WebView2 window is created.
    fn set_on_hwnd_created(&self, callback: Py<PyAny>) {
        *self.on_hwnd_created.borrow_mut() = Some(callback);
    }

    /// Test method to verify Python bindings work
    fn test_method(&self) -> PyResult<String> {
        Ok("test_method works!".to_string())
    }

    // === Lifecycle Methods ===

    /// Show WebView window (standalone mode or embedded mode)
    fn show(&self) -> PyResult<()> {
        use crate::webview::config::EmbedMode;
        let embed_mode = self.config.borrow().embed_mode;

        match embed_mode {
            EmbedMode::None => self.show_window(),
            #[cfg(target_os = "windows")]
            EmbedMode::Child => self.show_embedded(),
        }
    }

    /// Create WebView for standalone mode (creates its own window)
    fn show_window(&self) -> PyResult<()> {
        let title = self.config.borrow().title.clone();
        tracing::info!("Showing WebView (standalone mode): {}", title);

        let mut inner = self.inner.borrow_mut();
        let need_create = inner.as_ref().map_or(true, |e| e.event_loop.is_none());

        if need_create {
            let mut webview = WebViewInner::create_standalone(
                self.config.borrow().clone(),
                self.ipc_handler.clone(),
                self.message_queue.clone(),
            )
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            if let Some(proxy) = webview.event_loop_proxy.take() {
                *self.event_loop_proxy.borrow_mut() = Some(proxy);
            }
            *inner = Some(webview);
        }

        drop(inner);

        if let Some(webview_inner) = self.inner.borrow_mut().as_mut() {
            webview_inner.run_event_loop_blocking();
        }

        *self.inner.borrow_mut() = None;
        Ok(())
    }

    /// Show embedded WebView (non-blocking)
    #[cfg(target_os = "windows")]
    fn show_embedded(&self) -> PyResult<()> {
        let title = self.config.borrow().title.clone();
        tracing::info!("Creating embedded WebView: {}", title);

        // Track if we need to invoke the callback after releasing the borrow
        let mut created_hwnd: Option<u64> = None;

        {
            // Scope the mutable borrow to release it before callback invocation
            let mut inner = self.inner.borrow_mut();
            let need_create = inner.as_ref().map_or(true, |e| e.event_loop.is_none());

            if need_create {
                let config = self.config.borrow().clone();
                let parent_hwnd = config.parent_hwnd.unwrap_or(0);

                // Don't pass callback to create_embedded - we'll invoke it after releasing borrow
                // This avoids RefCell borrow conflict when callback tries to access self.inner
                let webview = WebViewInner::create_embedded(
                    parent_hwnd,
                    config.width,
                    config.height,
                    config,
                    self.ipc_handler.clone(),
                    self.message_queue.clone(),
                    None, // No callback - we'll call it manually after borrow is released
                )
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                // Get HWND before storing webview
                created_hwnd = webview.get_hwnd();

                *inner = Some(webview);
            }
        } // inner borrow released here

        // Now safe to invoke callback - self.inner is no longer borrowed
        if let Some(hwnd) = created_hwnd {
            if let Some(callback) = Python::attach(|py| {
                self.on_hwnd_created
                    .borrow()
                    .as_ref()
                    .map(|cb| cb.clone_ref(py))
            }) {
                tracing::info!(
                    "[OK] [show_embedded] Invoking on_hwnd_created callback with HWND 0x{:X}",
                    hwnd
                );
                Python::attach(|py| {
                    if let Err(e) = callback.call1(py, (hwnd,)) {
                        tracing::error!("Error calling on_hwnd_created callback: {:?}", e);
                        e.print(py);
                    }
                });
            }
        }

        Ok(())
    }

    /// Create WebView for embedded mode (for DCC integration)
    fn create_embedded(&self, parent_hwnd: u64, width: u32, height: u32) -> PyResult<()> {
        tracing::info!("Creating embedded WebView for parent HWND: {}", parent_hwnd);

        // Track if we need to invoke the callback after releasing the borrow
        let mut created_hwnd: Option<u64> = None;

        {
            // Scope the mutable borrow to release it before callback invocation
            let mut inner = self.inner.borrow_mut();
            if inner.is_none() {
                // Don't pass callback to create_embedded - we'll invoke it after releasing borrow
                // This avoids RefCell borrow conflict when callback tries to access self.inner
                let webview = WebViewInner::create_embedded(
                    parent_hwnd,
                    width,
                    height,
                    self.config.borrow().clone(),
                    self.ipc_handler.clone(),
                    self.message_queue.clone(),
                    None, // No callback - we'll call it manually after borrow is released
                )
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                // Get HWND before storing webview
                created_hwnd = webview.get_hwnd();

                *inner = Some(webview);
            }
        } // inner borrow released here

        // Now safe to invoke callback - self.inner is no longer borrowed
        if let Some(hwnd) = created_hwnd {
            if let Some(callback) = Python::attach(|py| {
                self.on_hwnd_created
                    .borrow()
                    .as_ref()
                    .map(|cb| cb.clone_ref(py))
            }) {
                tracing::info!(
                    "[OK] [create_embedded] Invoking on_hwnd_created callback with HWND 0x{:X}",
                    hwnd
                );
                Python::attach(|py| {
                    if let Err(e) = callback.call1(py, (hwnd,)) {
                        tracing::error!("Error calling on_hwnd_created callback: {:?}", e);
                        e.print(py);
                    }
                });
            }
        }

        Ok(())
    }

    /// Close the WebView window
    fn close(&self) -> PyResult<()> {
        use super::super::event_loop::UserEvent;

        if let Some(ref proxy) = *self.event_loop_proxy.borrow() {
            let _ = proxy.send_event(UserEvent::CloseWindow);
        }
        Ok(())
    }

    /// Process messages for DCC integration mode
    fn process_messages(&self) -> PyResult<bool> {
        let inner = self.inner.borrow();
        if let Some(webview) = inner.as_ref() {
            Ok(webview.process_messages())
        } else {
            Ok(false)
        }
    }

    /// Process IPC messages only (no window message pump)
    ///
    /// For DCC integration mode (create_for_dcc_async), this processes
    /// Python callbacks on the main thread while the background thread
    /// handles the actual WebView event loop.
    fn process_ipc_only(&self) -> PyResult<bool> {
        // Use try_borrow to avoid panic during initialization
        match self.inner.try_borrow() {
            Ok(inner_ref) => {
                if let Some(ref inner) = *inner_ref {
                    return Ok(inner.process_ipc_only());
                }
            }
            Err(_) => {
                tracing::trace!("[process_ipc_only] RefCell already borrowed, using async mode");
            }
        }

        // For background thread mode, inner is None
        let queue_len = self.message_queue.len();
        if queue_len > 0 {
            tracing::trace!(
                "[process_ipc_only] Async mode: {} messages in queue (handled by background)",
                queue_len
            );
        }

        Ok(false)
    }

    /// Cleanup timed-out JavaScript callbacks
    fn cleanup_timed_out_callbacks(&self) -> PyResult<usize> {
        Ok(self.js_callback_manager.cleanup_timed_out())
    }

    /// Get the default timeout for JavaScript callbacks
    fn get_js_timeout(&self) -> PyResult<u64> {
        Ok(self.js_callback_manager.default_timeout_ms())
    }

    /// Get the number of pending JavaScript callbacks
    fn pending_js_callbacks(&self) -> PyResult<usize> {
        Ok(self.js_callback_manager.pending_count())
    }

    // === Content Loading ===

    /// Load a URL in the WebView
    fn load_url(&self, url: &str) -> PyResult<()> {
        tracing::info!("Loading URL: {}", url);

        if let Some(webview) = self.inner.borrow_mut().as_mut() {
            webview
                .load_url(url)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            return Ok(());
        }

        let has_parent = self.config.borrow().parent_hwnd.is_some();
        if has_parent {
            self.message_queue
                .push(WebViewMessage::LoadUrl(url.to_string()));
            return Ok(());
        }

        let mut cfg = self.config.borrow_mut();
        cfg.url = Some(url.to_string());
        cfg.html = None;
        Ok(())
    }

    /// Load HTML content in the WebView
    fn load_html(&self, html: &str) -> PyResult<()> {
        tracing::info!("Loading HTML content ({} bytes)", html.len());

        if let Some(webview) = self.inner.borrow_mut().as_mut() {
            webview
                .load_html(html)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            return Ok(());
        }

        let has_parent = self.config.borrow().parent_hwnd.is_some();
        if has_parent {
            self.message_queue
                .push(WebViewMessage::LoadHtml(html.to_string()));
            return Ok(());
        }

        let mut cfg = self.config.borrow_mut();
        cfg.html = Some(html.to_string());
        cfg.url = None;
        Ok(())
    }

    /// Navigate to a URL (alias for load_url)
    fn navigate(&self, url: &str) -> PyResult<()> {
        self.load_url(url)
    }

    /// Reload the current page
    fn reload(&self) -> PyResult<()> {
        self.message_queue.push(WebViewMessage::Reload);
        Ok(())
    }

    /// Stop loading the current page
    fn stop_loading(&self) -> PyResult<()> {
        self.message_queue.push(WebViewMessage::StopLoading);
        Ok(())
    }

    // === Property Getters ===

    /// Get the window title
    #[getter]
    fn title(&self) -> PyResult<String> {
        Ok(self.config.borrow().title.clone())
    }

    /// Set the window title
    #[setter]
    fn set_title(&mut self, title: String) -> PyResult<()> {
        self.config.borrow_mut().title = title;
        Ok(())
    }

    /// Get the current URL
    #[getter]
    fn url(&self) -> PyResult<String> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            inner
                .get_url()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(self.config.borrow().url.clone().unwrap_or_default())
        }
    }

    /// Get the current load progress (0-100)
    #[getter]
    fn progress(&self) -> PyResult<u8> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            inner
                .load_progress()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(0)
        }
    }

    /// Check if page is loading
    #[getter]
    fn loading(&self) -> PyResult<bool> {
        let inner_ref = self.inner.borrow();
        if let Some(ref inner) = *inner_ref {
            inner
                .is_loading()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(false)
        }
    }

    /// Get the window handle (HWND on Windows)
    fn get_hwnd(&self) -> PyResult<Option<u64>> {
        // Use try_borrow to avoid panic if RefCell is already borrowed
        // This can happen during callback invocations from show_embedded/create_embedded
        match self.inner.try_borrow() {
            Ok(inner) => {
                if let Some(ref webview) = *inner {
                    Ok(webview.get_hwnd())
                } else {
                    Ok(None)
                }
            }
            Err(_) => {
                // RefCell already borrowed - return None instead of panicking
                tracing::debug!("[get_hwnd] RefCell already borrowed, returning None");
                Ok(None)
            }
        }
    }

    /// Get a thread-safe proxy for cross-thread operations
    ///
    /// Returns a `WebViewProxy` that can be safely shared across threads.
    /// Use this when you need to call `eval_js`, `emit`, etc. from a different
    /// thread than the one that created the WebView.
    ///
    /// This is essential for HWND mode where the WebView runs in a background
    /// thread but you need to call methods from the DCC main thread.
    ///
    /// Returns:
    ///     WebViewProxy: A thread-safe proxy for WebView operations
    ///
    /// Example:
    ///     >>> # In HWND mode - WebView runs in background thread
    ///     >>> def create_webview():
    ///     ...     webview = WebView(...)
    ///     ...     proxy = webview.get_proxy()  # Get thread-safe proxy
    ///     ...     self._proxy = proxy          # Store for later use
    ///     ...     webview.show_blocking()
    ///     ...
    ///     >>> # From DCC main thread - safe!
    ///     >>> self._proxy.eval_js("console.log('Hello!')")
    fn get_proxy(&self) -> crate::webview::proxy::WebViewProxy {
        crate::webview::proxy::WebViewProxy::new(
            self.message_queue.clone(),
            self.js_callback_manager.clone(),
        )
    }

    // === Lifecycle State Query APIs ===

    /// Check if the WebView is alive and ready for operations
    ///
    /// Returns True if the WebView has been created and is not destroyed.
    /// Use this to check if it's safe to call other WebView methods.
    ///
    /// Returns:
    ///     bool: True if WebView is alive, False otherwise
    ///
    /// Example:
    ///     >>> if webview.is_alive():
    ///     ...     webview.eval_js("console.log('Hello!')")
    fn is_alive(&self) -> PyResult<bool> {
        let inner = self.inner.borrow();
        if let Some(ref webview) = *inner {
            use crate::webview::lifecycle::LifecycleState;
            let state = webview.lifecycle_state();
            Ok(!matches!(
                state,
                LifecycleState::Destroyed
                    | LifecycleState::Destroying
                    | LifecycleState::CloseRequested
            ))
        } else {
            Ok(false)
        }
    }

    /// Get the current lifecycle state as a string
    ///
    /// Returns one of: "creating", "active", "close_requested", "destroying", "destroyed", "none"
    ///
    /// Returns:
    ///     str: Current lifecycle state
    ///
    /// Example:
    ///     >>> state = webview.lifecycle_state
    ///     >>> if state == "active":
    ///     ...     webview.eval_js("console.log('Ready!')")
    #[getter]
    fn lifecycle_state(&self) -> PyResult<String> {
        let inner = self.inner.borrow();
        if let Some(ref webview) = *inner {
            use crate::webview::lifecycle::LifecycleState;
            let state = webview.lifecycle_state();
            let state_str = match state {
                LifecycleState::Creating => "creating",
                LifecycleState::Active => "active",
                LifecycleState::CloseRequested => "close_requested",
                LifecycleState::Destroying => "destroying",
                LifecycleState::Destroyed => "destroyed",
            };
            Ok(state_str.to_string())
        } else {
            Ok("none".to_string())
        }
    }

    /// Reset the WebView state for reuse
    ///
    /// This clears internal state so the WebView can be shown again after being closed.
    /// Call this before calling show() a second time on the same instance.
    ///
    /// Example:
    ///     >>> webview.close()
    ///     >>> # ... later ...
    ///     >>> webview.reset()  # Clear old state
    ///     >>> webview.show()   # Show again
    fn reset(&self) -> PyResult<()> {
        tracing::info!("[AuroraView::reset] Resetting WebView state for reuse");

        // Clear the inner WebViewInner
        *self.inner.borrow_mut() = None;

        // Clear the event loop proxy
        *self.event_loop_proxy.borrow_mut() = None;

        // Clear the message queue
        self.message_queue.clear();

        tracing::info!("[AuroraView::reset] WebView state reset complete");
        Ok(())
    }
}
