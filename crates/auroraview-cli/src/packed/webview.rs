//! WebView creation and event loop
//!
//! Handles creating the WebView window and running the main event loop.

use anyhow::{Context, Result};
use auroraview_core::assets::{build_packed_init_script, get_loading_html};
use auroraview_core::plugins::{PluginRequest, PluginRouter, ScopeConfig};
use auroraview_core::protocol::MemoryAssets;
use auroraview_pack::{OverlayData, PackMode, PackedMetrics};
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
use super::utils::{escape_json_for_js, get_extensions_dir, get_webview_data_dir, has_extensions};

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

            // Check if this is a plugin command (plugin:*)
            if method.starts_with("plugin:") {
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

    // Create PluginRouter with secure default scope
    // Instead of permissive(), use a restricted scope that only allows:
    // - File system access within the application's working directory
    // - Shell commands for opening URLs and files (but not arbitrary command execution)
    // This follows the principle of least privilege for packed applications.
    let default_scope = {
        use auroraview_core::plugins::PathScope;
        use auroraview_core::plugins::ShellScope;

        // Get the current working directory as the base for file system access
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Create a restricted scope configuration
        ScopeConfig::new()
            .with_fs_scope(PathScope::new().allow(cwd))
            .with_shell_scope(ShellScope::new()) // Allows open_url/open_file but not arbitrary commands
    };

    let plugin_router = Arc::new(RwLock::new(PluginRouter::with_scope(default_scope)));

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
            let memory_assets = MemoryAssets::from_vec(overlay.assets.clone())
                .with_loading_html(get_loading_html());

            // Find index.html path for logging
            let index_path = memory_assets
                .list_paths()
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
                    let path = uri.path();

                    // Use MemoryAssets to handle the request
                    let response = memory_assets.handle_request(path);

                    // Security: Restrict CORS to same-origin only
                    // Using the custom protocol origin prevents cross-origin access from external sites
                    #[cfg(target_os = "windows")]
                    let allowed_origin = "https://auroraview.localhost";
                    #[cfg(not(target_os = "windows"))]
                    let allowed_origin = "auroraview://localhost";

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

    // Wrap webview in Rc<RefCell> for single-threaded access
    // Note: WebView is not Send+Sync, so we use Rc instead of Arc
    use std::cell::RefCell;
    use std::rc::Rc;
    let webview = Rc::new(RefCell::new(webview));
    let webview_for_event = webview.clone();

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
                            tracing::info!(
                                "[Rust:WebView] Received PluginEvent: event={}, data_len={}",
                                event,
                                data.len()
                            );
                            let escaped = escape_json_for_js(&data);
                            let script = format!(
                                r#"(function() {{
                                    if (window.auroraview && window.auroraview.trigger) {{
                                        try {{
                                            var data = JSON.parse("{}");
                                            console.log('[AuroraView] Triggering event:', '{}', data);
                                            window.auroraview.trigger('{}', data);
                                        }} catch (e) {{
                                            console.error('[AuroraView] Failed to parse event data:', e);
                                        }}
                                    }} else {{
                                        console.warn('[AuroraView] Bridge not ready for event:', '{}');
                                    }}
                                }})()"#,
                                escaped, event, event, event
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
                    }
                }
            }
            _ => {}
        }

        // Keep webview alive
        let _ = &webview;
    });

    // This is unreachable because event_loop.run() never returns
    #[allow(unreachable_code)]
    Ok(())
}
