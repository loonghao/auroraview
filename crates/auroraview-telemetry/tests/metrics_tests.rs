//! Tests for auroraview-telemetry metrics module.

use auroraview_telemetry::WebViewMetrics;

#[test]
fn test_webview_metrics_creation() {
    let metrics = WebViewMetrics::new();
    // Should not panic
    metrics.webview_created("test-window");
    metrics.webview_destroyed("test-window");
}

#[test]
fn test_webview_metrics_record_load_time() {
    let metrics = WebViewMetrics::new();
    metrics.record_load_time("test-window", 250.0);
}

#[test]
fn test_webview_metrics_record_ipc() {
    let metrics = WebViewMetrics::new();
    metrics.record_ipc_message("test-window", "js_to_rust");
    metrics.record_ipc_latency("test-window", "js_to_rust", 5.2);
}

#[test]
fn test_webview_metrics_record_js_eval() {
    let metrics = WebViewMetrics::new();
    metrics.record_js_eval("test-window", 12.5);
}

#[test]
fn test_webview_metrics_record_error() {
    let metrics = WebViewMetrics::new();
    metrics.record_error("test-window", "timeout");
}

#[test]
fn test_webview_metrics_record_navigation() {
    let metrics = WebViewMetrics::new();
    metrics.record_navigation("test-window", "https://example.com");
}

#[test]
fn test_webview_metrics_record_event_emit() {
    let metrics = WebViewMetrics::new();
    metrics.record_event_emit("test-window", "data_update");
}

#[test]
fn test_webview_metrics_record_memory() {
    let metrics = WebViewMetrics::new();
    metrics.record_memory("test-window", 1024 * 1024);
}
