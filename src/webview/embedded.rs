//! Embedded mode - WebView embedded in existing window
//!
//! This module handles creating WebView instances in embedded mode,
//! where the WebView is embedded as a child of an existing window (for DCC integration).

#[allow(unused_imports)]
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use tao::event_loop::EventLoopBuilder;
#[allow(unused_imports)]
use tao::window::WindowBuilder;
#[allow(unused_imports)]
use wry::WebViewBuilder as WryWebViewBuilder;

use super::config::WebViewConfig;
#[allow(unused_imports)]
use super::event_loop::UserEvent;
use super::webview_inner::WebViewInner;
#[allow(unused_imports)]
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};

/// Set parent window using Windows API
#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn set_parent_window(
    window: &tao::window::Window,
    parent_hwnd: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, IsWindow, SetParent, SetWindowLongPtrW, GWL_STYLE, WS_CHILD, WS_POPUP,
    };

    tracing::info!(
        "Setting parent window: parent_hwnd={} (0x{:x})",
        parent_hwnd,
        parent_hwnd
    );

    // Get the child window handle
    let window_handle = window.window_handle()?;
    let raw_handle = window_handle.as_raw();

    if let RawWindowHandle::Win32(handle) = raw_handle {
        let child_hwnd_value = handle.hwnd.get();
        let child_hwnd = HWND(child_hwnd_value as *mut c_void);

        // Convert u64 to HWND - handle both 32-bit and 64-bit systems
        // On Windows, HWND is a pointer-sized value
        let parent_hwnd_value = parent_hwnd as isize;
        let parent_hwnd_hwnd = HWND(parent_hwnd_value as *mut c_void);

        tracing::info!(
            "Child HWND: 0x{:x} (isize: {})",
            child_hwnd_value,
            child_hwnd_value
        );
        tracing::info!(
            "Parent HWND: 0x{:x} (isize: {})",
            parent_hwnd,
            parent_hwnd_value
        );

        // Validate window handles
        unsafe {
            let child_valid = IsWindow(child_hwnd).as_bool();
            let parent_valid = IsWindow(parent_hwnd_hwnd).as_bool();

            tracing::info!("Child window valid: {}", child_valid);
            tracing::info!("Parent window valid: {}", parent_valid);

            if !child_valid {
                tracing::error!(
                    "[ERROR] Child window handle is invalid: 0x{:x}",
                    child_hwnd_value
                );
                return Err("Child window handle is invalid".into());
            }

            if !parent_valid {
                tracing::error!(
                    "[ERROR] Parent window handle is invalid: 0x{:x}",
                    parent_hwnd
                );
                tracing::warn!("Parent HWND may be from a different process, not yet created, or already destroyed");
                tracing::warn!(
                    "Attempting SetParent anyway - it may still work if the handle is valid"
                );
            }

            // Set the parent window
            tracing::info!("Attempting to set parent window...");
            match SetParent(child_hwnd, parent_hwnd_hwnd) {
                Ok(previous_parent) => {
                    if previous_parent.is_invalid() {
                        tracing::info!("[OK] Window had no previous parent");
                    } else {
                        tracing::info!(
                            "[OK] Successfully set parent window: previous_parent=0x{:x}",
                            previous_parent.0 as usize
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("[ERROR] SetParent failed: {:?}", e);
                    tracing::error!(
                        "Parent HWND: 0x{:x}, Child HWND: 0x{:x}",
                        parent_hwnd,
                        child_hwnd_value
                    );
                    tracing::error!("This usually means:");
                    tracing::error!(
                        "  1. Parent window handle is invalid or from a different process"
                    );
                    tracing::error!("  2. Parent window was destroyed before SetParent was called");
                    tracing::error!("  3. The pointer passed is not a valid HWND");
                    return Err(format!("SetParent failed: {:?}", e).into());
                }
            }

            // Modify window style to be a child window
            // This prevents the creation of extra GDI+ windows
            tracing::info!("Modifying window style to WS_CHILD...");
            let current_style = GetWindowLongPtrW(child_hwnd, GWL_STYLE);
            if current_style != 0 {
                let new_style = (current_style & !(WS_POPUP.0 as isize)) | (WS_CHILD.0 as isize);
                let result = SetWindowLongPtrW(child_hwnd, GWL_STYLE, new_style);
                if result != 0 {
                    tracing::info!("[OK] Window style modified successfully");
                } else {
                    tracing::warn!(
                        "[WARNING] Failed to modify window style, but SetParent succeeded"
                    );
                }
            } else {
                tracing::warn!("[WARNING] Failed to get current window style");
            }
        }

        tracing::info!("[OK] Parent window set successfully");
        Ok(())
    } else {
        Err("Window handle is not a Win32 handle".into())
    }
}

/// Create embedded WebView for DCC integration (Windows)
#[cfg(target_os = "windows")]
pub fn create_embedded(
    parent_hwnd: u64,
    width: u32,
    height: u32,
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    use crate::webview::config::EmbedMode;
    use tao::platform::windows::WindowBuilderExtWindows;

    tracing::info!(
        "[OK] [create_embedded] Creating embedded WebView (parent_hwnd: {}, mode: {:?})",
        parent_hwnd,
        config.embed_mode
    );

    // CRITICAL: Embedded mode must NOT create an event loop!
    // The parent window (DCC app) will handle all events.
    // Creating an event loop here causes the main thread to block.
    tracing::info!(
        "[OK] [create_embedded] Skipping event loop creation (parent will handle events)"
    );

    // Create a temporary event loop ONLY for window creation
    #[cfg(target_os = "windows")]
    let temp_event_loop = {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
    };

    #[cfg(not(target_os = "windows"))]
    let temp_event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Create window builder
    let mut window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(tao::dpi::LogicalSize::new(width, height))
        .with_resizable(config.resizable)
        .with_decorations(config.decorations)
        .with_always_on_top(config.always_on_top)
        .with_transparent(config.transparent);

    // Set parent window based on embed mode
    match config.embed_mode {
        EmbedMode::Child => {
            // Child mode: WS_CHILD style
            tracing::info!("[OK] [create_embedded] Using Child mode (WS_CHILD)");
            window_builder = window_builder.with_parent_window(parent_hwnd as isize);
        }
        EmbedMode::Owner => {
            // Owner mode: GWLP_HWNDPARENT
            tracing::info!("[OK] [create_embedded] Using Owner mode (GWLP_HWNDPARENT)");
            window_builder = window_builder.with_owner_window(parent_hwnd as isize);
        }
        EmbedMode::None => {
            tracing::warn!(
                "[WARNING] [create_embedded] EmbedMode::None - creating standalone window"
            );
        }
    }

    // Build window using temporary event loop
    let window = window_builder
        .build(&temp_event_loop)
        .map_err(|e| format!("Failed to create window: {}", e))?;

    // Log window HWND for debugging
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let Ok(window_handle) = window.window_handle() {
            let raw_handle = window_handle.as_raw();
            if let RawWindowHandle::Win32(handle) = raw_handle {
                let hwnd_value = handle.hwnd.get();
                tracing::info!("[OK] [create_embedded] Window created successfully");
                tracing::info!(
                    "[OK] [create_embedded] WebView HWND: 0x{:X} ({})",
                    hwnd_value,
                    hwnd_value
                );
                tracing::info!(
                    "[OK] [create_embedded] Parent HWND: 0x{:X} ({})",
                    parent_hwnd,
                    parent_hwnd
                );
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    tracing::info!("[OK] [create_embedded] Window created successfully");

    // CRITICAL: Make window visible
    tracing::info!("[OK] [create_embedded] Setting window visible...");
    window.set_visible(true);
    tracing::info!("[OK] [create_embedded] Window is now visible");

    // Create WebView
    let webview = {
        let mut builder = WryWebViewBuilder::new();

        // Enable developer tools if configured
        if config.dev_tools {
            tracing::info!("[OK] [create_embedded] Enabling developer tools");
            builder = builder.with_devtools(true);
        }

        // Add event bridge script with full window.auroraview API
        let event_bridge_script = r#"
    (function() {
        console.log('[AuroraView] Initializing event bridge...');

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

        // Set IPC handler (same as standalone mode)
        let ipc_handler_clone = ipc_handler.clone();
        builder = builder.with_ipc_handler(move |request| {
            tracing::debug!("[OK] [embedded] IPC message received");

            let body_str = request.body();
            tracing::debug!("[OK] [embedded] IPC body: {}", body_str);

            if let Ok(message) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
                    if msg_type == "event" {
                        if let Some(event_name) = message.get("event").and_then(|v| v.as_str()) {
                            let detail = message
                                .get("detail")
                                .cloned()
                                .unwrap_or(serde_json::json!({}));
                            tracing::info!(
                                "[OK] [embedded] Event received: {} with detail: {}",
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
                                    tracing::info!("[OK] [embedded] Event handled successfully");
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[ERROR] [embedded] Error handling event: {}",
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
        builder
            .build(&window)
            .map_err(|e| format!("Failed to create WebView: {}", e))?
    };

    tracing::info!("[OK] [create_embedded] WebView created successfully");

    // Load initial content from config (same as standalone mode)
    if let Some(ref url) = config.url {
        tracing::info!("[OK] [create_embedded] Loading URL: {}", url);
        let script = format!("window.location.href = '{}';", url);
        webview
            .evaluate_script(&script)
            .map_err(|e| format!("Failed to load URL: {}", e))?;
    } else if let Some(ref html) = config.html {
        tracing::info!("[OK] [create_embedded] Loading HTML ({} bytes)", html.len());
        webview
            .load_html(html)
            .map_err(|e| format!("Failed to load HTML: {}", e))?;
    } else {
        tracing::warn!("[WARNING] [create_embedded] No initial content specified");
    }

    // Drop the temporary event loop - we don't need it anymore
    // The window is already created and will be handled by the parent's event loop
    drop(temp_event_loop);
    tracing::info!("[OK] [create_embedded] Temporary event loop dropped");

    // Create lifecycle manager
    use crate::webview::lifecycle::LifecycleManager;
    let lifecycle = Arc::new(LifecycleManager::new());
    lifecycle.set_state(crate::webview::lifecycle::LifecycleState::Active);

    // Create platform-specific window manager
    #[cfg(target_os = "windows")]
    let platform_manager = {
        use crate::webview::platform;
        let manager = platform::create_platform_manager(parent_hwnd);
        manager.setup_close_handlers(lifecycle.clone());
        Some(manager)
    };

    #[cfg(not(target_os = "windows"))]
    let platform_manager = None;

    tracing::info!("[OK] [create_embedded] Lifecycle manager and platform manager created");

    #[allow(clippy::arc_with_non_send_sync)]
    Ok(WebViewInner {
        webview: Arc::new(Mutex::new(webview)),
        window: Some(window),
        event_loop: None, // CRITICAL: No event loop in embedded mode!
        message_queue,
        event_loop_proxy: None, // Embedded mode doesn't use event loop proxy
        lifecycle,
        platform_manager,
        #[cfg(target_os = "windows")]
        backend: None, // Only used in DCC mode
    })
}

/// Create embedded WebView for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn create_embedded(
    _parent_hwnd: u64,
    _width: u32,
    _height: u32,
    _config: WebViewConfig,
    _ipc_handler: Arc<IpcHandler>,
    _message_queue: Arc<MessageQueue>,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    Err("Embedded mode is only supported on Windows".into())
}
