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

#[test]
fn test_error_display_sentry_config() {
    let err = TelemetryError::SentryConfig("invalid DSN".to_string());
    assert!(err.to_string().contains("Sentry"));
}

#[test]
fn test_error_tracing_init_contains_message() {
    let msg = "subscriber already set";
    let err = TelemetryError::TracingInit(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_metrics_init_contains_message() {
    let msg = "meter provider failed";
    let err = TelemetryError::MetricsInit(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_trace_init_contains_message() {
    let msg = "tracer provider failed";
    let err = TelemetryError::TraceInit(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_otlp_contains_message() {
    let msg = "connection refused";
    let err = TelemetryError::OtlpConfig(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_sentry_contains_message() {
    let msg = "malformed DSN";
    let err = TelemetryError::SentryConfig(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_debug_format() {
    let err = TelemetryError::AlreadyInitialized;
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("AlreadyInitialized"));
}

#[test]
fn test_error_debug_tracing_init() {
    let err = TelemetryError::TracingInit("details".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("TracingInit"));
    assert!(debug_str.contains("details"));
}

#[test]
fn test_error_is_std_error() {
    // Verify TelemetryError implements std::error::Error
    let err: &dyn std::error::Error = &TelemetryError::AlreadyInitialized;
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_tracing_init_source_is_none() {
    use std::error::Error;
    let err = TelemetryError::TracingInit("x".to_string());
    assert!(err.source().is_none());
}

#[test]
fn test_error_already_initialized_source_is_none() {
    use std::error::Error;
    let err = TelemetryError::AlreadyInitialized;
    assert!(err.source().is_none());
}

// ---------------------------------------------------------------------------
// Send + Sync bounds
// ---------------------------------------------------------------------------

#[test]
fn test_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<TelemetryError>();
}

#[test]
fn test_error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<TelemetryError>();
}

// ---------------------------------------------------------------------------
// Into<anyhow::Error> / display content validation
// ---------------------------------------------------------------------------

#[test]
fn test_error_tracing_init_display_not_empty() {
    let err = TelemetryError::TracingInit("anything".to_string());
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_metrics_init_display_not_empty() {
    let err = TelemetryError::MetricsInit("anything".to_string());
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_trace_init_display_not_empty() {
    let err = TelemetryError::TraceInit("anything".to_string());
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_otlp_display_not_empty() {
    let err = TelemetryError::OtlpConfig("anything".to_string());
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_sentry_display_not_empty() {
    let err = TelemetryError::SentryConfig("anything".to_string());
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_error_already_initialized_display_not_empty() {
    let err = TelemetryError::AlreadyInitialized;
    assert!(!err.to_string().is_empty());
}

// ---------------------------------------------------------------------------
// Debug format coverage
// ---------------------------------------------------------------------------

#[test]
fn test_error_debug_metrics_init() {
    let err = TelemetryError::MetricsInit("m".to_string());
    let s = format!("{err:?}");
    assert!(s.contains("MetricsInit"));
}

#[test]
fn test_error_debug_trace_init() {
    let err = TelemetryError::TraceInit("t".to_string());
    let s = format!("{err:?}");
    assert!(s.contains("TraceInit"));
}

#[test]
fn test_error_debug_otlp_config() {
    let err = TelemetryError::OtlpConfig("o".to_string());
    let s = format!("{err:?}");
    assert!(s.contains("OtlpConfig"));
}

#[test]
fn test_error_debug_sentry_config() {
    let err = TelemetryError::SentryConfig("s".to_string());
    let s = format!("{err:?}");
    assert!(s.contains("SentryConfig"));
}

// ---------------------------------------------------------------------------
// Unicode payload
// ---------------------------------------------------------------------------

#[test]
fn test_error_unicode_message() {
    let msg = "错误信息 — テスト";
    let err = TelemetryError::TracingInit(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_empty_message() {
    let err = TelemetryError::MetricsInit("".to_string());
    let s = err.to_string();
    assert!(!s.is_empty()); // Display should still produce some output
}
