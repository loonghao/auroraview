//! Packed application runtime module
//!
//! This module handles running packed (overlay) applications.
//! When an executable contains embedded overlay data, these functions
//! are used to extract and run the packed content.

use anyhow::{Context, Result};
use auroraview_core::assets::{get_event_bridge_js, get_loading_html};
use auroraview_core::plugins::{PluginRequest, PluginRouter, ScopeConfig};
use auroraview_pack::{
    BundleStrategy, LicenseConfig, LicenseReason, LicenseValidator, OverlayData, OverlayReader,
    PackMode, PackedMetrics, PythonBundleConfig, PythonRuntimeMeta,
};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows;
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use crate::{load_window_icon, normalize_url};

/// Start WebView2 warmup in background thread
///
/// This pre-initializes WebView2 environment while overlay is being read,
/// reducing cold-start latency by 2-4 seconds.
#[cfg(target_os = "windows")]
fn start_webview2_warmup() {
    use std::sync::OnceLock;
    use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
    use windows::core::PCWSTR;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    static WARMUP_STARTED: OnceLock<()> = OnceLock::new();

    // Only start warmup once
    WARMUP_STARTED.get_or_init(|| {
        let data_dir = get_webview_data_dir();

        thread::Builder::new()
            .name("webview2-warmup".to_string())
            .spawn(move || {
                let start = Instant::now();
                tracing::info!(
                    "[warmup] Starting WebView2 warmup (data_folder: {:?})",
                    data_dir
                );

                // Initialize COM in STA mode
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                }

                // Create data directory
                let _ = std::fs::create_dir_all(&data_dir);

                // Create WebView2 environment to trigger runtime discovery
                let data_dir_wide: Vec<u16> = data_dir
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let result = unsafe {
                    CreateCoreWebView2EnvironmentWithOptions(
                        PCWSTR::null(),
                        PCWSTR(data_dir_wide.as_ptr()),
                        None,
                        &CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
                            move |_env_result, _env| {
                                // Environment created successfully
                                Ok(())
                            },
                        )),
                    )
                };

                let duration = start.elapsed();
                match result {
                    Ok(_) => {
                        tracing::info!(
                            "[warmup] WebView2 warmup complete in {}ms",
                            duration.as_millis()
                        );
                    }
                    Err(e) => {
                        tracing::warn!("[warmup] WebView2 warmup failed: {:?}", e);
                    }
                }
            })
            .expect("Failed to spawn warmup thread");
    });
}

#[cfg(not(target_os = "windows"))]
fn start_webview2_warmup() {
    // No-op on non-Windows platforms
}

/// User event for communication between threads and WebView
#[derive(Debug, Clone)]
enum UserEvent {
    /// Python response to be sent to WebView
    PythonResponse(String),
    /// Plugin event to be sent to WebView
    PluginEvent { event: String, data: String },
    /// Python backend is ready, navigate to actual content
    PythonReady,
}

/// Python backend handle for IPC communication
struct PythonBackend {
    process: Mutex<Child>,
    stdin: Arc<Mutex<ChildStdin>>,
}

impl PythonBackend {
    /// Check if Python process is still running
    fn is_alive(&self) -> bool {
        if let Ok(mut process) = self.process.lock() {
            match process.try_wait() {
                Ok(None) => true,  // Still running
                Ok(Some(status)) => {
                    tracing::warn!("Python process exited with status: {:?}", status);
                    false
                }
                Err(e) => {
                    tracing::error!("Failed to check Python process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Send a JSON-RPC request to Python backend
    fn send_request(&self, request: &str) -> Result<()> {
        // Check if process is still alive before sending
        if !self.is_alive() {
            return Err(anyhow::anyhow!("Python backend process has exited"));
        }

        let mut stdin = self
            .stdin
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        writeln!(stdin, "{}", request)?;
        stdin.flush()?;
        Ok(())
    }
}

/// Run a packed application (overlay mode)
///
/// This function is called when the executable contains embedded overlay data.
/// It reads the overlay, initializes logging, validates license, and launches the WebView.
pub fn run_packed_app() -> Result<()> {
    // Start WebView2 warmup IMMEDIATELY - this runs in background while we read overlay
    // This is critical for reducing cold-start latency by 2-4 seconds
    start_webview2_warmup();

    // Start performance metrics
    let mut metrics = PackedMetrics::new();
    let startup_start = Instant::now();

    // Read overlay data from the executable with metrics
    // Note: WebView2 warmup is running in parallel during this I/O operation
    let exe_path = std::env::current_exe()?;
    let overlay = OverlayReader::read_with_metrics(&exe_path, Some(&mut metrics))
        .with_context(|| "Failed to read overlay data")?
        .ok_or_else(|| anyhow::anyhow!("No overlay data found in packed executable"))?;

    // Initialize logging
    let log_level = if overlay.config.debug {
        "debug"
    } else {
        "info"
    };
    let local_time = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339();

    // Configure tracing to output to stderr (stdout is used for JSON-RPC in packed mode)
    match local_time {
        Ok(timer) => {
            tracing_subscriber::fmt()
                .with_timer(timer)
                .with_writer(std::io::stderr)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
                )
                .with_target(false)
                .init();
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
                )
                .with_target(false)
                .init();
        }
    }

    tracing::info!(
        "Running packed application: {}",
        overlay.config.window.title
    );
    tracing::info!("Assets: {} files", overlay.assets.len());
    tracing::info!(
        "Overlay read completed in {:.2}ms",
        startup_start.elapsed().as_secs_f64() * 1000.0
    );

    // Inject environment variables
    inject_environment_variables(&overlay.config.env);

    // Validate license
    if let Some(ref license_config) = overlay.config.license {
        if !validate_license(license_config)? {
            return Ok(());
        }
    }

    run_packed_webview(overlay, metrics)
}

/// Run WebView from overlay data
///
/// Note: This function uses event_loop.run() which never returns.
/// It will call std::process::exit() when the window closes.
#[allow(unreachable_code)]
fn run_packed_webview(overlay: OverlayData, mut metrics: PackedMetrics) -> Result<()> {
    let config = &overlay.config;
    let is_fullstack = matches!(config.mode, PackMode::FullStack { .. });

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

    // Track if we're waiting for Python to be ready (for FullStack mode)
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
    let init_script = build_packed_init_script(is_fullstack);

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
            // Clone assets for the protocol handler
            let assets = overlay.assets.clone();

            // Find index.html path for logging
            let index_path = assets
                .iter()
                .find(|(path, _)| {
                    path == "index.html"
                        || path == "frontend/index.html"
                        || path.ends_with("/index.html")
                })
                .map(|(path, _)| path.clone())
                .unwrap_or_else(|| "index.html".to_string());

            tracing::info!("Loading embedded assets via auroraview:// protocol");
            tracing::info!("Index path: {}", index_path);

            // For FullStack mode, show loading screen while waiting for Python
            // For Frontend mode, load index.html directly
            let initial_url = if is_fullstack {
                tracing::info!("FullStack mode: showing loading screen while Python initializes");
                "auroraview://localhost/__loading__"
            } else {
                "auroraview://localhost/index.html"
            };

            // Get loading HTML for the protocol handler
            let loading_html = get_loading_html();

            WryWebViewBuilder::new_with_web_context(&mut web_context)
                .with_custom_protocol("auroraview".to_string(), move |_webview_id, request| {
                    let path = request.uri().path();
                    let path = path.trim_start_matches('/');

                    // Handle special loading page for FullStack mode
                    if path == "__loading__" {
                        return wry::http::Response::builder()
                            .status(200)
                            .header("Content-Type", "text/html; charset=utf-8")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(loading_html.clone().into_bytes().into())
                            .unwrap();
                    }

                    // Default to index.html for root path
                    let path = if path.is_empty() { "index.html" } else { path };

                    // Try different path variations
                    let content = assets
                        .iter()
                        .find(|(p, _)| {
                            p == path
                                || p == &format!("frontend/{}", path)
                                || p.ends_with(&format!("/{}", path))
                        })
                        .map(|(_, content)| content.clone());

                    match content {
                        Some(data) => {
                            let mime = mime_guess::from_path(path)
                                .first_or_octet_stream()
                                .to_string();
                            wry::http::Response::builder()
                                .status(200)
                                .header("Content-Type", mime)
                                .header("Access-Control-Allow-Origin", "*")
                                .body(data.into())
                                .unwrap()
                        }
                        None => {
                            tracing::warn!("Asset not found: {}", path);
                            wry::http::Response::builder()
                                .status(404)
                                .body(b"Not Found".to_vec().into())
                                .unwrap()
                        }
                    }
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
                    match user_event {
                        UserEvent::PythonReady => {
                            // Python backend is ready, navigate to actual content
                            tracing::info!("Python backend ready, navigating to application");
                            if let Ok(mut waiting) = waiting_for_python.write() {
                                *waiting = false;
                            }
                            // Navigate to the actual application
                            let _ = wv.load_url("auroraview://localhost/index.html");
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

/// Escape JSON string for embedding in JavaScript
pub fn escape_json_for_js(json: &str) -> String {
    json.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Build initialization script for packed mode
pub fn build_packed_init_script(is_fullstack: bool) -> String {
    let mut script = String::with_capacity(32768);

    // Add event bridge
    script.push_str(&get_event_bridge_js());
    script.push('\n');

    // Register API methods for fullstack mode
    if is_fullstack {
        script.push_str(
            r#"
// Register Gallery API methods
(function() {
    if (window.auroraview && window.auroraview._registerApiMethods) {
        window.auroraview._registerApiMethods('api', [
            'get_samples',
            'get_categories',
            'get_source',
            'run_sample',
            'kill_process',
            'send_to_process',
            'list_processes',
            'open_url'
        ]);
        console.log('[AuroraView] Registered Gallery API methods');
    }
})();
"#,
        );
    }

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
    tracing::debug!("IPC message received: {}", body);

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

            tracing::debug!("API call: {} (id: {})", method, id);

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
        }
        _ => {
            tracing::warn!("Unknown IPC message type: {}", msg_type);
        }
    }
}

/// Start Python backend process for FullStack mode with IPC support
fn start_python_backend_with_ipc(
    overlay: &OverlayData,
    python_config: &PythonBundleConfig,
    proxy: EventLoopProxy<UserEvent>,
    metrics: &mut PackedMetrics,
) -> Result<PythonBackend> {
    let func_start = Instant::now();

    // Determine Python executable path based on strategy
    let python_exe = match python_config.strategy {
        BundleStrategy::Standalone => {
            // Extract embedded Python runtime
            let runtime_start = Instant::now();
            let exe = extract_standalone_python(overlay)?;
            metrics.add_phase("python_runtime_extract", runtime_start.elapsed());
            metrics.mark_python_runtime_extract();
            exe
        }
        _ => {
            // Use system Python for other strategies
            PathBuf::from("python")
        }
    };

    // Create temp directory for Python files
    let temp_dir = get_python_extract_dir();
    fs::create_dir_all(&temp_dir)?;

    tracing::info!("Extracting Python files to: {}", temp_dir.display());

    // Collect Python files to extract
    let python_assets: Vec<_> = overlay
        .assets
        .iter()
        .filter(|(path, _)| path.starts_with("python/"))
        .collect();

    if !python_assets.is_empty() {
        let extract_start = Instant::now();

        // Pre-create all directories in batch (collect unique parent dirs)
        let dirs: HashSet<PathBuf> = python_assets
            .iter()
            .filter_map(|(path, _)| {
                let rel_path = path.strip_prefix("python/").unwrap_or(path);
                temp_dir.join(rel_path).parent().map(|p| p.to_path_buf())
            })
            .collect();

        for dir in &dirs {
            fs::create_dir_all(dir)?;
        }

        metrics.add_phase("python_dirs_create", extract_start.elapsed());

        // Parallel file extraction using rayon
        let write_start = Instant::now();
        let results: Vec<Result<String, anyhow::Error>> = python_assets
            .par_iter()
            .map(|(path, content)| {
                let rel_path = path.strip_prefix("python/").unwrap_or(path);
                let dest_path = temp_dir.join(rel_path);
                fs::write(&dest_path, content)
                    .with_context(|| format!("Failed to write: {}", dest_path.display()))?;
                Ok(rel_path.to_string())
            })
            .collect();

        // Check for errors
        let mut python_files = Vec::with_capacity(results.len());
        for result in results {
            python_files.push(result?);
        }

        metrics.add_phase("python_files_write", write_start.elapsed());
        metrics.mark_python_files_extract();

        tracing::info!(
            "Extracted {} Python files in {:.2}ms",
            python_files.len(),
            extract_start.elapsed().as_secs_f64() * 1000.0
        );
    }

    // Extract resource directories (examples, etc.) from overlay assets
    let resources_start = Instant::now();
    let resources_dir = extract_resources_parallel(overlay, &temp_dir)?;
    metrics.add_phase("resources_extract", resources_start.elapsed());
    metrics.mark_resources_extract();

    // Parse entry point (format: "module:function" or "file.py")
    let entry_point = &python_config.entry_point;
    let (module, function) = if entry_point.contains(':') {
        let parts: Vec<&str> = entry_point.split(':').collect();
        (parts[0], Some(parts.get(1).copied().unwrap_or("main")))
    } else {
        (entry_point.as_str(), None)
    };

    // Build module search paths from configuration
    let site_packages_dir = temp_dir.join("site-packages");
    let module_paths = build_module_search_paths(
        &python_config.module_search_paths,
        &temp_dir,
        &resources_dir,
        &site_packages_dir,
    );

    // Build Python command with module paths
    // Use runpy.run_path() to properly set __file__ and __name__ variables,
    // so developers don't need to handle packed mode specially in their code.
    let script_path = temp_dir.join(module);
    let python_code = if let Some(func) = function {
        // Import module and call function
        format!(
            "import sys; sys.path.insert(0, r'{}'); from {} import {}; {}()",
            temp_dir.display(),
            module.replace(['/', '\\'], ".").trim_end_matches(".py"),
            func,
            func
        )
    } else {
        // Use runpy.run_path() which properly sets __file__, __name__, etc.
        // This allows developers to use `if __name__ == "__main__"` and
        // Path(__file__).parent without any packed-mode specific handling.
        format!(
            r#"import sys; sys.path.insert(0, r'{}'); import runpy; runpy.run_path(r'{}', run_name='__main__')"#,
            temp_dir.display(),
            script_path.display()
        )
    };

    tracing::info!("Starting Python backend: {}", entry_point);
    tracing::info!("Using Python: {}", python_exe.display());
    tracing::debug!("Python code: {}", python_code);
    tracing::debug!("Module search paths: {:?}", module_paths);

    // Build PYTHONPATH from module search paths
    let pythonpath = module_paths.join(if cfg!(windows) { ";" } else { ":" });

    // Start Python process with environment variables
    let spawn_start = Instant::now();
    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &python_code])
        .current_dir(&temp_dir)
        .env("AURORAVIEW_PACKED", "1")
        .env("AURORAVIEW_RESOURCES_DIR", &resources_dir)
        .env("AURORAVIEW_PYTHON_PATH", &pythonpath)
        .env("PYTHONPATH", &pythonpath)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    // Windows: hide console window unless show_console is enabled
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;

        if python_config.show_console {
            tracing::debug!("Python console window enabled");
            cmd.creation_flags(CREATE_NEW_CONSOLE);
        } else {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
    }

    // Set specific resource paths as environment variables
    let examples_dir = resources_dir.join("examples");
    if examples_dir.exists() {
        cmd.env("AURORAVIEW_EXAMPLES_DIR", &examples_dir);
        tracing::info!("Examples directory: {}", examples_dir.display());
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to start Python backend: {}", python_exe.display()))?;

    metrics.add_phase("python_spawn", spawn_start.elapsed());
    metrics.mark_python_start();

    tracing::info!(
        "Python backend started (PID: {}) in {:.2}ms",
        child.id(),
        func_start.elapsed().as_secs_f64() * 1000.0
    );

    // Take ownership of stdin and stdout
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

    let stdin = Arc::new(Mutex::new(stdin));

    // Spawn thread to wait for Python ready signal and then read responses
    // This is non-blocking so WebView can show loading screen while waiting
    let ready_start = Instant::now();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut ready_line = String::new();

        // Read the first line - should be the ready signal
        match reader.read_line(&mut ready_line) {
            Ok(0) => {
                tracing::error!("Python process closed stdout before sending ready signal");
                return;
            }
            Ok(_) => {
                let ready_line_trimmed = ready_line.trim();
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(ready_line_trimmed) {
                    if msg.get("type").and_then(|v| v.as_str()) == Some("ready") {
                        let handlers = msg
                            .get("handlers")
                            .and_then(|v| v.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        tracing::info!(
                            "Python backend ready with {} handlers in {:.2}ms",
                            handlers,
                            ready_start.elapsed().as_secs_f64() * 1000.0
                        );
                        // Notify WebView to navigate to actual content
                        if let Err(e) = proxy.send_event(UserEvent::PythonReady) {
                            tracing::error!("Failed to send PythonReady event: {}", e);
                        }
                    } else {
                        tracing::warn!(
                            "Unexpected first message from Python (expected ready signal): {}",
                            ready_line_trimmed
                        );
                        // Still notify ready to avoid hanging on loading screen
                        let _ = proxy.send_event(UserEvent::PythonReady);
                    }
                } else {
                    tracing::warn!("Failed to parse Python ready signal: {}", ready_line_trimmed);
                    // Still notify ready to avoid hanging on loading screen
                    let _ = proxy.send_event(UserEvent::PythonReady);
                }
            }
            Err(e) => {
                tracing::error!("Failed to read Python ready signal: {}", e);
                return;
            }
        }

        // Continue reading Python stdout and forward responses
        for line in reader.lines() {
            match line {
                Ok(response) => {
                    if response.trim().is_empty() {
                        continue;
                    }
                    tracing::debug!("Python response: {}", response);
                    // Send response to event loop
                    if let Err(e) = proxy.send_event(UserEvent::PythonResponse(response)) {
                        tracing::error!("Failed to send response to event loop: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading Python stdout: {}", e);
                    break;
                }
            }
        }
        tracing::info!("Python stdout reader thread exiting");
    });

    Ok(PythonBackend {
        process: Mutex::new(child),
        stdin,
    })
}

/// Extract resource directories from overlay assets (parallel version)
///
/// Resources are stored with prefixes like "resources/examples/", "resources/data/", etc.
/// This function extracts them to the resources directory and returns the path.
fn extract_resources_parallel(overlay: &OverlayData, base_dir: &Path) -> Result<PathBuf> {
    let resources_dir = base_dir.join("resources");
    fs::create_dir_all(&resources_dir)?;

    // Collect resource files to extract
    let resource_assets: Vec<_> = overlay
        .assets
        .iter()
        .filter_map(|(path, content)| {
            if path.starts_with("resources/") {
                let rel_path = path.strip_prefix("resources/").unwrap_or(path);
                Some((resources_dir.join(rel_path), content))
            } else if path.starts_with("examples/") {
                Some((resources_dir.join(path), content))
            } else {
                None
            }
        })
        .collect();

    if resource_assets.is_empty() {
        return Ok(resources_dir);
    }

    // Pre-create all directories in batch
    let dirs: HashSet<PathBuf> = resource_assets
        .iter()
        .filter_map(|(path, _)| path.parent().map(|p| p.to_path_buf()))
        .collect();

    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    // Parallel file extraction
    let results: Vec<Result<(), anyhow::Error>> = resource_assets
        .par_iter()
        .map(|(dest_path, content)| {
            fs::write(dest_path, content)
                .with_context(|| format!("Failed to write: {}", dest_path.display()))?;
            Ok(())
        })
        .collect();

    // Check for errors
    for result in results {
        result?;
    }

    tracing::info!(
        "Extracted {} resource files to: {}",
        resource_assets.len(),
        resources_dir.display()
    );

    Ok(resources_dir)
}

#[allow(dead_code)]
/// Extract resource directories from overlay assets (sequential version, kept for reference)
fn extract_resources(overlay: &OverlayData, base_dir: &Path) -> Result<PathBuf> {
    let resources_dir = base_dir.join("resources");
    fs::create_dir_all(&resources_dir)?;

    let mut resource_count = 0;

    for (path, content) in &overlay.assets {
        // Check for resources with "resources/" prefix (from hooks.collect)
        if path.starts_with("resources/") {
            let rel_path = path.strip_prefix("resources/").unwrap_or(path);
            let dest_path = resources_dir.join(rel_path);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, content)?;
            resource_count += 1;
            tracing::debug!("Extracted resource: {}", rel_path);
        }
        // Also check for "examples/" prefix directly (legacy support)
        else if path.starts_with("examples/") {
            let dest_path = resources_dir.join(path);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, content)?;
            resource_count += 1;
            tracing::debug!("Extracted resource: {}", path);
        }
    }

    if resource_count > 0 {
        tracing::info!(
            "Extracted {} resource files to: {}",
            resource_count,
            resources_dir.display()
        );
    }

    Ok(resources_dir)
}

/// Extract embedded Python runtime for standalone mode
fn extract_standalone_python(overlay: &OverlayData) -> Result<PathBuf> {
    // Find Python runtime metadata
    let meta_data = overlay
        .assets
        .iter()
        .find(|(path, _)| path == "python_runtime.json")
        .map(|(_, content)| content.clone())
        .ok_or_else(|| anyhow::anyhow!("Python runtime metadata not found in overlay"))?;

    let meta: PythonRuntimeMeta = serde_json::from_slice(&meta_data)
        .with_context(|| "Failed to parse Python runtime metadata")?;

    // Find Python runtime archive
    let archive_data = overlay
        .assets
        .iter()
        .find(|(path, _)| path == "python_runtime.tar.gz")
        .map(|(_, content)| content.clone())
        .ok_or_else(|| anyhow::anyhow!("Python runtime archive not found in overlay"))?;

    tracing::info!(
        "Extracting Python {} runtime ({:.2} MB)...",
        meta.version,
        archive_data.len() as f64 / (1024.0 * 1024.0)
    );

    // Get app name for cache directory
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("auroraview"));
    let app_name = exe_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("auroraview");

    // Extract to cache directory
    let cache_dir = get_runtime_cache_dir(app_name);
    let version_marker = cache_dir.join(".version");

    // Check if already extracted with correct version
    if version_marker.exists() {
        if let Ok(cached_version) = fs::read_to_string(&version_marker) {
            if cached_version.trim() == meta.version {
                let python_path = get_python_exe_path(&cache_dir);
                if python_path.exists() {
                    tracing::info!("Using cached Python runtime: {}", cache_dir.display());
                    return Ok(python_path);
                }
            }
        }
    }

    // Clean up old extraction if exists
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(&cache_dir)?;

    tracing::info!("Extracting Python runtime to: {}", cache_dir.display());

    // Decompress and extract tar.gz
    let decoder = flate2::read::GzDecoder::new(&archive_data[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&cache_dir)
        .with_context(|| "Failed to extract Python runtime")?;

    // Write version marker
    fs::write(&version_marker, &meta.version)?;

    let python_path = get_python_exe_path(&cache_dir);
    if !python_path.exists() {
        return Err(anyhow::anyhow!(
            "Python executable not found after extraction: {}",
            python_path.display()
        ));
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&python_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&python_path, perms)?;
    }

    tracing::info!("Python runtime ready: {}", python_path.display());
    Ok(python_path)
}

/// Get the runtime cache directory for an app
pub fn get_runtime_cache_dir(app_name: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("runtime")
        .join(app_name)
}

/// Get the Python executable path within the extracted runtime
pub fn get_python_exe_path(cache_dir: &std::path::Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        cache_dir.join("python").join("python.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        cache_dir.join("python").join("bin").join("python3")
    }
}

/// Get the directory for extracting Python files
fn get_python_extract_dir() -> PathBuf {
    // Use a unique directory based on the executable path
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("auroraview"));
    let exe_name = exe_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("auroraview");

    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("python")
        .join(exe_name)
}

/// Build module search paths from configuration
///
/// Expands special variables:
/// - `$EXTRACT_DIR` - The directory where Python files are extracted
/// - `$RESOURCES_DIR` - The resources directory
/// - `$SITE_PACKAGES` - The site-packages directory
pub fn build_module_search_paths(
    config_paths: &[String],
    extract_dir: &Path,
    resources_dir: &Path,
    site_packages_dir: &Path,
) -> Vec<String> {
    config_paths
        .iter()
        .map(|path| {
            path.replace("$EXTRACT_DIR", &extract_dir.to_string_lossy())
                .replace("$RESOURCES_DIR", &resources_dir.to_string_lossy())
                .replace("$SITE_PACKAGES", &site_packages_dir.to_string_lossy())
        })
        .filter(|path| {
            // Only include paths that exist
            let p = Path::new(path);
            if p.exists() {
                true
            } else {
                tracing::debug!("Module search path does not exist: {}", path);
                false
            }
        })
        .collect()
}

/// Get the WebView2 user data directory in AppData
///
/// Returns a path like: `%LOCALAPPDATA%/AuroraView/WebView2`
/// This prevents WebView2 from creating data folders in the current directory.
pub fn get_webview_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("WebView2")
}

/// Inject environment variables from config
pub fn inject_environment_variables(env: &std::collections::HashMap<String, String>) {
    for (key, value) in env {
        tracing::debug!("Setting env: {}={}", key, value);
        std::env::set_var(key, value);
    }

    if !env.is_empty() {
        tracing::info!("Injected {} environment variables", env.len());
    }
}

/// Validate license and handle token input if needed
fn validate_license(license_config: &LicenseConfig) -> Result<bool> {
    let validator = LicenseValidator::new(license_config.clone());

    // First try without token
    let mut status = validator.validate(None);

    // If token is required and not embedded, prompt for it
    if status.reason == LicenseReason::TokenRequired {
        tracing::info!("Authorization token required");

        // Try to get token from environment variable first
        let env_token = std::env::var("AURORAVIEW_TOKEN").ok();

        if let Some(ref token) = env_token {
            tracing::debug!("Using token from AURORAVIEW_TOKEN environment variable");
            status = validator.validate(Some(token));
        } else {
            // Prompt user for token
            print!("Enter authorization token: ");
            io::stdout().flush()?;

            let mut token = String::new();
            io::stdin().read_line(&mut token)?;
            let token = token.trim();

            if token.is_empty() {
                eprintln!("Error: No token provided");
                return Ok(false);
            }

            status = validator.validate(Some(token));
        }
    }

    // Handle validation result
    if status.valid {
        if status.in_grace_period {
            if let Some(ref msg) = status.message {
                eprintln!("Warning: {}", msg);
            }
            if let Some(days) = status.days_remaining {
                eprintln!("Grace period: {} days remaining", days);
            }
        } else if let Some(days) = status.days_remaining {
            tracing::info!("License valid for {} more days", days);
        }
        Ok(true)
    } else {
        let error_msg = status
            .message
            .unwrap_or_else(|| format!("License validation failed: {:?}", status.reason));
        eprintln!("Error: {}", error_msg);
        Ok(false)
    }
}
