//! Standalone mode - WebView with its own window
//!
//! This module handles creating WebView instances in standalone mode,
//! where the WebView creates and manages its own window.
//!
//! # Examples
//!
//! From Python (recommended):
//! ```python
//! from auroraview._core import run_standalone
//!
//! run_standalone(
//!     title="My App",
//!     width=800,
//!     height=600,
//!     url="https://example.com"
//! )
//! ```
//!
//! From Rust (internal use):
//! ```ignore
//! // This module is internal, use Python bindings instead
//! use auroraview_core::webview::config::WebViewConfig;
//! use auroraview_core::ipc::{IpcHandler, MessageQueue};
//! use std::sync::Arc;
//!
//! let config = WebViewConfig {
//!     title: "My App".to_string(),
//!     width: 800,
//!     height: 600,
//!     url: Some("https://example.com".to_string()),
//!     ..Default::default()
//! };
//!
//! let ipc_handler = Arc::new(IpcHandler::new());
//! let message_queue = Arc::new(MessageQueue::new());
//!
//! // This will create a standalone window and run the event loop
//! // Note: This is a blocking call that will run until the window is closed
//! ```

use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopBuilder;
use tao::window::WindowBuilder;
use wry::WebViewBuilder as WryWebViewBuilder;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::js_assets;
use super::webview_inner::WebViewInner;
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};

/// Create standalone WebView with its own window
///
/// This function creates a WebView instance with its own window and event loop.
/// The window starts hidden to avoid white flash and shows a loading screen.
///
/// # Examples
///
/// ```ignore
/// // This is an internal function, use run_standalone from Python bindings instead
/// use auroraview_core::webview::config::WebViewConfig;
/// use auroraview_core::ipc::{IpcHandler, MessageQueue};
/// use std::sync::Arc;
///
/// let config = WebViewConfig {
///     title: "My App".to_string(),
///     width: 800,
///     height: 600,
///     url: Some("https://example.com".to_string()),
///     ..Default::default()
/// };
///
/// let ipc_handler = Arc::new(IpcHandler::new());
/// let message_queue = Arc::new(MessageQueue::new());
///
/// // Internal use only - called by run_standalone
/// // let webview = create_standalone(config, ipc_handler, message_queue).unwrap();
/// ```
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
        .with_resizable(config.resizable)
        .with_decorations(config.decorations)
        .with_transparent(config.transparent)
        .with_visible(false); // Start hidden to avoid white flash

    // If width or height is 0, maximize the window; otherwise set the size
    if config.width == 0 || config.height == 0 {
        tracing::info!(
            "[standalone] Maximizing window (width={}, height={})",
            config.width,
            config.height
        );
        window_builder = window_builder.with_maximized(true);
    } else {
        window_builder =
            window_builder.with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height));
    }

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

    // Register auroraview:// custom protocol for local asset loading
    if let Some(ref asset_root) = config.asset_root {
        let asset_root = asset_root.clone();
        tracing::info!(
            "[standalone] Registering auroraview:// protocol (asset_root: {:?})",
            asset_root
        );

        // On Windows, use HTTPS scheme for secure context support
        #[cfg(target_os = "windows")]
        {
            webview_builder = webview_builder.with_https_scheme(true);
        }

        webview_builder = webview_builder.with_custom_protocol(
            "auroraview".into(),
            move |_webview_id, request| {
                crate::webview::protocol_handlers::handle_auroraview_protocol(&asset_root, request)
            },
        );
    } else {
        tracing::debug!("[standalone] asset_root is None, auroraview:// protocol not registered");
    }

    // Register file:// protocol if enabled
    if config.allow_file_protocol {
        tracing::info!("[standalone] Enabling file:// protocol support");
        webview_builder = webview_builder
            .with_custom_protocol("file".into(), |_webview_id, request| {
                crate::webview::protocol_handlers::handle_file_protocol(request)
            });
    }

    // Build initialization script using js_assets module
    tracing::info!("[standalone] Building initialization script with js_assets");
    let event_bridge_script = js_assets::build_init_script(&config);

    // IMPORTANT: use initialization script so it reloads with every page load
    webview_builder = webview_builder.with_initialization_script(&event_bridge_script);

    // Store the target URL/HTML for later loading
    let target_url = config.url.clone();
    let target_html = config.html.clone();

    // Load loading screen first to avoid white screen
    let loading_html = js_assets::get_loading_html();
    tracing::info!("[standalone] Loading splash screen to avoid white screen");
    webview_builder = webview_builder.with_html(loading_html);

    // Add IPC handler to capture events and calls from JavaScript
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
                            .unwrap_or(serde_json::Value::Null);
                        tracing::info!(
                            "Event received from JavaScript: {} with detail: {}",
                            event_name,
                            detail
                        );

                        let ipc_message = IpcMessage {
                            event: event_name.to_string(),
                            data: detail,
                            id: None,
                        };

                        if let Err(e) = ipc_handler_clone.handle_message(ipc_message) {
                            tracing::error!("Error handling event: {}", e);
                        }
                    }
                } else if msg_type == "call" {
                    if let Some(method) = message.get("method").and_then(|v| v.as_str()) {
                        let params = message
                            .get("params")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        let id = message
                            .get("id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        tracing::info!(
                            "Call received from JavaScript: {} with params: {} id: {:?}",
                            method,
                            params,
                            id
                        );

                        let mut payload = serde_json::Map::new();
                        payload.insert("params".to_string(), params);
                        if let Some(ref call_id) = id {
                            payload.insert(
                                "id".to_string(),
                                serde_json::Value::String(call_id.clone()),
                            );
                        }

                        let ipc_message = IpcMessage {
                            event: method.to_string(),
                            data: serde_json::Value::Object(payload),
                            id,
                        };

                        if let Err(e) = ipc_handler_clone.handle_message(ipc_message) {
                            tracing::error!("Error handling call: {}", e);
                        }
                    }
                }
            }
        }
    });

    let webview = webview_builder.build(&window)?;

    tracing::info!("[standalone] WebView created successfully with loading screen");

    // Load the actual content after WebView is created
    // This happens in the background while the loading screen is visible
    if let Some(ref url) = target_url {
        tracing::info!("[standalone] Loading target URL in background: {}", url);
        let script = js_assets::build_load_url_script(url);
        webview.evaluate_script(&script)?;
    } else if let Some(ref html) = target_html {
        tracing::info!("[standalone] Loading target HTML in background");
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

/// Run standalone WebView with event_loop.run() (blocking until window closes)
///
/// This function is designed for standalone applications where the WebView owns
/// the event loop and the process should exit when the window closes.
/// It uses event_loop.run() which calls std::process::exit() on completion.
///
/// IMPORTANT: This will terminate the entire process when the window closes!
/// Only use this for standalone mode, NOT for DCC integration (embedded mode).
///
/// Use cases:
/// - Standalone Python scripts
/// - CLI applications
/// - Desktop applications
///
/// # Examples
///
/// From Python:
/// ```python
/// from auroraview._core import run_standalone
///
/// run_standalone(
///     title="My App",
///     width=1024,
///     height=768,
///     url="https://example.com"
/// )
/// ```
///
/// From Rust (internal use):
/// ```ignore
/// // This is an internal function, use Python bindings instead
/// use auroraview_core::webview::config::WebViewConfig;
/// use auroraview_core::ipc::{IpcHandler, MessageQueue};
/// use std::sync::Arc;
///
/// let config = WebViewConfig {
///     title: "My Standalone App".to_string(),
///     width: 1024,
///     height: 768,
///     url: Some("https://example.com".to_string()),
///     ..Default::default()
/// };
///
/// let ipc_handler = Arc::new(IpcHandler::new());
/// let message_queue = Arc::new(MessageQueue::new());
///
/// // This will block until the window is closed and then exit the process
/// // run_standalone(config, ipc_handler, message_queue).unwrap();
/// ```
pub fn run_standalone(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tao::event_loop::ControlFlow;

    // Create the WebView
    let mut webview_inner = create_standalone(config, ipc_handler, message_queue)?;

    // Take ownership of event loop and window using take()
    let event_loop = webview_inner
        .event_loop
        .take()
        .ok_or("Event loop is None")?;
    let window = webview_inner.window.take().ok_or("Window is None")?;
    let webview = webview_inner.webview.clone();

    // Window starts hidden - will be shown after a short delay to let loading screen render
    tracing::info!(
        "[Standalone] Window created (hidden), will show after loading screen renders..."
    );

    // Use a simple delay to ensure loading screen is rendered before showing window
    // This avoids the white flash that occurs when showing window before WebView is ready
    let show_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
    let mut window_shown = false;

    tracing::info!("[Standalone] Starting event loop with run()");

    // Run the event loop - this will block until window closes and then exit the process
    event_loop.run(move |event, _, control_flow| {
        // Poll frequently to check if we should show the window
        *control_flow = ControlFlow::Poll;

        // Keep webview alive
        let _ = &webview;

        // Show window after delay (once)
        if !window_shown && std::time::Instant::now() >= show_time {
            tracing::info!("[Standalone] Loading screen should be rendered, showing window now!");
            window.set_visible(true);
            window.request_redraw();
            window_shown = true;
            // Switch to Wait mode after showing window to reduce CPU usage
            *control_flow = ControlFlow::Wait;
        }

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            tracing::info!("[Standalone] Window close requested, exiting");
            // Set Exit control flow - WebView and Window will be dropped automatically
            // This helps avoid the Chrome_WidgetWin_0 unregister error
            *control_flow = ControlFlow::Exit;
        }
    });
}
