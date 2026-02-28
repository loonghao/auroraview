//! Telemetry configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the telemetry system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (default: true).
    pub enabled: bool,

    /// Service name reported to the backend.
    pub service_name: String,

    /// Service version.
    pub service_version: String,

    /// Log level filter (e.g., "info", "debug", "auroraview=debug,warn").
    pub log_level: String,

    /// Whether to export logs to stdout (default: true).
    pub log_to_stdout: bool,

    /// Whether to export logs as JSON format (default: false).
    pub log_json: bool,

    /// OTLP endpoint for exporting telemetry data.
    /// Set to `None` to disable OTLP export (logs/metrics go to stdout only).
    pub otlp_endpoint: Option<String>,

    /// Whether to enable metrics collection (default: true).
    pub metrics_enabled: bool,

    /// Metrics export interval in seconds (default: 60).
    pub metrics_interval_secs: u64,

    /// Whether to enable trace collection (default: true).
    pub traces_enabled: bool,

    /// Sampling ratio for traces (0.0 to 1.0, default: 1.0 = sample everything).
    pub trace_sample_ratio: f64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "auroraview".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            log_level: "info".to_string(),
            log_to_stdout: true,
            log_json: false,
            otlp_endpoint: None,
            metrics_enabled: true,
            metrics_interval_secs: 60,
            traces_enabled: true,
            trace_sample_ratio: 1.0,
        }
    }
}

impl TelemetryConfig {
    /// Create a minimal config for testing (stdout only, no OTLP).
    pub fn for_testing() -> Self {
        Self {
            enabled: true,
            service_name: "auroraview-test".to_string(),
            log_level: "debug".to_string(),
            log_to_stdout: true,
            log_json: false,
            otlp_endpoint: None,
            metrics_enabled: true,
            metrics_interval_secs: 5,
            traces_enabled: true,
            trace_sample_ratio: 1.0,
            ..Default::default()
        }
    }

    /// Create config with OTLP export enabled.
    #[cfg(feature = "otlp")]
    pub fn with_otlp(endpoint: impl Into<String>) -> Self {
        Self {
            otlp_endpoint: Some(endpoint.into()),
            ..Default::default()
        }
    }
}
