//! Integration tests for Timer functionality
//!
//! These tests verify timer behavior with actual time delays and threading.

use _core::webview::timer::Timer;
use rstest::*;
use std::thread;
use std::time::Duration;

/// Fixture: Create a timer with default interval
#[fixture]
fn timer() -> Timer {
    Timer::new(100)
}

#[rstest]
fn test_timer_throttling(timer: Timer) {
    // First tick should succeed
    assert!(timer.should_tick(), "First tick should succeed");

    // Immediate second tick should fail (throttled)
    assert!(
        !timer.should_tick(),
        "Immediate second tick should be throttled"
    );

    // Wait for interval
    thread::sleep(Duration::from_millis(110));

    // Now it should succeed
    assert!(timer.should_tick(), "Tick after interval should succeed");
}

#[rstest]
fn test_timer_throttling_precise() {
    // Use a longer interval to reduce sensitivity to scheduler variance
    // macOS CI runners have very high scheduler variance (50-100ms+)
    let timer = Timer::new(200);

    // First tick
    assert!(timer.should_tick(), "First tick should succeed");

    // Wait significantly less than interval (safe margin for CI environments)
    // Using 50ms which is well under the 200ms interval
    thread::sleep(Duration::from_millis(50));
    assert!(
        !timer.should_tick(),
        "Tick before interval should be throttled"
    );

    // Wait for remaining time plus very generous tolerance for CI environments
    // macOS/Linux CI runners can have extremely high scheduler variance (50-100ms+)
    // Total wait: 50ms + 200ms = 250ms, which gives 50ms tolerance over the 200ms interval
    thread::sleep(Duration::from_millis(200));
    assert!(
        timer.should_tick(),
        "Tick after full interval should succeed"
    );
}

#[rstest]
#[case(1)]
#[case(16)]
#[case(50)]
#[case(100)]
fn test_timer_throttling_various_intervals(#[case] interval_ms: u32) {
    let timer = Timer::new(interval_ms);

    // First tick should always succeed
    assert!(
        timer.should_tick(),
        "First tick should succeed for interval {}",
        interval_ms
    );

    // Immediate second tick should be throttled
    assert!(
        !timer.should_tick(),
        "Immediate tick should be throttled for interval {}",
        interval_ms
    );

    // Wait for interval + buffer
    thread::sleep(Duration::from_millis((interval_ms + 10) as u64));

    // Should succeed after waiting
    assert!(
        timer.should_tick(),
        "Tick after interval should succeed for interval {}",
        interval_ms
    );
}

#[rstest]
fn test_timer_multiple_ticks_over_time() {
    let timer = Timer::new(30);
    let mut successful_ticks = 0;

    // Try ticking multiple times over a period
    for _ in 0..5 {
        if timer.should_tick() {
            successful_ticks += 1;
        }
        thread::sleep(Duration::from_millis(35));
    }

    // Should have multiple successful ticks
    assert!(
        successful_ticks >= 4,
        "Should have at least 4 successful ticks, got {}",
        successful_ticks
    );
}
