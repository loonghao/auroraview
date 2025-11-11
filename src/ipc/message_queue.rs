//! Thread-safe message queue for cross-thread WebView communication
//!
//! This module provides a message queue system that allows safe communication
//! between the DCC main thread (e.g., Maya) and the WebView background thread.
//!
//! ## Problem
//! WryWebView is not Send/Sync, so we cannot call evaluate_script() from
//! a different thread than the one that created the WebView.
//!
//! ## Solution
//! Use a message queue with crossbeam-channel for high-performance communication:
//! 1. Main thread calls emit() -> pushes message to queue
//! 2. Background thread's event loop polls queue -> executes JavaScript
//!
//! This ensures all WebView operations happen on the correct thread.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::{Arc, Mutex};

// Import UserEvent from webview event_loop module
use crate::webview::event_loop::UserEvent;
use tao::event_loop::EventLoopProxy;

// Import DeadLetterQueue and Metrics
use super::dead_letter_queue::{DeadLetterQueue, FailureReason};
use super::metrics::IpcMetrics;

/// Message types that can be sent to the WebView
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WebViewMessage {
    /// Execute JavaScript code
    EvalJs(String),

    /// Emit an event to JavaScript
    EmitEvent {
        event_name: String,
        data: serde_json::Value,
    },

    /// Load a URL
    LoadUrl(String),

    /// Load HTML content
    LoadHtml(String),
}

/// Configuration for message queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MessageQueueConfig {
    /// Maximum number of messages in the queue (backpressure)
    pub capacity: usize,

    /// Whether to block when queue is full (true) or drop messages (false)
    pub block_on_full: bool,

    /// Maximum number of retry attempts for failed sends
    pub max_retries: u32,

    /// Delay between retry attempts (milliseconds)
    pub retry_delay_ms: u64,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            block_on_full: false,
            max_retries: 3,
            retry_delay_ms: 10,
        }
    }
}

/// Thread-safe message queue for WebView operations
///
/// Uses crossbeam-channel for high-performance lock-free communication.
/// Provides backpressure control to prevent unbounded memory growth.
#[derive(Clone)]
#[allow(dead_code)]
pub struct MessageQueue {
    /// Sender for pushing messages (lock-free)
    tx: Sender<WebViewMessage>,

    /// Receiver for popping messages (lock-free)
    rx: Receiver<WebViewMessage>,

    /// Event loop proxy for immediate wake-up
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,

    /// Dead letter queue for failed messages
    dlq: DeadLetterQueue,

    /// Performance metrics
    metrics: IpcMetrics,

    /// Configuration
    config: MessageQueueConfig,
}

impl MessageQueue {
    /// Create a new message queue with default configuration
    pub fn new() -> Self {
        Self::with_config(MessageQueueConfig::default())
    }

    /// Create a new message queue with custom configuration
    pub fn with_config(config: MessageQueueConfig) -> Self {
        let (tx, rx) = bounded(config.capacity);
        Self {
            tx,
            rx,
            event_loop_proxy: Arc::new(Mutex::new(None)),
            dlq: DeadLetterQueue::new(),
            metrics: IpcMetrics::new(),
            config,
        }
    }

    /// Set the event loop proxy for immediate wake-up
    pub fn set_event_loop_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        if let Ok(mut proxy_guard) = self.event_loop_proxy.lock() {
            *proxy_guard = Some(proxy);
            tracing::info!("Event loop proxy set in message queue");
        }
    }

    /// Push a message to the queue (thread-safe)
    ///
    /// This can be called from any thread, including the DCC main thread.
    /// After pushing the message, it will wake up the event loop immediately.
    ///
    /// # Backpressure
    /// - If `block_on_full` is true, this will block until space is available
    /// - If `block_on_full` is false, this will drop the message and log an error
    pub fn push(&self, message: WebViewMessage) {
        tracing::debug!(
            "[PUSH] [MessageQueue::push] Pushing message: {:?}",
            match &message {
                WebViewMessage::EvalJs(_) => "EvalJs",
                WebViewMessage::EmitEvent { event_name, .. } => event_name,
                WebViewMessage::LoadUrl(_) => "LoadUrl",
                WebViewMessage::LoadHtml(_) => "LoadHtml",
            }
        );

        // Try to send the message
        match self.tx.try_send(message.clone()) {
            Ok(_) => {
                self.metrics.record_send();
                let queue_len = self.len();
                self.metrics.update_peak_queue_length(queue_len);

                tracing::debug!(
                    "[PUSH] [MessageQueue::push] Message sent successfully (queue length: {})",
                    queue_len
                );

                // Wake up the event loop immediately
                self.wake_event_loop();
            }
            Err(TrySendError::Full(_)) => {
                if self.config.block_on_full {
                    // Block until space is available
                    tracing::warn!("[WARNING] [MessageQueue::push] Queue full, blocking...");
                    if let Err(e) = self.tx.send(message) {
                        self.metrics.record_failure();
                        tracing::error!(
                            "[ERROR] [MessageQueue::push] Failed to send message: {:?}",
                            e
                        );
                    } else {
                        self.metrics.record_send();
                        self.wake_event_loop();
                    }
                } else {
                    // Drop the message
                    self.metrics.record_drop();
                    tracing::error!("[ERROR] [MessageQueue::push] Queue full, dropping message!");
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                self.metrics.record_failure();
                tracing::error!("[ERROR] [MessageQueue::push] Channel disconnected!");
            }
        }
    }

    /// Push a message with retry logic (thread-safe)
    ///
    /// This method will retry sending the message if the queue is full,
    /// using the configured retry count and delay. Failed messages are
    /// automatically sent to the dead letter queue.
    ///
    /// # Returns
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(String)` if all retry attempts failed
    #[allow(dead_code)]
    pub fn push_with_retry(&self, message: WebViewMessage) -> Result<(), String> {
        let max_retries = self.config.max_retries;
        let retry_delay = std::time::Duration::from_millis(self.config.retry_delay_ms);
        let start_time = std::time::Instant::now();

        for attempt in 0..=max_retries {
            match self.tx.try_send(message.clone()) {
                Ok(_) => {
                    self.metrics.record_send();
                    let queue_len = self.len();
                    self.metrics.update_peak_queue_length(queue_len);

                    // Record latency
                    let latency_us = start_time.elapsed().as_micros() as u64;
                    self.metrics.record_latency(latency_us);

                    if attempt > 0 {
                        tracing::info!(
                            "[RETRY] Message sent successfully after {} attempts (latency: {}μs)",
                            attempt,
                            latency_us
                        );
                    }
                    self.wake_event_loop();
                    return Ok(());
                }
                Err(TrySendError::Full(_)) => {
                    self.metrics.record_retry();

                    if attempt < max_retries {
                        tracing::warn!(
                            "[RETRY] Queue full, attempt {}/{}, retrying in {:?}...",
                            attempt + 1,
                            max_retries,
                            retry_delay
                        );
                        std::thread::sleep(retry_delay);
                    } else {
                        // Send to DLQ
                        self.dlq.push(message, FailureReason::QueueFull, attempt);
                        self.metrics.record_failure();

                        let error_msg = format!(
                            "Failed to send message after {} attempts: queue full",
                            max_retries + 1
                        );
                        tracing::error!("[ERROR] {}", error_msg);
                        return Err(error_msg);
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    // Send to DLQ
                    self.dlq
                        .push(message, FailureReason::ChannelDisconnected, attempt);
                    self.metrics.record_failure();

                    let error_msg = "Channel disconnected".to_string();
                    tracing::error!("[ERROR] {}", error_msg);
                    return Err(error_msg);
                }
            }
        }

        Err("Unexpected retry loop exit".to_string())
    }

    /// Wake up the event loop
    fn wake_event_loop(&self) {
        if let Ok(proxy_guard) = self.event_loop_proxy.lock() {
            if let Some(proxy) = proxy_guard.as_ref() {
                tracing::debug!("[WAKE] [MessageQueue] Sending wake-up event...");
                match proxy.send_event(UserEvent::ProcessMessages) {
                    Ok(_) => {
                        tracing::debug!("[OK] [MessageQueue] Event loop woken up successfully!");
                    }
                    Err(e) => {
                        tracing::error!(
                            "[ERROR] [MessageQueue] Failed to wake up event loop: {:?}",
                            e
                        );
                    }
                }
            } else {
                tracing::debug!(
                    "[WARNING] [MessageQueue] Event loop proxy is None - cannot wake up event loop!"
                );
            }
        }
    }

    /// Pop a message from the queue (thread-safe)
    ///
    /// This should be called from the WebView thread only.
    pub fn pop(&self) -> Option<WebViewMessage> {
        let message = self.rx.try_recv().ok();
        if message.is_some() {
            self.metrics.record_receive();
        }
        message
    }

    /// Check if the queue is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// Get the number of pending messages
    pub fn len(&self) -> usize {
        self.rx.len()
    }

    /// Process all pending messages
    ///
    /// This should be called from the WebView thread's event loop.
    /// Returns the number of messages processed.
    pub fn process_all<F>(&self, mut handler: F) -> usize
    where
        F: FnMut(WebViewMessage),
    {
        let mut count = 0;

        while let Some(message) = self.pop() {
            handler(message);
            count += 1;
        }

        if count > 0 {
            tracing::debug!("Processed {} messages from queue", count);
        }

        count
    }

    /// Get a reference to the dead letter queue
    #[allow(dead_code)]
    pub fn dead_letter_queue(&self) -> &DeadLetterQueue {
        &self.dlq
    }

    /// Get statistics about failed messages
    #[allow(dead_code)]
    pub fn get_dlq_stats(&self) -> super::dead_letter_queue::DeadLetterStats {
        self.dlq.get_stats()
    }

    /// Get a reference to the metrics
    #[allow(dead_code)]
    pub fn metrics(&self) -> &IpcMetrics {
        &self.metrics
    }

    /// Get a snapshot of current metrics
    #[allow(dead_code)]
    pub fn get_metrics_snapshot(&self) -> super::metrics::IpcMetricsSnapshot {
        self.metrics.snapshot()
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_queue(block_on_full: bool, max_retries: u32) -> MessageQueue {
        let cfg = MessageQueueConfig {
            capacity: 1,
            block_on_full,
            max_retries,
            retry_delay_ms: 1,
        };
        MessageQueue::with_config(cfg)
    }

    #[test]
    fn test_push_pop_and_metrics() {
        let q = small_queue(false, 0);
        q.push(WebViewMessage::EvalJs("1+1".into()));
        let mut processed = 0;
        processed += q.process_all(|m| match m {
            WebViewMessage::EvalJs(s) => assert_eq!(s, "1+1"),
            _ => unreachable!(),
        });
        assert_eq!(processed, 1);
        let snap = q.get_metrics_snapshot();
        assert_eq!(snap.messages_sent, 1);
        assert_eq!(snap.messages_received, 1);
        assert!(snap.peak_queue_length >= 1);
    }

    #[test]
    fn test_backpressure_drop_and_retry_to_dlq() {
        let q = small_queue(false, 1);
        // Fill the queue
        q.push(WebViewMessage::EvalJs("a".into()));
        // This immediate push should drop due to full queue
        q.push(WebViewMessage::EvalJs("b".into()));
        let snap = q.get_metrics_snapshot();
        assert!(snap.messages_dropped >= 1);

        // push_with_retry should attempt and then go to DLQ
        let err = q
            .push_with_retry(WebViewMessage::EvalJs("c".into()))
            .unwrap_err();
        assert!(err.contains("queue full") || err.contains("Channel disconnected"));
        let stats = q.get_dlq_stats();
        assert!(stats.total >= 1);
        assert!(stats.queue_full >= 1 || stats.disconnected >= 1);

        // Drain queue to keep later tests stable
        let _ = q.process_all(|_| {});
    }
}
