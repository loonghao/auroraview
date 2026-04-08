//! Message processor tests

use auroraview_core::backend::{
    AtomicProcessorStats, MessagePriority, ProcessResult, ProcessingMode, ProcessorConfig,
    ProcessorStats, WakeController,
};
use rstest::rstest;
use std::{sync::Arc, time::Duration};

// ============================================================================
// ProcessResult
// ============================================================================

#[test]
fn test_process_result_continue_does_not_stop() {
    assert!(!ProcessResult::Continue.should_stop());
    assert!(!ProcessResult::Continue.as_bool());
}

#[test]
fn test_process_result_close_requested_stops() {
    assert!(ProcessResult::CloseRequested.should_stop());
    assert!(ProcessResult::CloseRequested.as_bool());
}

#[test]
fn test_process_result_error_stops() {
    assert!(ProcessResult::Error.should_stop());
    assert!(ProcessResult::Error.as_bool());
}

#[test]
fn test_process_result_eq() {
    assert_eq!(ProcessResult::Continue, ProcessResult::Continue);
    assert_eq!(
        ProcessResult::CloseRequested,
        ProcessResult::CloseRequested
    );
    assert_ne!(ProcessResult::Continue, ProcessResult::Error);
}

#[test]
fn test_process_result_clone_copy() {
    let r = ProcessResult::CloseRequested;
    let r2 = r;
    assert_eq!(r, r2);
}

// ============================================================================
// ProcessingMode
// ============================================================================

#[test]
fn test_processing_mode_default_is_full() {
    assert_eq!(ProcessingMode::default(), ProcessingMode::Full);
}

#[test]
fn test_processing_mode_eq() {
    assert_eq!(ProcessingMode::Full, ProcessingMode::Full);
    assert_eq!(ProcessingMode::IpcOnly, ProcessingMode::IpcOnly);
    assert_ne!(ProcessingMode::Full, ProcessingMode::IpcOnly);
}

#[test]
fn test_processing_mode_batch_eq() {
    assert_eq!(
        ProcessingMode::Batch { max_messages: 10 },
        ProcessingMode::Batch { max_messages: 10 }
    );
    assert_ne!(
        ProcessingMode::Batch { max_messages: 10 },
        ProcessingMode::Batch { max_messages: 20 }
    );
}

#[test]
fn test_processing_mode_batch_not_eq_ipc_only() {
    assert_ne!(
        ProcessingMode::Batch { max_messages: 5 },
        ProcessingMode::IpcOnly
    );
}

#[test]
fn test_processing_mode_debug() {
    let s = format!("{:?}", ProcessingMode::Batch { max_messages: 50 });
    assert!(s.contains("50"));
}

// ============================================================================
// ProcessorConfig
// ============================================================================

#[test]
fn test_processor_config_default() {
    let cfg = ProcessorConfig::default();
    assert_eq!(cfg.mode, ProcessingMode::Full);
    assert!(cfg.immediate_wake);
    assert_eq!(cfg.batch_interval_ms, 0);
    assert_eq!(cfg.max_messages_per_tick, 0);
}

#[test]
fn test_processor_config_standalone() {
    let cfg = ProcessorConfig::standalone();
    assert_eq!(cfg.mode, ProcessingMode::Full);
    assert!(cfg.immediate_wake);
    assert_eq!(cfg.batch_interval_ms, 0);
    assert_eq!(cfg.max_messages_per_tick, 0);
}

#[test]
fn test_processor_config_qt_embedded() {
    let cfg = ProcessorConfig::qt_embedded();
    assert_eq!(cfg.mode, ProcessingMode::IpcOnly);
    assert!(cfg.immediate_wake);
    assert_eq!(cfg.max_messages_per_tick, 100);
}

#[test]
fn test_processor_config_debug() {
    let cfg = ProcessorConfig::default();
    let s = format!("{:?}", cfg);
    assert!(s.contains("Full"));
}

// ============================================================================
// MessagePriority – ordering
// ============================================================================

#[test]
fn test_message_priority_default_is_normal() {
    assert_eq!(MessagePriority::default(), MessagePriority::Normal);
}

#[test]
fn test_message_priority_ord() {
    assert!(MessagePriority::Low < MessagePriority::Normal);
    assert!(MessagePriority::Normal < MessagePriority::High);
    assert!(MessagePriority::High < MessagePriority::Critical);
    assert!(MessagePriority::Low < MessagePriority::Critical);
}

#[test]
fn test_message_priority_sorting() {
    let mut priorities = vec![
        MessagePriority::High,
        MessagePriority::Low,
        MessagePriority::Critical,
        MessagePriority::Normal,
    ];
    priorities.sort();
    assert_eq!(
        priorities,
        vec![
            MessagePriority::Low,
            MessagePriority::Normal,
            MessagePriority::High,
            MessagePriority::Critical,
        ]
    );
}

#[rstest]
#[case(MessagePriority::Low, MessagePriority::Normal, true)]
#[case(MessagePriority::Normal, MessagePriority::High, true)]
#[case(MessagePriority::High, MessagePriority::Critical, true)]
#[case(MessagePriority::Critical, MessagePriority::Critical, false)]
#[case(MessagePriority::High, MessagePriority::Normal, false)]
fn test_message_priority_lt(
    #[case] a: MessagePriority,
    #[case] b: MessagePriority,
    #[case] expected: bool,
) {
    assert_eq!(a < b, expected);
}

#[test]
fn test_message_priority_ge_for_high_priority_threshold() {
    // threshold for immediate wake is >= High
    assert!(MessagePriority::High >= MessagePriority::High);
    assert!(MessagePriority::Critical >= MessagePriority::High);
    assert!(!(MessagePriority::Normal >= MessagePriority::High));
    assert!(!(MessagePriority::Low >= MessagePriority::High));
}

#[test]
fn test_message_priority_clone_copy() {
    let p = MessagePriority::High;
    let p2 = p;
    assert_eq!(p, p2);
}

// ============================================================================
// ProcessorStats
// ============================================================================

#[test]
fn test_processor_stats_default() {
    let s = ProcessorStats::default();
    assert_eq!(s.messages_processed, 0);
    assert_eq!(s.total_processing_time_us, 0);
    assert_eq!(s.peak_tick_time_us, 0);
    assert_eq!(s.ticks_processed, 0);
    assert_eq!(s.batch_skips, 0);
}

#[test]
fn test_processor_stats_avg_tick_time_zero_ticks() {
    let s = ProcessorStats::default();
    assert_eq!(s.avg_tick_time_us(), 0);
}

#[test]
fn test_processor_stats_avg_tick_time() {
    let s = ProcessorStats {
        total_processing_time_us: 300,
        ticks_processed: 3,
        ..Default::default()
    };
    assert_eq!(s.avg_tick_time_us(), 100);
}

#[test]
fn test_processor_stats_avg_messages_per_tick_zero_ticks() {
    let s = ProcessorStats::default();
    assert_eq!(s.avg_messages_per_tick(), 0.0);
}

#[test]
fn test_processor_stats_avg_messages_per_tick() {
    let s = ProcessorStats {
        messages_processed: 30,
        ticks_processed: 5,
        ..Default::default()
    };
    assert!((s.avg_messages_per_tick() - 6.0).abs() < f64::EPSILON);
}

// ============================================================================
// AtomicProcessorStats
// ============================================================================

#[test]
fn test_atomic_stats_initial_snapshot_is_zero() {
    let s = AtomicProcessorStats::new();
    let snap = s.snapshot();
    assert_eq!(snap.messages_processed, 0);
    assert_eq!(snap.ticks_processed, 0);
    assert_eq!(snap.peak_tick_time_us, 0);
    assert_eq!(snap.total_processing_time_us, 0);
    assert_eq!(snap.batch_skips, 0);
}

#[test]
fn test_atomic_stats_default_eq_new() {
    let s = AtomicProcessorStats::default();
    let snap = s.snapshot();
    assert_eq!(snap.messages_processed, 0);
}

#[test]
fn test_atomic_stats_record_tick_accumulates() {
    let s = AtomicProcessorStats::new();
    s.record_tick(5, Duration::from_micros(50));
    s.record_tick(10, Duration::from_micros(100));

    let snap = s.snapshot();
    assert_eq!(snap.messages_processed, 15);
    assert_eq!(snap.ticks_processed, 2);
    assert_eq!(snap.total_processing_time_us, 150);
    assert_eq!(snap.peak_tick_time_us, 100);
}

#[test]
fn test_atomic_stats_peak_tracks_max() {
    let s = AtomicProcessorStats::new();
    s.record_tick(1, Duration::from_micros(500));
    s.record_tick(1, Duration::from_micros(200));
    s.record_tick(1, Duration::from_micros(800));
    s.record_tick(1, Duration::from_micros(100));

    assert_eq!(s.snapshot().peak_tick_time_us, 800);
}

#[test]
fn test_atomic_stats_record_batch_skip() {
    let s = AtomicProcessorStats::new();
    s.record_batch_skip();
    s.record_batch_skip();
    assert_eq!(s.snapshot().batch_skips, 2);
}

#[test]
fn test_atomic_stats_reset_clears_all() {
    let s = AtomicProcessorStats::new();
    s.record_tick(100, Duration::from_micros(1000));
    s.record_batch_skip();

    s.reset();
    let snap = s.snapshot();
    assert_eq!(snap.messages_processed, 0);
    assert_eq!(snap.ticks_processed, 0);
    assert_eq!(snap.total_processing_time_us, 0);
    assert_eq!(snap.peak_tick_time_us, 0);
    assert_eq!(snap.batch_skips, 0);
}

#[test]
fn test_atomic_stats_reset_idempotent() {
    let s = AtomicProcessorStats::new();
    s.reset();
    s.reset();
    assert_eq!(s.snapshot().messages_processed, 0);
}

#[test]
fn test_atomic_stats_concurrent_record_tick() {
    use std::thread;

    let s = Arc::new(AtomicProcessorStats::new());
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let s = Arc::clone(&s);
            thread::spawn(move || {
                for _ in 0..100 {
                    s.record_tick(1, Duration::from_micros(10));
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let snap = s.snapshot();
    assert_eq!(snap.messages_processed, 1000);
    assert_eq!(snap.ticks_processed, 1000);
}

#[test]
fn test_atomic_stats_concurrent_batch_skip() {
    use std::thread;

    let s = Arc::new(AtomicProcessorStats::new());
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let s = Arc::clone(&s);
            thread::spawn(move || {
                for _ in 0..20 {
                    s.record_batch_skip();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(s.snapshot().batch_skips, 100);
}

// ============================================================================
// WakeController – no batching
// ============================================================================

#[test]
fn test_wake_controller_no_batching_always_wakes() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 0,
        immediate_wake: true,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats);

    for _ in 0..5 {
        assert!(wc.should_wake(MessagePriority::Low));
        assert!(wc.should_wake(MessagePriority::Normal));
        assert!(wc.should_wake(MessagePriority::High));
        assert!(wc.should_wake(MessagePriority::Critical));
    }
}

// ============================================================================
// WakeController – with batching
// ============================================================================

#[test]
fn test_wake_controller_batching_first_wake_succeeds() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: false,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats);
    assert!(wc.should_wake(MessagePriority::Normal));
}

#[test]
fn test_wake_controller_batching_immediate_second_wake_blocked() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: false,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats);
    assert!(wc.should_wake(MessagePriority::Normal));
    assert!(!wc.should_wake(MessagePriority::Normal));
}

#[test]
fn test_wake_controller_high_priority_bypasses_batching() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: true,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats.clone());

    // Consume first wake slot
    wc.should_wake(MessagePriority::Normal);
    // Immediate second would be batched, but High bypasses
    assert!(wc.should_wake(MessagePriority::High));
    assert!(wc.should_wake(MessagePriority::Critical));

    let snap = stats.snapshot();
    assert!(snap.batch_skips >= 2);
}

#[test]
fn test_wake_controller_high_priority_no_bypass_when_disabled() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: false,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats.clone());

    wc.should_wake(MessagePriority::Normal);
    // High priority does NOT bypass because immediate_wake = false
    assert!(!wc.should_wake(MessagePriority::High));

    assert_eq!(stats.snapshot().batch_skips, 0);
}

#[test]
fn test_wake_controller_set_immediate_wake_enable() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: false,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats.clone());

    wc.should_wake(MessagePriority::Normal);
    assert!(!wc.should_wake(MessagePriority::High));

    // Enable immediate wake
    wc.set_immediate_wake(true);
    assert!(wc.should_wake(MessagePriority::High));
    assert!(stats.snapshot().batch_skips >= 1);
}

#[test]
fn test_wake_controller_set_immediate_wake_disable() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: true,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats.clone());

    wc.should_wake(MessagePriority::Normal);

    // Disable immediate wake
    wc.set_immediate_wake(false);
    assert!(!wc.should_wake(MessagePriority::High));
    assert_eq!(stats.snapshot().batch_skips, 0);
}

#[test]
fn test_wake_controller_force_wake_resets_timer() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: false,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats);

    // First wake
    wc.should_wake(MessagePriority::Normal);
    // Now blocked
    assert!(!wc.should_wake(MessagePriority::Normal));

    // force_wake updates the last_wake timestamp, so next check will still be blocked
    // (last_wake was just set to now, so interval not elapsed)
    wc.force_wake();
    assert!(!wc.should_wake(MessagePriority::Normal));
}

#[test]
fn test_wake_controller_low_priority_no_bypass() {
    let cfg = ProcessorConfig {
        batch_interval_ms: 5000,
        immediate_wake: true,
        ..Default::default()
    };
    let stats = Arc::new(AtomicProcessorStats::new());
    let wc = WakeController::new(&cfg, stats.clone());

    wc.should_wake(MessagePriority::Normal);
    // Low priority does not bypass even with immediate_wake
    assert!(!wc.should_wake(MessagePriority::Low));
    assert_eq!(stats.snapshot().batch_skips, 0);
}
