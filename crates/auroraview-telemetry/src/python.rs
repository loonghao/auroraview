//! Python bindings for AuroraView Telemetry via PyO3
//!
//! Exposes OpenTelemetry-based telemetry to Python for performance debugging
//! and signal/thread profiling in DCC environments.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use parking_lot::Mutex;

use crate::config::TelemetryConfig;
use crate::guard::TelemetryGuard;
use crate::metrics::WebViewMetrics;
use crate::Telemetry;

// Global guard storage - keeps the telemetry alive across Python calls
static GUARD: Mutex<Option<TelemetryGuard>> = Mutex::new(None);

// ============================================================================
// PyTelemetryConfig
// ============================================================================

/// Telemetry configuration (Python-facing).
///
/// Example:
/// ```python
/// from auroraview._core import TelemetryConfig
///
/// config = TelemetryConfig()
/// config.service_name = "my-app"
/// config.log_level = "debug"
/// config.otlp_endpoint = "http://localhost:4317"
/// ```
#[pyclass(name = "TelemetryConfig")]
#[derive(Clone)]
pub struct PyTelemetryConfig {
    /// Whether telemetry is enabled.
    #[pyo3(get, set)]
    pub enabled: bool,

    /// Service name reported to the backend.
    #[pyo3(get, set)]
    pub service_name: String,

    /// Service version.
    #[pyo3(get, set)]
    pub service_version: String,

    /// Log level filter (e.g., "info", "debug", "auroraview=debug,warn").
    #[pyo3(get, set)]
    pub log_level: String,

    /// Whether to export logs to stdout.
    #[pyo3(get, set)]
    pub log_to_stdout: bool,

    /// Whether to export logs as JSON format.
    #[pyo3(get, set)]
    pub log_json: bool,

    /// OTLP endpoint (e.g., "http://localhost:4317"). None to disable.
    #[pyo3(get, set)]
    pub otlp_endpoint: Option<String>,

    /// Whether to enable metrics collection.
    #[pyo3(get, set)]
    pub metrics_enabled: bool,

    /// Metrics export interval in seconds.
    #[pyo3(get, set)]
    pub metrics_interval_secs: u64,

    /// Whether to enable trace collection.
    #[pyo3(get, set)]
    pub traces_enabled: bool,

    /// Sampling ratio for traces (0.0 to 1.0).
    #[pyo3(get, set)]
    pub trace_sample_ratio: f64,

    /// Optional Sentry DSN.
    #[pyo3(get, set)]
    pub sentry_dsn: Option<String>,

    /// Optional Sentry environment.
    #[pyo3(get, set)]
    pub sentry_environment: Option<String>,

    /// Optional Sentry release value.
    #[pyo3(get, set)]
    pub sentry_release: Option<String>,

    /// Sentry error sampling ratio (0.0 to 1.0).
    #[pyo3(get, set)]
    pub sentry_sample_rate: f32,

    /// Sentry transaction sampling ratio (0.0 to 1.0).
    #[pyo3(get, set)]
    pub sentry_traces_sample_rate: f32,
}

#[pymethods]
impl PyTelemetryConfig {
    /// Create a new config with default values.
    #[new]
    #[pyo3(signature = (
        enabled=true,
        service_name=None,
        service_version=None,
        log_level=None,
        log_to_stdout=true,
        log_json=false,
        otlp_endpoint=None,
        metrics_enabled=true,
        metrics_interval_secs=60,
        traces_enabled=true,
        trace_sample_ratio=1.0,
        sentry_dsn=None,
        sentry_environment=None,
        sentry_release=None,
        sentry_sample_rate=1.0,
        sentry_traces_sample_rate=0.0,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        enabled: bool,

        service_name: Option<String>,
        service_version: Option<String>,
        log_level: Option<String>,
        log_to_stdout: bool,
        log_json: bool,
        otlp_endpoint: Option<String>,
        metrics_enabled: bool,
        metrics_interval_secs: u64,
        traces_enabled: bool,
        trace_sample_ratio: f64,
        sentry_dsn: Option<String>,
        sentry_environment: Option<String>,
        sentry_release: Option<String>,
        sentry_sample_rate: f32,
        sentry_traces_sample_rate: f32,
    ) -> Self {
        let defaults = TelemetryConfig::default();
        Self {
            enabled,
            service_name: service_name.unwrap_or(defaults.service_name),
            service_version: service_version.unwrap_or(defaults.service_version),
            log_level: log_level.unwrap_or(defaults.log_level),
            log_to_stdout,
            log_json,
            otlp_endpoint,
            metrics_enabled,
            metrics_interval_secs,
            traces_enabled,
            trace_sample_ratio,
            sentry_dsn,
            sentry_environment,
            sentry_release,
            sentry_sample_rate,
            sentry_traces_sample_rate,
        }
    }

    /// Create a config for testing (stdout only, debug level).
    #[staticmethod]
    fn for_testing() -> Self {
        TelemetryConfig::for_testing().into()
    }

    fn __repr__(&self) -> String {
        format!(
            "TelemetryConfig(enabled={}, service_name='{}', log_level='{}', \
             otlp_endpoint={}, sentry_dsn={}, metrics={}, traces={})",
            self.enabled,
            self.service_name,
            self.log_level,
            self.otlp_endpoint
                .as_deref()
                .map_or("None".to_string(), |e| format!("'{e}'")),
            self.sentry_dsn
                .as_deref()
                .map_or("None".to_string(), |e| format!("'{e}'")),
            self.metrics_enabled,
            self.traces_enabled,
        )
    }
}

impl From<TelemetryConfig> for PyTelemetryConfig {
    fn from(c: TelemetryConfig) -> Self {
        Self {
            enabled: c.enabled,
            service_name: c.service_name,
            service_version: c.service_version,
            log_level: c.log_level,
            log_to_stdout: c.log_to_stdout,
            log_json: c.log_json,
            otlp_endpoint: c.otlp_endpoint,
            metrics_enabled: c.metrics_enabled,
            metrics_interval_secs: c.metrics_interval_secs,
            traces_enabled: c.traces_enabled,
            trace_sample_ratio: c.trace_sample_ratio,
            sentry_dsn: c.sentry_dsn,
            sentry_environment: c.sentry_environment,
            sentry_release: c.sentry_release,
            sentry_sample_rate: c.sentry_sample_rate,
            sentry_traces_sample_rate: c.sentry_traces_sample_rate,
        }
    }
}

impl From<&PyTelemetryConfig> for TelemetryConfig {
    fn from(c: &PyTelemetryConfig) -> Self {
        Self {
            enabled: c.enabled,
            service_name: c.service_name.clone(),
            service_version: c.service_version.clone(),
            log_level: c.log_level.clone(),
            log_to_stdout: c.log_to_stdout,
            log_json: c.log_json,
            otlp_endpoint: c.otlp_endpoint.clone(),
            metrics_enabled: c.metrics_enabled,
            metrics_interval_secs: c.metrics_interval_secs,
            traces_enabled: c.traces_enabled,
            trace_sample_ratio: c.trace_sample_ratio,
            sentry_dsn: c.sentry_dsn.clone(),
            sentry_environment: c.sentry_environment.clone(),
            sentry_release: c.sentry_release.clone(),
            sentry_sample_rate: c.sentry_sample_rate,
            sentry_traces_sample_rate: c.sentry_traces_sample_rate,
        }
    }
}

// ============================================================================
// PyWebViewMetrics
// ============================================================================

/// WebView metrics collector (Python-facing).
///
/// Records OpenTelemetry metrics for WebView operations.
///
/// Example:
/// ```python
/// from auroraview._core import WebViewMetrics
///
/// metrics = WebViewMetrics()
/// metrics.webview_created("main-window")
/// metrics.record_load_time("main-window", 180.0)
/// metrics.record_ipc_latency("main-window", "js_to_rust", 5.2)
/// metrics.webview_destroyed("main-window")
/// ```
#[pyclass(name = "WebViewMetrics")]
pub struct PyWebViewMetrics {
    inner: WebViewMetrics,
}

#[pymethods]
impl PyWebViewMetrics {
    /// Create a new metrics instance from the global meter provider.
    #[new]
    fn new() -> Self {
        Self {
            inner: WebViewMetrics::new(),
        }
    }

    /// Record a WebView instance creation.
    fn webview_created(&self, webview_id: &str) {
        self.inner.webview_created(webview_id);
    }

    /// Record a WebView instance destruction.
    fn webview_destroyed(&self, webview_id: &str) {
        self.inner.webview_destroyed(webview_id);
    }

    /// Record page load time (ms).
    fn record_load_time(&self, webview_id: &str, duration_ms: f64) {
        self.inner.record_load_time(webview_id, duration_ms);
    }

    /// Record IPC message latency (ms).
    fn record_ipc_latency(&self, webview_id: &str, direction: &str, latency_ms: f64) {
        self.inner
            .record_ipc_latency(webview_id, direction, latency_ms);
    }

    /// Increment IPC message counter.
    fn record_ipc_message(&self, webview_id: &str, direction: &str) {
        self.inner.record_ipc_message(webview_id, direction);
    }

    /// Record a JavaScript evaluation with duration (ms).
    fn record_js_eval(&self, webview_id: &str, duration_ms: f64) {
        self.inner.record_js_eval(webview_id, duration_ms);
    }

    /// Record an error.
    fn record_error(&self, webview_id: &str, error_type: &str) {
        self.inner.record_error(webview_id, error_type);
    }

    /// Record a navigation event.
    fn record_navigation(&self, webview_id: &str, url: &str) {
        self.inner.record_navigation(webview_id, url);
    }

    /// Record an event emission (Python -> JS).
    fn record_event_emit(&self, webview_id: &str, event_name: &str) {
        self.inner.record_event_emit(webview_id, event_name);
    }

    /// Record memory usage (bytes).
    fn record_memory(&self, webview_id: &str, bytes: u64) {
        self.inner.record_memory(webview_id, bytes);
    }

    fn __repr__(&self) -> String {
        "WebViewMetrics()".to_string()
    }
}

// ============================================================================
// Module-level functions
// ============================================================================

/// Initialize the telemetry system.
///
/// Must be called before recording any metrics or traces. The system remains
/// active until ``shutdown_telemetry()`` is called or the process exits.
///
/// Example:
/// ```python
/// from auroraview._core import init_telemetry, TelemetryConfig
///
/// config = TelemetryConfig(log_level="debug")
/// init_telemetry(config)
/// ```
#[pyfunction]
#[pyo3(signature = (config=None))]
fn init_telemetry(config: Option<&PyTelemetryConfig>) -> PyResult<()> {
    let rust_config = config.map_or_else(TelemetryConfig::default, TelemetryConfig::from);

    let guard = Telemetry::init(rust_config).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let mut slot = GUARD.lock();
    *slot = Some(guard);

    Ok(())
}

/// Shutdown the telemetry system, flushing all pending data.
#[pyfunction]
fn shutdown_telemetry() -> PyResult<()> {
    let mut slot = GUARD.lock();
    *slot = None; // Drop triggers TelemetryGuard::drop -> flush + shutdown
    Ok(())
}

/// Check if telemetry is currently enabled.
#[pyfunction]
fn is_telemetry_enabled() -> bool {
    Telemetry::is_enabled()
}

/// Disable telemetry at runtime.
#[pyfunction]
fn disable_telemetry() {
    Telemetry::disable();
}

/// Re-enable telemetry at runtime.
#[pyfunction]
fn enable_telemetry() {
    Telemetry::enable();
}

/// Convenience: record WebView load time (ms).
#[pyfunction]
fn record_webview_load_time(webview_id: &str, duration_ms: f64) {
    crate::metrics::record_webview_load_time(webview_id, duration_ms);
}

/// Convenience: record an IPC message with latency (ms).
#[pyfunction]
fn record_ipc_message(webview_id: &str, direction: &str, latency_ms: f64) {
    crate::metrics::record_ipc_message(webview_id, direction, latency_ms);
}

/// Convenience: record an error.
#[pyfunction]
fn record_telemetry_error(webview_id: &str, error_type: &str) {
    crate::metrics::record_error(webview_id, error_type);
}

/// Capture a message to Sentry.
///
/// Returns `True` when Sentry feature is active and the message was accepted.
#[pyfunction]
#[pyo3(signature = (message, level="error"))]
fn capture_sentry_message(message: &str, level: &str) -> bool {
    Telemetry::capture_sentry_message(message, level)
}

// ============================================================================
// Module registration
// ============================================================================

/// Register the telemetry submodule on the parent Python module.
pub fn register_module(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent.py(), "telemetry")?;

    // Classes
    m.add_class::<PyTelemetryConfig>()?;
    m.add_class::<PyWebViewMetrics>()?;

    // Functions
    m.add_function(wrap_pyfunction!(init_telemetry, &m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_telemetry, &m)?)?;
    m.add_function(wrap_pyfunction!(is_telemetry_enabled, &m)?)?;
    m.add_function(wrap_pyfunction!(disable_telemetry, &m)?)?;
    m.add_function(wrap_pyfunction!(enable_telemetry, &m)?)?;
    m.add_function(wrap_pyfunction!(record_webview_load_time, &m)?)?;
    m.add_function(wrap_pyfunction!(record_ipc_message, &m)?)?;
    m.add_function(wrap_pyfunction!(record_telemetry_error, &m)?)?;
    m.add_function(wrap_pyfunction!(capture_sentry_message, &m)?)?;

    parent.add_submodule(&m)?;
    Ok(())
}
