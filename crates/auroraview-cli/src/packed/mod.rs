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

mod backend;
mod events;
mod extract;
mod license;
mod utils;
mod warmup;
mod webview;

use anyhow::{Context, Result};
use auroraview_pack::{OverlayReader, PackedMetrics};
use std::time::Instant;

// Re-export public items
pub use utils::{
    build_module_search_paths, escape_json_for_js, get_python_exe_path, get_runtime_cache_dir,
    get_webview_data_dir, inject_environment_variables,
};
pub use webview::build_packed_init_script;

/// Run a packed application (overlay mode)
///
/// This function is called when the executable contains embedded overlay data.
/// It reads the overlay, initializes logging, validates license, and launches the WebView.
pub fn run_packed_app() -> Result<()> {
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
        "[Rust] Running packed application: {}",
        overlay.config.window.title
    );
    tracing::info!("[Rust] Assets: {} files", overlay.assets.len());
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
