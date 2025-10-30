//! WebViewInner - Core WebView implementation
//!
//! This module contains the internal WebView structure and core operations.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::config::WebViewConfig;
use super::embedded;
use super::event_loop::{EventLoopState, UserEvent, WebViewEventHandler};
use super::message_pump;
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
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        tracing::info!("🔴 [WebViewInner::drop] Cleaning up WebView resources");

        // Close the window if it exists
        if let Some(window) = self.window.take() {
            tracing::info!("🔴 [WebViewInner::drop] Setting window invisible");
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
                            "🔴 [WebViewInner::drop] Calling DestroyWindow on HWND: {:?}",
                            hwnd
                        );
                        unsafe {
                            let result = DestroyWindow(hwnd);
                            if result.is_ok() {
                                tracing::info!("✅ [WebViewInner::drop] DestroyWindow succeeded");

                                // Process pending messages to ensure proper cleanup
                                tracing::info!(
                                    "🔴 [WebViewInner::drop] Processing pending window messages..."
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
                                            "🔴 [WebViewInner::drop] Processing WM_DESTROY"
                                        );
                                    } else if msg.message == WM_NCDESTROY {
                                        tracing::info!(
                                            "🔴 [WebViewInner::drop] Processing WM_NCDESTROY"
                                        );
                                    }

                                    let _ = TranslateMessage(&msg);
                                    DispatchMessageW(&msg);
                                }

                                tracing::info!(
                                    "✅ [WebViewInner::drop] Processed {} messages",
                                    processed_count
                                );

                                // Small delay to ensure window disappears
                                std::thread::sleep(std::time::Duration::from_millis(50));
                            } else {
                                tracing::warn!(
                                    "⚠️ [WebViewInner::drop] DestroyWindow failed: {:?}",
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
            tracing::info!("🔴 [WebViewInner::drop] Event loop dropped");
        }

        tracing::info!("✅ [WebViewInner::drop] Cleanup completed");
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        embedded::create_embedded(parent_hwnd, width, height, config, ipc_handler)
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

    /// Process pending window messages (for embedded mode)
    ///
    /// This method processes all pending Windows messages without blocking.
    /// It should be called periodically (e.g., from a Maya timer) to keep
    /// the window responsive in embedded mode.
    ///
    /// Returns true if the window should be closed, false otherwise.
    pub fn process_events(&self) -> bool {
        // CRITICAL: For embedded windows, check if window handle is still valid
        // This is the ONLY reliable way to detect when user clicks the X button
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            use std::ffi::c_void;
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::IsWindow;

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get();
                        let hwnd = HWND(hwnd_value as *mut c_void);

                        // Check if window handle is still valid
                        // When user clicks X button on embedded window, the handle becomes invalid
                        let is_valid = unsafe { IsWindow(hwnd).as_bool() };

                        // Use println! for direct output to Maya Script Editor
                        println!("🔍 [process_events] Checking window validity...");
                        println!("🔍 [process_events] HWND: {:?}", hwnd);
                        println!("🔍 [process_events] is_valid: {}", is_valid);

                        if !is_valid {
                            println!("{}", "=".repeat(80));
                            println!(
                                "🔴 [process_events] ⚠️ Window handle is INVALID - user closed window!"
                            );
                            println!("🔴 [process_events] HWND: {:?}", hwnd);
                            println!("🔴 [process_events] Returning true to Python...");
                            println!("{}", "=".repeat(80));
                            return true;
                        }
                    }
                }
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
            message_pump::process_messages_for_hwnd(hwnd_value)
        } else {
            message_pump::process_all_messages()
        };

        if should_quit {
            tracing::info!("{}", "=".repeat(80));
            tracing::info!("🟢 [process_events] should_quit = true");
            tracing::info!("🟢 [process_events] Window close signal detected!");
            tracing::info!("🟢 [process_events] Returning true to Python...");
            tracing::info!("{}", "=".repeat(80));
            return true;
        }

        // Process message queue
        if let Ok(webview) = self.webview.lock() {
            let count = self.message_queue.process_all(|message| {
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
                            "🟢 [process_events] Emitting event: {} with data: {}",
                            event_name,
                            data
                        );
                        let script = format!(
                            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
                            event_name, data
                        );
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to emit event: {}", e);
                        } else {
                            tracing::debug!("✅ [process_events] Event emitted successfully");
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
                tracing::debug!(
                    "🟢 [process_events] Processed {} messages from queue",
                    count
                );
            }
        }

        false
    }
}
