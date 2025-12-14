//! IPC module tests

use auroraview_core::ipc::{IpcMessage, IpcMetrics, IpcMode};

// ============================================================================
// IpcMessage Tests (from message.rs)
// ============================================================================

#[test]
fn test_ipc_message_new() {
    let msg = IpcMessage::new("test_event", serde_json::json!({"key": "value"}));
    assert_eq!(msg.event, "test_event");
    assert!(msg.id.is_none());
}

#[test]
fn test_ipc_message_with_id() {
    let msg = IpcMessage::with_id("test", serde_json::json!(null), "msg_123");
    assert_eq!(msg.event, "test");
    assert_eq!(msg.id, Some("msg_123".to_string()));
}

#[test]
fn test_ipc_message_serialize() {
    let msg = IpcMessage::new("serialize_test", serde_json::json!({"a": 1}));
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("serialize_test"));
}

#[test]
fn test_ipc_message_deserialize() {
    let json = r#"{"event":"deser_test","data":{"key":"value"},"id":"id_1"}"#;
    let msg: IpcMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.event, "deser_test");
    assert_eq!(msg.id, Some("id_1".to_string()));
}

#[test]
fn test_ipc_mode_default() {
    let mode = IpcMode::default();
    assert_eq!(mode, IpcMode::Threaded);
}

#[test]
fn test_ipc_mode_equality() {
    assert_eq!(IpcMode::Threaded, IpcMode::Threaded);
    assert_ne!(IpcMode::Threaded, IpcMode::Process);
}

// ============================================================================
// IpcMetrics Tests (from metrics.rs)
// ============================================================================

#[test]
fn test_metrics_basic() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.record_send();
    metrics.record_failure();

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.messages_sent, 2);
    assert_eq!(snapshot.messages_failed, 1);
}

#[test]
fn test_metrics_latency() {
    let metrics = IpcMetrics::new();
    metrics.record_latency(100);
    metrics.record_latency(200);
    metrics.record_latency(300);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.avg_latency_us, 200);
}

#[test]
fn test_metrics_peak_queue() {
    let metrics = IpcMetrics::new();
    metrics.update_peak_queue_length(10);
    metrics.update_peak_queue_length(5);
    metrics.update_peak_queue_length(20);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.peak_queue_length, 20);
}

#[test]
fn test_metrics_reset() {
    let metrics = IpcMetrics::new();
    metrics.record_send();
    metrics.reset();

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.messages_sent, 0);
}
