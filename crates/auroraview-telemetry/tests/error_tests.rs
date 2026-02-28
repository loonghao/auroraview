//! Tests for auroraview-telemetry error module.

use auroraview_telemetry::TelemetryError;

#[test]
fn test_error_display_tracing_init() {
    let err = TelemetryError::TracingInit("test error".to_string());
    assert!(err.to_string().contains("tracing subscriber"));
}

#[test]
fn test_error_display_metrics_init() {
    let err = TelemetryError::MetricsInit("test error".to_string());
    assert!(err.to_string().contains("metrics provider"));
}

#[test]
fn test_error_display_trace_init() {
    let err = TelemetryError::TraceInit("test error".to_string());
    assert!(err.to_string().contains("trace provider"));
}

#[test]
fn test_error_display_otlp() {
    let err = TelemetryError::OtlpConfig("bad endpoint".to_string());
    assert!(err.to_string().contains("OTLP"));
}

#[test]
fn test_error_display_already_initialized() {
    let err = TelemetryError::AlreadyInitialized;
    assert!(err.to_string().contains("already initialized"));
}
