//! Improved event loop handling using ApplicationHandler pattern
//!
//! This module provides a better event loop implementation that:
//! - Uses a dedicated event handler structure
//! - Supports both blocking and non-blocking modes
//! - Properly manages window lifecycle
//! - Integrates better with Python's GIL
//! - Emits window events to Python callbacks (inspired by Tauri/Electron)

use std::sync::{Arc, Mutex};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::Window;
use wry::WebView as WryWebView;

use crate::ipc::{MessageQueue, WebViewMessage};
use crate::webview::js_assets;

#[cfg(feature = "python-bindings")]
use crate::ipc::{IpcHandler, WindowEventType};

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
}

/// Event loop state management
pub struct EventLoopState {
    /// Whether the event loop should continue running
    pub should_exit: Arc<Mutex<bool>>,
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
            should_exit: Arc::new(Mutex::new(false)),
            window: Some(window),
            webview: Some(Arc::new(Mutex::new(webview))),
            message_queue,
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            is_visible: false,
            is_focused: false,
        }
    }

    /// Create a new event loop state without webview (for later initialization)
    pub fn new_without_webview(window: Window, message_queue: Arc<MessageQueue>) -> Self {
        Self {
            should_exit: Arc::new(Mutex::new(false)),
            window: Some(window),
            webview: None,
            message_queue,
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
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

    /// Set the IPC handler for Python callbacks
    #[cfg(feature = "python-bindings")]
    #[allow(dead_code)]
    pub fn set_ipc_handler(&mut self, handler: Arc<IpcHandler>) {
        self.ipc_handler = Some(handler);
    }

    /// Signal the event loop to exit
    pub fn request_exit(&self) {
        // Shutdown the message queue first to prevent background threads
        // from sending messages after the event loop is closed
        self.message_queue.shutdown();

        if let Ok(mut should_exit) = self.should_exit.lock() {
            *should_exit = true;
        }
    }

    /// Check if exit was requested
    pub fn should_exit(&self) -> bool {
        self.should_exit.lock().map(|flag| *flag).unwrap_or(false)
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
                    // Process messages immediately when woken up
                    if let Ok(state_guard) = state_clone.lock() {
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

                            // Handle Close message - request exit
                            if matches!(&message, WebViewMessage::Close) {
                                tracing::info!("[EventLoop] Close message received, requesting exit");
                                state_guard.request_exit();
                                if let Some(ref window) = state_guard.window {
                                    window.set_visible(false);
                                }
                                return; // Skip other processing for Close
                            }

                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    match &message {
                                        WebViewMessage::EvalJs(script) => {
                                            tracing::debug!("[EventLoop] Executing EvalJs");
                                            if let Err(e) = webview.evaluate_script(script) {
                                                tracing::error!("[EventLoop] Failed to execute JavaScript: {}", e);
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to emit event '{}': {}", event_name, e);
                                            }
                                        }
                                        WebViewMessage::LoadUrl(url) => {
                                            // Use native WebView2 navigation
                                            tracing::info!("[EventLoop] Loading URL via native API: {}", url);
                                            if let Err(e) = webview.load_url(url) {
                                                tracing::error!("Failed to load URL '{}': {}", url, e);
                                            }
                                        }
                                        WebViewMessage::LoadHtml(html) => {
                                            if let Err(e) = webview.load_html(html) {
                                                tracing::error!("Failed to load HTML: {}", e);
                                            }
                                        }
                                        WebViewMessage::SetVisible(visible) => {
                                            // SetVisible is handled at window level, not webview
                                            // This is a no-op here, handled separately
                                            tracing::debug!("SetVisible({}) received in webview handler (no-op)", visible);
                                        }
                                        WebViewMessage::WindowEvent { event_type, data } => {
                                            // Window events are emitted to JavaScript
                                            let event_name = event_type.as_str();
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to emit window event '{}': {}", event_name, e);
                                            }
                                        }
                                        WebViewMessage::EvalJsAsync { script, callback_id } => {
                                            // Execute JavaScript and send result back via IPC
                                            let async_script = js_assets::build_eval_js_async_script(script, *callback_id);
                                            if let Err(e) = webview.evaluate_script(&async_script) {
                                                tracing::error!("Failed to execute async JavaScript (id={}): {}", callback_id, e);
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
                                            // Close is handled above, this branch should not be reached
                                        }
                                    }
                                } else {
                                    tracing::error!("[EventLoop] Failed to lock WebView");
                                }
                            } else {
                                tracing::warn!("[EventLoop] WebView is None");
                            }

                            // Handle SetVisible at window level
                            if let WebViewMessage::SetVisible(visible) = &message {
                                if let Some(ref window) = state_guard.window {
                                    tracing::debug!("[EventLoop] Setting window visibility: {}", visible);
                                    window.set_visible(*visible);
                                }
                            }
                        });

                        if count > 0 {
                            tracing::debug!("[EventLoop] Processed {} messages via UserEvent", count);
                        }

                        // Check if Close message was processed and exit immediately
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
                                (None, Arc::new(Mutex::new(false)))
                            }
                        };

                        if let Some(hwnd) = hwnd_opt {
                            // Process only this window's messages (isolated from DCC main thread)
                            // CRITICAL: Check return value for close intent
                            let should_close = message_pump::process_messages_for_hwnd(hwnd);
                            if should_close {
                                tracing::info!("[EventLoop] Close detected from message pump, requesting exit");
                                // Set exit flag directly without holding state lock
                                if let Ok(mut flag) = should_exit_arc.lock() {
                                    *flag = true;
                                }
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
                        // Process all pending messages
                        let count = state_guard.message_queue.process_all(|message| {
                            // Handle Close message - request exit
                            if matches!(&message, WebViewMessage::Close) {
                                tracing::info!("[EventLoop] Close message received in MainEventsCleared, requesting exit");
                                state_guard.request_exit();
                                if let Some(ref window) = state_guard.window {
                                    window.set_visible(false);
                                }
                                return; // Skip other processing for Close
                            }

                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    match &message {
                                        WebViewMessage::EvalJs(script) => {
                                            tracing::debug!("Processing EvalJs: {}", script);
                                            if let Err(e) = webview.evaluate_script(script) {
                                                tracing::error!("Failed to execute JavaScript: {}", e);
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to emit event '{}': {}", event_name, e);
                                            }
                                        }
                                        WebViewMessage::LoadUrl(url) => {
                                            // Use native WebView2 navigation
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
                                            tracing::debug!("SetVisible({}) in webview handler (no-op)", visible);
                                        }
                                        WebViewMessage::WindowEvent { event_type, data } => {
                                            let event_name = event_type.as_str();
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to emit window event '{}': {}", event_name, e);
                                            }
                                        }
                                        WebViewMessage::EvalJsAsync { script, callback_id } => {
                                            // Execute JavaScript and send result back via IPC
                                            let async_script = js_assets::build_eval_js_async_script(script, *callback_id);
                                            if let Err(e) = webview.evaluate_script(&async_script) {
                                                tracing::error!("Failed to execute async JavaScript (id={}): {}", callback_id, e);
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
                                            // Close is handled above, this branch should not be reached
                                        }
                                    }
                                }

                                // Handle SetVisible at window level
                                if let WebViewMessage::SetVisible(visible) = &message {
                                    if let Some(ref window) = state_guard.window {
                                        tracing::info!("[EventLoop] Setting window visibility: {}", visible);
                                        window.set_visible(*visible);
                                    }
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
                        state_guard.message_queue.process_all(|message| {
                            // Handle Close message - request exit
                            if matches!(&message, WebViewMessage::Close) {
                                tracing::info!("[poll_events_once] Close message received, requesting exit");
                                state_guard.request_exit();
                                if let Some(ref window) = state_guard.window {
                                    window.set_visible(false);
                                }
                                return;
                            }

                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    match &message {
                                        WebViewMessage::EvalJs(script) => {
                                            if let Err(e) = webview.evaluate_script(script) {
                                                tracing::error!(
                                                    "Failed to execute JavaScript: {}",
                                                    e
                                                );
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            let json_str = data.to_string();
                                            let escaped_json =
                                                json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(
                                                event_name,
                                                &escaped_json,
                                            );
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!(
                                                    "Failed to emit event '{}': {}",
                                                    event_name,
                                                    e
                                                );
                                            }
                                        }
                                        WebViewMessage::LoadUrl(url) => {
                                            // Use native WebView2 navigation
                                            tracing::info!("[EventLoop] Loading URL via native API: {}", url);
                                            if let Err(e) = webview.load_url(url) {
                                                tracing::error!(
                                                    "Failed to load URL '{}': {}",
                                                    url,
                                                    e
                                                );
                                            }
                                        }
                                        WebViewMessage::LoadHtml(html) => {
                                            if let Err(e) = webview.load_html(html) {
                                                tracing::error!("Failed to load HTML: {}", e);
                                            }
                                        }
                                        WebViewMessage::SetVisible(visible) => {
                                            tracing::debug!("SetVisible({}) in webview handler (no-op)", visible);
                                        }
                                        WebViewMessage::WindowEvent { event_type, data } => {
                                            let event_name = event_type.as_str();
                                            let json_str = data.to_string();
                                            let escaped_json =
                                                json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = js_assets::build_emit_event_script(
                                                event_name,
                                                &escaped_json,
                                            );
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!(
                                                    "Failed to emit window event '{}': {}",
                                                    event_name,
                                                    e
                                                );
                                            }
                                        }
                                        WebViewMessage::EvalJsAsync { script, callback_id } => {
                                            // Execute JavaScript and send result back via IPC
                                            let async_script = js_assets::build_eval_js_async_script(script, *callback_id);
                                            if let Err(e) = webview.evaluate_script(&async_script) {
                                                tracing::error!("Failed to execute async JavaScript (id={}): {}", callback_id, e);
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
                                            // Close is handled above, this branch should not be reached
                                        }
                                    }
                                }
                            }

                            // Handle SetVisible at window level
                            if let WebViewMessage::SetVisible(visible) = &message {
                                if let Some(ref window) = state_guard.window {
                                    tracing::info!("[EventLoop] Setting window visibility: {}", visible);
                                    window.set_visible(*visible);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::MessageQueue;
    use std::sync::{Arc, Mutex};
    use tao::event::WindowEvent;

    #[test]
    fn test_event_loop_state_flags() {
        let state = EventLoopState {
            should_exit: Arc::new(Mutex::new(false)),
            window: None,
            webview: None,
            message_queue: Arc::new(MessageQueue::new()),
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            is_visible: false,
            is_focused: false,
        };
        assert!(!state.should_exit());
        state.request_exit();
        assert!(state.should_exit());
    }

    #[test]
    fn test_handler_close_requested_sets_exit() {
        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState {
            should_exit: Arc::new(Mutex::new(false)),
            window: None,
            webview: None,
            message_queue: Arc::new(MessageQueue::new()),
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            is_visible: false,
            is_focused: false,
        }));
        let handler = WebViewEventHandler::new(state.clone());
        handler.handle_window_event(WindowEvent::CloseRequested);
        let guard = state.lock().unwrap();
        assert!(guard.should_exit());
    }
}

#[cfg(test)]
mod poll_tests {
    use super::*;
    use crate::ipc::{MessageQueue, WebViewMessage};
    use std::sync::{Arc, Mutex};
    use tao::event_loop::{EventLoop, EventLoopBuilder};
    #[cfg(windows)]
    use tao::platform::windows::EventLoopBuilderExtWindows;

    #[test]
    fn test_poll_events_once_process_messages_user_event() {
        let mut event_loop: EventLoop<UserEvent> = {
            let mut b = EventLoopBuilder::<UserEvent>::with_user_event();
            #[cfg(windows)]
            {
                b.with_any_thread(true);
            }
            b.build()
        };
        let q = Arc::new(MessageQueue::new());
        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState {
            should_exit: Arc::new(Mutex::new(false)),
            window: None,
            webview: None,
            message_queue: q.clone(),
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            is_visible: false,
            is_focused: false,
        }));

        // Attach proxy and queue a message
        let proxy = event_loop.create_proxy();
        state.lock().unwrap().set_event_loop_proxy(proxy.clone());
        q.push(WebViewMessage::EvalJs("1+1".into()));

        // Trigger processing via user event
        let _ = proxy.send_event(UserEvent::ProcessMessages);
        let should_close = WebViewEventHandler::poll_events_once(&mut event_loop, state.clone());
        assert!(!should_close);
        assert!(q.is_empty());
    }

    #[test]
    fn test_poll_events_once_close_window_user_event() {
        let mut event_loop: EventLoop<UserEvent> = {
            let mut b = EventLoopBuilder::<UserEvent>::with_user_event();
            #[cfg(windows)]
            {
                b.with_any_thread(true);
            }
            b.build()
        };
        let q = Arc::new(MessageQueue::new());
        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState {
            should_exit: Arc::new(Mutex::new(false)),
            window: None,
            webview: None,
            message_queue: q.clone(),
            event_loop_proxy: None,
            #[cfg(feature = "python-bindings")]
            ipc_handler: None,
            is_visible: false,
            is_focused: false,
        }));

        let proxy = event_loop.create_proxy();
        state.lock().unwrap().set_event_loop_proxy(proxy.clone());

        // Send close request
        let _ = proxy.send_event(UserEvent::CloseWindow);
        let _ = WebViewEventHandler::poll_events_once(&mut event_loop, state.clone());
        assert!(state.lock().unwrap().should_exit());
    }
}
