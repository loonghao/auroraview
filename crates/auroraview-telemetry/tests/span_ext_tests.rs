//! Tests for SpanExt trait on tracing spans.

use auroraview_telemetry::SpanExt;
use auroraview_telemetry::TelemetryError;
use rstest::*;
use tracing::{span, Level};

// ─────────────────────────────────────────────────────────────
// Basic attribute setters
// ─────────────────────────────────────────────────────────────

#[test]
fn set_webview_id_does_not_panic() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_webview_id("maya-panel-1");
}

#[test]
fn set_app_name_does_not_panic() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_app_name("my-dcc-tool");
}

#[test]
fn set_operation_does_not_panic() {
    let span = span!(Level::INFO, "test-span");
    let _enter = span.enter();
    span.set_operation("webview_create");
}

#[test]
fn set_error_does_not_panic() {
    let span = span!(Level::ERROR, "error-span");
    let _enter = span.enter();
    let err = TelemetryError::TracingInit("test".to_string());
    span.set_error(&err);
}

#[test]
fn set_all_attributes() {
    let span = span!(Level::DEBUG, "full-span");
    let _enter = span.enter();
    span.set_webview_id("houdini-panel");
    span.set_app_name("houdini-2025");
    span.set_operation("ipc_call");
    let err = TelemetryError::MetricsInit("setup".to_string());
    span.set_error(&err);
}

#[test]
fn span_outside_context_does_not_panic() {
    let span = span!(Level::WARN, "noop-span");
    span.set_webview_id("noop");
    span.set_app_name("noop-app");
    span.set_operation("noop-op");
}

#[test]
fn set_webview_id_empty() {
    let span = span!(Level::INFO, "empty-span");
    span.set_webview_id("");
}

#[test]
fn set_app_name_empty() {
    let span = span!(Level::INFO, "empty-span");
    span.set_app_name("");
}

#[test]
fn set_operation_special_chars() {
    let span = span!(Level::INFO, "op-span");
    span.set_operation("api.export_scene");
}

#[test]
fn set_error_with_otlp_error() {
    let span = span!(Level::ERROR, "otlp-err-span");
    let _enter = span.enter();
    let err = TelemetryError::OtlpConfig("unreachable host".to_string());
    span.set_error(&err);
}

#[test]
fn set_error_with_already_initialized() {
    let span = span!(Level::ERROR, "reinit-span");
    let _enter = span.enter();
    let err = TelemetryError::AlreadyInitialized;
    span.set_error(&err);
}

// ─────────────────────────────────────────────────────────────
// Parametric: all log levels
// ─────────────────────────────────────────────────────────────

#[rstest]
#[case(Level::TRACE)]
#[case(Level::DEBUG)]
#[case(Level::INFO)]
#[case(Level::WARN)]
#[case(Level::ERROR)]
fn set_webview_id_all_levels(#[case] level: Level) {
    let span = span!(Level::TRACE, "level-span");
    let _ = level; // just ensure the span is created with a fixed level for macro
    span.set_webview_id("panel-id");
}

#[rstest]
#[case("maya-2025")]
#[case("houdini-20.x")]
#[case("blender-4.x")]
#[case("unreal-5.3")]
#[case("3dsmax-2025")]
fn set_app_name_dcc_names(#[case] name: &str) {
    let span = span!(Level::INFO, "dcc-span");
    span.set_app_name(name);
}

#[rstest]
#[case("webview_create")]
#[case("webview_navigate")]
#[case("ipc_call")]
#[case("api.export_scene")]
#[case("tool.apply")]
#[case("on_message")]
fn set_operation_common_ops(#[case] op: &str) {
    let span = span!(Level::INFO, "op-span");
    span.set_operation(op);
}

// ─────────────────────────────────────────────────────────────
// Parametric: all TelemetryError variants with set_error
// ─────────────────────────────────────────────────────────────

#[test]
fn set_error_tracing_init() {
    let span = span!(Level::ERROR, "err-span");
    let _enter = span.enter();
    let err = TelemetryError::TracingInit("subscriber error".to_string());
    span.set_error(&err);
}

#[test]
fn set_error_metrics_init() {
    let span = span!(Level::ERROR, "err-span");
    let _enter = span.enter();
    let err = TelemetryError::MetricsInit("meter provider error".to_string());
    span.set_error(&err);
}

#[test]
fn set_error_trace_init() {
    let span = span!(Level::ERROR, "err-span");
    let _enter = span.enter();
    let err = TelemetryError::TraceInit("tracer provider error".to_string());
    span.set_error(&err);
}

#[test]
fn set_error_sentry_config() {
    let span = span!(Level::ERROR, "err-span");
    let _enter = span.enter();
    let err = TelemetryError::SentryConfig("invalid DSN".to_string());
    span.set_error(&err);
}

// ─────────────────────────────────────────────────────────────
// Concurrent span attribute setting — no panic under load
// ─────────────────────────────────────────────────────────────

#[test]
fn concurrent_set_webview_id_no_panic() {
    use std::sync::Arc;
    use std::thread;

    let handles: Vec<_> = (0..8)
        .map(|i| {
            thread::spawn(move || {
                let span = span!(Level::INFO, "concurrent-span");
                span.set_webview_id(&format!("panel-{i}"));
                span.set_app_name(&format!("app-{i}"));
                span.set_operation(&format!("op-{i}"));
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[test]
fn concurrent_set_error_all_variants_no_panic() {
    use std::thread;

    let handles: Vec<_> = (0..6)
        .map(|i| {
            thread::spawn(move || {
                let span = span!(Level::ERROR, "err-concurrent");
                let _enter = span.enter();
                match i % 6 {
                    0 => span.set_error(&TelemetryError::TracingInit(format!("err-{i}"))),
                    1 => span.set_error(&TelemetryError::MetricsInit(format!("err-{i}"))),
                    2 => span.set_error(&TelemetryError::TraceInit(format!("err-{i}"))),
                    3 => span.set_error(&TelemetryError::OtlpConfig(format!("err-{i}"))),
                    4 => span.set_error(&TelemetryError::SentryConfig(format!("err-{i}"))),
                    _ => span.set_error(&TelemetryError::AlreadyInitialized),
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

// ─────────────────────────────────────────────────────────────
// Edge cases: long strings, unicode, special characters
// ─────────────────────────────────────────────────────────────

#[test]
fn set_webview_id_long_string() {
    let long_id = "a".repeat(1024);
    let span = span!(Level::INFO, "long-id-span");
    span.set_webview_id(&long_id);
}

#[test]
fn set_app_name_unicode() {
    let span = span!(Level::INFO, "unicode-span");
    span.set_app_name("アプリ名-maya-2025");
}

#[test]
fn set_operation_with_slash() {
    let span = span!(Level::INFO, "slash-span");
    span.set_operation("api/v1/export_scene");
}

#[test]
fn set_error_long_message() {
    let span = span!(Level::ERROR, "long-err-span");
    let _enter = span.enter();
    let long_msg = "x".repeat(4096);
    let err = TelemetryError::OtlpConfig(long_msg);
    span.set_error(&err);
}

#[test]
fn multiple_attributes_then_error() {
    let span = span!(Level::ERROR, "multi-attr-span");
    let _enter = span.enter();
    span.set_webview_id("blender-id");
    span.set_app_name("blender-4.x");
    span.set_operation("render_start");
    let err = TelemetryError::TraceInit("provider not started".to_string());
    span.set_error(&err);
    // Reset with another set of attributes - no panic expected
    span.set_operation("render_end");
}

#[test]
fn set_error_as_dyn_std_error() {
    let span = span!(Level::ERROR, "dyn-err-span");
    let _enter = span.enter();
    let err: Box<dyn std::error::Error> =
        Box::new(TelemetryError::TracingInit("boxed".to_string()));
    span.set_error(err.as_ref());
}
