//! Integration tests for IPC Batch processing
//!
//! These tests verify the complete batch processing functionality including
//! message batching, flush conditions, and Python callback integration.

use _core::ipc::batch::{BatchedCallback, BatchedMessage, MessageBatch};
use pyo3::prelude::*;
use pyo3::types::PyList;
use rstest::*;
use std::thread;
use std::time::Duration;

#[rstest]
fn test_message_batch_flush_conditions() {
    let mut batch = MessageBatch::new();
    assert!(
        !batch.should_flush(2, 10_000),
        "Empty batch should not flush"
    );

    batch.add(BatchedMessage::new(
        "e".to_string(),
        serde_json::json!({"a":1}),
    ));
    assert!(
        !batch.should_flush(2, 10_000),
        "Single message should not flush with threshold 2"
    );

    batch.add(BatchedMessage::high_priority(
        "e".to_string(),
        serde_json::json!({"b":2}),
    ));
    assert!(
        batch.should_flush(2, 10_000),
        "High priority message should trigger flush"
    );

    // Age-based flush test
    let mut batch2 = MessageBatch::new();
    batch2.add(BatchedMessage::new("e".to_string(), serde_json::json!({})));
    thread::sleep(Duration::from_millis(5));
    assert!(
        batch2.should_flush(10, 1),
        "Old batch should flush based on age"
    );
}

#[rstest]
fn test_batched_callback_single_and_batch() {
    // Prepare a Python callback that collects inputs into `seen` list
    let (make_cb_obj, seen_obj) = Python::attach(|py| {
        let seen = PyList::new(py, Vec::<i32>::new()).unwrap();
        let m = pyo3::types::PyModule::from_code(
            py,
            c"def make_cb(seen):\n    def cb(x):\n        seen.append(x)\n    return cb\n",
            c"m.py",
            c"m",
        )
        .unwrap();
        (
            m.getattr("make_cb").unwrap().unbind(),
            seen.clone().unbind(),
        )
    });

    // 1) call_single
    let cb = Python::attach(|py| {
        let f = make_cb_obj.bind(py);
        let seen = seen_obj.bind(py);
        BatchedCallback::new(f.call1((seen,)).unwrap().clone().unbind(), true)
    });
    let msg = BatchedMessage::new("e".to_string(), serde_json::json!({"k": 1}));
    cb.call_single(&msg).expect("call_single should succeed");

    let single_len = Python::attach(|py| {
        let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
        seen.len()
    });
    assert_eq!(single_len, 1, "Should have 1 item after call_single");

    // 2) call_batch (batching enabled -> one list element appended)
    let cb2 = Python::attach(|py| {
        let f = make_cb_obj.bind(py);
        let seen = seen_obj.bind(py);
        BatchedCallback::new(f.call1((seen,)).unwrap().clone().unbind(), true)
    });
    let mut batch = MessageBatch::new();
    batch.add(BatchedMessage::new(
        "e".to_string(),
        serde_json::json!({"x": 1}),
    ));
    batch.add(BatchedMessage::high_priority(
        "e".to_string(),
        serde_json::json!({"y": 2}),
    ));
    cb2.call_batch(&batch).expect("call_batch should succeed");

    Python::attach(|py| {
        let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
        // After call_single (1) + call_batch (append one list) => total 2 entries
        assert_eq!(
            seen.len(),
            2,
            "Should have 2 items after call_single + call_batch"
        );
        let list_obj = seen.get_item(1).unwrap();
        let list_ref = list_obj.cast::<PyList>().unwrap();
        assert_eq!(list_ref.len(), 2, "Batch should contain 2 messages");
    });

    // 3) call_batch with batching disabled -> two individual appends
    let cb3 = Python::attach(|py| {
        let f = make_cb_obj.bind(py);
        let seen = seen_obj.bind(py);
        BatchedCallback::new(f.call1((seen,)).unwrap().clone().unbind(), false)
    });
    let mut batch2 = MessageBatch::new();
    batch2.add(BatchedMessage::new(
        "e".to_string(),
        serde_json::json!({"m": 1}),
    ));
    batch2.add(BatchedMessage::new(
        "e".to_string(),
        serde_json::json!({"n": 2}),
    ));
    cb3.call_batch(&batch2)
        .expect("fallback-to-single should succeed");

    Python::attach(|py| {
        let seen = seen_obj.bind(py).cast::<PyList>().unwrap();
        // Prior 2 entries + 2 more -> 4 entries total
        assert_eq!(seen.len(), 4, "Should have 4 items after all operations");
    });
}
