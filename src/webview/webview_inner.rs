//! WebViewInner - Core WebView implementation
//!
//! This module contains the internal WebView structure and core operations.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::config::WebViewConfig;
use super::embedded;
use super::event_loop::{EventLoopState, UserEvent, WebViewEventHandler};
use super::lifecycle::LifecycleManager;
use super::message_pump;
use super::platform::PlatformWindowManager;
use super::standalone;
use crate::ipc::{IpcHandler, MessageQueue};

/// Internal WebView structure - supports both standalone and embedded modes
pub struct WebViewInner {
    pub(crate) webview: Arc<Mutex<WryWebView>>,
    // For standalone mode only
    #[allow(dead_code)]
    pub(crate) window: Option<tao::window::Window>,
    #[allow(dead_code)]
    pub(crate) event_loop: Option<tao::event_loop::EventLoop<UserEvent>>,
    /// Message queue for thread-safe communication
    pub(crate) message_queue: Arc<MessageQueue>,
    /// Event loop proxy for sending close events (standalone mode only)
    pub(crate) event_loop_proxy: Option<tao::event_loop::EventLoopProxy<UserEvent>>,
    /// Cross-platform lifecycle manager
    pub(crate) lifecycle: Arc<LifecycleManager>,
    /// Platform-specific window manager
    pub(crate) platform_manager: Option<Box<dyn PlatformWindowManager>>,
    /// Backend instance for DCC mode - MUST be kept alive to prevent window destruction
    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    pub(crate) backend: Option<Box<super::backend::native::NativeBackend>>,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        use scopeguard::defer;

        defer! {
            tracing::warn!("[DROP] [WebViewInner::drop] Cleanup completed");
        }

        tracing::warn!("========================================");
        tracing::warn!("[DROP] WebViewInner is being dropped!");
        tracing::warn!("========================================");
        tracing::info!("[CLOSE] [WebViewInner::drop] Cleaning up WebView resources");

        // Execute lifecycle cleanup handlers
        self.lifecycle.execute_cleanup();

        // Cleanup platform-specific resources
        if let Some(platform_manager) = &self.platform_manager {
            platform_manager.cleanup();
        }

        // Close the window if it exists
        if let Some(window) = self.window.take() {
            tracing::info!("[CLOSE] [WebViewInner::drop] Setting window invisible");
            window.set_visible(false);

            // On Windows, explicitly destroy the window and process cleanup messages
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

                        tracing::info!(
                            "[CLOSE] [WebViewInner::drop] Calling DestroyWindow on HWND: {:?}",
                            hwnd
                        );
                        unsafe {
                            let result = DestroyWindow(hwnd);
                            if result.is_ok() {
                                tracing::info!("[OK] [WebViewInner::drop] DestroyWindow succeeded");

                                // Process pending messages to ensure proper cleanup
                                tracing::info!(
                                    "[CLOSE] [WebViewInner::drop] Processing pending window messages..."
                                );
                                let mut msg = MSG::default();
                                let mut processed_count = 0;
                                let max_iterations = 100;

                                while processed_count < max_iterations
                                    && PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool()
                                {
                                    processed_count += 1;

                                    if msg.message == WM_DESTROY {
                                        tracing::info!(
                                            "[CLOSE] [WebViewInner::drop] Processing WM_DESTROY"
                                        );
                                    } else if msg.message == WM_NCDESTROY {
                                        tracing::info!(
                                            "[CLOSE] [WebViewInner::drop] Processing WM_NCDESTROY"
                                        );
                                    }

                                    let _ = TranslateMessage(&msg);
                                    DispatchMessageW(&msg);
                                }

                                tracing::info!(
                                    "[OK] [WebViewInner::drop] Processed {} messages",
                                    processed_count
                                );

                                // Small delay to ensure window disappears
                                std::thread::sleep(std::time::Duration::from_millis(50));
                            } else {
                                tracing::warn!(
                                    "[WARNING] [WebViewInner::drop] DestroyWindow failed: {:?}",
                                    result
                                );
                            }
                        }
                    }
                }
            }
        }

        // Drop the event loop (this will clean up any associated resources)
        if let Some(_event_loop) = self.event_loop.take() {
            tracing::info!("[CLOSE] [WebViewInner::drop] Event loop dropped");
        }
    }
}

impl WebViewInner {
    /// Create standalone WebView with its own window
    pub fn create_standalone(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        standalone::create_standalone(config, ipc_handler, message_queue)
    }

    /// Create embedded WebView for DCC integration
    pub fn create_embedded(
        parent_hwnd: u64,
        width: u32,
        height: u32,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        embedded::create_embedded(
            parent_hwnd,
            width,
            height,
            config,
            ipc_handler,
            message_queue,
        )
    }

    /// Create WebView for DCC integration (no event loop)
    ///
    /// This method creates a WebView that integrates with DCC applications by
    /// reusing the DCC's Qt message pump instead of creating its own event loop.
    ///
    /// # Arguments
    /// * `parent_hwnd` - HWND of the DCC main window
    /// * `config` - WebView configuration
    /// * `ipc_handler` - IPC message handler
    /// * `message_queue` - Message queue for cross-thread communication
    ///
    /// # Returns
    /// A WebViewInner instance without an event loop
    #[cfg(target_os = "windows")]
    pub fn create_for_dcc(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use super::backend::native::NativeBackend;
        use super::backend::WebViewBackend;

        // Create backend using DCC integration mode
        let backend = NativeBackend::create_for_dcc(
            parent_hwnd,
            config.clone(),
            ipc_handler,
            message_queue.clone(),
        )?;

        // Extract webview reference (but keep backend alive!)
        let webview = backend.webview();

        tracing::info!("[OK] [create_for_dcc] Keeping backend alive to prevent window destruction");

        Ok(Self {
            webview,
            window: None,     // Window is owned by backend
            event_loop: None, // Event loop is owned by backend
            message_queue,
            event_loop_proxy: None,
            lifecycle: Arc::new(LifecycleManager::new()),
            platform_manager: None,
            backend: Some(Box::new(backend)), // CRITICAL: Keep backend alive!
        })
    }

    /// Create WebView for DCC integration (non-Windows platforms)
    #[cfg(not(target_os = "windows"))]
    pub fn create_for_dcc(
        _parent_hwnd: u64,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
        _message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Err("DCC integration mode is only supported on Windows".into())
    }

    /// Process messages for DCC integration mode
    ///
    /// This method should be called periodically from a Qt timer to process
    /// WebView messages without running a dedicated event loop.
    ///
    /// # Returns
    /// `true` if the window should be closed, `false` otherwise
    pub fn process_messages(&self) -> bool {
        // Process Windows messages for this window
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd = handle.hwnd.get() as u64;
                        let should_quit = message_pump::process_messages_for_hwnd(hwnd);

                        // Process message queue
                        if let Ok(webview) = self.webview.lock() {
                            self.message_queue.process_all(|message| {
                                use crate::ipc::WebViewMessage;
                                match message {
                                    WebViewMessage::EvalJs(script) => {
                                        if let Err(e) = webview.evaluate_script(&script) {
                                            tracing::error!("Failed to execute JavaScript: {}", e);
                                        }
                                    }
                                    WebViewMessage::EmitEvent { event_name, data } => {
                                        let json_str = data.to_string();
                                        let escaped_json =
                                            json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                        let script = format!(
                                            "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                                            event_name, escaped_json
                                        );
                                        if let Err(e) = webview.evaluate_script(&script) {
                                            tracing::error!("Failed to emit event: {}", e);
                                        }
                                    }
                                    WebViewMessage::LoadUrl(url) => {
                                        let script = format!("window.location.href = '{}';", url);
                                        if let Err(e) = webview.evaluate_script(&script) {
                                            tracing::error!("Failed to load URL: {}", e);
                                        }
                                    }
                                    WebViewMessage::LoadHtml(html) => {
                                        if let Err(e) = webview.load_html(&html) {
                                            tracing::error!("Failed to load HTML: {}", e);
                                        }
                                    }
                                }
                            });
                        }

                        should_quit
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    /// Load a URL
    pub fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Use JavaScript to navigate
        let script = format!("window.location.href = '{}';", url);
        if let Ok(webview) = self.webview.lock() {
            webview.evaluate_script(&script)?;
        }
        Ok(())
    }

    /// Load HTML content
    pub fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(webview) = self.webview.lock() {
            webview.load_html(html)?;
        }
        Ok(())
    }

    /// Execute JavaScript
    #[allow(dead_code)]
    pub fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(webview) = self.webview.lock() {
            webview.evaluate_script(script)?;
        }
        Ok(())
    }

    /// Emit an event to JavaScript
    #[allow(dead_code)]
    pub fn emit(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Mark events emitted from Python to avoid being re-forwarded by the bridge (feedback loop)
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: Object.assign({{}}, {{__aurora_from_python: true}}, {}) }}))",
            event_name, data
        );
        if let Ok(webview) = self.webview.lock() {
            webview.evaluate_script(&script)?;
        }
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

        // Get the webview from Arc<Mutex<>>
        // We need to lock it to get a reference
        let webview_guard = match self.webview.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to lock webview: {:?}", e);
                return;
            }
        };

        // We can't move the webview out of the Arc<Mutex<>>, so we need to
        // restructure this. Let's just pass None for now and fix the architecture later.
        drop(webview_guard);

        // TEMPORARY FIX: Create state without webview
        // TODO: Refactor EventLoopState to accept Arc<Mutex<WryWebView>>
        tracing::warn!("Creating EventLoopState without webview - this needs architectural fix");

        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState::new_without_webview(
            window,
            self.message_queue.clone(),
        )));

        // Store webview reference in state after creation
        if let Ok(mut state_guard) = state.lock() {
            state_guard.set_webview(self.webview.clone());
        }

        // Run the improved event loop
        WebViewEventHandler::run_blocking(event_loop, state);

        tracing::info!("Event loop exited");
    }

    /// Set window position
    ///
    /// Moves the window to the specified screen coordinates.
    /// This is useful for implementing custom window dragging in frameless windows.
    ///
    /// # Arguments
    /// * `x` - X coordinate in screen pixels
    /// * `y` - Y coordinate in screen pixels
    ///
    /// # Platform-specific behavior
    /// - Windows: Uses SetWindowPos API
    /// - macOS/Linux: Uses platform-specific window positioning
    pub fn set_window_position(&self, x: i32, y: i32) {
        if let Some(window) = &self.window {
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                use std::ffi::c_void;
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
                };

                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get();
                        let hwnd = HWND(hwnd_value as *mut c_void);

                        unsafe {
                            let result = SetWindowPos(
                                hwnd,
                                None,
                                x,
                                y,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );

                            if result.is_ok() {
                                tracing::debug!(
                                    "[OK] [set_window_position] Window moved to ({}, {})",
                                    x,
                                    y
                                );
                            } else {
                                tracing::error!(
                                    "[ERROR] [set_window_position] Failed to move window to ({}, {})",
                                    x,
                                    y
                                );
                            }
                        }
                    }
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                use tao::dpi::PhysicalPosition;
                window.set_outer_position(PhysicalPosition::new(x, y));
                tracing::debug!("[OK] [set_window_position] Window moved to ({}, {})", x, y);
            }
        } else {
            tracing::warn!("[WARNING] [set_window_position] No window available");
        }
    }

    /// Get window handle (HWND on Windows)
    ///
    /// Returns the native window handle for the WebView window.
    /// On Windows, this is the HWND value as a u64.
    ///
    /// # Returns
    /// - `Some(hwnd)` - Window handle on Windows
    /// - `None` - No window available or not on Windows
    ///
    /// # Example
    /// ```ignore
    /// let hwnd = webview_inner.get_hwnd();
    /// if let Some(hwnd) = hwnd {
    ///     println!("Window HWND: 0x{:x}", hwnd);
    /// }
    /// ```
    pub fn get_hwnd(&self) -> Option<u64> {
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get() as u64;
                        return Some(hwnd_value);
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Not supported on non-Windows platforms
        }

        None
    }

    /// Check if the window is still valid (Windows only)
    ///
    /// This method checks if the window handle is still valid.
    /// Useful for detecting when a window has been closed externally.
    ///
    /// Returns true if the window is valid, false otherwise.
    pub fn is_window_valid(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get() as u64;
                        return message_pump::is_window_valid(hwnd_value);
                    }
                }
            }
            false
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows platforms, assume window is valid if it exists
            self.window.is_some()
        }
    }

    /// Process pending window messages (for embedded mode)
    ///
    /// This method processes all pending Windows messages without blocking.
    /// It should be called periodically (e.g., from a Maya timer) to keep
    /// the window responsive in embedded mode.
    ///
    /// Returns true if the window should be closed, false otherwise.
    pub fn process_events(&self) -> bool {
        use crate::webview::lifecycle::LifecycleState;
        use scopeguard::defer;

        defer! {
            tracing::trace!("[process_events] tick completed");
        }

        // Check lifecycle state first
        match self.lifecycle.state() {
            LifecycleState::Destroyed => {
                tracing::warn!("[process_events] Window already destroyed");
                return true;
            }
            LifecycleState::CloseRequested | LifecycleState::Destroying => {
                tracing::info!("[process_events] Close already in progress");
                return true;
            }
            _ => {}
        }

        // Check for close signal from lifecycle manager
        if let Some(reason) = self.lifecycle.check_close_requested() {
            tracing::info!(
                "[process_events] Close requested via lifecycle: {:?}",
                reason
            );
            return true;
        }

        // Use platform-specific manager if available
        if let Some(platform_manager) = &self.platform_manager {
            if platform_manager.process_events() {
                tracing::info!("[process_events] Platform manager detected close");
                return true;
            }

            // Also check window validity
            if !platform_manager.is_window_valid() {
                tracing::info!("[process_events] Platform manager reports invalid window");
                return true;
            }
        }

        // Get the window HWND for targeted message processing
        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get() as u64;
                        Some(hwnd_value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        #[cfg(not(target_os = "windows"))]
        let hwnd: Option<u64> = None;

        // Process Windows messages with specific HWND if available
        let should_quit = if let Some(hwnd_value) = hwnd {
            let b1 = message_pump::process_messages_for_hwnd(hwnd_value);
            // Also service child/IPC windows (e.g., WebView2) in the same thread.
            // Use full-thread scan on Windows to ensure we don't miss SC_CLOSE/WM_CLOSE.
            #[cfg(target_os = "windows")]
            let b2 = message_pump::process_all_messages();
            #[cfg(not(target_os = "windows"))]
            let b2 = message_pump::process_all_messages_limited(1024);
            b1 || b2
        } else {
            // If we don't yet have a window handle, don't pull host messages.
            false
        };

        if should_quit {
            tracing::debug!(
                "[process_events] should_quit=true; close signal detected; returning to Python"
            );
            return true;
        }

        // Process message queue
        tracing::trace!("[process_events] processing queue");

        if let Ok(webview) = self.webview.lock() {
            let count = self.message_queue.process_all(|message| {
                tracing::trace!("[process_events] processing message: {:?}", message);
                use crate::ipc::WebViewMessage;
                match message {
                    WebViewMessage::EvalJs(script) => {
                        tracing::debug!("Processing EvalJs: {}", script);
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to execute JavaScript: {}", e);
                        }
                    }
                    WebViewMessage::EmitEvent { event_name, data } => {
                        tracing::debug!(
                            "[OK] [process_events] Emitting event: {} with data: {}",
                            event_name,
                            data
                        );
                        // Properly escape JSON data to avoid JavaScript syntax errors
                        let json_str = data.to_string();
                        let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                        let script = format!(
                            "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                            event_name, escaped_json
                        );
                        tracing::debug!("[CLOSE] [process_events] Generated script: {}", script);
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to emit event: {}", e);
                        } else {
                            tracing::debug!("[OK] [process_events] Event emitted successfully");
                        }
                    }
                    WebViewMessage::LoadUrl(url) => {
                        let script = format!("window.location.href = '{}';", url);
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to load URL: {}", e);
                        }
                    }
                    WebViewMessage::LoadHtml(html) => {
                        tracing::debug!("Processing LoadHtml ({} bytes)", html.len());
                        if let Err(e) = webview.load_html(&html) {
                            tracing::error!("Failed to load HTML: {}", e);
                        }
                    }
                }
            });

            if count > 0 {
                tracing::debug!("[process_events] processed {} messages from queue", count);
            } else {
                tracing::trace!("[process_events] no messages in queue");
            }
        } else {
            tracing::error!("[process_events] failed to lock WebView");
        }

        tracing::trace!("[process_events] end");

        false
    }
}
