//! Tests for auroraview-telemetry metrics module.

use auroraview_telemetry::metrics_api;
use auroraview_telemetry::WebViewMetrics;
use std::sync::Arc;
use std::thread;

// ────────────────────────────────────────────────────────────
// WebViewMetrics struct API
// ────────────────────────────────────────────────────────────

#[test]
fn test_webview_metrics_creation() {
    let metrics = WebViewMetrics::new();
    // Should not panic
    metrics.webview_created("test-window");
    metrics.webview_destroyed("test-window");
}

#[test]
fn test_webview_metrics_default() {
    // Default should use new() internally
    let metrics = WebViewMetrics::default();
    metrics.webview_created("default-window");
}

#[test]
fn test_webview_metrics_record_load_time() {
    let metrics = WebViewMetrics::new();
    metrics.record_load_time("test-window", 250.0);
}

#[test]
fn test_webview_metrics_record_load_time_zero() {
    let metrics = WebViewMetrics::new();
    metrics.record_load_time("fast-window", 0.0);
}

#[test]
fn test_webview_metrics_record_load_time_large() {
    let metrics = WebViewMetrics::new();
    metrics.record_load_time("slow-window", 9999.9);
}

#[test]
fn test_webview_metrics_record_ipc() {
    let metrics = WebViewMetrics::new();
    metrics.record_ipc_message("test-window", "js_to_rust");
    metrics.record_ipc_latency("test-window", "js_to_rust", 5.2);
}

#[test]
fn test_webview_metrics_record_ipc_rust_to_js() {
    let metrics = WebViewMetrics::new();
    metrics.record_ipc_message("test-window", "rust_to_js");
    metrics.record_ipc_latency("test-window", "rust_to_js", 1.0);
}

#[test]
fn test_webview_metrics_record_ipc_zero_latency() {
    let metrics = WebViewMetrics::new();
    metrics.record_ipc_latency("test-window", "js_to_rust", 0.0);
}

#[test]
fn test_webview_metrics_record_js_eval() {
    let metrics = WebViewMetrics::new();
    metrics.record_js_eval("test-window", 12.5);
}

#[test]
fn test_webview_metrics_record_js_eval_zero() {
    let metrics = WebViewMetrics::new();
    metrics.record_js_eval("test-window", 0.0);
}

#[test]
fn test_webview_metrics_record_error() {
    let metrics = WebViewMetrics::new();
    metrics.record_error("test-window", "timeout");
}

#[test]
fn test_webview_metrics_record_error_types() {
    let metrics = WebViewMetrics::new();
    metrics.record_error("w1", "ipc_error");
    metrics.record_error("w1", "navigation_error");
    metrics.record_error("w1", "js_error");
    metrics.record_error("w1", "crash");
}

#[test]
fn test_webview_metrics_record_navigation() {
    let metrics = WebViewMetrics::new();
    metrics.record_navigation("test-window", "https://example.com");
}

#[test]
fn test_webview_metrics_record_navigation_file_url() {
    let metrics = WebViewMetrics::new();
    metrics.record_navigation("test-window", "file:///index.html");
}

#[test]
fn test_webview_metrics_record_navigation_empty() {
    let metrics = WebViewMetrics::new();
    metrics.record_navigation("test-window", "");
}

#[test]
fn test_webview_metrics_record_event_emit() {
    let metrics = WebViewMetrics::new();
    metrics.record_event_emit("test-window", "data_update");
}

#[test]
fn test_webview_metrics_record_event_emit_multiple() {
    let metrics = WebViewMetrics::new();
    metrics.record_event_emit("w1", "echo_result");
    metrics.record_event_emit("w1", "scene_loaded");
    metrics.record_event_emit("w1", "tool_applied");
}

#[test]
fn test_webview_metrics_record_memory() {
    let metrics = WebViewMetrics::new();
    metrics.record_memory("test-window", 1024 * 1024);
}

#[test]
fn test_webview_metrics_record_memory_zero() {
    let metrics = WebViewMetrics::new();
    metrics.record_memory("test-window", 0);
}

#[test]
fn test_webview_metrics_record_memory_large() {
    let metrics = WebViewMetrics::new();
    // 200 MB
    metrics.record_memory("large-window", 200 * 1024 * 1024);
}

#[test]
fn test_webview_metrics_multiple_instances() {
    let m1 = WebViewMetrics::new();
    let m2 = WebViewMetrics::new();
    m1.webview_created("win-1");
    m2.webview_created("win-2");
    m1.record_load_time("win-1", 100.0);
    m2.record_load_time("win-2", 200.0);
    m1.webview_destroyed("win-1");
    m2.webview_destroyed("win-2");
}

#[test]
fn test_webview_metrics_lifecycle_sequence() {
    let m = WebViewMetrics::new();
    m.webview_created("maya-panel");
    m.record_load_time("maya-panel", 120.0);
    m.record_navigation("maya-panel", "file:///ui/panel.html");
    m.record_ipc_message("maya-panel", "js_to_rust");
    m.record_ipc_latency("maya-panel", "js_to_rust", 3.5);
    m.record_js_eval("maya-panel", 8.0);
    m.record_event_emit("maya-panel", "maya_scene_saved");
    m.record_memory("maya-panel", 48 * 1024 * 1024);
    m.webview_destroyed("maya-panel");
}

// ────────────────────────────────────────────────────────────
// Convenience API (metrics_api module)
// ────────────────────────────────────────────────────────────

#[test]
fn test_api_record_webview_load_time() {
    metrics_api::record_webview_load_time("api-window", 180.0);
}

#[test]
fn test_api_record_webview_load_time_zero() {
    metrics_api::record_webview_load_time("api-window", 0.0);
}

#[test]
fn test_api_record_ipc_message_js_to_rust() {
    metrics_api::record_ipc_message("api-window", "js_to_rust", 4.2);
}

#[test]
fn test_api_record_ipc_message_rust_to_js() {
    metrics_api::record_ipc_message("api-window", "rust_to_js", 1.1);
}

#[test]
fn test_api_record_error() {
    metrics_api::record_error("api-window", "navigation_failed");
}

#[test]
fn test_api_record_error_multiple_types() {
    metrics_api::record_error("api-window", "timeout");
    metrics_api::record_error("api-window", "crash");
    metrics_api::record_error("api-window", "ipc_error");
}

#[test]
fn test_api_multiple_webviews() {
    for i in 0..5 {
        let id = format!("window-{i}");
        metrics_api::record_webview_load_time(&id, (i * 50) as f64);
        metrics_api::record_ipc_message(&id, "js_to_rust", (i as f64) * 2.0);
    }
}

// ────────────────────────────────────────────────────────────
// Concurrent metrics recording
// ────────────────────────────────────────────────────────────

#[test]
fn test_webview_metrics_send_sync() {
    // Verify WebViewMetrics can be shared across threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WebViewMetrics>();
}

#[test]
fn test_concurrent_load_time_recording() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];

    for i in 0..8 {
        let m = Arc::clone(&metrics);
        let window = format!("thread-window-{i}");
        handles.push(thread::spawn(move || {
            m.webview_created(&window);
            for j in 0..10 {
                m.record_load_time(&window, (j * 10 + i * 100) as f64);
            }
            m.webview_destroyed(&window);
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_ipc_recording() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];

    for i in 0..4 {
        let m = Arc::clone(&metrics);
        let window = format!("ipc-window-{i}");
        handles.push(thread::spawn(move || {
            for _ in 0..20 {
                m.record_ipc_message(&window, "js_to_rust");
                m.record_ipc_latency(&window, "js_to_rust", 2.5);
                m.record_ipc_message(&window, "rust_to_js");
                m.record_ipc_latency(&window, "rust_to_js", 1.2);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_error_recording() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];
    let error_types = ["timeout", "crash", "ipc_error", "js_error", "navigation_error"];

    for (i, &err) in error_types.iter().enumerate() {
        let m = Arc::clone(&metrics);
        let window = format!("err-window-{i}");
        handles.push(thread::spawn(move || {
            for _ in 0..15 {
                m.record_error(&window, err);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_mixed_operations() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];

    for i in 0..6 {
        let m = Arc::clone(&metrics);
        let window = format!("mixed-window-{i}");
        handles.push(thread::spawn(move || {
            m.webview_created(&window);
            m.record_load_time(&window, (50 + i * 30) as f64);
            m.record_navigation(&window, "https://example.com");
            m.record_ipc_message(&window, "js_to_rust");
            m.record_ipc_latency(&window, "js_to_rust", (i as f64) * 1.5);
            m.record_js_eval(&window, (i as f64) * 5.0);
            m.record_event_emit(&window, "data_ready");
            m.record_memory(&window, (i as u64 + 1) * 10 * 1024 * 1024);
            m.record_error(&window, "none"); // record a no-op error type
            m.webview_destroyed(&window);
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_metrics_api() {
    let mut handles = vec![];

    for i in 0..8 {
        handles.push(thread::spawn(move || {
            let window = format!("api-concurrent-{i}");
            for j in 0..10 {
                metrics_api::record_webview_load_time(&window, (j * 15 + i) as f64);
                metrics_api::record_ipc_message(&window, "js_to_rust", (j as f64) * 0.5);
                metrics_api::record_error(&window, "test_error");
            }
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_many_windows_creation_destruction() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];

    for i in 0..16 {
        let m = Arc::clone(&metrics);
        handles.push(thread::spawn(move || {
            for j in 0..5 {
                let id = format!("window-{i}-{j}");
                m.webview_created(&id);
                m.record_load_time(&id, (i * 10 + j * 5) as f64);
                m.webview_destroyed(&id);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn test_concurrent_memory_recording() {
    let metrics = Arc::new(WebViewMetrics::new());
    let mut handles = vec![];

    for i in 0..4 {
        let m = Arc::clone(&metrics);
        let window = format!("mem-window-{i}");
        handles.push(thread::spawn(move || {
            for j in 0..10 {
                let bytes = ((i + 1) as u64) * 1024 * 1024 * (j + 1) as u64;
                m.record_memory(&window, bytes);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}
