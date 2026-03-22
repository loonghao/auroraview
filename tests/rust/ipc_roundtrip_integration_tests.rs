//! Integration tests for IPC roundtrip: Python handler → Rust dispatch → emit back
//!
//! These tests verify the complete IPC communication chain without requiring
//! a WebView window. The roundtrip path tested:
//!
//! 1. Register a Python callback for an event (simulating JS → Python routing)
//! 2. Send an IPC message through the handler (simulating JS `send_event`)
//! 3. Verify the Python callback is invoked with correct data
//! 4. Verify the handler can emit events back to the message queue
//!    (simulating Python `emit()` → JS `on()`)

use _core::ipc::handler::IpcHandler;
use _core::ipc::handler::{IpcMessage, MessageQueue, WebViewMessage};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use rstest::*;
use std::sync::Arc;

/// Creates a Python echo handler that:
/// 1. Appends received data to `seen` list
/// 2. Returns the data unchanged (for inspection)
///
/// Returns (callback, seen_list) as PyO3 handles
fn py_echo_handler() -> (Py<PyAny>, Py<PyAny>) {
    Python::attach(|py| {
        let seen = PyList::new(py, Vec::<i32>::new()).unwrap();
        let m = PyModule::from_code(
            py,
            c"def make_handler(seen):\n    def handler(data):\n        seen.append(data)\n    return handler\n",
            c"echo.py",
            c"echo",
        )
        .unwrap();
        let make_handler = m.getattr("make_handler").unwrap();
        let handler = make_handler
            .call1((seen.clone(),))
            .unwrap()
            .clone()
            .unbind();
        let seen_obj: Py<PyAny> = seen.clone().unbind().into();
        Ok::<_, PyErr>((handler, seen_obj))
    })
    .unwrap()
}

/// Creates a Python ping handler that:
/// 1. Records the received message
/// 2. Returns a fixed "pong" response
fn py_ping_handler() -> (Py<PyAny>, Py<PyAny>) {
    Python::attach(|py| {
        let results = PyList::new(py, Vec::<i32>::new()).unwrap();
        let m = PyModule::from_code(
            py,
            c"def make_handler(results):\n    def handler(data):\n        results.append({'received': data, 'response': 'pong'})\n    return handler\n",
            c"ping.py",
            c"ping",
        )
        .unwrap();
        let make_handler = m.getattr("make_handler").unwrap();
        let handler = make_handler
            .call1((results.clone(),))
            .unwrap()
            .clone()
            .unbind();
        let results_obj: Py<PyAny> = results.clone().unbind().into();
        Ok::<_, PyErr>((handler, results_obj))
    })
    .unwrap()
}

#[fixture]
fn handler_with_queue() -> (IpcHandler, Arc<MessageQueue>) {
    let mut handler = IpcHandler::new();
    let queue = Arc::new(MessageQueue::new());
    handler.set_message_queue(queue.clone());
    (handler, queue)
}

/// Test 1: Complete echo roundtrip
///
/// JS sends event → Python handler receives data → data matches original
#[rstest]
fn test_echo_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, _queue) = handler_with_queue;
    let (cb, seen_obj) = py_echo_handler();

    // Register Python echo handler
    handler.register_python_callback("echo", cb);

    // Simulate JS sending an event
    let msg = IpcMessage {
        event: "echo".to_string(),
        data: serde_json::json!({"message": "hello from JS", "count": 42}),
        id: None,
    };
    let result = handler.handle_message(msg);
    assert!(result.is_ok(), "Echo handler should succeed");

    // Verify Python received the correct data
    Python::attach(|py| {
        let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
        assert_eq!(seen.len(), 1, "Handler should have been called once");

        let received = seen.get_item(0).unwrap();
        let dict = received.cast::<PyDict>().unwrap();

        let message = dict
            .get_item("message")
            .unwrap()
            .unwrap()
            .extract::<String>()
            .unwrap();
        assert_eq!(message, "hello from JS");

        let count = dict
            .get_item("count")
            .unwrap()
            .unwrap()
            .extract::<i64>()
            .unwrap();
        assert_eq!(count, 42);
    });
}

/// Test 2: Ping-pong roundtrip
///
/// JS sends ping → Python handler processes → Python emits pong via queue
#[rstest]
fn test_ping_pong_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, queue) = handler_with_queue;
    let (cb, results_obj) = py_ping_handler();

    // Register Python ping handler
    handler.register_python_callback("ping", cb);

    // Simulate JS sending a ping
    let msg = IpcMessage {
        event: "ping".to_string(),
        data: serde_json::json!({"timestamp": 1234567890}),
        id: None,
    };
    let result = handler.handle_message(msg);
    assert!(result.is_ok(), "Ping handler should succeed");

    // Verify Python handler recorded the request
    Python::attach(|py| {
        let results = results_obj.bind(py).cast::<PyList>().unwrap();
        assert_eq!(
            results.len(),
            1,
            "Ping handler should have been called once"
        );
    });

    // Now simulate Python emitting a "pong" event back to JS
    let emit_result = handler.emit(
        "pong",
        serde_json::json!({"response": "pong", "timestamp": 1234567891}),
    );
    assert!(emit_result.is_ok(), "Emit to queue should succeed");

    // Verify the pong event landed in the message queue
    assert_eq!(queue.len(), 1, "Queue should have the pong message");
    if let Some(queued_msg) = queue.pop() {
        match queued_msg {
            WebViewMessage::EmitEvent {
                event_name, data, ..
            } => {
                assert_eq!(event_name, "pong");
                assert_eq!(data["response"], "pong");
                assert_eq!(data["timestamp"], 1234567891);
            }
            other => panic!("Expected EmitEvent, got {:?}", other),
        }
    }
}

/// Test 3: Multiple events roundtrip
///
/// Register multiple handlers, send messages to each, verify isolation
#[rstest]
fn test_multiple_events_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, queue) = handler_with_queue;
    let (echo_cb, echo_seen) = py_echo_handler();
    let (ping_cb, ping_results) = py_ping_handler();

    handler.register_python_callback("echo", echo_cb);
    handler.register_python_callback("ping", ping_cb);

    // Send echo event
    handler
        .handle_message(IpcMessage {
            event: "echo".to_string(),
            data: serde_json::json!({"type": "echo_data"}),
            id: None,
        })
        .unwrap();

    // Send ping event
    handler
        .handle_message(IpcMessage {
            event: "ping".to_string(),
            data: serde_json::json!({"type": "ping_data"}),
            id: None,
        })
        .unwrap();

    // Verify isolation: echo handler only got echo data
    Python::attach(|py| {
        let echo_list = echo_seen.bind(py).cast::<PyList>().unwrap();
        assert_eq!(echo_list.len(), 1);
        let echo_data = echo_list
            .get_item(0)
            .unwrap()
            .cast::<PyDict>()
            .unwrap()
            .get_item("type")
            .unwrap()
            .unwrap()
            .extract::<String>()
            .unwrap();
        assert_eq!(echo_data, "echo_data");

        let ping_list = ping_results.bind(py).cast::<PyList>().unwrap();
        assert_eq!(ping_list.len(), 1);
    });

    // Emit responses for both
    handler
        .emit("echo_response", serde_json::json!({"echoed": true}))
        .unwrap();
    handler
        .emit("pong", serde_json::json!({"ponged": true}))
        .unwrap();
    assert_eq!(queue.len(), 2, "Queue should have 2 response messages");
}

/// Test 4: Complex data roundtrip (Unicode, nested structures, arrays)
#[rstest]
fn test_complex_data_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, _queue) = handler_with_queue;
    let (cb, seen_obj) = py_echo_handler();

    handler.register_python_callback("complex", cb);

    let complex_data = serde_json::json!({
        "unicode": "Hello 你好 こんにちは 🌍",
        "nested": {
            "level1": {
                "level2": {
                    "value": [1, 2, 3]
                }
            }
        },
        "array": [
            {"name": "item1", "tags": ["a", "b"]},
            {"name": "item2", "tags": ["c"]}
        ],
        "numbers": {
            "int": 42,
            "float": 3.15,
            "negative": -100,
            "large": 999999999999i64
        },
        "booleans": {
            "yes": true,
            "no": false
        },
        "null_value": null
    });

    handler
        .handle_message(IpcMessage {
            event: "complex".to_string(),
            data: complex_data.clone(),
            id: None,
        })
        .unwrap();

    // Verify Python received all the complex data correctly
    Python::attach(|py| {
        let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
        assert_eq!(seen.len(), 1);

        let received = seen.get_item(0).unwrap();
        let dict = received.cast::<PyDict>().unwrap();

        // Verify Unicode
        let unicode = dict
            .get_item("unicode")
            .unwrap()
            .unwrap()
            .extract::<String>()
            .unwrap();
        assert_eq!(unicode, "Hello 你好 こんにちは 🌍");

        // Verify nested access
        let nested_tmp = dict.get_item("nested").unwrap().unwrap();
        let nested = nested_tmp.cast::<PyDict>().unwrap();
        let level1_tmp = nested.get_item("level1").unwrap().unwrap();
        let level1 = level1_tmp.cast::<PyDict>().unwrap();
        let level2_tmp = level1.get_item("level2").unwrap().unwrap();
        let level2 = level2_tmp.cast::<PyDict>().unwrap();
        let value_tmp = level2.get_item("value").unwrap().unwrap();
        let value = value_tmp.cast::<PyList>().unwrap();
        assert_eq!(value.len(), 3);

        // Verify null
        let null_val = dict.get_item("null_value").unwrap().unwrap();
        assert!(null_val.is_none());
    });
}

/// Test 5: Error handling - unregistered event
#[rstest]
fn test_unregistered_event_error(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, _queue) = handler_with_queue;

    let result = handler.handle_message(IpcMessage {
        event: "nonexistent".to_string(),
        data: serde_json::json!({}),
        id: None,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No handler registered"));
}

/// Test 6: Event deregistration roundtrip
#[rstest]
fn test_off_and_clear_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, _queue) = handler_with_queue;
    let (cb1, _) = py_echo_handler();
    let (cb2, _) = py_echo_handler();

    handler.register_python_callback("evt1", cb1);
    handler.register_python_callback("evt2", cb2);
    assert_eq!(handler.registered_event_count(), 2);

    // Remove one event
    handler.off("evt1");
    assert_eq!(handler.registered_event_count(), 1);

    // evt1 should fail now
    let result = handler.handle_message(IpcMessage {
        event: "evt1".to_string(),
        data: serde_json::json!({}),
        id: None,
    });
    assert!(result.is_err());

    // evt2 should still work
    let result = handler.handle_message(IpcMessage {
        event: "evt2".to_string(),
        data: serde_json::json!({}),
        id: None,
    });
    assert!(result.is_ok());

    // Clear all
    handler.clear();
    assert_eq!(handler.registered_event_count(), 0);
}

/// Test 7: Full emit → queue → pop roundtrip with multiple messages
#[rstest]
fn test_emit_queue_pop_roundtrip(handler_with_queue: (IpcHandler, Arc<MessageQueue>)) {
    let (handler, queue) = handler_with_queue;

    // Emit a sequence of events
    let events = vec![
        ("progress", serde_json::json!({"percent": 25})),
        ("progress", serde_json::json!({"percent": 50})),
        ("progress", serde_json::json!({"percent": 75})),
        ("complete", serde_json::json!({"result": "success"})),
    ];

    for (event, data) in &events {
        handler.emit(event, data.clone()).unwrap();
    }

    assert_eq!(queue.len(), 4, "All 4 events should be queued");

    // Pop and verify in order
    for (expected_event, expected_data) in &events {
        let msg = queue.pop().unwrap();
        match msg {
            WebViewMessage::EmitEvent {
                event_name, data, ..
            } => {
                assert_eq!(event_name, *expected_event);
                assert_eq!(data, *expected_data);
            }
            other => panic!("Expected EmitEvent, got {:?}", other),
        }
    }

    assert_eq!(queue.len(), 0, "Queue should be empty after popping all");
}
