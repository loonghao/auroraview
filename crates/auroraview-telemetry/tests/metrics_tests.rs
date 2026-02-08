//! Tests for auroraview-telemetry metrics module.

use auroraview_telemetry::WebViewMetrics;

#[test]
fn test_metrics_creation() {
    let metrics = WebViewMetrics::new();
    // Should not panic
    metrics.webview_created("test-1");
    metrics.webview_destroyed("test-1");
}

#[test]
fn test_metrics_default() {
    let metrics = WebViewMetrics::default();
    metrics.record_load_time("test-1", 150.0);
    metrics.record_ipc_latency("test-1", "js_to_rust", 5.0);
    metrics.record_ipc_message("test-1", "js_to_rust");
    metrics.record_js_eval("test-1", 10.0);
    metrics.record_error("test-1", "timeout");
    metrics.record_navigation("test-1", "https://example.com");
    metrics.record_event_emit("test-1", "data_loaded");
    metrics.record_memory("test-1", 1024 * 1024);
}

#[test]
fn test_convenience_functions() {
    auroraview_telemetry::metrics_api::record_webview_load_time("test-2", 200.0);
    auroraview_telemetry::metrics_api::record_ipc_message("test-2", "rust_to_js", 3.0);
    auroraview_telemetry::metrics_api::record_error("test-2", "crash");
}
