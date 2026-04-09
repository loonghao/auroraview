//! IPC module tests

use std::sync::Arc;
use std::thread;

use auroraview_core::ipc::{IpcMessage, IpcMetrics, IpcMetricsSnapshot, IpcMode, WebViewMessage, WindowEventType};
use rstest::rstest;

// ─── IpcMessage ──────────────────────────────────────────────────────────────

#[test]
fn message_new_no_id() {
    let msg = IpcMessage::new("test_event", serde_json::json!({"key": "value"}));
    assert_eq!(msg.event, "test_event");
    assert!(msg.id.is_none());
}

#[test]
fn message_with_id() {
    let msg = IpcMessage::with_id("test", serde_json::json!(null), "msg_123");
    assert_eq!(msg.event, "test");
    assert_eq!(msg.id, Some("msg_123".to_string()));
}

#[test]
fn message_serialize_roundtrip() {
    let msg = IpcMessage::new("serialize_test", serde_json::json!({"a": 1}));
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("serialize_test"));

    let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.event, msg.event);
}

#[test]
fn message_deserialize() {
    let json = r#"{"event":"deser_test","data":{"key":"value"},"id":"id_1"}"#;
    let msg: IpcMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.event, "deser_test");
    assert_eq!(msg.id, Some("id_1".to_string()));
}

#[test]
fn message_deserialize_null_data() {
    let json = r#"{"event":"null_data","data":null}"#;
    let msg: IpcMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.event, "null_data");
    assert!(msg.data.is_null());
    assert!(msg.id.is_none());
}

#[test]
fn message_data_complex() {
    let data = serde_json::json!({
        "list": [1, 2, 3],
        "nested": {"a": {"b": true}},
        "float": 2.5
    });

    let msg = IpcMessage::new("complex", data.clone());
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.data, data);
}

#[rstest]
#[case("event.foo", "id-1")]
#[case("event/bar", "uuid-abc")]
#[case("", "empty-event")]
#[case("very.long.event.name.with.many.parts", "123456789")]
fn message_with_various_ids(#[case] event: &str, #[case] id: &str) {
    let msg = IpcMessage::with_id(event, serde_json::json!(null), id);
    assert_eq!(msg.event, event);
    assert_eq!(msg.id, Some(id.to_string()));
}

#[test]
fn message_clone() {
    let original = IpcMessage::with_id("clone_test", serde_json::json!({"x": 1}), "id-clone");
    let cloned = original.clone();
    assert_eq!(cloned.event, original.event);
    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.data, original.data);
}

// ─── IpcMode ─────────────────────────────────────────────────────────────────

#[test]
fn ipc_mode_default() {
    let mode = IpcMode::default();
    assert_eq!(mode, IpcMode::Threaded);
}

#[test]
fn ipc_mode_equality() {
    assert_eq!(IpcMode::Threaded, IpcMode::Threaded);
    assert_ne!(IpcMode::Threaded, IpcMode::Process);
    assert_eq!(IpcMode::Process, IpcMode::Process);
}

#[test]
fn ipc_mode_clone() {
    let a = IpcMode::Process;
    let b = a;
    assert_eq!(a, b);
}

// ─── WebViewMessage ───────────────────────────────────────────────────────────

#[test]
fn webview_message_eval_js() {
    let msg = WebViewMessage::EvalJs("console.log('hello')".to_string());
    match msg {
        WebViewMessage::EvalJs(script) => assert_eq!(script, "console.log('hello')"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_eval_js_async() {
    let msg = WebViewMessage::EvalJsAsync {
        script: "1+1".to_string(),
        callback_id: 42,
    };
    match msg {
        WebViewMessage::EvalJsAsync { script, callback_id } => {
            assert_eq!(script, "1+1");
            assert_eq!(callback_id, 42);
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_emit_event() {
    let msg = WebViewMessage::EmitEvent {
        event_name: "my_event".to_string(),
        data: serde_json::json!({"key": "value"}),
    };
    match msg {
        WebViewMessage::EmitEvent { event_name, data } => {
            assert_eq!(event_name, "my_event");
            assert_eq!(data["key"], "value");
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_load_url() {
    let msg = WebViewMessage::LoadUrl("https://example.com".to_string());
    match msg {
        WebViewMessage::LoadUrl(url) => assert_eq!(url, "https://example.com"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_load_html() {
    let msg = WebViewMessage::LoadHtml("<html>test</html>".to_string());
    match msg {
        WebViewMessage::LoadHtml(html) => assert!(html.contains("test")),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_set_visible_true() {
    let msg = WebViewMessage::SetVisible(true);
    match msg {
        WebViewMessage::SetVisible(v) => assert!(v),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_set_visible_false() {
    let msg = WebViewMessage::SetVisible(false);
    match msg {
        WebViewMessage::SetVisible(v) => assert!(!v),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn webview_message_reload() {
    let msg = WebViewMessage::Reload;
    matches!(msg, WebViewMessage::Reload);
}

#[test]
fn webview_message_stop_loading() {
    let msg = WebViewMessage::StopLoading;
    matches!(msg, WebViewMessage::StopLoading);
}

#[test]
fn webview_message_close() {
    let msg = WebViewMessage::Close;
    matches!(msg, WebViewMessage::Close);
}

#[test]
fn webview_message_window_event() {
    let msg = WebViewMessage::WindowEvent {
        event_type: WindowEventType::Resized,
        data: serde_json::json!({"width": 800, "height": 600}),
    };
    match msg {
        WebViewMessage::WindowEvent { event_type, data } => {
            assert_eq!(event_type, WindowEventType::Resized);
            assert_eq!(data["width"], 800);
        }
        _ => panic!("Wrong variant"),
    }
}

// ─── WindowEventType ─────────────────────────────────────────────────────────

#[rstest]
#[case(WindowEventType::Shown, "shown")]
#[case(WindowEventType::Hidden, "hidden")]
#[case(WindowEventType::Closing, "closing")]
#[case(WindowEventType::Closed, "closed")]
#[case(WindowEventType::Focused, "focused")]
#[case(WindowEventType::Blurred, "blurred")]
#[case(WindowEventType::Minimized, "minimized")]
#[case(WindowEventType::Maximized, "maximized")]
#[case(WindowEventType::Restored, "restored")]
#[case(WindowEventType::Resized, "resized")]
#[case(WindowEventType::Moved, "moved")]
#[case(WindowEventType::LoadStarted, "load_started")]
#[case(WindowEventType::LoadFinished, "load_finished")]
#[case(WindowEventType::NavigationStarted, "navigation_started")]
#[case(WindowEventType::NavigationFinished, "navigation_finished")]
#[case(WindowEventType::WebView2Created, "webview2_created")]
fn window_event_type_as_str(#[case] event: WindowEventType, #[case] expected: &str) {
    assert_eq!(event.as_str(), expected);
}

#[rstest]
#[case(WindowEventType::Shown, "shown")]
#[case(WindowEventType::Resized, "resized")]
#[case(WindowEventType::NavigationFinished, "navigation_finished")]
fn window_event_type_display(#[case] event: WindowEventType, #[case] expected: &str) {
    assert_eq!(format!("{}", event), expected);
}

#[test]
fn window_event_type_equality() {
    assert_eq!(WindowEventType::Shown, WindowEventType::Shown);
    assert_ne!(WindowEventType::Shown, WindowEventType::Hidden);
}

#[test]
fn window_event_type_clone() {
    let a = WindowEventType::Focused;
    let b = a.clone();
    assert_eq!(a, b);
}

// ─── IpcMetrics basic ─────────────────────────────────────────────────────────

#[test]
fn metrics_new_all_zero() {
    let metrics = IpcMetrics::new();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 0);
    assert_eq!(snap.messages_failed, 0);
    assert_eq!(snap.messages_dropped, 0);
    assert_eq!(snap.retry_attempts, 0);
    assert_eq!(snap.avg_latency_us, 0);
    assert_eq!(snap.peak_queue_length, 0);
    assert_eq!(snap.messages_received, 0);
    assert_eq!(snap.success_rate, 100.0);
}

#[test]
fn metrics_default_all_zero() {
    let metrics = IpcMetrics::default();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 0);
}

#[test]
fn metrics_record_send() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_send();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 2);
}

#[test]
fn metrics_record_failure() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_send();
    metrics.record_failure();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 2);
    assert_eq!(snap.messages_failed, 1);
}

#[test]
fn metrics_record_drop() {
    let metrics = IpcMetrics::new();
    metrics.record_drop();
    metrics.record_drop();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_dropped, 2);
}

#[test]
fn metrics_record_retry() {
    let metrics = IpcMetrics::new();
    metrics.record_retry();
    metrics.record_retry();
    metrics.record_retry();
    let snap = metrics.snapshot();
    assert_eq!(snap.retry_attempts, 3);
}

#[test]
fn metrics_record_receive() {
    let metrics = IpcMetrics::new();
    metrics.record_receive();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_received, 1);
}

#[test]
fn metrics_latency_average() {
    let metrics = IpcMetrics::new();
    metrics.record_latency(100);
    metrics.record_latency(200);
    metrics.record_latency(300);
    let snap = metrics.snapshot();
    assert_eq!(snap.avg_latency_us, 200);
}

#[test]
fn metrics_latency_no_samples() {
    let metrics = IpcMetrics::new();
    let snap = metrics.snapshot();
    assert_eq!(snap.avg_latency_us, 0);
}

#[test]
fn metrics_peak_queue() {
    let metrics = IpcMetrics::new();
    metrics.update_peak_queue_length(10);
    metrics.update_peak_queue_length(5);
    metrics.update_peak_queue_length(20);
    let snap = metrics.snapshot();
    assert_eq!(snap.peak_queue_length, 20);
}

#[test]
fn metrics_peak_queue_monotonic() {
    let metrics = IpcMetrics::new();
    metrics.update_peak_queue_length(100);
    metrics.update_peak_queue_length(50);
    let snap = metrics.snapshot();
    assert_eq!(snap.peak_queue_length, 100); // Never decreases
}

#[test]
fn metrics_success_rate_all_success() {
    let metrics = IpcMetrics::new();
    for _ in 0..10 {
        metrics.record_send();
    }
    let snap = metrics.snapshot();
    assert_eq!(snap.success_rate, 100.0);
}

#[test]
fn metrics_success_rate_mixed() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_failure();
    let snap = metrics.snapshot();
    assert!((snap.success_rate - 50.0).abs() < 0.01);
}

#[test]
fn metrics_success_rate_no_messages() {
    let metrics = IpcMetrics::new();
    let snap = metrics.snapshot();
    assert_eq!(snap.success_rate, 100.0);
}

#[test]
fn metrics_reset() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_failure();
    metrics.record_latency(1000);
    metrics.reset();
    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 0);
    assert_eq!(snap.messages_failed, 0);
    assert_eq!(snap.avg_latency_us, 0);
}

// ─── IpcMetricsSnapshot ──────────────────────────────────────────────────────

#[test]
fn metrics_snapshot_format_contains_key_fields() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_failure();
    metrics.record_latency(500);
    let snap = metrics.snapshot();
    let formatted = snap.format();

    assert!(formatted.contains("Messages Sent"));
    assert!(formatted.contains("Messages Failed"));
    assert!(formatted.contains("Success Rate"));
    assert!(formatted.contains("Avg Latency"));
}

#[test]
fn metrics_snapshot_serde_roundtrip() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_latency(200);
    let snap = metrics.snapshot();

    let json = serde_json::to_string(&snap).unwrap();
    let parsed: IpcMetricsSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.messages_sent, snap.messages_sent);
    assert_eq!(parsed.avg_latency_us, snap.avg_latency_us);
    assert!((parsed.success_rate - snap.success_rate).abs() < 0.001);
}

#[test]
fn metrics_snapshot_clone() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    let snap = metrics.snapshot();
    let cloned = snap.clone();
    assert_eq!(cloned.messages_sent, snap.messages_sent);
}

// ─── IpcMetrics clone/Arc shared ─────────────────────────────────────────────

#[test]
fn metrics_clone_shares_counters() {
    let metrics = IpcMetrics::new();
    let cloned = metrics.clone();

    metrics.record_send();
    cloned.record_send();

    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 2);
}

// ─── Concurrent metrics ───────────────────────────────────────────────────────

#[test]
fn metrics_concurrent_sends() {
    let metrics = Arc::new(IpcMetrics::new());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let m = Arc::clone(&metrics);
        let h = thread::spawn(move || {
            for _ in 0..100 {
                m.record_send();
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    let snap = metrics.snapshot();
    assert_eq!(snap.messages_sent, 1000);
}

#[test]
fn metrics_concurrent_mixed_ops() {
    let metrics = Arc::new(IpcMetrics::new());
    let mut handles = Vec::new();

    for i in 0..8 {
        let m = Arc::clone(&metrics);
        let h = thread::spawn(move || {
            for j in 0..50 {
                if (i + j) % 3 == 0 {
                    m.record_send();
                } else if (i + j) % 3 == 1 {
                    m.record_failure();
                } else {
                    m.record_latency(((i * 10 + j) as u64) % 500 + 1);
                }
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    // Just verify no panic and snapshot is consistent
    let snap = metrics.snapshot();
    assert!(snap.messages_sent + snap.messages_failed > 0 || snap.avg_latency_us == 0);
}

#[test]
fn metrics_concurrent_peak_queue() {
    let metrics = Arc::new(IpcMetrics::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let m = Arc::clone(&metrics);
        let h = thread::spawn(move || {
            m.update_peak_queue_length(i * 7 + 3);
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    let snap = metrics.snapshot();
    // Max of (3, 10, 17, 24, 31, 38, 45, 52, 59, 66) = 66
    assert_eq!(snap.peak_queue_length, 66);
}
