//! Unified Message Processor
//!
//! Provides a single source of truth for message processing across all modes:
//! - Standalone blocking
//! - Standalone threaded
//! - Embedded host pump (Qt/DCC)
//! - Embedded self pump
//!
//! This addresses the P2 finding about inconsistent event processing strategies.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Result of processing messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    /// Continue processing
    Continue,
    /// Window close requested
    CloseRequested,
    /// Error occurred during processing
    Error,
}

impl ProcessResult {
    /// Check if processing should stop
    pub fn should_stop(&self) -> bool {
        matches!(self, ProcessResult::CloseRequested | ProcessResult::Error)
    }

    /// Convert to bool (true = should close)
    pub fn as_bool(&self) -> bool {
        self.should_stop()
    }
}

/// Processing mode determines how messages are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessingMode {
    /// Full processing - handle all messages and window events
    #[default]
    Full,
    /// IPC only - only process IPC messages, skip window events
    /// Used when host application owns the message pump (Qt/DCC)
    IpcOnly,
    /// Batch processing with limit
    Batch { max_messages: usize },
}

/// Configuration for message processing
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Processing mode
    pub mode: ProcessingMode,
    /// Enable immediate wake for high-priority messages
    /// Addresses P0: Message queue batching latency
    pub immediate_wake: bool,
    /// Batch interval in milliseconds (0 = no batching)
    pub batch_interval_ms: u64,
    /// Maximum messages per tick (0 = unlimited)
    pub max_messages_per_tick: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            mode: ProcessingMode::Full,
            immediate_wake: true, // Default to immediate wake for responsiveness
            batch_interval_ms: 0, // No batching by default
            max_messages_per_tick: 0, // Unlimited
        }
    }
}

impl ProcessorConfig {
    /// Create config for standalone mode
    pub fn standalone() -> Self {
        Self {
            mode: ProcessingMode::Full,
            immediate_wake: true,
            batch_interval_ms: 0,
            max_messages_per_tick: 0,
        }
    }

    /// Create config for Qt/DCC embedded mode
    pub fn qt_embedded() -> Self {
        Self {
            mode: ProcessingMode::IpcOnly,
            immediate_wake: true,
            batch_interval_ms: 0,
            max_messages_per_tick: 100, // Limit for busy DCC main threads
        }
    }

    /// Create config for legacy embedded mode
    pub fn legacy_embedded() -> Self {
        Self {
            mode: ProcessingMode::Full,
            immediate_wake: true,
            batch_interval_ms: 0,
            max_messages_per_tick: 50,
        }
    }
}

/// Message priority for wake-up decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum MessagePriority {
    /// Low priority - can be batched
    Low = 0,
    /// Normal priority - default
    #[default]
    Normal = 1,
    /// High priority - immediate wake
    High = 2,
    /// Critical - bypass all batching
    Critical = 3,
}

/// Statistics for message processing
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    /// Total messages processed
    pub messages_processed: u64,
    /// Total processing time in microseconds
    pub total_processing_time_us: u64,
    /// Peak processing time for a single tick
    pub peak_tick_time_us: u64,
    /// Number of ticks processed
    pub ticks_processed: u64,
    /// Number of times batching was skipped due to high priority
    pub batch_skips: u64,
}

impl ProcessorStats {
    /// Get average processing time per tick in microseconds
    pub fn avg_tick_time_us(&self) -> u64 {
        if self.ticks_processed == 0 {
            0
        } else {
            self.total_processing_time_us / self.ticks_processed
        }
    }

    /// Get average messages per tick
    pub fn avg_messages_per_tick(&self) -> f64 {
        if self.ticks_processed == 0 {
            0.0
        } else {
            self.messages_processed as f64 / self.ticks_processed as f64
        }
    }
}

/// Atomic statistics for thread-safe updates
pub struct AtomicProcessorStats {
    messages_processed: AtomicU64,
    total_processing_time_us: AtomicU64,
    peak_tick_time_us: AtomicU64,
    ticks_processed: AtomicU64,
    batch_skips: AtomicU64,
}

impl Default for AtomicProcessorStats {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicProcessorStats {
    /// Create new atomic stats
    pub fn new() -> Self {
        Self {
            messages_processed: AtomicU64::new(0),
            total_processing_time_us: AtomicU64::new(0),
            peak_tick_time_us: AtomicU64::new(0),
            ticks_processed: AtomicU64::new(0),
            batch_skips: AtomicU64::new(0),
        }
    }

    /// Record a processing tick
    pub fn record_tick(&self, messages: u64, duration: Duration) {
        let us = duration.as_micros() as u64;

        self.messages_processed
            .fetch_add(messages, Ordering::Relaxed);
        self.total_processing_time_us
            .fetch_add(us, Ordering::Relaxed);
        self.ticks_processed.fetch_add(1, Ordering::Relaxed);

        // Update peak if this tick was longer
        let mut current_peak = self.peak_tick_time_us.load(Ordering::Relaxed);
        while us > current_peak {
            match self.peak_tick_time_us.compare_exchange_weak(
                current_peak,
                us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    /// Record a batch skip
    pub fn record_batch_skip(&self) {
        self.batch_skips.fetch_add(1, Ordering::Relaxed);
    }

    /// Get snapshot of current stats
    pub fn snapshot(&self) -> ProcessorStats {
        ProcessorStats {
            messages_processed: self.messages_processed.load(Ordering::Relaxed),
            total_processing_time_us: self.total_processing_time_us.load(Ordering::Relaxed),
            peak_tick_time_us: self.peak_tick_time_us.load(Ordering::Relaxed),
            ticks_processed: self.ticks_processed.load(Ordering::Relaxed),
            batch_skips: self.batch_skips.load(Ordering::Relaxed),
        }
    }

    /// Reset all stats
    pub fn reset(&self) {
        self.messages_processed.store(0, Ordering::Relaxed);
        self.total_processing_time_us.store(0, Ordering::Relaxed);
        self.peak_tick_time_us.store(0, Ordering::Relaxed);
        self.ticks_processed.store(0, Ordering::Relaxed);
        self.batch_skips.store(0, Ordering::Relaxed);
    }
}

/// Wake-up controller for message queue
///
/// Addresses P0: Message queue wake-up batching latency
pub struct WakeController {
    /// Last wake time
    last_wake: std::sync::Mutex<Option<Instant>>,
    /// Batch interval
    batch_interval: Duration,
    /// Whether immediate wake is enabled
    immediate_wake_enabled: AtomicBool,
    /// Stats
    stats: Arc<AtomicProcessorStats>,
}

impl WakeController {
    /// Create a new wake controller
    pub fn new(config: &ProcessorConfig, stats: Arc<AtomicProcessorStats>) -> Self {
        Self {
            last_wake: std::sync::Mutex::new(None),
            batch_interval: Duration::from_millis(config.batch_interval_ms),
            immediate_wake_enabled: AtomicBool::new(config.immediate_wake),
            stats,
        }
    }

    /// Check if a wake-up should be triggered
    pub fn should_wake(&self, priority: MessagePriority) -> bool {
        // High priority always wakes immediately
        if priority >= MessagePriority::High && self.immediate_wake_enabled.load(Ordering::Relaxed)
        {
            self.stats.record_batch_skip();
            return true;
        }

        // No batching configured
        if self.batch_interval.is_zero() {
            return true;
        }

        // Check batch interval
        if let Ok(mut last_wake) = self.last_wake.lock() {
            let now = Instant::now();
            match *last_wake {
                Some(last) if now.duration_since(last) < self.batch_interval => false,
                _ => {
                    *last_wake = Some(now);
                    true
                }
            }
        } else {
            // Lock poisoned, wake anyway
            true
        }
    }

    /// Force a wake-up (bypasses batching)
    pub fn force_wake(&self) {
        if let Ok(mut last_wake) = self.last_wake.lock() {
            *last_wake = Some(Instant::now());
        }
    }

    /// Enable or disable immediate wake for high priority messages
    pub fn set_immediate_wake(&self, enabled: bool) {
        self.immediate_wake_enabled
            .store(enabled, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_result() {
        assert!(!ProcessResult::Continue.should_stop());
        assert!(ProcessResult::CloseRequested.should_stop());
        assert!(ProcessResult::Error.should_stop());
    }

    #[test]
    fn test_processor_config_presets() {
        let standalone = ProcessorConfig::standalone();
        assert_eq!(standalone.mode, ProcessingMode::Full);
        assert!(standalone.immediate_wake);

        let qt = ProcessorConfig::qt_embedded();
        assert_eq!(qt.mode, ProcessingMode::IpcOnly);
        assert_eq!(qt.max_messages_per_tick, 100);
    }

    #[test]
    fn test_atomic_stats() {
        let stats = AtomicProcessorStats::new();

        stats.record_tick(10, Duration::from_micros(100));
        stats.record_tick(20, Duration::from_micros(200));

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.messages_processed, 30);
        assert_eq!(snapshot.ticks_processed, 2);
        assert_eq!(snapshot.peak_tick_time_us, 200);
    }

    #[test]
    fn test_wake_controller_no_batching() {
        let config = ProcessorConfig {
            batch_interval_ms: 0,
            ..Default::default()
        };
        let stats = Arc::new(AtomicProcessorStats::new());
        let controller = WakeController::new(&config, stats);

        // Should always wake with no batching
        assert!(controller.should_wake(MessagePriority::Low));
        assert!(controller.should_wake(MessagePriority::Normal));
        assert!(controller.should_wake(MessagePriority::High));
    }

    #[test]
    fn test_wake_controller_with_batching() {
        let config = ProcessorConfig {
            batch_interval_ms: 100,
            immediate_wake: true,
            ..Default::default()
        };
        let stats = Arc::new(AtomicProcessorStats::new());
        let controller = WakeController::new(&config, stats.clone());

        // First wake should succeed
        assert!(controller.should_wake(MessagePriority::Normal));

        // Immediate second wake should be batched
        assert!(!controller.should_wake(MessagePriority::Normal));

        // High priority should bypass batching
        assert!(controller.should_wake(MessagePriority::High));
        assert_eq!(stats.snapshot().batch_skips, 1);
    }
}
