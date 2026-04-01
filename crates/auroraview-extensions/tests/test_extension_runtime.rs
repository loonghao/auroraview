//! Integration tests for ExtensionRuntime and RuntimeManager (src/runtime.rs)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::json;

use auroraview_extensions::runtime::{
    create_runtime_manager, ExtensionMessage, ExtensionRuntime, MessageType, Port, RuntimeManager,
    RuntimeState,
};

// ─── ExtensionMessage ────────────────────────────────────────────────────────

#[test]
fn test_message_new() {
    let msg = ExtensionMessage::new("ext-a".to_string(), json!({"key": "value"}));
    assert_eq!(msg.source, "ext-a");
    assert!(msg.target.is_none());
    assert_eq!(msg.message_type, MessageType::Message);
    assert!(msg.callback_id.is_none());
    assert!(!msg.id.is_empty());
    assert!(msg.timestamp > 0);
}

#[test]
fn test_message_to_sets_target() {
    let msg = ExtensionMessage::new("ext-a".to_string(), json!(null))
        .to("ext-b".to_string());
    assert_eq!(msg.target, Some("ext-b".to_string()));
}

#[test]
fn test_message_request_has_callback_id() {
    let req = ExtensionMessage::request("ext-a".to_string(), json!({"action": "ping"}));
    assert_eq!(req.message_type, MessageType::Request);
    assert!(req.callback_id.is_some());
    let cb = req.callback_id.as_ref().unwrap();
    assert!(!cb.is_empty());
}

#[test]
fn test_message_response_fields() {
    let req = ExtensionMessage::request("ext-a".to_string(), json!({"x": 1}));
    let resp = ExtensionMessage::response(&req, json!({"result": 42}));
    assert_eq!(resp.message_type, MessageType::Response);
    assert_eq!(resp.callback_id, req.callback_id);
    assert_eq!(resp.target, Some("ext-a".to_string()));
    assert_eq!(resp.payload, json!({"result": 42}));
}

// ─── ExtensionRuntime lifecycle ───────────────────────────────────────────────

#[test]
fn test_runtime_initial_state() {
    let rt = ExtensionRuntime::new("test-ext".to_string());
    assert_eq!(rt.state(), RuntimeState::Stopped);
    assert_eq!(rt.extension_id(), "test-ext");
}

#[test]
fn test_runtime_start_stop() {
    let rt = ExtensionRuntime::new("test-ext".to_string());
    rt.start().unwrap();
    assert_eq!(rt.state(), RuntimeState::Running);
    rt.stop().unwrap();
    assert_eq!(rt.state(), RuntimeState::Stopped);
}

#[test]
fn test_runtime_start_idempotent() {
    let rt = ExtensionRuntime::new("test-ext".to_string());
    rt.start().unwrap();
    // Second start should not error
    rt.start().unwrap();
    assert_eq!(rt.state(), RuntimeState::Running);
}

#[test]
fn test_runtime_stop_clears_ports() {
    let rt = ExtensionRuntime::new("test-ext".to_string());
    rt.start().unwrap();
    let port = rt.connect("my-port", None);
    assert!(rt.get_port(&port.id).is_some());
    rt.stop().unwrap();
    // After stop, port should be gone (ports.clear())
    assert!(rt.get_port(&port.id).is_none());
}

// ─── Message handling ─────────────────────────────────────────────────────────

#[test]
fn test_send_message_while_running() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();
    rt.on_message("h1", move |_msg, _sender| {
        counter2.fetch_add(1, Ordering::SeqCst);
        None
    });

    let msg = ExtensionMessage::new("ext-b".to_string(), json!({"data": 1}));
    rt.send_message(msg).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_send_message_queued_when_stopped() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    // Runtime is stopped — message should be queued
    let msg = ExtensionMessage::new("ext-b".to_string(), json!({"queued": true}));
    let result = rt.send_message(msg).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_pending_messages_flushed_on_start() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let rt = ExtensionRuntime::new("ext-a".to_string());

    // Queue two messages while stopped
    rt.send_message(ExtensionMessage::new("ext-b".to_string(), json!(1)))
        .unwrap();
    rt.send_message(ExtensionMessage::new("ext-b".to_string(), json!(2)))
        .unwrap();

    rt.on_message("h1", move |_msg, _sender| {
        counter2.fetch_add(1, Ordering::SeqCst);
        None
    });

    // Start should flush the queue
    rt.start().unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_handler_returns_value() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();
    rt.on_message("echo", |msg, _sender| {
        Some(msg.payload.clone())
    });

    let msg = ExtensionMessage::new("caller".to_string(), json!({"echo": "hello"}));
    let resp = rt.send_message(msg).unwrap();
    assert_eq!(resp, Some(json!({"echo": "hello"})));
}

#[test]
fn test_remove_handler() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();
    rt.on_message("h1", move |_msg, _sender| {
        counter2.fetch_add(1, Ordering::SeqCst);
        None
    });

    let msg1 = ExtensionMessage::new("x".to_string(), json!(null));
    rt.send_message(msg1).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    rt.remove_handler("h1");
    let msg2 = ExtensionMessage::new("x".to_string(), json!(null));
    rt.send_message(msg2).unwrap();
    // Handler removed — count should still be 1
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// ─── Async message / response ─────────────────────────────────────────────────

#[test]
fn test_send_message_async_and_handle_response() {
    let received = Arc::new(AtomicUsize::new(0));
    let received2 = received.clone();

    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let req = ExtensionMessage::request("caller".to_string(), json!({"op": "test"}));
    let cb_id = req.callback_id.clone().unwrap();

    rt.send_message_async(req, 5000, move |payload| {
        if payload == json!({"ok": true}) {
            received2.fetch_add(1, Ordering::SeqCst);
        }
    })
    .unwrap();

    rt.handle_response(&cb_id, json!({"ok": true}));
    assert_eq!(received.load(Ordering::SeqCst), 1);
}

#[test]
fn test_handle_response_unknown_callback_noop() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    // Should not panic
    rt.handle_response("nonexistent-cb-id", json!(null));
}

#[test]
fn test_cleanup_expired_responses() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let req = ExtensionMessage::request("caller".to_string(), json!({}));
    // Use a very short timeout that is already expired (negative offset)
    rt.send_message_async(req, -10000, |_| {}).unwrap();

    // cleanup should remove the expired entry without panicking
    rt.cleanup_expired_responses();
    // Sending a response for a cleaned-up callback should be a noop
    rt.handle_response("whatever", json!(null));
}

// ─── Port connections ─────────────────────────────────────────────────────────

#[test]
fn test_port_connect_and_get() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let port = rt.connect("chan", Some("ext-b"));
    assert!(port.connected);
    assert_eq!(port.name, "chan");
    assert_eq!(port.source_extension_id, "ext-a");
    assert_eq!(port.target_extension_id, Some("ext-b".to_string()));

    let fetched: Port = rt.get_port(&port.id).unwrap();
    assert_eq!(fetched.id, port.id);
}

#[test]
fn test_port_disconnect() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let port = rt.connect("c", None);
    rt.disconnect(&port.id);

    let updated = rt.get_port(&port.id).unwrap();
    assert!(!updated.connected);
}

#[test]
fn test_port_post_message_success() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let port = rt.connect("data", None);
    // Should succeed since port is connected and runtime is running
    rt.port_post_message(&port.id, json!({"msg": "hello"}))
        .unwrap();
}

#[test]
fn test_port_post_message_disconnected_error() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let port = rt.connect("data", None);
    rt.disconnect(&port.id);

    let result = rt.port_post_message(&port.id, json!({"msg": "hello"}));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("disconnected") || err.contains("Port"));
}

#[test]
fn test_port_post_message_not_found_error() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    let result = rt.port_post_message("no-such-port-id", json!(null));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Port"));
}

// ─── Event listener ───────────────────────────────────────────────────────────

#[test]
fn test_add_remove_event_listener() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    rt.add_event_listener("install", "l1");
    rt.add_event_listener("install", "l2");
    // Duplicate add should not duplicate
    rt.add_event_listener("install", "l1");

    rt.remove_event_listener("install", "l1");
    // After removal, l2 should still be there; just verify no panic
    rt.add_event_listener("install", "l3");
}

#[test]
fn test_dispatch_event_while_running() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    rt.start().unwrap();

    rt.add_event_listener("startup", "l1");
    // dispatch_event sends a message; runtime is running so handlers are invoked
    rt.dispatch_event("startup", json!({"reason": "install"}))
        .unwrap();
}

#[test]
fn test_dispatch_event_while_stopped_queues() {
    let rt = ExtensionRuntime::new("ext-a".to_string());
    // stopped → queued
    rt.dispatch_event("startup", json!({})).unwrap();
}

// ─── RuntimeManager ──────────────────────────────────────────────────────────

#[test]
fn test_manager_create_and_get_runtime() {
    let mgr = RuntimeManager::new();
    let rt = mgr.create_runtime("ext-a".to_string()).unwrap();
    assert_eq!(rt.extension_id(), "ext-a");

    let got = mgr.get_runtime("ext-a").unwrap();
    assert_eq!(got.extension_id(), "ext-a");
}

#[test]
fn test_manager_create_duplicate_errors() {
    let mgr = RuntimeManager::new();
    mgr.create_runtime("ext-a".to_string()).unwrap();
    let result = mgr.create_runtime("ext-a".to_string());
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("ext-a"));
}

#[test]
fn test_manager_get_nonexistent_returns_none() {
    let mgr = RuntimeManager::new();
    assert!(mgr.get_runtime("no-such-ext").is_none());
}

#[test]
fn test_manager_remove_runtime() {
    let mgr = RuntimeManager::new();
    mgr.create_runtime("ext-a".to_string()).unwrap();
    let removed = mgr.remove_runtime("ext-a");
    assert!(removed.is_some());
    assert!(mgr.get_runtime("ext-a").is_none());
}

#[test]
fn test_manager_remove_nonexistent_returns_none() {
    let mgr = RuntimeManager::new();
    assert!(mgr.remove_runtime("not-there").is_none());
}

#[test]
fn test_manager_extension_ids() {
    let mgr = RuntimeManager::new();
    mgr.create_runtime("ext-a".to_string()).unwrap();
    mgr.create_runtime("ext-b".to_string()).unwrap();

    let mut ids = mgr.extension_ids();
    ids.sort();
    assert_eq!(ids, vec!["ext-a".to_string(), "ext-b".to_string()]);
}

#[test]
fn test_manager_start_all_stop_all() {
    let mgr = RuntimeManager::new();
    let rt_a = mgr.create_runtime("ext-a".to_string()).unwrap();
    let rt_b = mgr.create_runtime("ext-b".to_string()).unwrap();

    mgr.start_all().unwrap();
    assert_eq!(rt_a.state(), RuntimeState::Running);
    assert_eq!(rt_b.state(), RuntimeState::Running);

    mgr.stop_all().unwrap();
    assert_eq!(rt_a.state(), RuntimeState::Stopped);
    assert_eq!(rt_b.state(), RuntimeState::Stopped);
}

#[test]
fn test_manager_send_to_existing() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();

    let mgr = RuntimeManager::new();
    let rt = mgr.create_runtime("ext-a".to_string()).unwrap();
    rt.start().unwrap();
    rt.on_message("h1", move |_msg, _sender| {
        counter2.fetch_add(1, Ordering::SeqCst);
        None
    });

    let msg = ExtensionMessage::new("caller".to_string(), json!({}));
    mgr.send_to("ext-a", msg).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_manager_send_to_nonexistent_errors() {
    let mgr = RuntimeManager::new();
    let msg = ExtensionMessage::new("caller".to_string(), json!({}));
    let result = mgr.send_to("no-ext", msg);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("no-ext"));
}

#[test]
fn test_manager_broadcast_reaches_all() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mgr = RuntimeManager::new();
    for name in ["ext-a", "ext-b", "ext-c"] {
        let c = counter.clone();
        let rt = mgr.create_runtime(name.to_string()).unwrap();
        rt.start().unwrap();
        rt.on_message("count", move |_msg, _sender| {
            c.fetch_add(1, Ordering::SeqCst);
            None
        });
    }

    let msg = ExtensionMessage::new("broadcaster".to_string(), json!({"event": "all"}));
    mgr.broadcast(msg).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[test]
fn test_manager_broadcast_empty_noop() {
    let mgr = RuntimeManager::new();
    // Should not error on empty manager
    let msg = ExtensionMessage::new("broadcaster".to_string(), json!(null));
    mgr.broadcast(msg).unwrap();
}

#[test]
fn test_manager_dispatch_event_to_all() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mgr = RuntimeManager::new();
    for name in ["ext-a", "ext-b"] {
        let c = counter.clone();
        let rt = mgr.create_runtime(name.to_string()).unwrap();
        rt.start().unwrap();
        rt.on_message("ev", move |msg, _sender| {
            if msg.message_type == MessageType::Event {
                c.fetch_add(1, Ordering::SeqCst);
            }
            None
        });
    }

    mgr.dispatch_event("startup", json!({"reason": "test"}))
        .unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_manager_default() {
    let mgr = RuntimeManager::default();
    assert!(mgr.extension_ids().is_empty());
}

#[test]
fn test_create_runtime_manager_fn() {
    let mgr = create_runtime_manager();
    assert!(mgr.extension_ids().is_empty());
}
