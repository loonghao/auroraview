//! Integration tests for Timer functionality
//!
//! These tests verify timer behavior with actual time delays and threading.

use auroraview_core::webview::timer::Timer;
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
    let timer = Timer::new(50);

    // First tick
    assert!(timer.should_tick(), "First tick should succeed");

    // Wait less than interval
    thread::sleep(Duration::from_millis(30));
    assert!(
        !timer.should_tick(),
        "Tick before interval should be throttled"
    );

    // Wait for remaining time
    thread::sleep(Duration::from_millis(25));
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
