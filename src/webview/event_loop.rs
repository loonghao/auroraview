//! Improved event loop handling using ApplicationHandler pattern
//!
//! This module provides a better event loop implementation that:
//! - Uses a dedicated event handler structure
//! - Supports both blocking and non-blocking modes
//! - Properly manages window lifecycle
//! - Integrates better with Python's GIL
//! - Emits window events to Python callbacks (inspired by Tauri/Electron)
//!
//! ## Thread Safety Optimizations
//!
//! - Uses `AtomicBool` for `should_exit` flag (lock-free)
//! - Uses `crossbeam-channel` for message queue (lock-free MPMC)
//! - Minimizes lock contention in hot paths
//!
//! ## Error Handling
//!
//! This module uses [`EventLoopError`] for structured error handling with
//! actionable information for debugging production issues.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::Window;
use thiserror::Error;
use wry::WebView as WryWebView;

use crate::ipc::{MessageQueue, WebViewMessage};
use crate::webview::js_assets;

#[cfg(feature = "python-bindings")]
use crate::ipc::{IpcHandler, WindowEventType};

/// Event loop errors with context for debugging
///
/// These errors provide structured information for diagnosing issues
/// in production environments. Each variant includes relevant context
/// to help identify the root cause.
#[derive(Debug, Error)]
pub enum EventLoopError {
    /// Failed to acquire WebView lock
    #[error("WebView lock failed: {context}")]
    WebViewLock {
        /// Description of what operation was attempted
        context: String,
    },

    /// JavaScript execution failed
    #[error("JavaScript execution failed: {script_preview}")]
    ScriptExecution {
        /// First 100 chars of the script for identification
        script_preview: String,
        /// The underlying error message
        message: String,
    },

    /// Message queue is full (backpressure)
    #[error("Message queue full: {queue_len} messages pending")]
    QueueFull {
        /// Current queue length
        queue_len: usize,
    },

    /// UTF-8 encoding error in message data
    #[error("Encoding error: {message}")]
    Encoding {
        /// Description of the encoding issue
        message: String,
    },

    /// Event loop proxy error (loop may be closed)
    #[error("Event loop closed or unavailable")]
    EventLoopClosed,

    /// Window operation failed
    #[error("Window operation failed: {operation}")]
    WindowOperation {
        /// The operation that failed
        operation: String,
    },

    /// Message processing timeout
    #[error("Message processing timed out after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },
}

/// Result type for event loop operations
pub type EventLoopResult<T> = Result<T, EventLoopError>;

impl EventLoopError {
    /// Create a WebView lock error with context
    pub fn webview_lock(context: impl Into<String>) -> Self {
        Self::WebViewLock {
            context: context.into(),
        }
    }

    /// Create a script execution error with preview
    pub fn script_execution(script: &str, message: impl Into<String>) -> Self {
        let preview = if script.len() > 100 {
            format!("{}...", &script[..100])
        } else {
            script.to_string()
        };
        Self::ScriptExecution {
            script_preview: preview,
            message: message.into(),
        }
    }

    /// Create an encoding error
    pub fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding {
            message: message.into(),
        }
    }

    /// Create a window operation error
    pub fn window_operation(operation: impl Into<String>) -> Self {
        Self::WindowOperation {
            operation: operation.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }
}

/// Custom user event for waking up the event loop
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// Wake up the event loop to process pending messages
    ProcessMessages,
    /// Request to close the window
    CloseWindow,
    /// Tray menu item clicked
    TrayMenuClick(String),
    /// Tray icon clicked (for show/hide window)
    TrayIconClick,
    /// Tray icon double-clicked
    TrayIconDoubleClick,
    /// Request to start native window drag
    DragWindow,
    /// Plugin event to be forwarded to WebView
    PluginEvent { event: String, data: String },
    /// Request to create a new child WebView window
    CreateChildWindow {
        url: String,
        width: u32,
        height: u32,
    },
}

/// Hints for applying Win32 window styles at the correct time.
///
/// On Windows 11, tao/wry may adjust window styles when the window becomes visible.
/// For frameless/transparent tool windows we keep a small set of hints so we can
/// re-apply the desired styles right after `set_visible(true)`.
#[derive(Debug, Clone, Copy)]
pub struct WindowStyleHints {
    #[cfg(target_os = "windows")]
    pub decorations: bool,
    #[cfg(target_os = "windows")]
    pub tool_window: bool,
    #[cfg(target_os = "windows")]
    pub undecorated_shadow: bool,
    #[cfg(target_os = "windows")]
    pub transparent: bool,
}

/// Event loop state management
///
/// ## Thread Safety
/// - `should_exit`: `AtomicBool` for lock-free exit signaling
/// - `webview`: `Arc<Mutex<>>` because WryWebView is not Send/Sync
/// - `message_queue`: Uses `crossbeam-channel` internally (lock-free MPMC)
pub struct EventLoopState {
    /// Whether the event loop should continue running (atomic for lock-free access)
    pub should_exit: Arc<AtomicBool>,
    /// Window reference
    pub window: Option<Window>,
    /// WebView reference for processing messages (wrapped in Arc<Mutex<>> for thread safety)
    pub webview: Option<Arc<Mutex<WryWebView>>>,
    /// Message queue for cross-thread communication
    pub message_queue: Arc<MessageQueue>,
    /// Event loop proxy for waking up the event loop
    pub event_loop_proxy: Option<EventLoopProxy<UserEvent>>,
    /// IPC handler for Python callbacks (window events)
    #[cfg(feature = "python-bindings")]
    pub ipc_handler: Option<Arc<IpcHandler>>,
    /// Optional window style hints for post-show re-application
    pub window_style_hints: Option<WindowStyleHints>,
    /// Track window visibility state for event deduplication
    is_visible: bool,
    /// Track window focus state for event deduplication
    is_focused: bool,
}

impl EventLoopState {
    /// Create a new event loop state
    #[allow(dead_code)]
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(window: Window, webview: WryWebView, message_queue: Arc<MessageQueue>) -> Self {
        Self {
            should_exit: Arc::new(AtomicBool::new(false)),
            window: Some(window),
            webview: Some(Arc::new(Mutex::new(webview))),
            message_queue,
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            window_style_hints: None,
            is_visible: false,
            is_focused: false,
        }
    }

    /// Create a new event loop state without webview (for later initialization)
    pub fn new_without_webview(window: Window, message_queue: Arc<MessageQueue>) -> Self {
        Self {
            should_exit: Arc::new(AtomicBool::new(false)),
            window: Some(window),
            webview: None,
            message_queue,
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            window_style_hints: None,
            is_visible: false,
            is_focused: false,
        }
    }

    /// Set the webview reference
    pub fn set_webview(&mut self, webview: Arc<Mutex<WryWebView>>) {
        self.webview = Some(webview);
    }

    /// Set the event loop proxy
    pub fn set_event_loop_proxy(&mut self, proxy: EventLoopProxy<UserEvent>) {
        self.event_loop_proxy = Some(proxy);
    }

    /// Set window style hints for post-show re-application.
    #[allow(dead_code)]
    pub fn set_window_style_hints(&mut self, hints: Option<WindowStyleHints>) {
        self.window_style_hints = hints;
    }

    /// Set the IPC handler for Python callbacks
    #[cfg(feature = "python-bindings")]
    #[allow(dead_code)]
    pub fn set_ipc_handler(&mut self, handler: Arc<IpcHandler>) {
        self.ipc_handler = Some(handler);
    }

    /// Signal the event loop to exit
    ///
    /// Uses ipckit's graceful shutdown mechanism to wait for pending operations.
    /// This method is lock-free for the exit flag (uses AtomicBool).
    pub fn request_exit(&self) {
        // Shutdown the message queue first to prevent background threads
        // from sending messages after the event loop is closed
        self.message_queue.shutdown();

        // Wait for pending operations to complete (with short timeout)
        // This uses ipckit's wait_for_drain mechanism
        if let Err(e) = self
            .message_queue
            .wait_for_drain(Some(std::time::Duration::from_millis(500)))
        {
            tracing::warn!("[EventLoopState] Drain timeout during exit: {}", e);
        }

        // Set exit flag atomically (lock-free)
        self.should_exit.store(true, Ordering::SeqCst);
    }

    /// Check if exit was requested (lock-free)
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::SeqCst)
    }

    /// Emit a window event to Python callbacks
    #[cfg(feature = "python-bindings")]
    pub fn emit_window_event(&self, event_type: WindowEventType, data: serde_json::Value) {
        if let Some(handler) = &self.ipc_handler {
            let event_name = event_type.as_str();
            tracing::debug!(
                "Emitting window event: {} with data: {:?}",
                event_name,
                data
            );

            // Call Python callbacks for this event
            let message = crate::ipc::IpcMessage {
                event: event_name.to_string(),
                data: data.clone(),
                id: None,
            };

            if let Err(e) = handler.handle_message(message) {
                // No handler registered is OK - just means no Python callback was set
                if !e.contains("No handler registered") {
                    tracing::warn!("Window event callback error: {}", e);
                }
            }

            // Also emit to JavaScript via message queue
            self.message_queue.push(WebViewMessage::EmitEvent {
                event_name: event_name.to_string(),
                data,
            });
        }
    }

    /// Update visibility state and emit event if changed
    pub fn set_visible(&mut self, visible: bool) {
        if self.is_visible != visible {
            self.is_visible = visible;
            #[cfg(feature = "python-bindings")]
            {
                let event_type = if visible {
                    WindowEventType::Shown
                } else {
                    WindowEventType::Hidden
                };
                self.emit_window_event(event_type, serde_json::json!({}));
            }
        }
    }

    /// Update focus state and emit event if changed
    pub fn set_focused(&mut self, focused: bool) {
        if self.is_focused != focused {
            self.is_focused = focused;
            #[cfg(feature = "python-bindings")]
            {
                let event_type = if focused {
                    WindowEventType::Focused
                } else {
                    WindowEventType::Blurred
                };
                self.emit_window_event(event_type, serde_json::json!({}));
            }
        }
    }

    /// Get the window HWND (Windows only)
    ///
    /// Returns the native window handle for targeted message processing.
    /// This is used to isolate message pump processing to only this window's messages.
    #[cfg(target_os = "windows")]
    pub fn get_hwnd(&self) -> Option<u64> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if let Some(window) = &self.window {
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    return Some(handle.hwnd.get() as u64);
                }
            }
        }
        None
    }
}

/// Improved event loop handler
pub struct WebViewEventHandler {
    state: Arc<Mutex<EventLoopState>>,
}

/// Process a single WebView message
///
/// This is extracted as a separate function to avoid code duplication
/// across different event handlers (UserEvent::ProcessMessages, MainEventsCleared, poll_events_once).
fn process_webview_message(
    message: &WebViewMessage,
    webview: &WryWebView,
    window: Option<&Window>,
    state: &EventLoopState,
) {
    match message {
        WebViewMessage::EvalJs(script) => {
            tracing::debug!("[EventLoop] Executing EvalJs");
            if let Err(e) = webview.evaluate_script(script) {
                tracing::error!("[EventLoop] Failed to execute JavaScript: {}", e);
            }
        }
        WebViewMessage::EmitEvent { event_name, data } => {
            // JSON is already valid JavaScript literal
            let json_str = data.to_string();
            let script = js_assets::build_emit_event_script(event_name, &json_str);
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("Failed to emit event '{}': {}", event_name, e);
            }
        }
        WebViewMessage::LoadUrl(url) => {
            tracing::info!("[EventLoop] Loading URL via native API: {}", url);
            if let Err(e) = webview.load_url(url) {
                tracing::error!("Failed to load URL '{}': {}", url, e);
            }
        }
        WebViewMessage::LoadHtml(html) => {
            tracing::debug!("Processing LoadHtml ({} bytes)", html.len());
            if let Err(e) = webview.load_html(html) {
                tracing::error!("Failed to load HTML: {}", e);
            }
        }
        WebViewMessage::SetVisible(visible) => {
            // Handle at window level
            if let Some(win) = window {
                tracing::debug!("[EventLoop] Setting window visibility: {}", visible);
                win.set_visible(*visible);
            }
        }
        WebViewMessage::WindowEvent { event_type, data } => {
            let event_name = event_type.as_str();
            let json_str = data.to_string();
            let script = js_assets::build_emit_event_script(event_name, &json_str);
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("Failed to emit window event '{}': {}", event_name, e);
            }
        }
        WebViewMessage::EvalJsAsync { script, callback_id } => {
            let async_script = js_assets::build_eval_js_async_script(script, *callback_id);
            if let Err(e) = webview.evaluate_script(&async_script) {
                tracing::error!(
                    "Failed to execute async JavaScript (id={}): {}",
                    callback_id,
                    e
                );
            }
        }
        WebViewMessage::Reload => {
            if let Err(e) = webview.evaluate_script("location.reload()") {
                tracing::error!("Failed to reload: {}", e);
            }
        }
        WebViewMessage::StopLoading => {
            if let Err(e) = webview.evaluate_script("window.stop()") {
                tracing::error!("Failed to stop loading: {}", e);
            }
        }
        WebViewMessage::Close => {
            tracing::info!("[EventLoop] Close message received, requesting exit");
            state.request_exit();
            if let Some(win) = window {
                win.set_visible(false);
            }
        }
    }
}

impl WebViewEventHandler {
    /// Create a new event handler
    pub fn new(state: Arc<Mutex<EventLoopState>>) -> Self {
        Self { state }
    }

    /// Handle window events and emit to Python callbacks
    pub fn handle_window_event(&self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested - emitting closing event");
                if let Ok(state) = self.state.lock() {
                    // Emit closing event before requesting exit
                    #[cfg(feature = "python-bindings")]
                    state.emit_window_event(WindowEventType::Closing, serde_json::json!({}));

                    state.request_exit();

                    // Emit closed event
                    #[cfg(feature = "python-bindings")]
                    state.emit_window_event(WindowEventType::Closed, serde_json::json!({}));
                }
            }
            WindowEvent::Resized(size) => {
                tracing::debug!("Window resized: {:?}", size);
                #[cfg(feature = "python-bindings")]
                if let Ok(state) = self.state.lock() {
                    state.emit_window_event(
                        WindowEventType::Resized,
                        serde_json::json!({
                            "width": size.width,
                            "height": size.height
                        }),
                    );
                }
            }
            WindowEvent::Moved(position) => {
                tracing::debug!("Window moved: {:?}", position);
                #[cfg(feature = "python-bindings")]
                if let Ok(state) = self.state.lock() {
                    state.emit_window_event(
                        WindowEventType::Moved,
                        serde_json::json!({
                            "x": position.x,
                            "y": position.y
                        }),
                    );
                }
            }
            WindowEvent::Focused(focused) => {
                tracing::debug!("Window focus changed: {}", focused);
                if let Ok(mut state) = self.state.lock() {
                    state.set_focused(focused);
                }
            }
            _ => {}
        }
    }

    /// Run the event loop (blocking)
    ///
    /// CRITICAL: Uses run_return() instead of run() to prevent process exit.
    /// The run() method calls std::process::exit() when the event loop exits,
    /// which would terminate the entire DCC application (Maya, Houdini, etc.).
    /// The run_return() method returns normally, allowing the DCC to continue running.
    ///
    /// # Arguments
    /// * `event_loop` - The event loop to run
    /// * `state` - The event loop state
    /// * `auto_show` - Whether to automatically show the window
    pub fn run_blocking(
        mut event_loop: EventLoop<UserEvent>,
        state: Arc<Mutex<EventLoopState>>,
        auto_show: bool,
    ) {
        tracing::info!(
            "[WARNING] [run_blocking] Starting event loop (blocking mode with run_return, auto_show={})",
            auto_show
        );

        // Create event loop proxy and store it in state
        tracing::info!("[WARNING] [run_blocking] Creating event loop proxy...");
        let proxy = event_loop.create_proxy();

        tracing::info!("[WARNING] [run_blocking] Storing proxy in EventLoopState...");
        if let Ok(mut state_guard) = state.lock() {
            state_guard.set_event_loop_proxy(proxy.clone());
            tracing::info!("[OK] [run_blocking] Event loop proxy stored in EventLoopState");
        } else {
            tracing::error!("[ERROR] [run_blocking] Failed to lock state for storing proxy");
        }

        // Also store proxy in message queue for immediate wake-up
        tracing::info!("[WARNING] [run_blocking] Storing proxy in MessageQueue...");
        if let Ok(state_guard) = state.lock() {
            state_guard.message_queue.set_event_loop_proxy(proxy);
            tracing::info!("[OK] [run_blocking] Event loop proxy stored in MessageQueue");
        } else {
            tracing::error!(
                "[ERROR] [run_blocking] Failed to lock state for storing proxy in MessageQueue"
            );
        }

        // Only show window if auto_show is true
        if auto_show {
            tracing::info!("[WARNING] [run_blocking] Making window visible (auto_show=true)...");
            if let Ok(mut state_guard) = state.lock() {
                // First set visible to emit the event
                state_guard.set_visible(true);

                if let Some(window) = &state_guard.window {
                    window.set_visible(true);
                    tracing::info!("[OK] [run_blocking] Window is now visible");

                    // Re-apply Win32 window styles right after showing the window.
                    // On Windows 11, tao/wry may adjust styles at show-time.
                    #[cfg(target_os = "windows")]
                    {
                        if let (Some(hints), Some(hwnd)) =
                            (state_guard.window_style_hints, state_guard.get_hwnd())
                        {
                            use auroraview_core::builder::{
                                apply_frameless_popup_window_style, apply_tool_window_style,
                                disable_window_shadow, extend_frame_into_client_area,
                                optimize_transparent_window_resize, remove_clip_children_style,
                            };

                            if !hints.decorations {
                                let _ = apply_frameless_popup_window_style(hwnd as isize);
                            }

                            if hints.tool_window {
                                apply_tool_window_style(hwnd as isize);
                            }
                            if !hints.undecorated_shadow {
                                disable_window_shadow(hwnd as isize);
                            }
                            if hints.transparent {
                                remove_clip_children_style(hwnd as isize);
                                extend_frame_into_client_area(hwnd as isize);
                                optimize_transparent_window_resize(hwnd as isize);
                            }
                        }
                    }

                    // CRITICAL FIX: Request a redraw to wake up the event loop

                    // Without this, run_return() may hang on Windows waiting for the first event
                    window.request_redraw();
                    tracing::info!(
                        "[OK] [run_blocking] Requested window redraw to wake event loop"
                    );
                } else {
                    tracing::warn!("[WARNING] [run_blocking] Window is None");
                }
            } else {
                tracing::error!("[ERROR] [run_blocking] Failed to lock state for showing window");
            }
        } else {
            tracing::info!("[OK] [run_blocking] Window stays hidden (auto_show=false)");
            // Still request redraw to wake event loop
            if let Ok(state_guard) = state.lock() {
                if let Some(window) = &state_guard.window {
                    window.request_redraw();
                    tracing::info!(
                        "[OK] [run_blocking] Requested window redraw to wake event loop"
                    );
                }
            }
        }

        let state_clone = state.clone();
        let exit_code = event_loop.run_return(move |event, _, control_flow| {
            // CRITICAL: Use Poll mode for WebView2 compatibility on Windows
            // WebView2 requires continuous message pump processing for:
            // - COM message handling
            // - IPC message processing
            // - JavaScript execution results
            // - Rendering updates
            // Poll mode ensures the Windows message loop runs continuously
            *control_flow = ControlFlow::Poll;

            match event {
                Event::UserEvent(UserEvent::ProcessMessages) => {
                    tracing::debug!("[EventLoop] Processing UserEvent::ProcessMessages");
                    if let Ok(state_guard) = state_clone.lock() {
                        let window_ref = state_guard.window.as_ref();
                        let count = state_guard.message_queue.process_all(|message| {
                            tracing::debug!("[EventLoop] Processing message: {:?}",
                                match &message {
                                    WebViewMessage::EvalJs(_) => "EvalJs",
                                    WebViewMessage::EvalJsAsync { .. } => "EvalJsAsync",
                                    WebViewMessage::EmitEvent { event_name, .. } => event_name.as_str(),
                                    WebViewMessage::LoadUrl(_) => "LoadUrl",
                                    WebViewMessage::LoadHtml(_) => "LoadHtml",
                                    WebViewMessage::SetVisible(v) => if *v { "SetVisible(true)" } else { "SetVisible(false)" },
                                    WebViewMessage::Reload => "Reload",
                                    WebViewMessage::StopLoading => "StopLoading",
                                    WebViewMessage::WindowEvent { event_type, .. } => event_type.as_str(),
                                    WebViewMessage::Close => "Close",
                                }
                            );

                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    process_webview_message(&message, &webview, window_ref, &state_guard);
                                } else {
                                    tracing::error!("[EventLoop] Failed to lock WebView");
                                }
                            } else {
                                tracing::warn!("[EventLoop] WebView is None");
                            }
                        });

                        if count > 0 {
                            tracing::debug!("[EventLoop] Processed {} messages via UserEvent", count);
                        }

                        if state_guard.should_exit() {
                            tracing::info!("[EventLoop] Exit requested after processing messages, exiting");
                            *control_flow = ControlFlow::Exit;
                        }
                    } else {
                        tracing::error!("[EventLoop] Failed to lock state");
                    }
                }
                Event::UserEvent(UserEvent::CloseWindow) => {
                    tracing::info!("[CLOSE] [EventLoop] UserEvent::CloseWindow received");
                    tracing::info!("[CLOSE] [EventLoop] Requesting window close via event loop");

                    // Request exit through the state
                    if let Ok(state_guard) = state_clone.lock() {
                        state_guard.request_exit();

                        // Hide window immediately
                        if let Some(window) = &state_guard.window {
                            window.set_visible(false);
                            tracing::info!("[OK] [EventLoop] Window hidden");
                        }

                        // Set control flow to exit
                        *control_flow = ControlFlow::Exit;
                        tracing::info!("[OK] [EventLoop] Control flow set to Exit");
                    } else {
                        tracing::error!("[ERROR] [EventLoop] Failed to lock state for close");
                    }
                }
                Event::UserEvent(UserEvent::DragWindow) => {
                    tracing::debug!("[EventLoop] UserEvent::DragWindow received");
                    if let Ok(state_guard) = state_clone.lock() {
                        if let Some(window) = &state_guard.window {
                            if let Err(e) = window.drag_window() {
                                tracing::warn!("[EventLoop] Failed to start window drag: {:?}", e);
                            } else {
                                tracing::debug!("[EventLoop] Window drag started");
                            }
                        }
                    }
                }
                Event::UserEvent(UserEvent::PluginEvent { event, data }) => {
                    tracing::debug!("[EventLoop] UserEvent::PluginEvent received: {}", event);
                    if let Ok(state_guard) = state_clone.lock() {
                        if let Some(webview_arc) = &state_guard.webview {
                            if let Ok(webview) = webview_arc.lock() {
                                // data is already a JSON string, no escaping needed
                                let script = js_assets::build_emit_event_script(&event, &data);
                                if let Err(e) = webview.evaluate_script(&script) {
                                    tracing::error!("[EventLoop] Failed to emit plugin event '{}': {}", event, e);
                                }
                            }
                        }
                    }
                }
                Event::UserEvent(UserEvent::CreateChildWindow { url, width, height }) => {
                    tracing::info!("[EventLoop] UserEvent::CreateChildWindow received: {}", url);
                    #[cfg(target_os = "windows")]
                    {
                        // Create child window on the main thread
                        if let Err(e) = super::child_window::create_child_webview_window(&url, width, height) {
                            tracing::error!("[EventLoop] Failed to create child window: {}", e);
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        // Suppress unused variable warnings on non-Windows platforms
                        let _ = (width, height);
                        // On non-Windows platforms, fall back to system browser
                        tracing::warn!("[EventLoop] Child window not supported on this platform, opening in browser");
                        if let Err(e) = open::that(&url) {
                            tracing::error!("[EventLoop] Failed to open URL in browser: {}", e);
                        }
                    }
                }
                Event::UserEvent(_) => {
                    // Handle other user events (TrayMenuClick, TrayIconClick, etc.) - currently no-op
                }
                Event::WindowEvent { event, .. } => {
                    tracing::debug!("Window event: {:?}", event);
                    let handler = WebViewEventHandler::new(state_clone.clone());
                    handler.handle_window_event(event);

                    // Check if we should exit after handling the event
                    if let Ok(state_guard) = state_clone.lock() {
                        if state_guard.should_exit() {
                            tracing::info!("Window close requested, hiding window and exiting event loop");
                            // Hide the window before exiting to prevent visual artifacts
                            if let Some(window) = &state_guard.window {
                                window.set_visible(false);
                                tracing::info!("Window hidden");
                            }
                            *control_flow = ControlFlow::Exit;
                            tracing::info!("Control flow set to Exit");
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // CRITICAL: Explicitly pump Windows messages for WebView2
                    // tao's run_return() may not process all Windows messages that
                    // WebView2 needs for proper rendering and COM message handling.
                    // This ensures the WebView2 message pump stays active.
                    //
                    // OPTIMIZATION: Only process messages for the WebView window (not all thread messages)
                    // This prevents interfering with DCC host application's message pump.
                    #[cfg(target_os = "windows")]
                    {
                        use super::message_pump;

                        // Get the window HWND for targeted message processing
                        // CRITICAL: Release the lock BEFORE calling process_messages_for_hwnd
                        // because DestroyWindow may trigger WM_DESTROY which could cause deadlock
                        let (hwnd_opt, should_exit_arc) = {
                            if let Ok(state_guard) = state_clone.lock() {
                                (state_guard.get_hwnd(), state_guard.should_exit.clone())
                            } else {
                                (None, Arc::new(AtomicBool::new(false)))
                            }
                        };

                        if let Some(hwnd) = hwnd_opt {
                            // Process only this window's messages (isolated from DCC main thread)
                            // CRITICAL: Check return value for close intent
                            let should_close = message_pump::process_messages_for_hwnd(hwnd);
                            if should_close {
                                tracing::info!("[EventLoop] Close detected from message pump, requesting exit");
                                // Set exit flag directly using AtomicBool (lock-free)
                                should_exit_arc.store(true, Ordering::SeqCst);
                            }
                        } else {
                            // Fallback: If HWND not available, use limited global processing
                            // This should rarely happen (only during initialization)
                            tracing::trace!("[event_loop] HWND not available, using fallback message pump");
                            let _ = message_pump::process_all_messages_limited(100);
                        }
                    }

                    // Process pending messages from the queue
                    if let Ok(state_guard) = state_clone.lock() {
                        let window_ref = state_guard.window.as_ref();
                        let count = state_guard.message_queue.process_all(|message| {
                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    process_webview_message(&message, &webview, window_ref, &state_guard);
                                }
                            }
                        });

                        if count > 0 {
                            tracing::debug!("Processed {} messages in MainEventsCleared", count);
                        }

                        // Check if we should exit
                        if state_guard.should_exit() {
                            tracing::info!("Exit requested in MainEventsCleared, exiting event loop gracefully");
                            *control_flow = ControlFlow::Exit;
                        }
                    }

                    // In Poll mode, add a small sleep to reduce CPU usage
                    // This is a workaround for WebView2 compatibility - we need Poll mode
                    // for WebView2 message pump, but pure polling uses 100% CPU
                    // 2ms sleep gives us ~500 iterations/second which is plenty for UI
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Event::LoopDestroyed => {
                    tracing::info!("Event loop destroyed");
                }
                _ => {}
            }
        });

        tracing::info!("Event loop exited with code: {}", exit_code);
    }

    /// Process events once (non-blocking) for embedded mode
    ///
    /// This method processes pending window events without blocking.
    /// It should be called periodically (e.g., from a timer) to keep the window responsive.
    ///
    /// Returns true if the window should be closed, false otherwise.
    #[allow(dead_code)]
    pub fn poll_events_once(
        event_loop: &mut EventLoop<UserEvent>,
        state: Arc<Mutex<EventLoopState>>,
    ) -> bool {
        use tao::event_loop::ControlFlow;

        let should_close = false;
        let state_clone = state.clone();

        // Process events with ControlFlow::Poll (non-blocking)
        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll; // Non-blocking mode

            match event {
                Event::UserEvent(UserEvent::ProcessMessages) => {
                    tracing::debug!("[OK] [poll_events_once] Processing messages");
                    if let Ok(state_guard) = state_clone.lock() {
                        let window_ref = state_guard.window.as_ref();
                        state_guard.message_queue.process_all(|message| {
                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    process_webview_message(&message, &webview, window_ref, &state_guard);
                                }
                            }
                        });
                    }
                }
                Event::WindowEvent { event, .. } => {
                    tracing::debug!("[OK] [poll_events_once] Window event: {:?}", event);
                    let handler = WebViewEventHandler::new(state_clone.clone());
                    handler.handle_window_event(event);

                    // Check if we should exit
                    if let Ok(state_guard) = state_clone.lock() {
                        if state_guard.should_exit() {
                            tracing::info!("[OK] [poll_events_once] Window close requested");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                Event::UserEvent(UserEvent::CloseWindow) => {
                    tracing::info!("[OK] [poll_events_once] CloseWindow user event received");
                    if let Ok(state_guard) = state_clone.lock() {
                        state_guard.request_exit();
                    }
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => {
                    // Exit immediately after processing all events (non-blocking)
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });

        should_close
    }
}
