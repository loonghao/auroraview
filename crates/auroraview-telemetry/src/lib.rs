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
mod sentry_support;
mod span_ext;

/// Python bindings via PyO3 (when `python` feature is enabled).
#[cfg(feature = "python")]
pub mod python;

/// Telemetry configuration: exporters, sampling, and feature toggles.
pub use config::TelemetryConfig;
/// Telemetry error types.
pub use error::TelemetryError;
/// RAII guard that shuts down telemetry pipelines on drop.
pub use guard::TelemetryGuard;
/// WebView performance metrics (FPS, load time, memory, IPC latency).
pub use metrics::WebViewMetrics;
/// Extension trait for `tracing::Span` with OpenTelemetry attributes.
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

    /// Check if telemetry has been initialized.
    ///
    /// Returns `true` after a successful [`Telemetry::init`] call and before
    /// the returned [`TelemetryGuard`] is dropped.
    pub fn is_initialized() -> bool {
        guard::is_initialized()
    }

    /// Capture a message to Sentry when Sentry integration is enabled.
    ///
    /// Returns `true` if Sentry capture path is active, otherwise `false`.
    pub fn capture_sentry_message(message: &str, level: &str) -> bool {
        sentry_support::capture_message(message, level)
    }
}
