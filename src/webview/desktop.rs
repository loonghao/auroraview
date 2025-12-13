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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopBuilder;
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::WindowBuilder;
use wry::WebViewBuilder as WryWebViewBuilder;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::js_assets;
use super::tray::TrayManager;
use super::webview_inner::WebViewInner;
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};

// Use shared builder utilities from auroraview-core
#[cfg(target_os = "windows")]
use auroraview_core::builder::{
    apply_child_window_style, apply_owner_window_style, apply_tool_window_style,
    disable_window_shadow, ChildWindowStyleOptions,
};
use auroraview_core::builder::{get_background_color, init_com_sta, log_background_color};

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
        .with_always_on_top(config.always_on_top)
        .with_visible(false); // Start hidden to avoid white flash

    // On Windows, apply platform-specific window styles
    // NOTE: with_undecorated_shadow and with_skip_taskbar are NOT applied here
    // because they cause WebView2 creation to fail with HRESULT 0x80070057.
    // These styles are applied AFTER WebView2 creation via apply_tool_window_style().
    #[cfg(target_os = "windows")]
    {
        // WindowBuilderExtWindows is imported but styles are applied post-creation
        // to avoid WebView2 initialization issues.
    }

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
                    // RECOMMENDED for embedding: Use WS_CHILD for true child window
                    // - wry's build_as_child() is designed for this
                    // - WebView2's "Windowed Hosting" is the simplest option
                    tracing::info!("Creating WS_CHILD window (RECOMMENDED for embedding)");
                    window_builder = window_builder
                        .with_decorations(false)
                        .with_parent_window(parent as isize);
                }
                EmbedMode::Owner => {
                    // RECOMMENDED for floating windows: Owner relationship
                    // - Window stays above owner in Z-order
                    // - Hidden when owner is minimized
                    // - Destroyed when owner is destroyed
                    // Owner is set after window creation via SetWindowLongPtrW
                    tracing::info!("Creating owned window (RECOMMENDED for floating windows)");
                    // Don't set parent here - we'll set owner after creation
                }
                EmbedMode::None => {
                    // Standalone window mode - no parent relationship
                    tracing::info!("EmbedMode::None with parent_hwnd - ignoring parent");
                }
            }
        }
    }

    let window = window_builder.build(&event_loop)?;

    // Apply window styles based on embed mode
    #[cfg(target_os = "windows")]
    {
        use crate::webview::config::EmbedMode;
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if let Ok(window_handle) = window.window_handle() {
            if let RawWindowHandle::Win32(handle) = window_handle.as_raw() {
                let hwnd = handle.hwnd.get();

                if let Some(parent) = config.parent_hwnd {
                    match config.embed_mode {
                        EmbedMode::Child => {
                            let _ = apply_child_window_style(
                                hwnd,
                                parent as isize,
                                ChildWindowStyleOptions::for_standalone(),
                            );
                        }
                        EmbedMode::Owner => {
                            // Set owner relationship using GWLP_HWNDPARENT
                            apply_owner_window_style(hwnd, parent, config.tool_window);
                        }
                        EmbedMode::None => {}
                    }
                }
                // NOTE: Tool window style for standalone windows (without owner) is
                // applied AFTER WebView2 creation to avoid HRESULT 0x80070057 error.
                // See the apply_tool_window_style call after webview_builder.build().
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

    // Set background color based on transparency setting
    // Transparent windows need both:
    // 1. with_transparent(true) on WebViewBuilder
    // 2. with_background_color((0,0,0,0)) for fully transparent background
    if config.transparent {
        webview_builder = webview_builder
            .with_transparent(true)
            .with_background_color((0, 0, 0, 0));
        tracing::info!("[standalone] Transparent window: WebView transparency enabled");
    } else {
        let background_color = get_background_color();
        webview_builder = webview_builder.with_background_color(background_color);
        log_background_color(background_color);
    }

    // Register auroraview:// custom protocol for local asset loading
    // Always register this protocol to support https://auroraview.localhost/file/... URLs
    let asset_root_for_protocol = config.asset_root.clone();
    tracing::info!(
        "[standalone] Registering auroraview:// protocol (asset_root: {:?})",
        asset_root_for_protocol
    );

    // On Windows, use HTTPS scheme for secure context support
    #[cfg(target_os = "windows")]
    {
        webview_builder = webview_builder.with_https_scheme(true);
    }

    // Use a default empty path if no asset_root is provided
    let default_asset_root = std::env::current_dir().unwrap_or_default();
    let protocol_asset_root = asset_root_for_protocol.unwrap_or(default_asset_root);

    webview_builder =
        webview_builder.with_custom_protocol("auroraview".into(), move |_webview_id, request| {
            crate::webview::protocol_handlers::handle_auroraview_protocol(
                &protocol_asset_root,
                request,
            )
        });

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

    // Handle new window requests (window.open())
    // On Windows WebView2, NewWindowResponse::Allow may not work reliably.
    // Instead, we open external links in the system default browser.
    if config.allow_new_window {
        tracing::info!("[standalone] Allowing new windows (opens in system browser)");
        webview_builder = webview_builder.with_new_window_req_handler(|url, _features| {
            tracing::info!("[standalone] New window requested: {}", url);
            // Open in system default browser instead of creating a new WebView window
            // This is more reliable on WebView2 and matches user expectations
            if let Err(e) = open::that(&url) {
                tracing::error!("[standalone] Failed to open URL in browser: {}", e);
            }
            // Deny creating a new WebView window since we opened in external browser
            wry::NewWindowResponse::Deny
        });
    } else {
        tracing::debug!("[standalone] New windows blocked by default");
    }

    // Determine initial content to load
    // Priority: 1. Target HTML, 2. Target URL (via splash), 3. Splash screen only
    let target_url = config.url.clone();
    let target_html = config.html.clone();

    // If we have target HTML, load it directly (no splash screen needed)
    // If we have target URL, show splash screen first then navigate
    // If neither, just show splash screen
    if let Some(ref html) = target_html {
        tracing::info!(
            "[standalone] Loading target HTML directly ({} bytes)",
            html.len()
        );
        webview_builder = webview_builder.with_html(html);
    } else {
        // Load splash screen for URL navigation or empty state
        let loading_html = js_assets::get_loading_html();
        tracing::info!("[standalone] Loading splash screen");
        webview_builder = webview_builder.with_html(loading_html);
    }

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
                        let has_params = message.get("params").is_some();
                        let params = message.get("params").cloned();
                        let id = message
                            .get("id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        tracing::info!(
                            "Call received from JavaScript: {} with params: {:?} id: {:?}",
                            method,
                            params,
                            id
                        );

                        let mut payload = serde_json::Map::new();
                        // Only include params if it was present in the original message
                        if has_params {
                            if let Some(p) = params {
                                payload.insert("params".to_string(), p);
                            } else {
                                payload.insert("params".to_string(), serde_json::Value::Null);
                            }
                        }
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

                            // Dispatch call_result event to JavaScript using auroraview.trigger()
                            let escaped_json = result_payload.to_string().replace('\\', "\\\\").replace('\'', "\\'");
                            let script = format!(
                                "(function() {{ \
                                    if (window.auroraview && window.auroraview.trigger) {{ \
                                        window.auroraview.trigger('__auroraview_call_result', JSON.parse('{}')); \
                                    }} else {{ \
                                        console.error('[AuroraView] Event bridge not ready, cannot emit call_result'); \
                                    }} \
                                }})();",
                                escaped_json
                            );
                            message_queue_clone.push(crate::ipc::WebViewMessage::EvalJs(script));
                        }
                    }
                }
            }
        }
    });

    // Build the WebView - this is the slowest part of initialization
    // On first run, WebView2 needs to:
    // 1. Create WebView2 Environment (discover runtime, spawn browser process)
    // 2. Create WebView2 Controller
    // 3. Initialize the WebView with HTML content
    tracing::info!("[standalone] Building WebView (this may take a few seconds on first run)...");
    let build_start = std::time::Instant::now();
    let webview = webview_builder.build(&window)?;
    let build_duration = build_start.elapsed();

    tracing::info!(
        "[standalone] WebView created successfully in {:.2}s",
        build_duration.as_secs_f64()
    );

    // Load target URL if specified (HTML was already loaded via with_html)
    // This navigates from splash screen to the target URL
    if let Some(ref url) = target_url {
        tracing::info!("[standalone] Loading target URL: {}", url);
        webview.load_url(url)?;
    }
    // Note: target_html was already loaded via with_html() above

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

    // Apply tool window style AFTER WebView2 is created
    // Doing this before WebView2 creation causes HRESULT 0x80070057 (E_INVALIDARG)
    #[cfg(target_os = "windows")]
    if config.tool_window {
        if let Some(hwnd) = cached_hwnd {
            apply_tool_window_style(hwnd as isize);
        }
    }

    // Disable window shadow for transparent frameless windows
    // undecorated_shadow=false means we want to disable the shadow
    #[cfg(target_os = "windows")]
    if !config.undecorated_shadow {
        if let Some(hwnd) = cached_hwnd {
            disable_window_shadow(hwnd as isize);
            tracing::info!("[standalone] Disabled window shadow (undecorated_shadow=false)");
        }
    }

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
    use tray_icon::menu::MenuEvent;
    use tray_icon::TrayIconEvent;

    // Save config values before config is consumed
    let auto_show = config.auto_show;
    let headless = config.headless;
    let tray_config = config.tray.clone();
    let window_icon = config.icon.clone();

    // Create the WebView
    let mut webview_inner = create_desktop(config, ipc_handler, message_queue)?;

    // Take ownership of event loop and window using take()
    let mut event_loop = webview_inner
        .event_loop
        .take()
        .ok_or("Event loop is None")?;
    let window = webview_inner.window.take().ok_or("Window is None")?;
    let webview = webview_inner.webview.clone();

    // Create system tray if configured
    let tray_manager = if let Some(ref tray_cfg) = tray_config {
        match TrayManager::new(tray_cfg, window_icon.as_ref()) {
            Ok(manager) => Some(manager),
            Err(e) => {
                tracing::warn!("[Standalone] Failed to create system tray: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Get tray settings for event handling
    let hide_on_close = tray_config.as_ref().is_some_and(|t| t.hide_on_close);
    let show_on_click = tray_config.as_ref().is_some_and(|t| t.show_on_click);
    let show_on_double_click = tray_config.as_ref().is_some_and(|t| t.show_on_double_click);

    // Store menu IDs for event handling
    let menu_ids = tray_manager
        .as_ref()
        .map(|m| Arc::new(m.menu_ids.clone()))
        .unwrap_or_else(|| Arc::new(std::collections::HashMap::new()));
    let menu_ids_clone = menu_ids.clone();

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
    let mut window_visible = false;

    // Set up Ctrl+C handler with atomic flag
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_flag = ctrlc_pressed.clone();

    // Try to set up Ctrl+C handler (may fail if already set)
    if let Err(e) = ctrlc::try_set_handler(move || {
        tracing::info!("[Standalone] Ctrl+C received, requesting exit...");
        ctrlc_flag.store(true, Ordering::SeqCst);
    }) {
        tracing::warn!("[Standalone] Could not set Ctrl+C handler: {}", e);
    }

    let tray_enabled = tray_manager.is_some();
    tracing::info!(
        "[Standalone] Starting event loop with run_return() (Ctrl+C enabled, tray={})",
        tray_enabled
    );

    // Run the event loop using run_return() to allow Ctrl+C handling
    // This returns normally instead of calling std::process::exit()
    event_loop.run_return(move |event, _, control_flow| {
        // Check for Ctrl+C signal
        if ctrlc_pressed.load(Ordering::SeqCst) {
            tracing::info!("[Standalone] Ctrl+C detected, exiting event loop");
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Process tray icon events
        if tray_enabled {
            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        ..
                    } => {
                        if show_on_click && !window_visible {
                            tracing::debug!("[Standalone] Tray click - showing window");
                            window.set_visible(true);
                            window.set_focus();
                            window_visible = true;
                        }
                    }
                    TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    } => {
                        if show_on_double_click {
                            tracing::debug!("[Standalone] Tray double-click - toggling window");
                            if window_visible {
                                window.set_visible(false);
                                window_visible = false;
                            } else {
                                window.set_visible(true);
                                window.set_focus();
                                window_visible = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Process menu events
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(id) = menu_ids_clone.get(&event.id) {
                    tracing::info!("[Standalone] Tray menu clicked: {}", id);
                    // Handle built-in menu actions
                    match id.as_str() {
                        "quit" | "exit" => {
                            tracing::info!("[Standalone] Quit menu clicked, exiting");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        "show" => {
                            window.set_visible(true);
                            window.set_focus();
                            window_visible = true;
                        }
                        "hide" => {
                            window.set_visible(false);
                            window_visible = false;
                        }
                        _ => {
                            // Custom menu item - emit event to JavaScript
                            if let Ok(wv) = webview.lock() {
                                let js = format!(
                                    r#"(function() {{
                                        if (window.auroraview && window.auroraview.trigger) {{
                                            window.auroraview.trigger('tray_menu_click', {{ id: '{}' }});
                                        }}
                                    }})()"#,
                                    id
                                );
                                let _ = wv.evaluate_script(&js);
                            }
                        }
                    }
                }
            }
        }

        // Poll frequently to check if we should show the window (only if auto_show)
        // Also poll to check for Ctrl+C signal and tray events
        if (auto_show && !window_shown) || tray_enabled {
            *control_flow = ControlFlow::Poll;
        } else {
            // Use WaitUntil with a timeout to periodically check for Ctrl+C
            *control_flow = ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(100),
            );
        }

        // Keep webview and tray alive
        let _ = &webview;
        let _ = &tray_manager;

        // Show window after delay (once) - only if auto_show is enabled and not headless
        if !headless && auto_show && !window_shown && std::time::Instant::now() >= show_time {
            tracing::info!("[Standalone] Loading screen should be rendered, showing window now!");
            window.set_visible(true);
            window.request_redraw();
            window_shown = true;
            window_visible = true;
        }

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            if hide_on_close && tray_enabled {
                // Hide to tray instead of closing
                tracing::info!("[Standalone] Window close requested, hiding to tray");
                window.set_visible(false);
                window_visible = false;
            } else {
                tracing::info!("[Standalone] Window close requested, exiting");
                // Set Exit control flow - WebView and Window will be dropped automatically
                // This helps avoid the Chrome_WidgetWin_0 unregister error
                *control_flow = ControlFlow::Exit;
            }
        }
    });

    tracing::info!("[Standalone] Event loop exited normally");
    Ok(())
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
