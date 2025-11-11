//! Improved event loop handling using ApplicationHandler pattern
//!
//! This module provides a better event loop implementation that:
//! - Uses a dedicated event handler structure
//! - Supports both blocking and non-blocking modes
//! - Properly manages window lifecycle
//! - Integrates better with Python's GIL

use std::sync::{Arc, Mutex};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::Window;
use wry::WebView as WryWebView;

use crate::ipc::{MessageQueue, WebViewMessage};

/// Custom user event for waking up the event loop
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// Wake up the event loop to process pending messages
    ProcessMessages,
    /// Request to close the window
    CloseWindow,
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

    /// Signal the event loop to exit
    pub fn request_exit(&self) {
        if let Ok(mut should_exit) = self.should_exit.lock() {
            *should_exit = true;
        }
    }

    /// Check if exit was requested
    pub fn should_exit(&self) -> bool {
        self.should_exit.lock().map(|flag| *flag).unwrap_or(false)
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

    /// Handle window events
    pub fn handle_window_event(&self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested");
                if let Ok(state) = self.state.lock() {
                    state.request_exit();
                }
            }
            WindowEvent::Resized(size) => {
                tracing::debug!("Window resized: {:?}", size);
                // Handle resize if needed
            }
            WindowEvent::Focused(focused) => {
                tracing::debug!("Window focus changed: {}", focused);
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
    pub fn run_blocking(mut event_loop: EventLoop<UserEvent>, state: Arc<Mutex<EventLoopState>>) {
        tracing::info!(
            "[WARNING] [run_blocking] Starting event loop (blocking mode with run_return)"
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

        // Show the window
        tracing::info!("[WARNING] [run_blocking] Making window visible...");
        if let Ok(state_guard) = state.lock() {
            if let Some(window) = &state_guard.window {
                window.set_visible(true);
                tracing::info!("[OK] [run_blocking] Window is now visible");
            } else {
                tracing::warn!("[WARNING] [run_blocking] Window is None");
            }
        } else {
            tracing::error!("[ERROR] [run_blocking] Failed to lock state for showing window");
        }

        let state_clone = state.clone();
        let exit_code = event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(UserEvent::ProcessMessages) => {
                    tracing::info!("[OK] [EventLoop] Received UserEvent::ProcessMessages - processing queue immediately");
                    // Process messages immediately when woken up
                    if let Ok(state_guard) = state_clone.lock() {
                        tracing::info!("[OK] [EventLoop] State lock acquired");
                        let queue_len = state_guard.message_queue.len();
                        tracing::info!("[OK] [EventLoop] Queue length: {}", queue_len);

                        let count = state_guard.message_queue.process_all(|message| {
                            tracing::info!("[OK] [EventLoop] Processing message: {:?}",
                                match &message {
                                    WebViewMessage::EvalJs(_) => "EvalJs",
                                    WebViewMessage::EmitEvent { event_name, .. } => event_name.as_str(),
                                    WebViewMessage::LoadUrl(_) => "LoadUrl",
                                    WebViewMessage::LoadHtml(_) => "LoadHtml",
                                }
                            );

                            if let Some(webview_arc) = &state_guard.webview {
                                tracing::info!("[OK] [EventLoop] WebView exists, locking...");
                                if let Ok(webview) = webview_arc.lock() {
                                    tracing::info!("[OK] [EventLoop] WebView locked, executing message...");
                                    match message {
                                        WebViewMessage::EvalJs(script) => {
                                            tracing::info!("[OK] [EventLoop] Executing EvalJs: {}", script);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("[ERROR] [EventLoop] Failed to execute JavaScript: {}", e);
                                            } else {
                                                tracing::info!("[OK] [EventLoop] EvalJs executed successfully");
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            tracing::info!("[OK] [EventLoop] Emitting event: {}", event_name);
                                            // Properly escape JSON data to avoid JavaScript syntax errors
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = format!(
                                                "window.dispatchEvent(new CustomEvent('{}', {{ detail: Object.assign({{}}, {{__aurora_from_python: true}}, JSON.parse('{}')) }}))",
                                                event_name, escaped_json
                                            );
                                            tracing::debug!("[CLOSE] [EventLoop] Generated script: {}", script);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("[ERROR] [EventLoop] Failed to emit event: {}", e);
                                            } else {
                                                tracing::info!("[OK] [EventLoop] Event emitted successfully");
                                            }
                                        }
                                        WebViewMessage::LoadUrl(url) => {
                                            tracing::info!("[OK] [EventLoop] Loading URL: {}", url);
                                            let script = format!("window.location.href = '{}';", url);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("[ERROR] [EventLoop] Failed to load URL: {}", e);
                                            } else {
                                                tracing::info!("[OK] [EventLoop] URL loaded successfully");
                                            }
                                        }
                                        WebViewMessage::LoadHtml(html) => {
                                            tracing::info!("[OK] [EventLoop] Loading HTML ({} bytes)", html.len());
                                            if let Err(e) = webview.load_html(&html) {
                                                tracing::error!("[ERROR] [EventLoop] Failed to load HTML: {}", e);
                                            } else {
                                                tracing::info!("[OK] [EventLoop] HTML loaded successfully");
                                            }
                                        }
                                    }
                                } else {
                                    tracing::error!("[ERROR] [EventLoop] Failed to lock WebView");
                                }
                            } else {
                                tracing::error!("[ERROR] [EventLoop] WebView is None!");
                            }
                        });

                        if count > 0 {
                            tracing::info!("[OK] [EventLoop] Processed {} messages immediately via UserEvent", count);
                        } else {
                            tracing::warn!("[WARNING] [EventLoop] No messages processed (queue was empty)");
                        }
                    } else {
                        tracing::error!("[ERROR] [EventLoop] Failed to lock state");
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
                    // Process pending messages from the queue
                    if let Ok(state_guard) = state_clone.lock() {
                        // Process all pending messages
                        let count = state_guard.message_queue.process_all(|message| {
                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    match message {
                                        WebViewMessage::EvalJs(script) => {
                                            tracing::debug!("Processing EvalJs: {}", script);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to execute JavaScript: {}", e);
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            tracing::debug!("Processing EmitEvent: {}", event_name);
                                            // Properly escape JSON data to avoid JavaScript syntax errors
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = format!(
                                                "window.dispatchEvent(new CustomEvent('{}', {{ detail: Object.assign({{}}, {{__aurora_from_python: true}}, JSON.parse('{}')) }}))",
                                                event_name, escaped_json
                                            );
                                            tracing::debug!("[CLOSE] [EmitEvent] Generated script: {}", script);
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to emit event: {}", e);
                                            }
                                        }
                                        WebViewMessage::LoadUrl(url) => {
                                            tracing::debug!("Processing LoadUrl: {}", url);
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
            *control_flow = ControlFlow::Poll;  // Non-blocking mode

            match event {
                Event::UserEvent(UserEvent::ProcessMessages) => {
                    tracing::debug!("[OK] [poll_events_once] Processing messages");
                    if let Ok(state_guard) = state_clone.lock() {
                        state_guard.message_queue.process_all(|message| {
                            if let Some(webview_arc) = &state_guard.webview {
                                if let Ok(webview) = webview_arc.lock() {
                                    match message {
                                        WebViewMessage::EvalJs(script) => {
                                            if let Err(e) = webview.evaluate_script(&script) {
                                                tracing::error!("Failed to execute JavaScript: {}", e);
                                            }
                                        }
                                        WebViewMessage::EmitEvent { event_name, data } => {
                                            // Properly escape JSON data to avoid JavaScript syntax errors
                                            let json_str = data.to_string();
                                            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                            let script = format!(
                                                "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                                                event_name, escaped_json
                                            );
                                            tracing::debug!("[CLOSE] [EmitEvent] Generated script: {}", script);
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
        }));

        let proxy = event_loop.create_proxy();
        state.lock().unwrap().set_event_loop_proxy(proxy.clone());

        // Send close request
        let _ = proxy.send_event(UserEvent::CloseWindow);
        let _ = WebViewEventHandler::poll_events_once(&mut event_loop, state.clone());
        assert!(state.lock().unwrap().should_exit());
    }
}
