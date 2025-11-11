//! Standalone mode - WebView with its own window
//!
//! This module handles creating WebView instances in standalone mode,
//! where the WebView creates and manages its own window.

use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopBuilder;
use tao::window::WindowBuilder;
use wry::WebViewBuilder as WryWebViewBuilder;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::webview_inner::WebViewInner;
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};

/// Create standalone WebView with its own window
pub fn create_standalone(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    // Allow event loop to be created on any thread (required for DCC integration)
    // Use UserEvent for custom events (wake-up for immediate message processing)
    #[cfg(target_os = "windows")]
    let event_loop = {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
    };

    #[cfg(not(target_os = "windows"))]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
        .with_resizable(config.resizable)
        .with_decorations(config.decorations)
        .with_transparent(config.transparent);

    // Parent/owner on Windows
    #[cfg(target_os = "windows")]
    {
        use crate::webview::config::EmbedMode;
        use tao::platform::windows::WindowBuilderExtWindows;

        if let Some(parent) = config.parent_hwnd {
            match config.embed_mode {
                EmbedMode::Child => {
                    tracing::info!("Creating WS_CHILD window (same-thread parenting required)");
                    // Child windows typically have no decorations
                    window_builder = window_builder
                        .with_decorations(false)
                        .with_parent_window(parent as isize);
                }
                EmbedMode::Owner => {
                    tracing::info!("Creating owned window (cross-thread safe)");
                    window_builder = window_builder.with_owner_window(parent as isize);
                }
                EmbedMode::None => {}
            }
        }
    }

    let window = window_builder.build(&event_loop)?;

    // No manual SetParent needed when using builder-ext on Windows

    // Create the WebView with IPC handler
    let mut webview_builder = WryWebViewBuilder::new();
    if config.dev_tools {
        webview_builder = webview_builder.with_devtools(true);
    }

    // Inject event bridge as initialization script so it persists across navigations
    let event_bridge_script: &str = r#"
    (function() {
        console.log('[AuroraView] Initializing event bridge...');

        // Event handlers registry
        const eventHandlers = new Map();

        // Listen for events from Python (Python -> JS)
        window.addEventListener('message', function(event) {
            console.log('[AuroraView] Received message event:', event);
            try {
                const data = JSON.parse(event.data);
                console.log('[AuroraView] Parsed message data:', data);

                if (data.type === 'event' && data.event) {
                    const handlers = eventHandlers.get(data.event);
                    if (handlers && handlers.length > 0) {
                        console.log('[AuroraView] Dispatching to', handlers.length, 'handler(s) for event:', data.event);
                        handlers.forEach(handler => {
                            try {
                                handler(data.detail || {});
                            } catch (e) {
                                console.error('[AuroraView] Error in event handler:', e);
                            }
                        });
                    } else {
                        console.warn('[AuroraView] No handlers registered for event:', data.event);
                    }
                }
            } catch (e) {
                console.error('[AuroraView] Failed to parse message:', e);
            }
        });

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

        // Intercept CustomEvent dispatch for backward compatibility
        const originalDispatchEvent = window.dispatchEvent;
        window.dispatchEvent = function(event) {
            if (event instanceof CustomEvent) {
                // Ignore events emitted from Python to avoid feedback loop
                if (event.detail && event.detail.__aurora_from_python === true) {
                    return originalDispatchEvent.call(this, event);
                }
                try {
                    const message = {
                        type: 'event',
                        event: event.type,
                        detail: event.detail
                    };
                    window.ipc.postMessage(JSON.stringify(message));
                } catch (e) {
                    console.error('[AuroraView] Failed to send event via IPC:', e);
                }
            }
            return originalDispatchEvent.call(this, event);
        };

        console.log('[AuroraView] ✓ Bridge initialized');
        console.log('[AuroraView] ✓ Low-level API: window.auroraview.send_event() / .on()');
        console.log('[AuroraView] ✓ High-level API: window.aurora.emit() / .on()');
        console.log('[AuroraView] ✓ Qt-style class: new AuroraView()');
    })();
    "#;

    // IMPORTANT: use initialization script so it reloads with every page load
    webview_builder = webview_builder.with_initialization_script(event_bridge_script);

    // Add IPC handler to capture events from JavaScript
    let ipc_handler_clone = ipc_handler.clone();
    webview_builder = webview_builder.with_ipc_handler(move |request| {
        tracing::debug!("IPC message received");

        // The request body is a String
        let body_str = request.body();
        tracing::debug!("IPC body: {}", body_str);

        if let Ok(message) = serde_json::from_str::<serde_json::Value>(body_str) {
            if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
                if msg_type == "event" {
                    if let Some(event_name) = message.get("event").and_then(|v| v.as_str()) {
                        let detail = message
                            .get("detail")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));
                        tracing::info!(
                            "Event received from JavaScript: {} with detail: {}",
                            event_name,
                            detail
                        );

                        // Create IPC message and handle it
                        let ipc_message = IpcMessage {
                            event: event_name.to_string(),
                            data: detail,
                            id: None,
                        };

                        // Call the IPC handler to invoke Python callbacks
                        match ipc_handler_clone.handle_message(ipc_message) {
                            Ok(_) => {
                                tracing::info!("Event handled successfully");
                            }
                            Err(e) => {
                                tracing::error!("Error handling event: {}", e);
                            }
                        }
                    }
                }
            }
        }
    });

    let webview = webview_builder.build(&window)?;

    // Apply initial content from config if provided
    if let Some(ref url) = config.url {
        let script = format!("window.location.href = '{}';", url);
        webview.evaluate_script(&script)?;
    } else if let Some(ref html) = config.html {
        webview.load_html(html)?;
    }

    // Create event loop proxy for sending close events
    let event_loop_proxy = event_loop.create_proxy();

    // Create lifecycle manager
    use crate::webview::lifecycle::LifecycleManager;
    let lifecycle = Arc::new(LifecycleManager::new());
    lifecycle.set_state(crate::webview::lifecycle::LifecycleState::Active);

    // Standalone mode doesn't need platform manager (uses event loop instead)
    let platform_manager = None;

    #[allow(clippy::arc_with_non_send_sync)]
    Ok(WebViewInner {
        webview: Arc::new(Mutex::new(webview)),
        window: Some(window),
        event_loop: Some(event_loop),
        message_queue,
        event_loop_proxy: Some(event_loop_proxy),
        lifecycle,
        platform_manager,
        #[cfg(target_os = "windows")]
        backend: None, // Only used in DCC mode
    })
}
