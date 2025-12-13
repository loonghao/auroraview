//! Packed application runtime module
//!
//! This module handles running packed (overlay) applications.
//! When an executable contains embedded overlay data, these functions
//! are used to extract and run the packed content.

use anyhow::{Context, Result};
use auroraview_pack::{
    read_overlay, BundleStrategy, LicenseConfig, LicenseReason, LicenseValidator, OverlayData,
    PackMode, PythonBundleConfig, PythonRuntimeMeta,
};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tao::event_loop::{ControlFlow, EventLoop};
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use crate::{load_window_icon, normalize_url};

/// Run a packed application (overlay mode)
///
/// This function is called when the executable contains embedded overlay data.
/// It reads the overlay, initializes logging, validates license, and launches the WebView.
pub fn run_packed_app() -> Result<()> {
    // Read overlay data from the executable
    let overlay = read_overlay()
        .with_context(|| "Failed to read overlay data")?
        .ok_or_else(|| anyhow::anyhow!("No overlay data found in packed executable"))?;

    // Initialize logging
    let log_level = if overlay.config.debug {
        "debug"
    } else {
        "info"
    };
    let local_time = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339();

    match local_time {
        Ok(timer) => {
            tracing_subscriber::fmt()
                .with_timer(timer)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
                )
                .with_target(false)
                .init();
        }
        Err(_) => {
            tracing_subscriber::fmt()
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

    // Inject environment variables
    inject_environment_variables(&overlay.config.env);

    // Validate license
    if let Some(ref license_config) = overlay.config.license {
        if !validate_license(license_config)? {
            return Ok(());
        }
    }

    run_packed_webview(overlay)
}

/// Run WebView from overlay data
fn run_packed_webview(overlay: OverlayData) -> Result<()> {
    let config = &overlay.config;

    // For FullStack mode, extract Python files and start backend
    let mut python_process = if let PackMode::FullStack { ref python, .. } = config.mode {
        Some(start_python_backend(&overlay, python)?)
    } else {
        None
    };

    // Create event loop
    let event_loop = EventLoop::new();

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

    // Create WebContext
    let data_dir = get_webview_data_dir();
    let mut web_context = WebContext::new(Some(data_dir));

    // Create WebView based on pack mode
    let webview = match &config.mode {
        PackMode::Url { url } => {
            let normalized_url = normalize_url(url)?;
            tracing::info!("Loading URL: {}", normalized_url);

            WryWebViewBuilder::new_with_web_context(&mut web_context)
                .with_url(&normalized_url)
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

            WryWebViewBuilder::new_with_web_context(&mut web_context)
                .with_custom_protocol("auroraview".to_string(), move |_webview_id, request| {
                    let path = request.uri().path();
                    let path = path.trim_start_matches('/');

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
                // Use with_url to load from custom protocol, avoiding CORS issues
                .with_url("auroraview://localhost/index.html")
                .with_devtools(config.debug)
                .build(&window)
                .with_context(|| "Failed to create WebView")?
        }
    };

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            // Kill Python process on close
            if let Some(ref mut process) = python_process {
                tracing::info!("Stopping Python backend...");
                let _ = process.kill();
            }
            *control_flow = ControlFlow::Exit;
        }

        // Keep webview alive
        let _ = &webview;
    });
}

/// Start Python backend process for FullStack mode
fn start_python_backend(
    overlay: &OverlayData,
    python_config: &PythonBundleConfig,
) -> Result<Child> {
    // Determine Python executable path based on strategy
    let python_exe = match python_config.strategy {
        BundleStrategy::Standalone => {
            // Extract embedded Python runtime
            extract_standalone_python(overlay)?
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

    // Extract Python files from overlay assets
    let mut python_files = Vec::new();
    for (path, content) in &overlay.assets {
        if path.starts_with("python/") {
            let rel_path = path.strip_prefix("python/").unwrap_or(path);
            let dest_path = temp_dir.join(rel_path);

            // Create parent directories
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, content)?;
            python_files.push(rel_path.to_string());
            tracing::debug!("Extracted: {}", rel_path);
        }
    }

    tracing::info!("Extracted {} Python files", python_files.len());

    // Extract resource directories (examples, etc.) from overlay assets
    let resources_dir = extract_resources(overlay, &temp_dir)?;

    // Parse entry point (format: "module:function" or "file.py")
    let entry_point = &python_config.entry_point;
    let (module, function) = if entry_point.contains(':') {
        let parts: Vec<&str> = entry_point.split(':').collect();
        (parts[0], Some(parts.get(1).copied().unwrap_or("main")))
    } else {
        (entry_point.as_str(), None)
    };

    // Build Python command
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
        // Just run the file
        format!(
            "import sys; sys.path.insert(0, r'{}'); exec(open(r'{}/{}').read())",
            temp_dir.display(),
            temp_dir.display(),
            module
        )
    };

    tracing::info!("Starting Python backend: {}", entry_point);
    tracing::info!("Using Python: {}", python_exe.display());
    tracing::debug!("Python code: {}", python_code);

    // Start Python process with environment variables
    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &python_code])
        .current_dir(&temp_dir)
        .env("AURORAVIEW_PACKED", "1")
        .env("AURORAVIEW_RESOURCES_DIR", &resources_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Set specific resource paths as environment variables
    let examples_dir = resources_dir.join("examples");
    if examples_dir.exists() {
        cmd.env("AURORAVIEW_EXAMPLES_DIR", &examples_dir);
        tracing::info!("Examples directory: {}", examples_dir.display());
    }

    let child = cmd
        .spawn()
        .with_context(|| format!("Failed to start Python backend: {}", python_exe.display()))?;

    tracing::info!("Python backend started (PID: {})", child.id());

    Ok(child)
}

/// Extract resource directories from overlay assets
///
/// Resources are stored with prefixes like "resources/examples/", "resources/data/", etc.
/// This function extracts them to the resources directory and returns the path.
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
        tracing::info!("Extracted {} resource files to: {}", resource_count, resources_dir.display());
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
fn get_runtime_cache_dir(app_name: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("runtime")
        .join(app_name)
}

/// Get the Python executable path within the extracted runtime
fn get_python_exe_path(cache_dir: &std::path::Path) -> PathBuf {
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
fn inject_environment_variables(env: &std::collections::HashMap<String, String>) {
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
