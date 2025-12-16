//! WebView creation and event loop
//!
//! Handles creating the WebView window and running the main event loop.

use anyhow::{Context, Result};
use auroraview_core::assets::{build_packed_init_script, get_loading_html};
use auroraview_core::plugins::{PluginRequest, PluginRouter, ScopeConfig};
use auroraview_core::protocol::MemoryAssets;
use auroraview_pack::{OverlayData, PackMode, PackedMetrics};
use std::sync::{Arc, RwLock};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use crate::{load_window_icon, normalize_url};

use super::backend::{start_python_backend_with_ipc, PythonBackend};
use super::events::UserEvent;
use super::utils::{escape_json_for_js, get_webview_data_dir};

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
                    let router = plugin_router.read().unwrap();
                    let response = router.handle(request);

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
            let router = plugin_router.read().unwrap();
            let response = router.handle(request);

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

    // Create PluginRouter for handling native plugin commands
    let plugin_router = Arc::new(RwLock::new(PluginRouter::with_scope(
        ScopeConfig::permissive(),
    )));

    // Set up event callback for plugins to emit events to WebView
    let proxy_for_events = proxy.clone();
    {
        let router = plugin_router.read().unwrap();
        router.set_event_callback(Arc::new(
            move |event_name: &str, data: serde_json::Value| {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                let _ = proxy_for_events.send_event(UserEvent::PluginEvent {
                    event: event_name.to_string(),
                    data: data_str,
                });
            },
        ));
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

    // Set window icon
    if let Some(icon) = load_window_icon() {
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

                    wry::http::Response::builder()
                        .status(response.status)
                        .header("Content-Type", &response.mime_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(response.data.into_owned().into())
                        .unwrap()
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
                // Kill Python process on close
                if let Some(ref _backend) = python_backend_arc {
                    tracing::info!("Stopping Python backend...");
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
                            let is_python_ready = *python_ready.read().unwrap();
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
                        UserEvent::PythonReady => {
                            // Python backend is ready
                            tracing::info!("[Rust] ========================================");
                            tracing::info!("[Rust] Python backend ready");
                            if let Ok(mut ready) = python_ready.write() {
                                *ready = true;
                            }
                            // If loading screen is ready, send backend_ready event to frontend
                            let is_loading_ready = *loading_screen_ready.read().unwrap();
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
                            let escaped = escape_json_for_js(&data);
                            let script = format!(
                                r#"(function() {{
                                    if (window.auroraview && window.auroraview.trigger) {{
                                        try {{
                                            var data = JSON.parse("{}");
                                            window.auroraview.trigger('{}', data);
                                        }} catch (e) {{
                                            console.error('[AuroraView] Failed to parse event data:', e);
                                        }}
                                    }}
                                }})()"#,
                                escaped, event
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
