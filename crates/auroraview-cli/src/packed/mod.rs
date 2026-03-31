//! Packed application runtime module
//!
//! This module handles running packed (overlay) applications.
//! When an executable contains embedded overlay data, these functions
//! are used to extract and run the packed content.
//!
//! ## Module Structure
//!
//! - `warmup` - WebView2 pre-initialization for faster cold starts
//! - `events` - User events for thread communication
//! - `backend` - Python backend process and IPC communication
//! - `webview` - WebView creation and event loop
//! - `extract` - File extraction (Python runtime, resources)
//! - `license` - License validation
//! - `utils` - Utility functions (paths, environment variables)
//!
//! ## Debug Mode
//!
//! Set `AURORAVIEW_DEBUG=1` environment variable to enable debug mode:
//! - On Windows: Opens a console window for log output
//! - Writes logs to `%TEMP%/auroraview_packed.log`
//! - Enables verbose logging

mod backend;
mod events;
mod extract;
mod license;
mod utils;
mod warmup;
mod webview;

use std::time::Instant;

use anyhow::{Context, Result};
use auroraview_pack::{OverlayReader, PackedMetrics};

// Re-export public items
pub use utils::{
    build_css_injection_script, build_module_search_paths, escape_json_for_js,
    get_python_exe_path, get_runtime_cache_dir_with_hash, get_webview_data_dir,
    inject_environment_variables,
};

// Re-export from auroraview-core
pub use auroraview_core::assets::build_packed_init_script_with_csp;

/// Check if debug mode is enabled via environment variable
fn is_debug_env() -> bool {
    std::env::var("AURORAVIEW_DEBUG")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

fn parse_env_bool(value: Option<&str>) -> bool {
    value
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn parse_env_f32(value: Option<&str>, default: f32) -> f32 {
    value.and_then(|v| v.parse::<f32>().ok()).unwrap_or(default)
}

fn init_packed_telemetry(log_level: &str) -> Option<auroraview_telemetry::TelemetryGuard> {
    if parse_env_bool(
        std::env::var("AURORAVIEW_GALLERY_RUST_TELEMETRY_DISABLED")
            .ok()
            .as_deref(),
    ) {
        return None;
    }

    let sentry_dsn = std::env::var("AURORAVIEW_GALLERY_RUST_SENTRY_DSN").ok();
    let otlp_endpoint = std::env::var("AURORAVIEW_GALLERY_RUST_OTLP_ENDPOINT").ok();

    if sentry_dsn.is_none() && otlp_endpoint.is_none() {
        return None;
    }

    let config = auroraview_telemetry::TelemetryConfig {
        enabled: true,
        service_name: std::env::var("AURORAVIEW_GALLERY_RUST_SERVICE_NAME")
            .unwrap_or_else(|_| "auroraview-packed-client".to_string()),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
        log_level: std::env::var("AURORAVIEW_GALLERY_RUST_LOG_LEVEL")
            .unwrap_or_else(|_| log_level.to_string()),
        log_to_stdout: false,
        log_json: false,
        otlp_endpoint,
        metrics_enabled: true,
        metrics_interval_secs: 30,
        traces_enabled: true,
        trace_sample_ratio: 0.2,
        sentry_dsn,
        sentry_environment: Some(
            std::env::var("AURORAVIEW_GALLERY_RUST_SENTRY_ENV")
                .unwrap_or_else(|_| "packed".to_string()),
        ),
        sentry_release: std::env::var("AURORAVIEW_GALLERY_RUST_SENTRY_RELEASE").ok(),
        sentry_sample_rate: parse_env_f32(
            std::env::var("AURORAVIEW_GALLERY_RUST_SENTRY_SAMPLE_RATE")
                .ok()
                .as_deref(),
            1.0,
        ),
        sentry_traces_sample_rate: parse_env_f32(
            std::env::var("AURORAVIEW_GALLERY_RUST_SENTRY_TRACES_SAMPLE_RATE")
                .ok()
                .as_deref(),
            0.2,
        ),
    };

    match auroraview_telemetry::Telemetry::init(config) {
        Ok(guard) => Some(guard),
        Err(err) => {
            eprintln!("[packed] telemetry init failed: {err}");
            None
        }
    }
}

/// Allocate a console window on Windows for debug output
#[cfg(target_os = "windows")]
fn allocate_console() {
    use windows::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        // Try to attach to parent console first (if run from command line)
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            // If no parent console, allocate a new one
            let _ = AllocConsole();
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn allocate_console() {
    // No-op on non-Windows platforms
}

/// Run a packed application (overlay mode)
///
/// This function is called when the executable contains embedded overlay data.
/// It reads the overlay, initializes logging, validates license, and launches the WebView.
pub fn run_packed_app() -> Result<()> {
    // Check for debug mode early
    let debug_env = is_debug_env();

    // Allocate console on Windows if debug mode is enabled
    if debug_env {
        allocate_console();
        eprintln!("[DEBUG] AuroraView packed app starting in debug mode...");
    }

    // Start WebView2 warmup IMMEDIATELY - this runs in background while we read overlay
    // This is critical for reducing cold-start latency by 2-4 seconds
    warmup::start_webview2_warmup();

    // Start performance metrics
    let mut metrics = PackedMetrics::new();
    let startup_start = Instant::now();

    // Read overlay data from the executable with metrics
    // Note: WebView2 warmup is running in parallel during this I/O operation
    let exe_path = std::env::current_exe()?;
    let overlay = OverlayReader::read_with_metrics(&exe_path, Some(&mut metrics))
        .with_context(|| "Failed to read overlay data")?
        .ok_or_else(|| anyhow::anyhow!("No overlay data found in packed executable"))?;

    // Initialize logging/telemetry
    // Use debug level if either config.debug or AURORAVIEW_DEBUG env is set
    let log_level = if overlay.config.debug || debug_env {
        "debug"
    } else {
        "info"
    };

    let _telemetry_guard = init_packed_telemetry(log_level);
    if _telemetry_guard.is_none() {
        let local_time = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339();

        // Configure tracing to output to stderr (stdout is used for JSON-RPC in packed mode)
        // Disable ANSI colors on Windows to avoid garbled output in parent process console
        #[cfg(target_os = "windows")]
        let use_ansi = false;
        #[cfg(not(target_os = "windows"))]
        let use_ansi = true;

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
                    .with_ansi(use_ansi)
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
                    .with_ansi(use_ansi)
                    .init();
            }
        }
    }

    // Write debug log file if debug mode is enabled
    if debug_env {
        let log_path = std::env::temp_dir().join("auroraview_packed.log");
        tracing::info!("[DEBUG] Log file: {:?}", log_path);
        if let Ok(mut file) = std::fs::File::create(&log_path) {
            use std::io::Write;
            let _ = writeln!(file, "AuroraView Packed App Debug Log");
            let _ = writeln!(file, "================================");
            let _ = writeln!(file, "Exe: {:?}", exe_path);
            let _ = writeln!(file, "Title: {}", overlay.config.window.title);
            let _ = writeln!(file, "Assets: {} files", overlay.assets.len());
            let _ = writeln!(file, "Mode: {:?}", overlay.config.mode);
            let _ = writeln!(file);
            let _ = writeln!(file, "Assets list:");
            for (i, (path, data)) in overlay.assets.iter().enumerate() {
                let _ = writeln!(file, "  [{}] {} ({} bytes)", i, path, data.len());
            }
        }
    }

    tracing::info!(
        "[Rust] Running packed application: {}",
        overlay.config.window.title
    );
    tracing::info!("[Rust] Assets: {} files", overlay.assets.len());
    tracing::info!("[Rust] Mode: {:?}", overlay.config.mode);
    tracing::info!(
        "[Rust] Overlay read completed in {:.2}ms",
        startup_start.elapsed().as_secs_f64() * 1000.0
    );

    // Inject environment variables
    inject_environment_variables(&overlay.config.env);

    // Validate license
    if let Some(ref license_config) = overlay.config.license {
        if !license::validate_license(license_config)? {
            return Ok(());
        }
    }

    webview::run_packed_webview(overlay, metrics)
}
