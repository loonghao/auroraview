//! Performance metrics for IPC communication
//!
//! This module provides detailed metrics tracking for IPC operations,
//! including message throughput, latency, and failure rates.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// IPC performance metrics
///
/// All counters are atomic and can be safely accessed from multiple threads.
#[derive(Clone)]
pub struct IpcMetrics {
    /// Total number of messages sent successfully
    messages_sent: Arc<AtomicU64>,

    /// Total number of messages that failed to send
    messages_failed: Arc<AtomicU64>,

    /// Total number of messages dropped (queue full, no retry)
    messages_dropped: Arc<AtomicU64>,

    /// Total number of retry attempts
    retry_attempts: Arc<AtomicU64>,

    /// Total latency in microseconds (for calculating average)
    total_latency_us: Arc<AtomicU64>,

    /// Number of latency samples
    latency_samples: Arc<AtomicU64>,

    /// Peak queue length observed
    peak_queue_length: Arc<AtomicU64>,

    /// Total number of messages received
    messages_received: Arc<AtomicU64>,
}

impl IpcMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            messages_sent: Arc::new(AtomicU64::new(0)),
            messages_failed: Arc::new(AtomicU64::new(0)),
            messages_dropped: Arc::new(AtomicU64::new(0)),
            retry_attempts: Arc::new(AtomicU64::new(0)),
            total_latency_us: Arc::new(AtomicU64::new(0)),
            latency_samples: Arc::new(AtomicU64::new(0)),
            peak_queue_length: Arc::new(AtomicU64::new(0)),
            messages_received: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a successful message send
    pub fn record_send(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed message send
    pub fn record_failure(&self) {
        self.messages_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a dropped message
    pub fn record_drop(&self) {
        self.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.retry_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record message latency in microseconds
    pub fn record_latency(&self, latency_us: u64) {
        self.total_latency_us
            .fetch_add(latency_us, Ordering::Relaxed);
        self.latency_samples.fetch_add(1, Ordering::Relaxed);
    }

    /// Update peak queue length if current length is higher
    pub fn update_peak_queue_length(&self, current_length: usize) {
        let current = current_length as u64;
        let mut peak = self.peak_queue_length.load(Ordering::Relaxed);

        while current > peak {
            match self.peak_queue_length.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }

    /// Record a received message
    pub fn record_receive(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> IpcMetricsSnapshot {
        let messages_sent = self.messages_sent.load(Ordering::Relaxed);
        let messages_failed = self.messages_failed.load(Ordering::Relaxed);
        let messages_dropped = self.messages_dropped.load(Ordering::Relaxed);
        let retry_attempts = self.retry_attempts.load(Ordering::Relaxed);
        let total_latency_us = self.total_latency_us.load(Ordering::Relaxed);
        let latency_samples = self.latency_samples.load(Ordering::Relaxed);
        let peak_queue_length = self.peak_queue_length.load(Ordering::Relaxed);
        let messages_received = self.messages_received.load(Ordering::Relaxed);

        let avg_latency_us = if latency_samples > 0 {
            total_latency_us / latency_samples
        } else {
            0
        };

        let success_rate = if messages_sent + messages_failed > 0 {
            (messages_sent as f64) / ((messages_sent + messages_failed) as f64) * 100.0
        } else {
            100.0
        };

        IpcMetricsSnapshot {
            messages_sent,
            messages_failed,
            messages_dropped,
            retry_attempts,
            avg_latency_us,
            peak_queue_length,
            messages_received,
            success_rate,
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.messages_sent.store(0, Ordering::Relaxed);
        self.messages_failed.store(0, Ordering::Relaxed);
        self.messages_dropped.store(0, Ordering::Relaxed);
        self.retry_attempts.store(0, Ordering::Relaxed);
        self.total_latency_us.store(0, Ordering::Relaxed);
        self.latency_samples.store(0, Ordering::Relaxed);
        self.peak_queue_length.store(0, Ordering::Relaxed);
        self.messages_received.store(0, Ordering::Relaxed);
    }
}

impl Default for IpcMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of IPC metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMetricsSnapshot {
    /// Total messages sent successfully
    pub messages_sent: u64,

    /// Total messages that failed
    pub messages_failed: u64,

    /// Total messages dropped
    pub messages_dropped: u64,

    /// Total retry attempts
    pub retry_attempts: u64,

    /// Average latency in microseconds
    pub avg_latency_us: u64,

    /// Peak queue length
    pub peak_queue_length: u64,

    /// Total messages received
    pub messages_received: u64,

    /// Success rate (percentage)
    pub success_rate: f64,
}

impl IpcMetricsSnapshot {
    /// Format metrics as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "IPC Metrics:\n\
             - Messages Sent: {}\n\
             - Messages Failed: {}\n\
             - Messages Dropped: {}\n\
             - Retry Attempts: {}\n\
             - Success Rate: {:.2}%\n\
             - Avg Latency: {}μs\n\
             - Peak Queue Length: {}\n\
             - Messages Received: {}",
            self.messages_sent,
            self.messages_failed,
            self.messages_dropped,
            self.retry_attempts,
            self.success_rate,
            self.avg_latency_us,
            self.peak_queue_length,
            self.messages_received
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_basic() {
        let metrics = IpcMetrics::new();

        metrics.record_send();
        metrics.record_send();
        metrics.record_failure();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.messages_sent, 2);
        assert_eq!(snapshot.messages_failed, 1);
        assert_eq!(snapshot.success_rate, 66.66666666666666);
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
        metrics.update_peak_queue_length(15);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.peak_queue_length, 20);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = IpcMetrics::new();

        metrics.record_send();
        metrics.record_failure();
        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.messages_sent, 0);
        assert_eq!(snapshot.messages_failed, 0);
    }
}
