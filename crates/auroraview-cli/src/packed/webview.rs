//! WebView creation and event loop
//!
//! Handles creating the WebView window and running the main event loop.

use anyhow::{Context, Result};
use auroraview_core::assets::{build_error_page, build_packed_init_script, get_loading_html};
use auroraview_core::plugins::{PathScope, PluginRequest, ScopeConfig, ShellScope};
use auroraview_core::protocol::MemoryAssets;
use auroraview_pack::{OverlayData, PackMode, PackedMetrics};
use auroraview_plugins::PluginRouter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use crate::{load_window_icon, load_window_icon_from_bytes, normalize_url};

use super::backend::{start_python_backend_with_ipc, PythonBackend};
use super::events::UserEvent;
use super::utils::{escape_json_for_js, get_webview_data_dir};
#[cfg(target_os = "windows")]
use super::utils::{get_extensions_dir, has_extensions};

/// Regex pattern for valid handler names: alphanumeric, underscore, dot, colon, hyphen
/// This prevents injection attacks via malicious handler names
static VALID_HANDLER_PATTERN: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^[A-Za-z0-9_\.:\-]+$").unwrap());

/// Build JavaScript to register API methods in frontend
///
/// Groups handlers by namespace (e.g., "api.get_samples" -> namespace "api", method "get_samples")
/// and generates a script that calls `window.auroraview._registerApiMethods()` for each namespace.
///
/// Security: Uses serde_json for proper escaping and validates handler names against a whitelist pattern.
fn build_api_registration_script(handlers: &[String]) -> String {
    if handlers.is_empty() {
        return String::new();
    }

    // Group handlers by namespace, validating each handler name
    let mut namespaces: HashMap<String, Vec<String>> = HashMap::new();
    for handler in handlers {
        // Validate handler name against whitelist pattern
        if !VALID_HANDLER_PATTERN.is_match(handler) {
            tracing::warn!(
                "[Rust] Skipping invalid handler name (must match [A-Za-z0-9_.:-]+): {}",
                handler
            );
            continue;
        }

        if let Some(dot_pos) = handler.find('.') {
            let namespace = &handler[..dot_pos];
            let method = &handler[dot_pos + 1..];
            namespaces
                .entry(namespace.to_string())
                .or_default()
                .push(method.to_string());
        }
    }

    if namespaces.is_empty() {
        return String::new();
    }

    // Generate registration script using serde_json for safe escaping
    let mut script = String::from(
        "(function() {\n\
        if (!window.auroraview || !window.auroraview._registerApiMethods) {\n\
            console.warn('[AuroraView] Event bridge not ready for API registration');\n\
            return;\n\
        }\n",
    );

    for (namespace, methods) in &namespaces {
        // Use serde_json for proper JS string escaping (handles quotes, backslashes, unicode, etc.)
        let namespace_json =
            serde_json::to_string(namespace).unwrap_or_else(|_| "\"\"".to_string());
        let methods_json = serde_json::to_string(&methods).unwrap_or_else(|_| "[]".to_string());

        script.push_str(&format!(
            "window.auroraview._registerApiMethods({}, {});\n",
            namespace_json, methods_json
        ));
        tracing::debug!(
            "[Rust] Registering {} methods in namespace '{}'",
            methods.len(),
            namespace
        );
    }

    script.push_str("})()");
    script
}

/// Handle extension resource requests in the custom protocol
///
/// Maps URLs like `https://auroraview.localhost/extension/{extensionId}/{path}`
/// to local files in `%LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}`
#[cfg(target_os = "windows")]
fn handle_extension_resource_request(
    ext_path: &str,
    allowed_origin: &str,
) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    use mime_guess::from_path;
    use std::borrow::Cow;

    tracing::debug!("[Protocol] extension resource request: {}", ext_path);

    // Parse extension ID and resource path
    // Format: {extensionId}/{path/to/resource}
    let parts: Vec<&str> = ext_path.splitn(2, '/').collect();
    if parts.is_empty() {
        tracing::warn!("[Protocol] Invalid extension path: {}", ext_path);
        return wry::http::Response::builder()
            .status(400)
            .body(Cow::Borrowed(b"Bad Request: Invalid extension path" as &[u8]))
            .unwrap();
    }

    let extension_id = parts[0];
    let resource_path = if parts.len() > 1 {
        parts[1]
    } else {
        "index.html"
    };

    // Get the extensions directory
    let extensions_dir = get_extensions_dir();

    // Build full path to the resource
    let full_path = extensions_dir.join(extension_id).join(resource_path);

    tracing::debug!(
        "[Protocol] Extension resource: {} -> {:?}",
        ext_path,
        full_path
    );

    // Security check: ensure the path is within the extension directory
    let canonical_ext_dir = match extensions_dir.join(extension_id).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                "[Protocol] Extension directory not found: {} ({})",
                extension_id,
                e
            );
            return wry::http::Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Extension not found" as &[u8]))
                .unwrap();
        }
    };

    let canonical_full_path = match full_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                "[Protocol] Extension resource not found: {:?} ({})",
                full_path,
                e
            );
            return wry::http::Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Resource not found" as &[u8]))
                .unwrap();
        }
    };

    // Verify the resource is within the extension directory (prevent directory traversal)
    if !canonical_full_path.starts_with(&canonical_ext_dir) {
        tracing::warn!(
            "[Protocol] Directory traversal attempt in extension: {:?}",
            full_path
        );
        return wry::http::Response::builder()
            .status(403)
            .body(Cow::Borrowed(b"Forbidden: Directory traversal" as &[u8]))
            .unwrap();
    }

    // Read and serve the file
    match std::fs::read(&full_path) {
        Ok(data) => {
            let mime_type = from_path(&full_path).first_or_octet_stream().to_string();
            tracing::debug!(
                "[Protocol] Loaded extension resource: {} ({} bytes, {})",
                ext_path,
                data.len(),
                mime_type
            );

            wry::http::Response::builder()
                .status(200)
                .header("Content-Type", mime_type)
                .header("Access-Control-Allow-Origin", allowed_origin)
                .body(Cow::Owned(data))
                .unwrap()
        }
        Err(e) => {
            tracing::warn!(
                "[Protocol] Failed to read extension resource: {:?} ({})",
                full_path,
                e
            );
            wry::http::Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Resource not found" as &[u8]))
                .unwrap()
        }
    }
}

/// Handle window commands from JavaScript
///
/// These commands are handled directly by Rust since the window is controlled by Rust.
/// Supported commands:
/// - close: Close the window and exit the application
/// - (future) show, hide, minimize, maximize, etc.
fn handle_window_command(
    method: &str,
    _params: &serde_json::Value,
    proxy: &EventLoopProxy<UserEvent>,
) -> Result<serde_json::Value, String> {
    tracing::info!("[Rust] Window command: {}", method);

    match method {
        "close" => {
            tracing::info!("[Rust] Closing window via window.close() API");
            // Send close event to event loop
            if let Err(e) = proxy.send_event(UserEvent::CloseWindow) {
                tracing::error!("[Rust] Failed to send CloseWindow event: {}", e);
                return Err(format!("Failed to close window: {}", e));
            }
            Ok(serde_json::json!({"success": true}))
        }
        // Future: Add more window commands here
        // "show" => { ... }
        // "hide" => { ... }
        // "minimize" => { ... }
        // "maximize" => { ... }
        // "setTitle" => { ... }
        // etc.
        _ => {
            tracing::warn!("[Rust] Unknown window command: {}", method);
            Err(format!("Unknown window command: {}", method))
        }
    }
}

/// Handle IPC message from WebView
///
/// This function routes messages to either:
/// 1. PluginRouter - for native plugin commands (plugin:*, shell, process, etc.)
/// 2. Python backend - for application-specific API calls (api.*)
fn handle_ipc_message(
    body: &str,
    python_backend: Option<&Arc<PythonBackend>>,
    plugin_router: &Arc<RwLock<PluginRouter>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    tracing::debug!("[Rust] IPC message received: {}", body);

    // Parse the message
    let msg: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to parse IPC message: {}", e);
            return;
        }
    };

    let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "call" => {
            // Handle API call
            let id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let method = msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
            let params = msg
                .get("params")
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            tracing::debug!("[Rust] API call: {} (id: {})", method, id);

            // Check if this is a window command (window.*)
            // These are handled directly by Rust since Rust controls the window
            if method.starts_with("window.") {
                let window_method = &method[7..]; // Strip "window." prefix
                let result = handle_window_command(window_method, &params, proxy);
                let response = serde_json::json!({
                    "id": id,
                    "ok": result.is_ok(),
                    "result": result.as_ref().ok(),
                    "error": result.as_ref().err().map(|e| serde_json::json!({
                        "name": "WindowError",
                        "message": e.to_string()
                    }))
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(response.to_string()));
            }
            // Check if this is a plugin command (plugin:*)
            else if method.starts_with("plugin:") {
                // Handle via PluginRouter
                if let Some(request) = PluginRequest::from_invoke(method, params) {
                    // Use map_err to handle lock poisoning gracefully
                    let response = match plugin_router.read() {
                        Ok(router) => router.handle(request),
                        Err(e) => {
                            tracing::error!("Plugin router lock poisoned: {}", e);
                            let error_response = serde_json::json!({
                                "id": id,
                                "ok": false,
                                "result": null,
                                "error": {
                                    "name": "InternalError",
                                    "message": "Plugin router unavailable"
                                }
                            });
                            let _ = proxy
                                .send_event(UserEvent::PythonResponse(error_response.to_string()));
                            return;
                        }
                    };

                    // Send response back via event loop
                    let result = serde_json::json!({
                        "id": id,
                        "ok": response.success,
                        "result": response.data,
                        "error": response.error
                    });
                    let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
                } else {
                    tracing::warn!("Invalid plugin command format: {}", method);
                }
            } else if let Some(backend) = python_backend {
                // Forward to Python backend for api.* calls
                let request = serde_json::json!({
                    "id": id,
                    "method": method,
                    "params": params
                });

                if let Err(e) = backend.send_request(&request.to_string()) {
                    tracing::error!("Failed to send request to Python: {}", e);
                    // Send error response back to WebView so it doesn't hang
                    let error_response = serde_json::json!({
                        "id": id,
                        "ok": false,
                        "result": null,
                        "error": {
                            "name": "PythonBackendError",
                            "message": format!("{}", e)
                        }
                    });
                    let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
                }
            } else {
                tracing::warn!("No Python backend available for API call: {}", method);
                // Send error response for missing backend
                let error_response = serde_json::json!({
                    "id": id,
                    "ok": false,
                    "result": null,
                    "error": {
                        "name": "NoPythonBackend",
                        "message": "No Python backend available for API calls"
                    }
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
            }
        }
        "plugin" => {
            // Direct plugin invocation (alternative format)
            let plugin = msg.get("plugin").and_then(|v| v.as_str()).unwrap_or("");
            let command = msg.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let args = msg.get("args").cloned().unwrap_or(serde_json::Value::Null);
            let id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            tracing::debug!("Plugin call: {}|{} (id: {})", plugin, command, id);

            let request = PluginRequest::new(plugin, command, args);
            // Use map_err to handle lock poisoning gracefully
            let response = match plugin_router.read() {
                Ok(router) => router.handle(request),
                Err(e) => {
                    tracing::error!("Plugin router lock poisoned: {}", e);
                    let error_response = serde_json::json!({
                        "id": id,
                        "ok": false,
                        "result": null,
                        "error": {
                            "name": "InternalError",
                            "message": "Plugin router unavailable"
                        }
                    });
                    let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
                    return;
                }
            };

            // Send response back via event loop
            let result = serde_json::json!({
                "id": id,
                "ok": response.success,
                "result": response.data,
                "error": response.error
            });
            let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
        }
        "event" => {
            // Handle event (fire-and-forget)
            let event = msg.get("event").and_then(|v| v.as_str()).unwrap_or("");
            tracing::debug!("Event received: {}", event);

            // Handle loading screen ready event
            if event == "loading_screen_ready" {
                tracing::info!("[Rust] Received loading_screen_ready event from WebView");
                let _ = proxy.send_event(UserEvent::LoadingScreenReady);
            }
            // Handle navigate to app event (frontend requests navigation)
            else if event == "navigate_to_app" {
                tracing::info!("[Rust] Received navigate_to_app event from WebView");
                let _ = proxy.send_event(UserEvent::NavigateToApp);
            }
            // Handle page ready event (auroraview bridge initialized after navigation)
            else if event == "__auroraview_ready" {
                tracing::info!(
                    "[Rust] Received __auroraview_ready event - page bridge initialized"
                );
                let _ = proxy.send_event(UserEvent::PageReady);
            }
        }
        "set_html" => {
            // Handle set_html command from Python (for dynamic HTML like Browser component)
            let html = msg
                .get("html")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = msg.get("title").and_then(|v| v.as_str()).map(String::from);

            tracing::info!(
                "[Rust] Received set_html command (html_len: {}, title: {:?})",
                html.len(),
                title
            );

            let _ = proxy.send_event(UserEvent::SetHtml { html, title });
        }
        "invoke" => {
            // Handle Tauri-style invoke command (used by frontend SDK)
            // Format: {"type": "invoke", "id": "...", "cmd": "plugin:name|command", "args": {...}}
            let id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let cmd = msg.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
            let args = msg.get("args").cloned().unwrap_or(serde_json::Value::Null);

            tracing::debug!("[Rust] Invoke command: {} (id: {})", cmd, id);

            // Handle plugin commands (plugin:name|command format)
            if cmd.starts_with("plugin:") {
                // Parse plugin:name|command format
                let plugin_part = cmd.strip_prefix("plugin:").unwrap_or("");
                if let Some((plugin, command)) = plugin_part.split_once('|') {
                    tracing::debug!(
                        "[Rust] Plugin invoke: plugin={}, command={}, id={}",
                        plugin,
                        command,
                        id
                    );

                    let request = PluginRequest::new(plugin, command, args);
                    let response = match plugin_router.read() {
                        Ok(router) => router.handle(request),
                        Err(e) => {
                            tracing::error!("Plugin router lock poisoned: {}", e);
                            let error_response = serde_json::json!({
                                "id": id,
                                "ok": false,
                                "result": null,
                                "error": {
                                    "name": "InternalError",
                                    "message": "Plugin router unavailable"
                                }
                            });
                            let _ = proxy
                                .send_event(UserEvent::PythonResponse(error_response.to_string()));
                            return;
                        }
                    };

                    // Send response back via event loop
                    let result = serde_json::json!({
                        "id": id,
                        "ok": response.success,
                        "result": response.data,
                        "error": response.error
                    });
                    let _ = proxy.send_event(UserEvent::PythonResponse(result.to_string()));
                } else {
                    tracing::warn!("Invalid invoke plugin command format: {}", cmd);
                    let error_response = serde_json::json!({
                        "id": id,
                        "ok": false,
                        "result": null,
                        "error": {
                            "name": "InvalidFormat",
                            "message": format!("Invalid plugin command format: {}", cmd)
                        }
                    });
                    let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
                }
            } else {
                tracing::warn!("Unsupported invoke command: {}", cmd);
                let error_response = serde_json::json!({
                    "id": id,
                    "ok": false,
                    "result": null,
                    "error": {
                        "name": "UnsupportedCommand",
                        "message": format!("Unsupported invoke command: {}", cmd)
                    }
                });
                let _ = proxy.send_event(UserEvent::PythonResponse(error_response.to_string()));
            }
        }
        _ => {
            tracing::warn!("Unknown IPC message type: {}", msg_type);
        }
    }
}

/// Run WebView from overlay data
///
/// Note: This function uses event_loop.run() which never returns.
/// It will call std::process::exit() when the window closes.
#[allow(unreachable_code)]
pub fn run_packed_webview(overlay: OverlayData, mut metrics: PackedMetrics) -> Result<()> {
    let config = &overlay.config;
    let is_fullstack = matches!(config.mode, PackMode::FullStack { .. });

    tracing::info!(
        "[Rust] Pack mode: {:?}, is_fullstack: {}",
        config.mode,
        is_fullstack
    );

    // Create event loop with user event support
    #[cfg(target_os = "windows")]
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::<UserEvent>::with_user_event()
        .with_any_thread(true)
        .build();

    #[cfg(not(target_os = "windows"))]
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();

    // Create PluginRouter with all built-in plugins and secure default scope
    // Using create_router_with_scope() ensures all plugins (fs, clipboard, shell, dialog,
    // process, browser_bridge, extensions) are registered with the router.
    let default_scope = {
        // Get the current working directory as the base for file system access
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Build fs scope with cwd
        let mut fs_scope = PathScope::new().allow(&cwd);

        // On Windows, also allow access to extensions directory
        #[cfg(target_os = "windows")]
        {
            let extensions_dir = get_extensions_dir();
            fs_scope = fs_scope.allow(&extensions_dir);
        }

        // Create a restricted scope configuration
        ScopeConfig::new()
            .with_fs_scope(fs_scope)
            .with_shell_scope(ShellScope::new()) // Allows open_url/open_file but not arbitrary commands
    };

    let plugin_router = Arc::new(RwLock::new(auroraview_plugins::create_router_with_scope(
        default_scope,
    )));

    // Set up event callback for plugins to emit events to WebView
    let proxy_for_events = proxy.clone();
    {
        // Handle lock poisoning gracefully during initialization
        if let Ok(router) = plugin_router.read() {
            router.set_event_callback(Arc::new(
                move |event_name: &str, data: serde_json::Value| {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let _ = proxy_for_events.send_event(UserEvent::PluginEvent {
                        event: event_name.to_string(),
                        data: data_str,
                    });
                },
            ));
        } else {
            tracing::error!("Failed to set event callback: plugin router lock poisoned");
        }
    }

    // For FullStack mode, start Python backend BEFORE creating window
    // This allows Python to initialize while the window is being created
    // We'll show a loading screen while waiting for Python to be ready
    let python_backend = if let PackMode::FullStack { ref python, .. } = config.mode {
        tracing::info!("Starting Python backend before window creation...");
        Some(start_python_backend_with_ipc(
            &overlay,
            python,
            proxy.clone(),
            &mut metrics,
        )?)
    } else {
        None
    };

    // Track loading state for FullStack mode
    // We need both loading screen ready AND Python ready before navigating
    let loading_screen_ready = Arc::new(RwLock::new(false));
    let python_ready = Arc::new(RwLock::new(false));
    let waiting_for_python = Arc::new(RwLock::new(python_backend.is_some()));
    // Store registered handlers for API method registration
    let registered_handlers: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    // Ready timeout guard for packed FullStack (avoid infinite loading)
    // Two-stage timeout:
    // 1. First timeout (10s): Show warning, allow Python to continue initializing
    // 2. Final timeout (20s): Show error page if still not ready
    if python_backend.is_some() {
        let python_ready_for_timeout = python_ready.clone();
        let proxy_for_timeout = proxy.clone();
        std::thread::spawn(move || {
            // First timeout: 10 seconds
            std::thread::sleep(std::time::Duration::from_secs(10));
            let is_ready = python_ready_for_timeout.read().map(|r| *r).unwrap_or(false);
            if !is_ready {
                tracing::warn!("[Rust] Python backend not ready after 10s, showing warning...");
                let _ = proxy_for_timeout.send_event(UserEvent::BackendError {
                    message: "Python backend initialization taking longer than expected...".to_string(),
                    source: "startup".to_string(),
                });
                let _ = proxy_for_timeout.send_event(UserEvent::LoadingUpdate {
                    progress: None,
                    text: Some("Backend initialization slow, please wait...".to_string()),
                    step_id: None,
                    step_text: None,
                    step_status: None,
                });
                
                // Second timeout: additional 20 seconds (total 30s)
                std::thread::sleep(std::time::Duration::from_secs(20));
                let is_ready_final = python_ready_for_timeout.read().map(|r| *r).unwrap_or(false);
                if !is_ready_final {
                    tracing::error!("[Rust] Python backend ready timeout after 30s, showing error page");
                    let _ = proxy_for_timeout.send_event(UserEvent::ShowError {
                        code: 503,
                        title: "Backend Initialization Failed".to_string(),
                        message: "The Python backend failed to initialize within the expected time.\n\nThis could be caused by:\n- Missing Python dependencies\n- Syntax errors in your application code\n- Import errors in your modules".to_string(),
                        details: Some("The backend process may have crashed or is stuck.\nCheck the console output for more details.".to_string()),
                        source: "python".to_string(),
                    });
                    // Also send empty PythonReady to unblock any waiting code
                    let _ = proxy_for_timeout.send_event(UserEvent::PythonReady {
                        handlers: Vec::new(),
                    });
                }
            }
        });
    }

    // Create window
    let mut window_builder = tao::window::WindowBuilder::new()
        .with_title(&config.window.title)
        .with_inner_size(tao::dpi::LogicalSize::new(
            config.window.width,
            config.window.height,
        ))
        .with_resizable(config.window.resizable)
        .with_decorations(!config.window.frameless)
        .with_transparent(config.window.transparent)
        .with_always_on_top(config.window.always_on_top);

    // Set minimum size if specified
    if let (Some(min_w), Some(min_h)) = (config.window.min_width, config.window.min_height) {
        window_builder =
            window_builder.with_min_inner_size(tao::dpi::LogicalSize::new(min_w, min_h));
    }

    // Set window icon (custom from config or default)
    let icon = if let Some(ref icon_data) = config.window_icon {
        tracing::info!("Using custom window icon ({} bytes)", icon_data.len());
        load_window_icon_from_bytes(icon_data)
    } else {
        tracing::info!("Using default window icon");
        load_window_icon()
    };
    if let Some(icon) = icon {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let window = window_builder
        .build(&event_loop)
        .with_context(|| "Failed to create window")?;

    metrics.mark_window_created();

    // Create WebContext
    let data_dir = get_webview_data_dir();
    let mut web_context = WebContext::new(Some(data_dir));

    // Build initialization script with JS bridge
    // Note: API methods are registered dynamically by Python backend
    let init_script = build_packed_init_script();

    // Clone for IPC handler
    let python_backend_arc = python_backend.map(Arc::new);
    let python_backend_for_ipc = python_backend_arc.clone();
    let plugin_router_for_ipc = plugin_router.clone();
    let proxy_for_ipc = proxy.clone();

    // Create WebView based on pack mode
    let webview = match &config.mode {
        PackMode::Url { url } => {
            let normalized_url = normalize_url(url)?;
            tracing::info!("Loading URL: {}", normalized_url);

            WryWebViewBuilder::new_with_web_context(&mut web_context)
                .with_url(&normalized_url)
                .with_initialization_script(&init_script)
                .with_ipc_handler(move |request| {
                    handle_ipc_message(
                        request.body(),
                        python_backend_for_ipc.as_ref(),
                        &plugin_router_for_ipc,
                        &proxy_for_ipc,
                    );
                })
                .with_devtools(config.debug)
                .build(&window)
                .with_context(|| "Failed to create WebView")?
        }
        PackMode::Frontend { .. } | PackMode::FullStack { .. } => {
            // Create MemoryAssets from overlay assets
            let mut memory_assets = MemoryAssets::from_vec(overlay.assets.clone())
                .with_loading_html(get_loading_html());

            // Add auroraview-assets resources for loading/error pages
            // These are required because loading HTML references relative JS/CSS paths
            for path in auroraview_assets::list_assets() {
                if let Some(data) = auroraview_assets::get_asset(&path) {
                    memory_assets.insert(path, data.into_owned());
                }
            }
            tracing::debug!(
                "Added {} auroraview-assets resources to MemoryAssets",
                auroraview_assets::list_assets().len()
            );

            // Find index.html path for logging
            let all_paths = memory_assets.list_paths();
            let index_path = all_paths
                .iter()
                .find(|path| {
                    **path == "index.html"
                        || **path == "frontend/index.html"
                        || path.ends_with("/index.html")
                })
                .map(|p| (*p).clone())
                .unwrap_or_else(|| "index.html".to_string());

            tracing::info!("Loading embedded assets via auroraview:// protocol");
            tracing::info!("Index path: {}", index_path);
            
            // Debug: Print all available assets for troubleshooting
            tracing::info!("[DEBUG] Available assets ({} total):", all_paths.len());
            for (i, path) in all_paths.iter().enumerate() {
                if i < 50 {
                    // Limit to first 50 to avoid log spam
                    tracing::info!("  [{}] {}", i, path);
                } else if i == 50 {
                    tracing::info!("  ... and {} more", all_paths.len() - 50);
                    break;
                }
            }

            // For FullStack mode, show loading screen while waiting for Python
            // For Frontend mode, load index.html directly
            //
            // URL format depends on platform:
            // - Windows with with_https_scheme(true): https://auroraview.localhost/path
            // - Other platforms: auroraview://localhost/path
            #[cfg(target_os = "windows")]
            let (initial_url, _app_url) = if is_fullstack {
                tracing::info!("FullStack mode: showing loading screen while Python initializes");
                (
                    "https://auroraview.localhost/__loading__",
                    "https://auroraview.localhost/index.html",
                )
            } else {
                (
                    "https://auroraview.localhost/index.html",
                    "https://auroraview.localhost/index.html",
                )
            };

            #[cfg(not(target_os = "windows"))]
            let (initial_url, _app_url) = if is_fullstack {
                tracing::info!("FullStack mode: showing loading screen while Python initializes");
                (
                    "auroraview://localhost/__loading__",
                    "auroraview://localhost/index.html",
                )
            } else {
                (
                    "auroraview://localhost/index.html",
                    "auroraview://localhost/index.html",
                )
            };

            tracing::info!("Initial URL: {}", initial_url);
            tracing::info!("App URL (after Python ready): {}", _app_url);
            tracing::info!("Available assets: {} total", memory_assets.len());

            #[allow(unused_mut)]
            let mut builder = WryWebViewBuilder::new_with_web_context(&mut web_context);

            // On Windows, use HTTPS scheme for secure context support
            // This is required for custom protocols to work correctly
            #[cfg(target_os = "windows")]
            let builder = builder.with_https_scheme(true);

            // Enable browser extensions if the extensions directory exists and has extensions
            #[cfg(target_os = "windows")]
            let builder = {
                let ext_dir = get_extensions_dir();
                tracing::info!("[packed] Extensions directory: {}", ext_dir.display());

                // List all entries in extensions directory for debugging
                if ext_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&ext_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let manifest_exists = path.join("manifest.json").exists();
                            tracing::info!(
                                "[packed] Extension entry: {} (is_dir={}, has_manifest={})",
                                path.display(),
                                path.is_dir(),
                                manifest_exists
                            );
                        }
                    }
                }

                if has_extensions() {
                    tracing::info!(
                        "[packed] Enabling browser extensions from: {}",
                        ext_dir.display()
                    );
                    // Create extensions directory if needed
                    let _ = std::fs::create_dir_all(&ext_dir);
                    builder
                        .with_browser_extensions_enabled(true)
                        .with_extensions_path(ext_dir)
                } else {
                    tracing::warn!(
                        "[packed] No valid extensions found in: {}",
                        ext_dir.display()
                    );
                    builder
                }
            };

            // Set remote debugging port for CDP (Chrome DevTools Protocol) connections
            // This allows Playwright/Puppeteer to connect to WebView2
            #[cfg(target_os = "windows")]
            let builder = if let Some(port) = config.remote_debugging_port {
                let args = format!("--remote-debugging-port={}", port);
                tracing::info!("[packed] Set WebView2 remote debugging port: {}", port);
                builder.with_additional_browser_args(&args)
            } else {
                builder
            };

            builder
                .with_custom_protocol("auroraview".to_string(), move |_webview_id, request| {
                    let uri = request.uri();
                    let path = uri.path().trim_start_matches('/');

                    // Security: Restrict CORS to same-origin only
                    // Using the custom protocol origin prevents cross-origin access from external sites
                    #[cfg(target_os = "windows")]
                    let allowed_origin = "https://auroraview.localhost";
                    #[cfg(not(target_os = "windows"))]
                    let allowed_origin = "auroraview://localhost";

                    // Handle extension resources: /extension/{extensionId}/{path}
                    // Maps to %LOCALAPPDATA%/AuroraView/Extensions/{extensionId}/{path}
                    #[cfg(target_os = "windows")]
                    if let Some(ext_path) = path.strip_prefix("extension/") {
                        return handle_extension_resource_request(ext_path, allowed_origin);
                    }

                    // Use MemoryAssets to handle the request
                    let response = memory_assets.handle_request(path);

                    wry::http::Response::builder()
                        .status(response.status)
                        .header("Content-Type", &response.mime_type)
                        .header("Access-Control-Allow-Origin", allowed_origin)
                        .body(response.data.into_owned().into())
                        .unwrap_or_else(|_| {
                            wry::http::Response::builder()
                                .status(500)
                                .body(Vec::new().into())
                                .unwrap()
                        })
                })
                .with_initialization_script(&init_script)
                .with_ipc_handler(move |request| {
                    handle_ipc_message(
                        request.body(),
                        python_backend_for_ipc.as_ref(),
                        &plugin_router_for_ipc,
                        &proxy_for_ipc,
                    );
                })
                // Use with_url to load from custom protocol, avoiding CORS issues
                .with_url(initial_url)
                .with_devtools(config.debug)
                .build(&window)
                .with_context(|| "Failed to create WebView")?
        }
    };

    metrics.mark_webview_created();
    metrics.mark_total();

    // Log performance report
    tracing::info!(
        "Startup completed in {:.2}ms",
        metrics.elapsed().as_secs_f64() * 1000.0
    );
    // Always log performance report for debugging startup issues
    metrics.log_report();

    // Start process monitor thread to detect Python crashes
    // This thread checks if Python process is still alive every 500ms
    // If Python crashes, it sends a PythonCrash event to display error page
    if let Some(ref backend) = python_backend_arc {
        let backend_for_monitor = Arc::clone(backend);
        let proxy_for_monitor = proxy.clone();
        let python_ready_for_monitor = python_ready.clone();
        std::thread::spawn(move || {
            // Wait a bit for initial startup
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                // Check if shutdown has been initiated
                if backend_for_monitor.is_shutting_down() {
                    tracing::debug!("[ProcessMonitor] Shutdown detected, stopping monitor");
                    break;
                }
                
                // Check if process is still alive
                if !backend_for_monitor.is_alive() {
                    let exit_code = backend_for_monitor.get_exit_code();
                    let stderr_output = backend_for_monitor.get_last_stderr();
                    let during_startup = !python_ready_for_monitor.read().map(|r| *r).unwrap_or(false);
                    
                    tracing::error!(
                        "[ProcessMonitor] Python process crashed! exit_code={:?}, during_startup={}, stderr_len={}",
                        exit_code,
                        during_startup,
                        stderr_output.len()
                    );
                    
                    // Send crash event
                    let _ = proxy_for_monitor.send_event(UserEvent::PythonCrash {
                        exit_code,
                        stderr_output,
                        during_startup,
                    });
                    break;
                }
            }
            tracing::info!("[ProcessMonitor] Monitor thread exiting");
        });
    }

    // Wrap webview and window in Rc<RefCell> for single-threaded access
    // Note: WebView is not Send+Sync, so we use Rc instead of Arc
    use std::cell::RefCell;
    use std::rc::Rc;
    let webview = Rc::new(RefCell::new(webview));
    let webview_for_event = webview.clone();
    let window = Rc::new(RefCell::new(window));
    let window_for_event = window.clone();

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                // Kill all managed processes on close
                if let Ok(router) = plugin_router.read() {
                    let request = PluginRequest::new("process", "kill_all", serde_json::json!({}));
                    let _ = router.handle(request);
                }
                // Gracefully shutdown Python backend using ipckit
                if let Some(ref backend) = python_backend_arc {
                    tracing::info!("Stopping Python backend...");
                    backend.shutdown();
                }
                *control_flow = ControlFlow::Exit;
            }
            tao::event::Event::UserEvent(user_event) => {
                if let Ok(wv) = webview_for_event.try_borrow() {
                    // Helper function to navigate to app
                    let do_navigate = |wv: &wry::WebView| {
                        #[cfg(target_os = "windows")]
                        let target_url = "https://auroraview.localhost/index.html";
                        #[cfg(not(target_os = "windows"))]
                        let target_url = "auroraview://localhost/index.html";

                        tracing::info!("[Rust] Navigating to: {}", target_url);
                        match wv.load_url(target_url) {
                            Ok(_) => {
                                tracing::info!("[Rust] Navigation initiated successfully");
                            }
                            Err(e) => {
                                tracing::error!("[Rust] Failed to navigate to application: {}", e);
                            }
                        }
                    };

                    match user_event {
                        UserEvent::LoadingScreenReady => {
                            tracing::info!("[Rust] Loading screen is ready (DOM rendered)");
                            if let Ok(mut ready) = loading_screen_ready.write() {
                                *ready = true;
                            }
                            // If Python is already ready, send backend_ready event to frontend
                            let is_python_ready = python_ready.read().map(|r| *r).unwrap_or(false);
                            if is_python_ready {
                                tracing::info!(
                                    "[Rust] Python already ready, sending backend_ready to frontend"
                                );
                                let script = r#"
                                    (function() {
                                        if (window.auroraview && window.auroraview.trigger) {
                                            window.auroraview.trigger('backend_ready', { ready: true });
                                        }
                                        // Also dispatch a custom event for the loading page
                                        window.dispatchEvent(new CustomEvent('auroraview:backend_ready', { detail: { ready: true } }));
                                    })()
                                "#;
                                let _ = wv.evaluate_script(script);
                            }
                        }
                        UserEvent::PythonReady { handlers } => {
                            // Python backend is ready
                            tracing::info!("[Rust] ========================================");
                            tracing::info!(
                                "[Rust] Python backend ready with {} handlers",
                                handlers.len()
                            );
                            if let Ok(mut ready) = python_ready.write() {
                                *ready = true;
                            }

                            // Store handlers for later use (e.g., after navigation)
                            if let Ok(mut stored) = registered_handlers.write() {
                                *stored = handlers.clone();
                            }

                            // Register API methods in frontend
                            let register_script = build_api_registration_script(&handlers);
                            if !register_script.is_empty() {
                                let _ = wv.evaluate_script(&register_script);
                            }

                            // If loading screen is ready, send backend_ready event to frontend
                            let is_loading_ready = loading_screen_ready.read().map(|r| *r).unwrap_or(false);
                            if is_loading_ready {
                                tracing::info!(
                                    "[Rust] Loading screen ready, sending backend_ready to frontend"
                                );
                                let script = r#"
                                    (function() {
                                        if (window.auroraview && window.auroraview.trigger) {
                                            window.auroraview.trigger('backend_ready', { ready: true });
                                        }
                                        // Also dispatch a custom event for the loading page
                                        window.dispatchEvent(new CustomEvent('auroraview:backend_ready', { detail: { ready: true } }));
                                    })()
                                "#;
                                let _ = wv.evaluate_script(script);
                            }
                        }
                        UserEvent::NavigateToApp => {
                            // Frontend requested navigation to app
                            tracing::info!("[Rust] Frontend requested navigation to app");
                            if let Ok(mut w) = waiting_for_python.write() {
                                *w = false;
                            }
                            do_navigate(&wv);
                        }
                        UserEvent::PageReady => {
                            // Page bridge is ready after navigation
                            // Re-register API methods that were registered during PythonReady
                            tracing::info!("[Rust] Page ready - re-registering API methods");
                            if let Ok(handlers) = registered_handlers.read() {
                                if !handlers.is_empty() {
                                    let register_script = build_api_registration_script(&handlers);
                                    if !register_script.is_empty() {
                                        tracing::info!(
                                            "[Rust] Re-registering {} API handlers",
                                            handlers.len()
                                        );
                                        let _ = wv.evaluate_script(&register_script);
                                    }
                                }
                            }
                        }
                        UserEvent::PythonResponse(response) => {
                            // Send response back to WebView
                            let escaped = escape_json_for_js(&response);
                            let script = format!(
                                r#"(function() {{
                                    if (window.auroraview && window.auroraview.trigger) {{
                                        try {{
                                            var data = JSON.parse("{}");
                                            window.auroraview.trigger('__auroraview_call_result', data);
                                        }} catch (e) {{
                                            console.error('[AuroraView] Failed to parse response:', e);
                                        }}
                                    }}
                                }})()"#,
                                escaped
                            );
                            let _ = wv.evaluate_script(&script);
                        }
                        UserEvent::PluginEvent { event, data } => {
                            // Send plugin event to WebView
                            // The `data` is already a valid JSON string from Python
                            // We insert it directly as a JavaScript object literal
                            tracing::info!(
                                "[Rust:WebView] Received PluginEvent: event={}, data_len={}",
                                event,
                                data.len()
                            );
                            tracing::debug!("[Rust:WebView] Event data: {}", data);
                            // Insert JSON directly as JavaScript object literal (no JSON.parse needed)
                            let script = format!(
                                r#"(function() {{
                                    if (window.auroraview && window.auroraview.trigger) {{
                                        try {{
                                            var data = {};
                                            console.log('[AuroraView] Triggering event:', '{}', data);
                                            window.auroraview.trigger('{}', data);
                                        }} catch (e) {{
                                            console.error('[AuroraView] Failed to process event data:', e);
                                        }}
                                    }} else {{
                                        console.warn('[AuroraView] Bridge not ready for event:', '{}');
                                    }}
                                }})()"#,
                                data, event, event, event
                            );
                            let _ = wv.evaluate_script(&script);
                        }
                        UserEvent::LoadingUpdate {
                            progress,
                            text,
                            step_id,
                            step_text,
                            step_status,
                        } => {
                            // Update loading screen progress and text
                            let mut updates = Vec::new();

                            if let Some(p) = progress {
                                updates.push(format!("window.auroraLoading.setProgress({});", p));
                            }
                            if let Some(t) = text {
                                let escaped_text = t.replace('\\', "\\\\").replace('\'', "\\'");
                                updates.push(format!(
                                    "window.auroraLoading.setText('{}');",
                                    escaped_text
                                ));
                            }
                            if let (Some(id), Some(txt), Some(status)) =
                                (step_id, step_text, step_status)
                            {
                                let escaped_id = id.replace('\\', "\\\\").replace('\'', "\\'");
                                let escaped_txt = txt.replace('\\', "\\\\").replace('\'', "\\'");
                                let escaped_status =
                                    status.replace('\\', "\\\\").replace('\'', "\\'");
                                updates.push(format!(
                                    "window.auroraLoading.setStep('{}', '{}', '{}');",
                                    escaped_id, escaped_txt, escaped_status
                                ));
                            }

                            if !updates.is_empty() {
                                let script = format!(
                                    "(function() {{ if (window.auroraLoading) {{ {} }} }})()",
                                    updates.join(" ")
                                );
                                let _ = wv.evaluate_script(&script);
                            }
                        }
                        UserEvent::BackendError { message, source } => {
                            // Send backend error to frontend for display
                            let escaped_msg = message
                                .replace('\\', "\\\\")
                                .replace('\'', "\\'")
                                .replace('\n', "\\n")
                                .replace('\r', "");
                            let escaped_source = source.replace('\\', "\\\\").replace('\'', "\\'");
                            let script = format!(
                                r#"(function() {{
                                    if (window.auroraLoading && window.auroraLoading.addError) {{
                                        window.auroraLoading.addError('{}', '{}');
                                    }}
                                    // Also trigger event for custom handling
                                    if (window.auroraview && window.auroraview.trigger) {{
                                        window.auroraview.trigger('backend_error', {{
                                            message: '{}',
                                            source: '{}'
                                        }});
                                    }}
                                    console.error('[Backend:{}] {}');
                                }})()"#,
                                escaped_msg, escaped_source, escaped_msg, escaped_source, escaped_source, escaped_msg
                            );
                            let _ = wv.evaluate_script(&script);
                        }
                        UserEvent::SetHtml { html, title } => {
                            // Load dynamic HTML content (for Browser component in packed mode)
                            tracing::info!(
                                "[Rust] SetHtml event: loading {} bytes of HTML",
                                html.len()
                            );

                            // Set window title if provided
                            if let Some(new_title) = title {
                                if let Ok(win) = window_for_event.try_borrow() {
                                    win.set_title(&new_title);
                                    tracing::debug!("[Rust] Window title set to: {}", new_title);
                                }
                            }

                            // Use set_html to load the HTML content
                            // WRY WebView doesn't have a direct set_html method, so we use
                            // navigate to a data URL or use evaluate_script to replace document
                            let escaped_html = html
                                .replace('\\', "\\\\")
                                .replace('`', "\\`")
                                .replace("${", "\\${");

                            let script = format!(
                                r#"(function() {{
                                    document.open();
                                    document.write(`{}`);
                                    document.close();
                                }})()"#,
                                escaped_html
                            );

                            match wv.evaluate_script(&script) {
                                Ok(_) => {
                                    tracing::info!(
                                        "[Rust] SetHtml: Successfully loaded dynamic HTML"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[Rust] SetHtml: Failed to load HTML: {}",
                                        e
                                    );
                                }
                            }
                        }
                        UserEvent::CloseWindow => {
                            // Close window triggered by JavaScript window.close() API
                            tracing::info!("[Rust] CloseWindow event received from JavaScript");

                            // Kill all managed processes on close
                            if let Ok(router) = plugin_router.read() {
                                let request =
                                    PluginRequest::new("process", "kill_all", serde_json::json!({}));
                                let _ = router.handle(request);
                            }
                            // Gracefully shutdown Python backend using ipckit
                            if let Some(ref backend) = python_backend_arc {
                                tracing::info!("Stopping Python backend...");
                                backend.shutdown();
                            }
                            *control_flow = ControlFlow::Exit;
                        }
                        UserEvent::PythonCrash {
                            exit_code,
                            stderr_output,
                            during_startup,
                        } => {
                            // Python process crashed - display error page
                            tracing::error!(
                                "[Rust] PythonCrash: exit_code={:?}, during_startup={}, stderr_len={}",
                                exit_code,
                                during_startup,
                                stderr_output.len()
                            );

                            // Build error message
                            let exit_info = match exit_code {
                                Some(code) => format!("Process exited with code: {}", code),
                                None => "Process terminated unexpectedly".to_string(),
                            };

                            let title = if during_startup {
                                "Python Backend Failed to Start"
                            } else {
                                "Python Backend Crashed"
                            };

                            let message = if during_startup {
                                format!(
                                    "The Python backend failed to start properly.\n\n{}\n\nCommon causes:\n- Syntax errors in Python code\n- Missing dependencies\n- Import errors\n- Invalid configuration",
                                    exit_info
                                )
                            } else {
                                format!(
                                    "The Python backend has unexpectedly terminated.\n\n{}\n\nThis may be caused by:\n- Unhandled exceptions\n- Memory issues\n- External termination",
                                    exit_info
                                )
                            };

                            // Format stderr as details
                            let details = if stderr_output.is_empty() {
                                Some("No error output captured.".to_string())
                            } else {
                                // Limit stderr to last 50 lines for display
                                let lines: Vec<&str> = stderr_output.lines().collect();
                                let truncated = if lines.len() > 50 {
                                    format!(
                                        "... ({} lines truncated)\n{}",
                                        lines.len() - 50,
                                        lines[lines.len() - 50..].join("\n")
                                    )
                                } else {
                                    stderr_output.clone()
                                };
                                Some(truncated)
                            };

                            // Build and display error page
                            let error_html = build_error_page(
                                500,
                                title,
                                &message,
                                details.as_deref(),
                                None, // No retry URL for crashes
                            );

                            // Load error page using document.write
                            let escaped_html = error_html
                                .replace('\\', "\\\\")
                                .replace('`', "\\`")
                                .replace("${", "\\${");

                            let script = format!(
                                r#"(function() {{
                                    document.open();
                                    document.write(`{}`);
                                    document.close();
                                }})()"#,
                                escaped_html
                            );

                            match wv.evaluate_script(&script) {
                                Ok(_) => {
                                    tracing::info!(
                                        "[Rust] PythonCrash: Error page displayed successfully"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[Rust] PythonCrash: Failed to display error page: {}",
                                        e
                                    );
                                }
                            }
                        }
                        UserEvent::ShowError {
                            code,
                            title,
                            message,
                            details,
                            source,
                        } => {
                            // Navigate to error page with full diagnostics
                            tracing::error!(
                                "[Rust] ShowError: {} - {} (source: {})",
                                code, title, source
                            );

                            // Build and display error page
                            let error_html = build_error_page(
                                code,
                                &title,
                                &message,
                                details.as_deref(),
                                None, // No retry URL
                            );

                            // Add source info via JavaScript
                            let error_html_with_source = error_html.replace(
                                "</body>",
                                &format!(
                                    r#"<script>
                                    if (window._errorInfo) {{
                                        window._errorInfo.source = '{}';
                                    }}
                                    </script></body>"#,
                                    source
                                ),
                            );

                            // Load error page using document.write
                            let escaped_html = error_html_with_source
                                .replace('\\', "\\\\")
                                .replace('`', "\\`")
                                .replace("${", "\\${");

                            let script = format!(
                                r#"(function() {{
                                    document.open();
                                    document.write(`{}`);
                                    document.close();
                                }})()"#,
                                escaped_html
                            );

                            match wv.evaluate_script(&script) {
                                Ok(_) => {
                                    tracing::info!(
                                        "[Rust] ShowError: Error page displayed successfully"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[Rust] ShowError: Failed to display error page: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Keep webview and window alive
        let _ = &webview;
        let _ = &window;
    });

    // This is unreachable because event_loop.run() never returns
    #[allow(unreachable_code)]
    Ok(())
}
