//! AuroraView Telemetry - OpenTelemetry-based observability for AuroraView.
//!
//! Provides unified logging, tracing, and metrics collection using OpenTelemetry.
//! Designed to monitor WebView performance, app lifecycle, and error tracking.
//!
//! # Features
//!
//! - **Logging**: Structured log collection via `tracing` -> OpenTelemetry bridge
//! - **Traces**: Distributed tracing for WebView operations (creation, navigation, IPC)
//! - **Metrics**: Performance counters (FPS, load time, memory, IPC latency)
//! - **OTLP Export**: Optional export to any OpenTelemetry-compatible backend
//!
//! # Quick Start
//!
//! ```no_run
//! use auroraview_telemetry::{Telemetry, TelemetryConfig};
//!
//! // Initialize with default settings (stdout export, enabled by default)
//! let _guard = Telemetry::init(TelemetryConfig::default()).unwrap();
//!
//! // Use tracing macros as normal - they flow through OpenTelemetry
//! tracing::info!(webview_id = "main", "WebView created");
//!
//! // Record metrics
//! auroraview_telemetry::metrics_api::record_webview_load_time("main", 250.0);
//! ```

mod config;
mod error;
mod guard;
mod metrics;
mod provider;
mod span_ext;

pub use config::TelemetryConfig;
pub use error::TelemetryError;
pub use guard::TelemetryGuard;
pub use metrics::WebViewMetrics;
pub use span_ext::SpanExt;

/// Re-export metrics module for direct access.
pub mod metrics_api {
    pub use crate::metrics::*;
}

/// Main telemetry entry point.
pub struct Telemetry;

impl Telemetry {
    /// Initialize the telemetry system.
    ///
    /// Returns a guard that shuts down telemetry when dropped.
    /// The guard must be held for the lifetime of the application.
    pub fn init(config: TelemetryConfig) -> Result<TelemetryGuard, TelemetryError> {
        provider::init(config)
    }

    /// Check if telemetry is currently enabled.
    pub fn is_enabled() -> bool {
        guard::is_enabled()
    }

    /// Disable telemetry at runtime.
    pub fn disable() {
        guard::set_enabled(false);
    }

    /// Re-enable telemetry at runtime.
    pub fn enable() {
        guard::set_enabled(true);
    }
}
