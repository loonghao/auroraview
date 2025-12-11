//! Desktop mode - WebView with its own window
//!
//! This module handles creating WebView instances in desktop mode,
//! where the WebView creates and manages its own window.
//!
//! # Examples
//!
//! From Python (recommended):
//! ```python
//! from auroraview._core import run_desktop
//!
//! run_desktop(
//!     title="My App",
//!     width=800,
//!     height=600,
//!     url="https://example.com"
//! )
//!
//! # Or use the legacy alias:
//! from auroraview._core import run_standalone
//! run_standalone(...)
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
//! // This will create a desktop window and run the event loop
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

// Use shared builder utilities from auroraview-core
use auroraview_core::builder::{get_background_color, log_background_color};
#[cfg(target_os = "windows")]
use auroraview_core::builder::{apply_child_window_style, init_com_sta, ChildWindowStyleOptions};

/// Create desktop WebView with its own window
///
/// This function creates a WebView instance with its own window and event loop.
/// The window starts hidden to avoid white flash and shows a loading screen.
///
/// # Examples
///
/// ```ignore
/// // This is an internal function, use run_desktop from Python bindings instead
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
/// // Internal use only - called by run_desktop
/// // let webview = create_desktop(config, ipc_handler, message_queue).unwrap();
/// ```
pub fn create_desktop(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    // Initialize COM for WebView2 on Windows (using shared utility)
    init_com_sta();

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

    // Set window icon (custom or default)
    if let Some(icon) = load_window_icon(config.icon.as_ref()) {
        window_builder = window_builder.with_window_icon(Some(icon));
        tracing::debug!("[standalone] Window icon set successfully");
    } else {
        tracing::debug!("[standalone] No window icon available");
    }

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
                    // RECOMMENDED: Use WS_CHILD for true child window embedding
                    // - wry's build_as_child() is designed for this
                    // - WebView2's "Windowed Hosting" is the simplest option
                    tracing::info!("Creating WS_CHILD window (RECOMMENDED mode)");
                    window_builder = window_builder
                        .with_decorations(false)
                        .with_parent_window(parent as isize);
                }
                EmbedMode::None => {
                    // Standalone window mode - no parent relationship
                    tracing::info!("EmbedMode::None with parent_hwnd - ignoring parent");
                }
            }
        }
    }

    let window = window_builder.build(&event_loop)?;

    // Apply WS_CHILD style for Child mode to prevent independent dragging
    #[cfg(target_os = "windows")]
    {
        use crate::webview::config::EmbedMode;
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if let Some(parent) = config.parent_hwnd {
            if matches!(config.embed_mode, EmbedMode::Child) {
                if let Ok(window_handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(handle) = window_handle.as_raw() {
                        let hwnd = handle.hwnd.get();
                        let _ = apply_child_window_style(
                            hwnd,
                            parent as isize,
                            ChildWindowStyleOptions::for_standalone(),
                        );
                    }
                }
            }
        }
    }

    // No manual SetParent needed when using builder-ext on Windows

    // Create WebContext with shared user data folder for better performance
    // Priority: 1. config.data_directory, 2. shared warmup folder, 3. system default
    let mut web_context = if let Some(ref data_dir) = config.data_directory {
        tracing::info!("[standalone] Using custom data directory: {:?}", data_dir);
        wry::WebContext::new(Some(data_dir.clone()))
    } else {
        // Try to use shared user data folder from warmup (Windows only)
        #[cfg(target_os = "windows")]
        let shared_folder = crate::platform::windows::warmup::get_shared_user_data_folder();
        #[cfg(not(target_os = "windows"))]
        let shared_folder: Option<std::path::PathBuf> = None;

        if let Some(ref shared_dir) = shared_folder {
            tracing::info!(
                "[standalone] Using shared warmup data directory: {:?}",
                shared_dir
            );
            wry::WebContext::new(Some(shared_dir.clone()))
        } else {
            tracing::debug!("[standalone] Using default data directory");
            wry::WebContext::default()
        }
    };

    // Create the WebView with IPC handler and web context
    let mut webview_builder = WryWebViewBuilder::new_with_web_context(&mut web_context);
    if config.dev_tools {
        webview_builder = webview_builder.with_devtools(true);
    }

    // Set remote debugging port for CDP (Chrome DevTools Protocol) connections
    // This allows Playwright/Puppeteer to connect to WebView2
    #[cfg(target_os = "windows")]
    if let Some(port) = config.remote_debugging_port {
        let args = format!("--remote-debugging-port={}", port);
        webview_builder = webview_builder.with_additional_browser_args(&args);
        tracing::info!(
            "[standalone] Set WebView2 additional browser args: {}",
            args
        );
    }

    // Set background color to match app background (dark theme) using shared utility
    let background_color = get_background_color();
    webview_builder = webview_builder.with_background_color(background_color);
    log_background_color(background_color);

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

    // Add navigation handler for security filtering
    if config.block_external_navigation {
        let allowed_domains = config.allowed_navigation_domains.clone();
        webview_builder = webview_builder.with_navigation_handler(move |uri| {
            // Always allow custom protocols
            if uri.starts_with("auroraview://")
                || uri.starts_with("data:")
                || uri.starts_with("about:")
                || uri.starts_with("blob:")
            {
                return true;
            }

            // Parse the URI to get the domain
            if let Ok(url) = url::Url::parse(&uri) {
                if let Some(host) = url.host_str() {
                    // Check if domain is in allowed list
                    for allowed in &allowed_domains {
                        if host == allowed || host.ends_with(&format!(".{}", allowed)) {
                            tracing::debug!("[standalone] Navigation allowed: {}", uri);
                            return true;
                        }
                    }
                    tracing::warn!(
                        "[standalone] Navigation blocked: {} (domain not allowed)",
                        uri
                    );
                    return false;
                }
            }

            // Block by default if can't parse
            tracing::warn!("[standalone] Navigation blocked: {} (invalid URL)", uri);
            false
        });
    }

    // Store the target URL/HTML for later loading
    let target_url = config.url.clone();
    let target_html = config.html.clone();

    // Load loading screen first to avoid white screen
    let loading_html = js_assets::get_loading_html();
    tracing::info!("[standalone] Loading splash screen to avoid white screen");
    webview_builder = webview_builder.with_html(loading_html);

    // Add native file drag-drop handler using shared builder module
    // This provides full file paths that browsers cannot access due to security restrictions
    let ipc_handler_for_drop = ipc_handler.clone();
    webview_builder = webview_builder.with_drag_drop_handler(
        auroraview_core::builder::create_drag_drop_handler(move |event_name, data| {
            let ipc_message = IpcMessage {
                event: event_name.to_string(),
                data,
                id: None,
            };

            if let Err(e) = ipc_handler_for_drop.handle_message(ipc_message) {
                tracing::error!("[standalone] Error handling {}: {}", event_name, e);
            }
        }),
    );

    // Add IPC handler to capture events and calls from JavaScript
    let ipc_handler_clone = ipc_handler.clone();
    let message_queue_clone = message_queue.clone();

    // Create plugin router for handling plugin commands
    let plugin_router = Arc::new(std::sync::RwLock::new(
        auroraview_core::plugins::PluginRouter::with_scope(
            auroraview_core::plugins::ScopeConfig::permissive(),
        ),
    ));
    let plugin_router_clone = plugin_router.clone();

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
                } else if msg_type == "invoke" {
                    // Handle plugin invoke commands
                    let cmd = message.get("cmd").and_then(|v| v.as_str());
                    let args = message
                        .get("args")
                        .cloned()
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    let id = message
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if let Some(invoke_cmd) = cmd {
                        tracing::info!(
                            "Invoke received from JavaScript: {} with args: {} id: {:?}",
                            invoke_cmd,
                            args,
                            id
                        );

                        // Handle plugin command
                        let response = if let Ok(router) = plugin_router_clone.read() {
                            if let Some(request) =
                                auroraview_core::plugins::PluginRequest::from_invoke(
                                    invoke_cmd, args,
                                )
                            {
                                router.handle(request)
                            } else {
                                auroraview_core::plugins::PluginResponse::err(
                                    format!("Invalid plugin command: {}", invoke_cmd),
                                    "INVALID_COMMAND",
                                )
                            }
                        } else {
                            auroraview_core::plugins::PluginResponse::err(
                                "Plugin router lock failed",
                                "INTERNAL_ERROR",
                            )
                        };

                        // Send result back to JavaScript
                        if let Some(call_id) = id {
                            let result_payload = if response.success {
                                serde_json::json!({
                                    "type": "call_result",
                                    "id": call_id,
                                    "ok": true,
                                    "result": response.data
                                })
                            } else {
                                serde_json::json!({
                                    "type": "call_result",
                                    "id": call_id,
                                    "ok": false,
                                    "error": {
                                        "name": "PluginError",
                                        "message": response.error.unwrap_or_default(),
                                        "code": response.code
                                    }
                                })
                            };

                            // Dispatch call_result event to JavaScript
                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('__auroraview_call_result', {{ detail: {} }}));",
                                result_payload
                            );
                            message_queue_clone.push(crate::ipc::WebViewMessage::EvalJs(script));
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
    //
    // IMPORTANT: Use wry's native load_url() instead of evaluate_script() with
    // window.location.href. The JavaScript approach fails because:
    // 1. The splash screen HTML may not be fully loaded when evaluate_script runs
    // 2. window.location.href assignment can be blocked or ignored in certain contexts
    // 3. Native load_url() is more reliable and handles all edge cases
    if let Some(ref url) = target_url {
        tracing::info!("[standalone] Loading target URL in background: {}", url);
        webview.load_url(url)?;
    } else if let Some(ref html) = target_html {
        tracing::info!("[standalone] Loading target HTML in background");
        webview.load_html(html)?;
    }

    // Create event loop proxy for sending close events
    let event_loop_proxy = event_loop.create_proxy();

    // CRITICAL: Set event loop proxy in message queue for immediate wake-up
    // Without this, messages pushed to queue won't wake the event loop!
    message_queue.set_event_loop_proxy(event_loop_proxy.clone());
    tracing::info!("[standalone] Event loop proxy set in message queue for wake-up");

    // Create lifecycle manager
    use crate::webview::lifecycle::LifecycleManager;
    let lifecycle = Arc::new(LifecycleManager::new());
    lifecycle.set_state(crate::webview::lifecycle::LifecycleState::Active);

    // Determine auto_show: false in headless mode
    let auto_show = config.auto_show && !config.headless;

    // Cache HWND before creating WebViewInner (window may be moved during event loop)
    #[cfg(target_os = "windows")]
    let cached_hwnd = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
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
    };

    #[allow(clippy::arc_with_non_send_sync)]
    Ok(WebViewInner {
        webview: Arc::new(Mutex::new(webview)),
        window: Some(window),
        event_loop: Some(event_loop),
        message_queue,
        event_loop_proxy: Some(event_loop_proxy),
        lifecycle,
        auto_show,
        #[cfg(target_os = "windows")]
        backend: None, // Only used in DCC mode
        #[cfg(target_os = "windows")]
        cached_hwnd,
    })
}

/// Run desktop WebView with event_loop.run() (blocking until window closes)
///
/// This function is designed for desktop applications where the WebView owns
/// the event loop and the process should exit when the window closes.
/// It uses event_loop.run() which calls std::process::exit() on completion.
///
/// IMPORTANT: This will terminate the entire process when the window closes!
/// Only use this for desktop mode, NOT for DCC integration (embedded mode).
///
/// Use cases:
/// - Desktop Python scripts
/// - CLI applications
/// - Desktop applications
///
/// # Examples
///
/// From Python:
/// ```python
/// from auroraview._core import run_desktop
///
/// run_desktop(
///     title="My App",
///     width=1024,
///     height=768,
///     url="https://example.com"
/// )
///
/// # Or use the legacy alias:
/// from auroraview._core import run_standalone
/// run_standalone(...)
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
///     title: "My Desktop App".to_string(),
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
/// // run_desktop(config, ipc_handler, message_queue).unwrap();
/// ```
pub fn run_desktop(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tao::event_loop::ControlFlow;

    // Save auto_show and headless before config is consumed
    let auto_show = config.auto_show;
    let headless = config.headless;

    // Create the WebView
    let mut webview_inner = create_desktop(config, ipc_handler, message_queue)?;

    // Take ownership of event loop and window using take()
    let event_loop = webview_inner
        .event_loop
        .take()
        .ok_or("Event loop is None")?;
    let window = webview_inner.window.take().ok_or("Window is None")?;
    let webview = webview_inner.webview.clone();

    // Window starts hidden - will be shown after a short delay to let loading screen render
    // (only if auto_show is enabled and not in headless mode)
    if headless {
        tracing::info!("[Standalone] Headless mode enabled, window will remain hidden");
    } else if auto_show {
        tracing::info!(
            "[Standalone] Window created (hidden), will show after loading screen renders..."
        );
    } else {
        tracing::info!(
            "[Standalone] Window created (hidden), auto_show=false, window will stay hidden"
        );
    }

    // Use a simple delay to ensure loading screen is rendered before showing window
    // This avoids the white flash that occurs when showing window before WebView is ready
    let show_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
    // Window should only be shown if: auto_show is true AND headless is false
    let mut window_shown = !auto_show || headless;

    tracing::info!("[Standalone] Starting event loop with run()");

    // Run the event loop - this will block until window closes and then exit the process
    event_loop.run(move |event, _, control_flow| {
        // Poll frequently to check if we should show the window (only if auto_show)
        if auto_show && !window_shown {
            *control_flow = ControlFlow::Poll;
        } else {
            *control_flow = ControlFlow::Wait;
        }

        // Keep webview alive
        let _ = &webview;

        // Show window after delay (once) - only if auto_show is enabled and not headless
        if !headless && auto_show && !window_shown && std::time::Instant::now() >= show_time {
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

/// Embedded window icon (32x32 PNG) - used as fallback
const DEFAULT_ICON_PNG_BYTES: &[u8] = include_bytes!("../../assets/icons/auroraview-32.png");

/// Load window icon from custom path or use embedded default
///
/// # Arguments
/// * `custom_icon` - Optional path to custom icon file (PNG, ICO, JPEG, BMP, GIF)
///
/// # Icon Requirements
/// - **Format**: PNG (recommended), ICO, JPEG, BMP, GIF
/// - **Recommended sizes**: 32x32 (taskbar), 64x64 (alt-tab), 256x256 (high-DPI)
/// - **Color depth**: 32-bit RGBA recommended for transparency support
fn load_window_icon(custom_icon: Option<&std::path::PathBuf>) -> Option<tao::window::Icon> {
    use ::image::GenericImageView;

    // Try custom icon first
    if let Some(icon_path) = custom_icon {
        if icon_path.exists() {
            match ::image::open(icon_path) {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    let rgba = img.into_rgba8().into_raw();
                    if let Ok(icon) = tao::window::Icon::from_rgba(rgba, width, height) {
                        tracing::info!("[standalone] Loaded custom icon from {:?}", icon_path);
                        return Some(icon);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "[standalone] Failed to load custom icon {:?}: {}, using default",
                        icon_path,
                        e
                    );
                }
            }
        } else {
            tracing::warn!(
                "[standalone] Custom icon path does not exist: {:?}, using default",
                icon_path
            );
        }
    }

    // Fall back to embedded default icon
    let img = ::image::load_from_memory(DEFAULT_ICON_PNG_BYTES).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();

    tao::window::Icon::from_rgba(rgba, width, height).ok()
}

// ============================================================
// Backward compatibility aliases
// ============================================================

/// Alias for `create_desktop` (backward compatibility)
///
/// This function is deprecated in favor of `create_desktop`.
/// It will be removed in a future version.
#[inline]
pub fn create_standalone(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    create_desktop(config, ipc_handler, message_queue)
}

/// Alias for `run_desktop` (backward compatibility)
///
/// This function is deprecated in favor of `run_desktop`.
/// It will be removed in a future version.
#[allow(dead_code)]
#[inline]
pub fn run_standalone(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_desktop(config, ipc_handler, message_queue)
}

#[cfg(test)]
mod tests {
    use crate::webview::config::WebViewConfig;
    use std::path::PathBuf;

    /// Test that config with width=0 should trigger maximize
    #[test]
    fn test_should_maximize_when_width_zero() {
        let config = WebViewConfig {
            width: 0,
            height: 600,
            ..Default::default()
        };
        let should_maximize = config.width == 0 || config.height == 0;
        assert!(should_maximize);
    }

    /// Test that config with height=0 should trigger maximize
    #[test]
    fn test_should_maximize_when_height_zero() {
        let config = WebViewConfig {
            width: 800,
            height: 0,
            ..Default::default()
        };
        let should_maximize = config.width == 0 || config.height == 0;
        assert!(should_maximize);
    }

    /// Test that config with both dimensions zero should trigger maximize
    #[test]
    fn test_should_maximize_when_both_zero() {
        let config = WebViewConfig {
            width: 0,
            height: 0,
            ..Default::default()
        };
        let should_maximize = config.width == 0 || config.height == 0;
        assert!(should_maximize);
    }

    /// Test that config with normal dimensions should NOT maximize
    #[test]
    fn test_should_not_maximize_with_normal_dimensions() {
        let config = WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        };
        let should_maximize = config.width == 0 || config.height == 0;
        assert!(!should_maximize);
    }

    /// Test asset_root config conversion
    #[test]
    fn test_asset_root_config() {
        let config = WebViewConfig {
            asset_root: Some(PathBuf::from("/tmp/assets")),
            ..Default::default()
        };
        assert!(config.asset_root.is_some());
        assert_eq!(
            config.asset_root.as_ref().unwrap().to_str().unwrap(),
            "/tmp/assets"
        );
    }

    /// Test asset_root with None
    #[test]
    fn test_asset_root_none() {
        let config = WebViewConfig::default();
        assert!(config.asset_root.is_none());
    }

    /// Test allow_file_protocol config
    #[test]
    fn test_allow_file_protocol_enabled() {
        let config = WebViewConfig {
            allow_file_protocol: true,
            ..Default::default()
        };
        assert!(config.allow_file_protocol);
    }

    /// Test allow_file_protocol default
    #[test]
    fn test_allow_file_protocol_default() {
        let config = WebViewConfig::default();
        assert!(!config.allow_file_protocol);
    }

    /// Test always_on_top config
    #[test]
    fn test_always_on_top_enabled() {
        let config = WebViewConfig {
            always_on_top: true,
            ..Default::default()
        };
        assert!(config.always_on_top);
    }

    /// Test always_on_top default
    #[test]
    fn test_always_on_top_default() {
        let config = WebViewConfig::default();
        assert!(!config.always_on_top);
    }

    /// Test combined asset_root and allow_file_protocol
    #[test]
    fn test_combined_local_file_options() {
        let config = WebViewConfig {
            asset_root: Some(PathBuf::from("./assets")),
            allow_file_protocol: true,
            ..Default::default()
        };
        assert!(config.asset_root.is_some());
        assert!(config.allow_file_protocol);
    }

    /// Test window visibility starts hidden
    #[test]
    fn test_config_transparent() {
        let config = WebViewConfig {
            transparent: true,
            ..Default::default()
        };
        assert!(config.transparent);
    }

    /// Test decorations config
    #[test]
    fn test_config_decorations() {
        let config = WebViewConfig {
            decorations: false,
            ..Default::default()
        };
        assert!(!config.decorations);
    }

    /// Test resizable config
    #[test]
    fn test_config_resizable() {
        let config = WebViewConfig {
            resizable: false,
            ..Default::default()
        };
        assert!(!config.resizable);
    }

    /// Test dev_tools config
    #[test]
    fn test_config_dev_tools() {
        let config = WebViewConfig {
            dev_tools: true,
            ..Default::default()
        };
        assert!(config.dev_tools);
    }

    /// Test dev_tools default is true (enabled for development convenience)
    #[test]
    fn test_config_dev_tools_default() {
        let config = WebViewConfig::default();
        assert!(config.dev_tools);
    }

    /// Test headless config
    #[test]
    fn test_config_headless() {
        let config = WebViewConfig {
            headless: true,
            ..Default::default()
        };
        assert!(config.headless);
    }

    /// Test headless default is false
    #[test]
    fn test_config_headless_default() {
        let config = WebViewConfig::default();
        assert!(!config.headless);
    }

    /// Test auto_show config
    #[test]
    fn test_config_auto_show() {
        let config = WebViewConfig {
            auto_show: false,
            ..Default::default()
        };
        assert!(!config.auto_show);
    }

    /// Test auto_show default is true
    #[test]
    fn test_config_auto_show_default() {
        let config = WebViewConfig::default();
        assert!(config.auto_show);
    }

    /// Test auto_show logic with headless mode
    #[test]
    fn test_auto_show_disabled_in_headless() {
        let config = WebViewConfig {
            auto_show: true,
            headless: true,
            ..Default::default()
        };
        // auto_show should be effectively false when headless is true
        let effective_auto_show = config.auto_show && !config.headless;
        assert!(!effective_auto_show);
    }

    /// Test URL config
    #[test]
    fn test_config_url() {
        let config = WebViewConfig {
            url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.url, Some("https://example.com".to_string()));
    }

    /// Test HTML config
    #[test]
    fn test_config_html() {
        let config = WebViewConfig {
            html: Some("<html><body>Hello</body></html>".to_string()),
            ..Default::default()
        };
        assert!(config.html.is_some());
        assert!(config.html.as_ref().unwrap().contains("Hello"));
    }

    /// Test title config
    #[test]
    fn test_config_title() {
        let config = WebViewConfig {
            title: "My Custom Title".to_string(),
            ..Default::default()
        };
        assert_eq!(config.title, "My Custom Title");
    }

    /// Test default title
    #[test]
    fn test_config_title_default() {
        let config = WebViewConfig::default();
        assert!(!config.title.is_empty());
    }

    /// Test icon config
    #[test]
    fn test_config_icon() {
        let config = WebViewConfig {
            icon: Some(PathBuf::from("/path/to/icon.png")),
            ..Default::default()
        };
        assert!(config.icon.is_some());
        assert_eq!(
            config.icon.as_ref().unwrap().to_str().unwrap(),
            "/path/to/icon.png"
        );
    }

    /// Test icon default is None
    #[test]
    fn test_config_icon_default() {
        let config = WebViewConfig::default();
        assert!(config.icon.is_none());
    }

    /// Test data_directory config
    #[test]
    fn test_config_data_directory() {
        let config = WebViewConfig {
            data_directory: Some(PathBuf::from("/tmp/webview_data")),
            ..Default::default()
        };
        assert!(config.data_directory.is_some());
    }

    /// Test data_directory default is None
    #[test]
    fn test_config_data_directory_default() {
        let config = WebViewConfig::default();
        assert!(config.data_directory.is_none());
    }

    /// Test remote_debugging_port config
    #[test]
    fn test_config_remote_debugging_port() {
        let config = WebViewConfig {
            remote_debugging_port: Some(9222),
            ..Default::default()
        };
        assert_eq!(config.remote_debugging_port, Some(9222));
    }

    /// Test remote_debugging_port default is None
    #[test]
    fn test_config_remote_debugging_port_default() {
        let config = WebViewConfig::default();
        assert!(config.remote_debugging_port.is_none());
    }

    /// Test block_external_navigation config
    #[test]
    fn test_config_block_external_navigation() {
        let config = WebViewConfig {
            block_external_navigation: true,
            ..Default::default()
        };
        assert!(config.block_external_navigation);
    }

    /// Test block_external_navigation default
    #[test]
    fn test_config_block_external_navigation_default() {
        let config = WebViewConfig::default();
        assert!(!config.block_external_navigation);
    }

    /// Test allowed_navigation_domains config
    #[test]
    fn test_config_allowed_navigation_domains() {
        let config = WebViewConfig {
            allowed_navigation_domains: vec!["example.com".to_string(), "test.org".to_string()],
            ..Default::default()
        };
        assert_eq!(config.allowed_navigation_domains.len(), 2);
        assert!(config
            .allowed_navigation_domains
            .contains(&"example.com".to_string()));
    }

    /// Test allowed_navigation_domains default is empty
    #[test]
    fn test_config_allowed_navigation_domains_default() {
        let config = WebViewConfig::default();
        assert!(config.allowed_navigation_domains.is_empty());
    }

    /// Test parent_hwnd config
    #[test]
    fn test_config_parent_hwnd() {
        let config = WebViewConfig {
            parent_hwnd: Some(12345),
            ..Default::default()
        };
        assert_eq!(config.parent_hwnd, Some(12345));
    }

    /// Test parent_hwnd default is None
    #[test]
    fn test_config_parent_hwnd_default() {
        let config = WebViewConfig::default();
        assert!(config.parent_hwnd.is_none());
    }

    /// Test combined config for DCC embedding
    #[test]
    fn test_config_dcc_embedding() {
        let config = WebViewConfig {
            parent_hwnd: Some(0x12345678),
            decorations: false,
            resizable: true,
            transparent: false,
            ..Default::default()
        };
        assert!(config.parent_hwnd.is_some());
        assert!(!config.decorations);
        assert!(config.resizable);
        assert!(!config.transparent);
    }

    /// Test combined config for standalone app
    #[test]
    fn test_config_standalone_app() {
        let config = WebViewConfig {
            title: "My App".to_string(),
            width: 1024,
            height: 768,
            url: Some("https://myapp.com".to_string()),
            dev_tools: true,
            decorations: true,
            resizable: true,
            ..Default::default()
        };
        assert_eq!(config.title, "My App");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(config.url.is_some());
        assert!(config.dev_tools);
        assert!(config.decorations);
        assert!(config.resizable);
    }

    /// Test combined config for headless testing
    #[test]
    fn test_config_headless_testing() {
        let config = WebViewConfig {
            headless: true,
            auto_show: false,
            remote_debugging_port: Some(9222),
            url: Some("https://test.example.com".to_string()),
            ..Default::default()
        };
        assert!(config.headless);
        assert!(!config.auto_show);
        assert_eq!(config.remote_debugging_port, Some(9222));
    }

    /// Test combined config for local development
    #[test]
    fn test_config_local_development() {
        let config = WebViewConfig {
            asset_root: Some(PathBuf::from("./frontend/dist")),
            allow_file_protocol: true,
            dev_tools: true,
            url: Some("auroraview://localhost/index.html".to_string()),
            ..Default::default()
        };
        assert!(config.asset_root.is_some());
        assert!(config.allow_file_protocol);
        assert!(config.dev_tools);
    }
}
