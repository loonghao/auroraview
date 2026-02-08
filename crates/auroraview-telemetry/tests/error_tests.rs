//! Tests for auroraview-telemetry error module.

use auroraview_telemetry::TelemetryError;

#[test]
fn test_error_display() {
    let err = TelemetryError::TracingInit("test error".to_string());
    assert!(err.to_string().contains("test error"));

    let err = TelemetryError::MetricsInit("metrics error".to_string());
    assert!(err.to_string().contains("metrics error"));

    let err = TelemetryError::TraceInit("trace error".to_string());
    assert!(err.to_string().contains("trace error"));

    let err = TelemetryError::OtlpConfig("otlp error".to_string());
    assert!(err.to_string().contains("otlp error"));

    let err = TelemetryError::AlreadyInitialized;
    assert!(err.to_string().contains("already initialized"));
}
