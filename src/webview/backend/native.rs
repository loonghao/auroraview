//! Native backend - WebView embedded using platform-specific APIs
//!
//! This backend uses native window parenting (HWND on Windows) to embed
//! the WebView into existing DCC application windows.

#[allow(unused_imports)]
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use tao::event_loop::EventLoopBuilder;
#[allow(unused_imports)]
use tao::window::WindowBuilder;
use wry::WebView as WryWebView;
use wry::WebViewBuilder as WryWebViewBuilder;

use super::WebViewBackend;
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};
use crate::webview::config::WebViewConfig;
use crate::webview::event_loop::UserEvent;
use crate::webview::message_pump;

/// Native backend implementation
///
/// This backend creates a WebView that can be embedded into existing windows
/// using platform-specific APIs (e.g., Windows HWND parenting).
#[allow(dead_code)]
pub struct NativeBackend {
    webview: Arc<Mutex<WryWebView>>,
    window: Option<tao::window::Window>,
    event_loop: Option<tao::event_loop::EventLoop<UserEvent>>,
    message_queue: Arc<MessageQueue>,
}

impl WebViewBackend for NativeBackend {
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Determine if this is embedded or standalone mode
        if let Some(parent_hwnd) = config.parent_hwnd {
            Self::create_embedded(parent_hwnd, config, ipc_handler, message_queue)
        } else {
            Self::create_standalone(config, ipc_handler, message_queue)
        }
    }

    fn webview(&self) -> Arc<Mutex<WryWebView>> {
        self.webview.clone()
    }

    fn message_queue(&self) -> Arc<MessageQueue> {
        self.message_queue.clone()
    }

    fn window(&self) -> Option<&tao::window::Window> {
        self.window.as_ref()
    }

    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>> {
        self.event_loop.take()
    }

    fn process_events(&self) -> bool {
        // Check if window handle is still valid (for embedded mode)
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

                        let is_valid = unsafe { IsWindow(hwnd).as_bool() };

                        if !is_valid {
                            tracing::info!("[CLOSE] [NativeBackend::process_events] Window handle invalid - user closed window");
                            return true;
                        }
                    }
                }
            }
        }

        // Get window HWND for targeted message processing
        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        Some(handle.hwnd.get() as u64)
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

        // Process Windows messages
        let should_quit = if let Some(hwnd_value) = hwnd {
            message_pump::process_messages_for_hwnd(hwnd_value)
        } else {
            message_pump::process_all_messages()
        };

        if should_quit {
            tracing::info!("[CLOSE] [NativeBackend::process_events] Window close signal detected");
            return true;
        }

        // Process message queue
        if let Ok(webview) = self.webview.lock() {
            let count = self.message_queue.process_all(|message| {
                use crate::ipc::WebViewMessage;
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
                        tracing::debug!("[CLOSE] [NativeBackend] Generated script: {}", script);
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

            if count > 0 {
                tracing::debug!(
                    "[OK] [NativeBackend::process_events] Processed {} messages",
                    count
                );
            }
        }

        false
    }

    fn run_event_loop_blocking(&mut self) {
        use crate::webview::event_loop::{EventLoopState, WebViewEventHandler};

        tracing::info!("[OK] [NativeBackend::run_event_loop_blocking] Starting event loop");

        if self.window.is_none() || self.event_loop.is_none() {
            tracing::error!("Window or event loop is None!");
            return;
        }

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

        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState::new_without_webview(
            window,
            self.message_queue.clone(),
        )));

        if let Ok(mut state_guard) = state.lock() {
            state_guard.set_webview(self.webview.clone());
        }

        WebViewEventHandler::run_blocking(event_loop, state);
        tracing::info!("Event loop exited");
    }
}

impl NativeBackend {
    /// Create standalone WebView with its own window
    #[allow(dead_code)]
    fn create_standalone(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Delegate to standalone module for now
        // We need to use the existing standalone implementation
        // and convert it to NativeBackend structure
        let mut inner = crate::webview::standalone::create_standalone(
            config,
            ipc_handler,
            message_queue.clone(),
        )?;

        // Extract fields from WebViewInner
        // We can safely take these because we own the WebViewInner
        let webview = inner.webview.clone();
        let window = inner.window.take();
        let event_loop = inner.event_loop.take();

        Ok(Self {
            webview,
            window,
            event_loop,
            message_queue,
        })
    }

    /// Create embedded WebView for DCC integration
    #[cfg(target_os = "windows")]
    fn create_embedded(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::webview::config::EmbedMode;
        use tao::platform::windows::WindowBuilderExtWindows;

        tracing::info!(
            "[OK] [NativeBackend::create_embedded] Creating embedded WebView (parent_hwnd: {}, mode: {:?})",
            parent_hwnd,
            config.embed_mode
        );

        // Create event loop
        let event_loop = {
            use tao::platform::windows::EventLoopBuilderExtWindows;
            EventLoopBuilder::<UserEvent>::with_user_event()
                .with_any_thread(true)
                .build()
        };

        // Create window builder
        let mut window_builder = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
            .with_resizable(config.resizable)
            .with_decorations(config.decorations)
            .with_always_on_top(config.always_on_top)
            .with_transparent(config.transparent);

        // Set parent window based on embed mode
        match config.embed_mode {
            EmbedMode::Child => {
                tracing::info!("[OK] [NativeBackend] Using Child mode (WS_CHILD)");
                window_builder = window_builder.with_parent_window(parent_hwnd as isize);
            }
            EmbedMode::Owner => {
                tracing::info!("[OK] [NativeBackend] Using Owner mode (GWLP_HWNDPARENT)");
                window_builder = window_builder.with_owner_window(parent_hwnd as isize);
            }
            EmbedMode::None => {
                tracing::warn!(
                    "[WARNING] [NativeBackend] EmbedMode::None - creating standalone window"
                );
            }
        }

        // Build window
        let window = window_builder
            .build(&event_loop)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Log window HWND
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    let hwnd_value = handle.hwnd.get();
                    tracing::info!(
                        "[OK] [NativeBackend] Window created: HWND 0x{:X}",
                        hwnd_value
                    );
                }
            }
        }

        // Make window visible
        window.set_visible(true);

        // Create WebView with IPC handler
        let webview = Self::create_webview(&window, &config, ipc_handler)?;

        #[allow(clippy::arc_with_non_send_sync)]
        Ok(Self {
            webview: Arc::new(Mutex::new(webview)),
            window: Some(window),
            event_loop: Some(event_loop),
            message_queue,
        })
    }

    /// Create embedded WebView for non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    fn create_embedded(
        _parent_hwnd: u64,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
        _message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Embedded mode is only supported on Windows".into())
    }

    /// Create WebView instance with IPC handler
    #[allow(dead_code)]
    fn create_webview(
        window: &tao::window::Window,
        config: &WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
    ) -> Result<WryWebView, Box<dyn std::error::Error>> {
        let mut builder = WryWebViewBuilder::new();

        // Enable developer tools if configured
        if config.dev_tools {
            tracing::info!("[OK] [NativeBackend] Enabling developer tools");
            builder = builder.with_devtools(true);
        }

        // Add event bridge script with full window.auroraview API
        let event_bridge_script = r#"
    (function() {
        console.log('Initializing AuroraView event bridge...');

        // Event handlers registry for Python -> JS communication
        const eventHandlers = new Map();

        // Create low-level window.auroraview API
        window.auroraview = {
            // Send event to Python (JS -> Python)
            send_event: function(eventName, data) {
                console.log('[AuroraView] Sending event to Python:', eventName, data);
                try {
                    window.ipc.postMessage(JSON.stringify({
                        type: 'event',
                        event: eventName,
                        detail: data || {}
                    }));
                } catch (e) {
                    console.error('[AuroraView] Failed to send event via IPC:', e);
                }
            },

            // Register event handler for Python -> JS communication
            on: function(eventName, callback) {
                console.log('[AuroraView] Registering handler for event:', eventName);
                if (!eventHandlers.has(eventName)) {
                    eventHandlers.set(eventName, []);
                }
                eventHandlers.get(eventName).push(callback);
            }
        };

        // Create high-level AuroraView helper class (Qt-style API)
        window.AuroraView = class {
            constructor() {
                this.ready = true; // Always ready since we're in init script
                console.log('[AuroraView] Helper class initialized');
            }

            // Qt-style emit (JavaScript -> Python)
            emit(signal, data = {}) {
                window.auroraview.send_event(signal, data);
                return this;
            }

            // Qt-style connect (Python -> JavaScript)
            on(signal, slot) {
                if (typeof slot !== 'function') {
                    console.error('[AuroraView] Slot must be a function');
                    return this;
                }
                window.auroraview.on(signal, slot);
                return this;
            }

            // Alias for consistency
            connect(signal, slot) {
                return this.on(signal, slot);
            }

            // Check if ready (always true in init script)
            isReady() {
                return this.ready;
            }
        };

        // Create default instance for convenience
        window.aurora = new window.AuroraView();

        // Listen for events from Python
        window.addEventListener('message', function(event) {
            try {
                const message = JSON.parse(event.data);
                if (message.type === 'python_event') {
                    const eventName = message.event;
                    const data = message.detail || {};
                    console.log('[AuroraView] Received event from Python:', eventName, data);

                    const handlers = eventHandlers.get(eventName);
                    if (handlers) {
                        handlers.forEach(handler => {
                            try {
                                handler(data);
                            } catch (e) {
                                console.error('[AuroraView] Error in event handler:', e);
                            }
                        });
                    }
                }
            } catch (e) {
                console.error('[AuroraView] Error processing message from Python:', e);
            }
        });

        console.log('[AuroraView] ✓ Bridge initialized');
        console.log('[AuroraView] ✓ Low-level API: window.auroraview.send_event() / .on()');
        console.log('[AuroraView] ✓ High-level API: window.aurora.emit() / .on()');
        console.log('[AuroraView] ✓ Qt-style class: new AuroraView()');
    })();
    "#;
        builder = builder.with_initialization_script(event_bridge_script);

        // Set IPC handler
        let ipc_handler_clone = ipc_handler.clone();
        builder = builder.with_ipc_handler(move |request| {
            tracing::debug!("[OK] [NativeBackend] IPC message received");

            let body_str = request.body();
            tracing::debug!("[OK] [NativeBackend] IPC body: {}", body_str);

            if let Ok(message) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
                    if msg_type == "event" {
                        if let Some(event_name) = message.get("event").and_then(|v| v.as_str()) {
                            let detail = message
                                .get("detail")
                                .cloned()
                                .unwrap_or(serde_json::json!({}));
                            tracing::info!(
                                "[OK] [NativeBackend] Event received: {} with detail: {}",
                                event_name,
                                detail
                            );

                            let ipc_message = IpcMessage {
                                event: event_name.to_string(),
                                data: detail,
                                id: None,
                            };

                            match ipc_handler_clone.handle_message(ipc_message) {
                                Ok(_) => {
                                    tracing::info!(
                                        "[OK] [NativeBackend] Event handled successfully"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[ERROR] [NativeBackend] Error handling event: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });

        // Build WebView
        let webview = builder
            .build(window)
            .map_err(|e| format!("Failed to create WebView: {}", e))?;

        tracing::info!("[OK] [NativeBackend] WebView created successfully");

        // Load initial content
        if let Some(ref url) = config.url {
            tracing::info!("[OK] [NativeBackend] Loading URL: {}", url);
            let script = format!("window.location.href = '{}';", url);
            webview
                .evaluate_script(&script)
                .map_err(|e| format!("Failed to load URL: {}", e))?;
        } else if let Some(ref html) = config.html {
            tracing::info!("[OK] [NativeBackend] Loading HTML ({} bytes)", html.len());
            webview
                .load_html(html)
                .map_err(|e| format!("Failed to load HTML: {}", e))?;
        }

        Ok(webview)
    }
}
