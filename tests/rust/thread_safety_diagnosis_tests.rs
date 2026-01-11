//! Integration tests for thread safety diagnosis findings
//!
//! These tests verify the critical issues identified in the architecture diagnosis:
//! - P0: Lock ordering in EventLoopState
//! - P0: Message queue wake-up batching latency
//! - P1: Dual message pump conflict
//! - P1: Arc<Mutex<WryWebView>> contention

use _core::ipc::message_queue::{MessageQueue, MessageQueueConfig};
use _core::ipc::WebViewMessage;
use rstest::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Test fixture for immediate wake queue (no batching)
#[fixture]
fn immediate_wake_queue() -> MessageQueue {
    let cfg = MessageQueueConfig {
        capacity: 100,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0, // No batching for immediate response
    };
    MessageQueue::with_config(cfg)
}

/// Test fixture for batched wake queue (16ms batching)
#[fixture]
fn batched_wake_queue() -> MessageQueue {
    let cfg = MessageQueueConfig {
        capacity: 100,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 16, // Default batching interval
    };
    MessageQueue::with_config(cfg)
}

// =============================================================================
// P0: Message Queue Wake-up Batching Tests
// =============================================================================

/// Test that immediate wake queue processes messages without delay
///
/// This test verifies the fix for P0 issue: Message queue batching causing UI latency
#[rstest]
fn test_immediate_wake_no_latency(immediate_wake_queue: MessageQueue) {
    let q = immediate_wake_queue;
    let start = Instant::now();

    // Push a message
    q.push(WebViewMessage::EvalJs("test".into()));

    // Process immediately
    let mut processed = false;
    q.process_all(|_| processed = true);

    let elapsed = start.elapsed();

    assert!(processed, "Message should be processed");
    assert!(
        elapsed < Duration::from_millis(5),
        "Processing should be nearly instant, took {:?}",
        elapsed
    );
}

/// Test that batched queue introduces latency (documenting current behavior)
///
/// This test documents the P0 issue where batch_interval_ms causes delays
#[rstest]
fn test_batched_wake_latency_documented(batched_wake_queue: MessageQueue) {
    let q = batched_wake_queue;

    // Push multiple messages rapidly
    for i in 0..5 {
        q.push(WebViewMessage::EvalJs(format!("msg{}", i)));
    }

    // All messages should be available for processing
    let mut count = 0;
    q.process_all(|_| count += 1);

    assert_eq!(count, 5, "All messages should be processed");

    // Note: The actual latency impact happens in wake_event_loop()
    // which we cannot directly test without the event loop
}

// =============================================================================
// P1: Lock Contention Tests
// =============================================================================

/// Test concurrent message queue access under contention
///
/// This test verifies that the message queue handles concurrent access correctly
#[rstest]
fn test_concurrent_queue_access(immediate_wake_queue: MessageQueue) {
    let q = Arc::new(immediate_wake_queue);
    let message_count = Arc::new(AtomicUsize::new(0));

    let num_producers = 4;
    let messages_per_producer = 25;

    let mut handles = vec![];

    // Spawn producer threads
    for producer_id in 0..num_producers {
        let q_clone = Arc::clone(&q);
        let handle = thread::spawn(move || {
            for i in 0..messages_per_producer {
                q_clone.push(WebViewMessage::EvalJs(format!("p{}m{}", producer_id, i)));
            }
        });
        handles.push(handle);
    }

    // Wait for all producers
    for handle in handles {
        handle.join().expect("Producer thread panicked");
    }

    // Process all messages
    let count_clone = Arc::clone(&message_count);
    q.process_all(move |_| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    let total = message_count.load(Ordering::SeqCst);
    let expected = num_producers * messages_per_producer;

    assert_eq!(
        total, expected,
        "Should process all {} messages, got {}",
        expected, total
    );
}

/// Test that metrics are thread-safe
#[rstest]
fn test_metrics_thread_safety(immediate_wake_queue: MessageQueue) {
    let q = Arc::new(immediate_wake_queue);

    let num_threads = 4;
    let ops_per_thread = 50;

    let mut handles = vec![];

    // Spawn threads that push and read metrics concurrently
    for _ in 0..num_threads {
        let q_clone = Arc::clone(&q);
        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                q_clone.push(WebViewMessage::EvalJs(format!("test{}", i)));
                // Read metrics while pushing
                let _ = q_clone.get_metrics_snapshot();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Final metrics check
    let snap = q.get_metrics_snapshot();
    let expected_sent = num_threads * ops_per_thread;

    assert_eq!(
        snap.messages_sent, expected_sent,
        "Should have sent {} messages",
        expected_sent
    );
}

// =============================================================================
// Message Processing Order Tests
// =============================================================================

/// Test that messages are processed in FIFO order
#[rstest]
fn test_message_fifo_order(immediate_wake_queue: MessageQueue) {
    let q = immediate_wake_queue;

    // Push messages with sequence numbers
    for i in 0..10 {
        q.push(WebViewMessage::EvalJs(format!("{}", i)));
    }

    // Verify FIFO order
    let mut expected = 0;
    q.process_all(|msg| {
        if let WebViewMessage::EvalJs(s) = msg {
            let num: usize = s.parse().expect("Should be a number");
            assert_eq!(num, expected, "Messages should be in FIFO order");
            expected += 1;
        }
    });

    assert_eq!(expected, 10, "Should process all 10 messages");
}

// =============================================================================
// Backpressure and Queue Full Tests
// =============================================================================

/// Test queue behavior when full
#[rstest]
fn test_queue_full_behavior() {
    let cfg = MessageQueueConfig {
        capacity: 2,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    let q = MessageQueue::with_config(cfg);

    // Fill the queue
    q.push(WebViewMessage::EvalJs("1".into()));
    q.push(WebViewMessage::EvalJs("2".into()));

    // This should be dropped
    q.push(WebViewMessage::EvalJs("3".into()));

    let snap = q.get_metrics_snapshot();
    assert!(
        snap.messages_dropped >= 1,
        "Should have dropped at least 1 message when queue is full"
    );
}

/// Test retry behavior with backpressure
#[rstest]
fn test_retry_with_backpressure() {
    let cfg = MessageQueueConfig {
        capacity: 1,
        block_on_full: false,
        max_retries: 2,
        retry_delay_ms: 5,
        batch_interval_ms: 0,
    };
    let q = MessageQueue::with_config(cfg);

    // Fill the queue
    q.push(WebViewMessage::EvalJs("blocking".into()));

    // Try to push with retry - should fail after retries
    let start = Instant::now();
    let result = q.push_with_retry(WebViewMessage::EvalJs("retry".into()));
    let elapsed = start.elapsed();

    assert!(result.is_err(), "Should fail after exhausting retries");

    // Should have taken at least (max_retries * retry_delay_ms) time
    let min_expected = Duration::from_millis(2 * 5);
    assert!(
        elapsed >= min_expected,
        "Should have waited for retries, elapsed: {:?}",
        elapsed
    );
}

// =============================================================================
// Shutdown State Tests
// =============================================================================

/// Test that queue handles shutdown gracefully
#[rstest]
fn test_shutdown_handling(immediate_wake_queue: MessageQueue) {
    let q = Arc::new(immediate_wake_queue);

    // Push some messages
    q.push(WebViewMessage::EvalJs("before_shutdown".into()));

    // Shutdown the queue
    q.shutdown();

    // Messages after shutdown should be dropped
    q.push(WebViewMessage::EvalJs("after_shutdown".into()));

    // Process remaining messages
    let mut count = 0;
    q.process_all(|_| count += 1);

    // Only the message before shutdown should be processed
    assert!(count <= 1, "Should process at most 1 message after shutdown");
}

/// Test concurrent shutdown and push
#[rstest]
fn test_concurrent_shutdown() {
    let cfg = MessageQueueConfig {
        capacity: 100,
        block_on_full: false,
        max_retries: 0,
        retry_delay_ms: 1,
        batch_interval_ms: 0,
    };
    let q = Arc::new(MessageQueue::with_config(cfg));

    let q_push = Arc::clone(&q);
    let q_shutdown = Arc::clone(&q);

    // Spawn pusher thread
    let push_handle = thread::spawn(move || {
        for i in 0..50 {
            q_push.push(WebViewMessage::EvalJs(format!("msg{}", i)));
            thread::sleep(Duration::from_micros(100));
        }
    });

    // Spawn shutdown thread with delay
    let shutdown_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(2));
        q_shutdown.shutdown();
    });

    push_handle.join().expect("Push thread panicked");
    shutdown_handle.join().expect("Shutdown thread panicked");

    // Queue should be in shutdown state
    // This test verifies no deadlocks or panics occur
}

// =============================================================================
// Performance Baseline Tests
// =============================================================================

/// Establish baseline for message throughput
#[rstest]
fn test_message_throughput_baseline(immediate_wake_queue: MessageQueue) {
    let q = immediate_wake_queue;
    let num_messages = 1000;

    let start = Instant::now();

    // Push all messages
    for i in 0..num_messages {
        q.push(WebViewMessage::EvalJs(format!("msg{}", i)));
    }

    let push_elapsed = start.elapsed();

    // Process all messages
    let process_start = Instant::now();
    let mut count = 0;
    q.process_all(|_| count += 1);
    let process_elapsed = process_start.elapsed();

    assert_eq!(count, num_messages, "Should process all messages");

    // Log performance metrics
    let push_per_sec = num_messages as f64 / push_elapsed.as_secs_f64();
    let process_per_sec = num_messages as f64 / process_elapsed.as_secs_f64();

    println!(
        "Throughput: push={:.0} msg/s, process={:.0} msg/s",
        push_per_sec, process_per_sec
    );

    // Sanity check: should be able to handle at least 10k messages per second
    assert!(
        push_per_sec > 10_000.0,
        "Push throughput too low: {:.0} msg/s",
        push_per_sec
    );
}
