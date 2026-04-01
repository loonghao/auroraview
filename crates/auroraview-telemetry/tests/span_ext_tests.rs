//! Tests for SpanExt trait on tracing spans.

use auroraview_telemetry::SpanExt;
use tracing::span;
use tracing::Level;

#[test]
fn test_set_webview_id() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_webview_id("maya-panel-1");
}

#[test]
fn test_set_app_name() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_app_name("my-dcc-tool");
}

#[test]
fn test_set_operation() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_operation("webview_create");
}

#[test]
fn test_set_error() {
    use auroraview_telemetry::TelemetryError;
    let span = span!(Level::ERROR, "error-span");
    let _enter = span.enter();
    let err = TelemetryError::TracingInit("test".to_string());
    span.set_error(&err);
}

#[test]
fn test_set_all_attributes() {
    use auroraview_telemetry::TelemetryError;
    let span = span!(Level::DEBUG, "full-span");
    let _enter = span.enter();
    span.set_webview_id("houdini-panel");
    span.set_app_name("houdini-2025");
    span.set_operation("ipc_call");
    let err = TelemetryError::MetricsInit("setup".to_string());
    span.set_error(&err);
}

#[test]
fn test_span_outside_context_does_not_panic() {
    // Span without enter - should not panic
    let span = span!(Level::WARN, "noop-span");
    span.set_webview_id("noop");
    span.set_app_name("noop-app");
    span.set_operation("noop-op");
}

#[test]
fn test_set_webview_id_empty() {
    let span = span!(Level::INFO, "empty-span");
    span.set_webview_id("");
}

#[test]
fn test_set_app_name_empty() {
    let span = span!(Level::INFO, "empty-span");
    span.set_app_name("");
}

#[test]
fn test_set_operation_special_chars() {
    let span = span!(Level::INFO, "op-span");
    span.set_operation("api.export_scene");
}

#[test]
fn test_set_error_with_otlp_error() {
    use auroraview_telemetry::TelemetryError;
    let span = span!(Level::ERROR, "otlp-err-span");
    let _enter = span.enter();
    let err = TelemetryError::OtlpConfig("unreachable host".to_string());
    span.set_error(&err);
}

#[test]
fn test_set_error_with_already_initialized() {
    use auroraview_telemetry::TelemetryError;
    let span = span!(Level::ERROR, "reinit-span");
    let _enter = span.enter();
    let err = TelemetryError::AlreadyInitialized;
    span.set_error(&err);
}
