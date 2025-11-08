//! Dead Letter Queue (DLQ) for failed IPC messages
//!
//! This module provides a mechanism to capture and store messages that failed
//! to be delivered, allowing for debugging, monitoring, and potential recovery.

#![allow(dead_code)]

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

use super::message_queue::WebViewMessage;

/// Reason why a message was sent to the DLQ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureReason {
    /// Queue was full and all retry attempts failed
    QueueFull,
    /// Channel was disconnected
    ChannelDisconnected,
    /// Message processing timeout
    Timeout,
    /// Custom error message
    Custom(String),
}

/// A failed message with metadata
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeadLetter {
    /// The original message that failed
    pub message: WebViewMessage,
    /// Reason for failure
    pub reason: FailureReason,
    /// Timestamp when the message failed (milliseconds since epoch)
    pub timestamp: u64,
    /// Number of retry attempts made
    pub retry_count: u32,
}

impl DeadLetter {
    /// Create a new dead letter
    #[allow(dead_code)]
    pub fn new(message: WebViewMessage, reason: FailureReason, retry_count: u32) -> Self {
        Self {
            message,
            reason,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            retry_count,
        }
    }
}

/// Configuration for the dead letter queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeadLetterQueueConfig {
    /// Maximum number of dead letters to store
    pub max_size: usize,
    /// Whether to log dead letters to tracing
    pub log_failures: bool,
}

impl Default for DeadLetterQueueConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            log_failures: true,
        }
    }
}

/// Dead Letter Queue for storing failed messages
///
/// This queue is thread-safe and can be accessed from multiple threads.
/// It uses a circular buffer to prevent unbounded memory growth.
#[derive(Clone)]
#[allow(dead_code)]
pub struct DeadLetterQueue {
    /// Internal storage for dead letters
    letters: Arc<Mutex<VecDeque<DeadLetter>>>,
    /// Configuration
    config: DeadLetterQueueConfig,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue with default configuration
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_config(DeadLetterQueueConfig::default())
    }

    /// Create a new dead letter queue with custom configuration
    pub fn with_config(config: DeadLetterQueueConfig) -> Self {
        Self {
            letters: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_size))),
            config,
        }
    }

    /// Add a failed message to the DLQ
    #[allow(dead_code)]
    pub fn push(&self, message: WebViewMessage, reason: FailureReason, retry_count: u32) {
        let dead_letter = DeadLetter::new(message, reason.clone(), retry_count);

        if self.config.log_failures {
            tracing::error!(
                "[DLQ] Message failed after {} retries: {:?}",
                retry_count,
                reason
            );
        }

        let mut letters = self.letters.lock();

        // If at capacity, remove oldest message
        if letters.len() >= self.config.max_size {
            letters.pop_front();
            tracing::warn!("[DLQ] Queue full, removing oldest dead letter");
        }

        letters.push_back(dead_letter);
    }

    /// Get the number of dead letters in the queue
    pub fn len(&self) -> usize {
        self.letters.lock().len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.letters.lock().is_empty()
    }

    /// Get all dead letters (for debugging/monitoring)
    pub fn get_all(&self) -> Vec<DeadLetter> {
        self.letters.lock().iter().cloned().collect()
    }

    /// Get the most recent N dead letters
    pub fn get_recent(&self, count: usize) -> Vec<DeadLetter> {
        let letters = self.letters.lock();
        letters.iter().rev().take(count).cloned().collect()
    }

    /// Clear all dead letters
    pub fn clear(&self) {
        self.letters.lock().clear();
        tracing::info!("[DLQ] Cleared all dead letters");
    }

    /// Get statistics about failures
    pub fn get_stats(&self) -> DeadLetterStats {
        let letters = self.letters.lock();
        let total = letters.len();

        let mut queue_full = 0;
        let mut disconnected = 0;
        let mut timeout = 0;
        let mut custom = 0;

        for letter in letters.iter() {
            match letter.reason {
                FailureReason::QueueFull => queue_full += 1,
                FailureReason::ChannelDisconnected => disconnected += 1,
                FailureReason::Timeout => timeout += 1,
                FailureReason::Custom(_) => custom += 1,
            }
        }

        DeadLetterStats {
            total,
            queue_full,
            disconnected,
            timeout,
            custom,
        }
    }
}

impl Default for DeadLetterQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about dead letters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterStats {
    /// Total number of dead letters
    pub total: usize,
    /// Number of failures due to queue full
    pub queue_full: usize,
    /// Number of failures due to channel disconnection
    pub disconnected: usize,
    /// Number of failures due to timeout
    pub timeout: usize,
    /// Number of failures with custom reasons
    pub custom: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlq_basic() {
        let dlq = DeadLetterQueue::new();
        assert_eq!(dlq.len(), 0);
        assert!(dlq.is_empty());

        dlq.push(
            WebViewMessage::EvalJs("test".to_string()),
            FailureReason::QueueFull,
            3,
        );

        assert_eq!(dlq.len(), 1);
        assert!(!dlq.is_empty());
    }

    #[test]
    fn test_dlq_capacity() {
        let config = DeadLetterQueueConfig {
            max_size: 5,
            log_failures: false,
        };
        let dlq = DeadLetterQueue::with_config(config);

        // Add 10 messages
        for i in 0..10 {
            dlq.push(
                WebViewMessage::EvalJs(format!("test{}", i)),
                FailureReason::QueueFull,
                1,
            );
        }

        // Should only keep the last 5
        assert_eq!(dlq.len(), 5);
    }

    #[test]
    fn test_dlq_stats() {
        let dlq = DeadLetterQueue::new();

        dlq.push(
            WebViewMessage::EvalJs("test1".to_string()),
            FailureReason::QueueFull,
            1,
        );
        dlq.push(
            WebViewMessage::EvalJs("test2".to_string()),
            FailureReason::ChannelDisconnected,
            1,
        );
        dlq.push(
            WebViewMessage::EvalJs("test3".to_string()),
            FailureReason::QueueFull,
            1,
        );

        let stats = dlq.get_stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.queue_full, 2);
        assert_eq!(stats.disconnected, 1);
    }
}
