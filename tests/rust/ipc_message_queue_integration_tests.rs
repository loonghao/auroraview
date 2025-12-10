//! Integration tests for IPC MessageQueue
//!
//! These tests verify the complete message queue functionality including
//! push/pop operations, backpressure handling, and retry logic.

use _core::ipc::message_queue::{MessageQueue, MessageQueueConfig};
use _core::ipc::WebViewMessage;
use rstest::*;

#[fixture]
fn small_queue_no_retry() -> MessageQueue {
    let cfg = MessageQueueConfig {
        capacity: 1,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    MessageQueue::with_config(cfg)
}

#[fixture]
fn small_queue_with_retry() -> MessageQueue {
    let cfg = MessageQueueConfig {
        capacity: 1,
        block_on_full: false,
        max_retries: 1,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    MessageQueue::with_config(cfg)
}

#[rstest]
fn test_push_pop_and_metrics(small_queue_no_retry: MessageQueue) {
    let q = small_queue_no_retry;
    q.push(WebViewMessage::EvalJs("1+1".into()));

    let mut processed = 0;
    processed += q.process_all(|m| match m {
        WebViewMessage::EvalJs(s) => assert_eq!(s, "1+1"),
        _ => unreachable!(),
    });

    assert_eq!(processed, 1, "Should process exactly one message");

    let snap = q.get_metrics_snapshot();
    assert_eq!(snap.messages_sent, 1, "Should have sent 1 message");
    assert_eq!(snap.messages_received, 1, "Should have received 1 message");
    assert!(
        snap.peak_queue_length >= 1,
        "Peak queue length should be at least 1"
    );
}

#[rstest]
fn test_backpressure_drop_and_retry(small_queue_with_retry: MessageQueue) {
    let q = small_queue_with_retry;

    // Fill the queue
    q.push(WebViewMessage::EvalJs("a".into()));

    // This immediate push should drop due to full queue
    q.push(WebViewMessage::EvalJs("b".into()));

    let snap = q.get_metrics_snapshot();
    assert!(
        snap.messages_dropped >= 1,
        "Should have dropped at least 1 message due to full queue"
    );

    // push_with_retry should fail after exhausting retries
    let err = q
        .push_with_retry(WebViewMessage::EvalJs("c".into()))
        .unwrap_err();

    assert!(
        err.contains("queue full") || err.contains("Channel disconnected"),
        "Error should indicate queue full or disconnected, got: {}",
        err
    );

    // Drain queue to keep later tests stable
    let _ = q.process_all(|_| {});
}

#[rstest]
fn test_message_queue_creation() {
    let cfg = MessageQueueConfig {
        capacity: 10,
        block_on_full: false,
        max_retries: 3,
        retry_delay_ms: 10,
        batch_interval_ms: 16,
    };
    let q = MessageQueue::with_config(cfg);

    let snap = q.get_metrics_snapshot();
    assert_eq!(
        snap.messages_sent, 0,
        "New queue should have 0 messages sent"
    );
    assert_eq!(
        snap.messages_received, 0,
        "New queue should have 0 messages received"
    );
}

#[rstest]
fn test_multiple_message_types(small_queue_no_retry: MessageQueue) {
    let q = small_queue_no_retry;

    q.push(WebViewMessage::EvalJs("test".into()));

    let mut count = 0;
    q.process_all(|msg| match msg {
        WebViewMessage::EvalJs(s) => {
            assert_eq!(s, "test");
            count += 1;
        }
        _ => unreachable!("Should only have EvalJs messages"),
    });

    assert_eq!(count, 1, "Should process exactly one message");
}

/// Test LoadUrl message type
#[rstest]
fn test_load_url_message() {
    let cfg = MessageQueueConfig {
        capacity: 10,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    let q = MessageQueue::with_config(cfg);

    // Test various URL types
    let urls = vec![
        "https://example.com",
        "http://localhost:8080",
        "file:///path/to/file.html",
    ];

    for url in &urls {
        q.push(WebViewMessage::LoadUrl(url.to_string()));
    }

    let mut received_urls = Vec::new();
    q.process_all(|msg| {
        if let WebViewMessage::LoadUrl(url) = msg {
            received_urls.push(url);
        }
    });

    assert_eq!(received_urls.len(), urls.len(), "Should receive all URLs");
    for (i, url) in urls.iter().enumerate() {
        assert_eq!(received_urls[i], *url, "URL should match at index {}", i);
    }
}

/// Test that event loop proxy warning is not triggered when proxy is set
#[rstest]
fn test_event_loop_proxy_not_set_initially() {
    let cfg = MessageQueueConfig {
        capacity: 10,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    let q = MessageQueue::with_config(cfg);

    // Push a message without setting event loop proxy
    // This should not panic, just log a debug message
    q.push(WebViewMessage::EvalJs("test".into()));

    // Message should still be in queue
    let mut count = 0;
    q.process_all(|_| count += 1);
    assert_eq!(count, 1, "Message should be queued even without proxy");
}
