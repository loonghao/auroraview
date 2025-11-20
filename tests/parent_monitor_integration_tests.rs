//! Integration tests for ParentWindowMonitor
//!
//! These tests verify the parent window monitoring functionality with actual threading.

#[cfg(target_os = "windows")]
use auroraview_core::webview::parent_monitor::ParentWindowMonitor;
use rstest::*;
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(target_os = "windows")]
use std::sync::Arc;
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;

#[rstest]
#[cfg(target_os = "windows")]
fn test_parent_monitor_invalid_hwnd() {
    // Use an invalid HWND (0)
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = callback_count.clone();

    let mut monitor = ParentWindowMonitor::new(
        0, // Invalid HWND
        move || {
            callback_count_clone.fetch_add(1, Ordering::Relaxed);
        },
        100, // Check every 100ms
    );

    // Wait for callback to be invoked
    thread::sleep(Duration::from_millis(500));

    // Callback should have been invoked
    assert!(
        callback_count.load(Ordering::Relaxed) > 0,
        "Callback should be invoked for invalid HWND"
    );

    monitor.stop();
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_parent_monitor_stop() {
    let mut monitor = ParentWindowMonitor::new(0, || {}, 100);

    assert!(
        monitor.is_running(),
        "Monitor should be running after creation"
    );

    monitor.stop();

    // Give it a moment to stop
    thread::sleep(Duration::from_millis(200));

    assert!(
        !monitor.is_running(),
        "Monitor should be stopped after stop()"
    );
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_parent_monitor_multiple_callbacks() {
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = callback_count.clone();

    let mut monitor = ParentWindowMonitor::new(
        0,
        move || {
            callback_count_clone.fetch_add(1, Ordering::Relaxed);
        },
        100, // Check every 100ms
    );

    // Wait for multiple callbacks (longer duration for reliability)
    thread::sleep(Duration::from_millis(500));

    let count = callback_count.load(Ordering::Relaxed);
    assert!(
        count >= 1,
        "Should have at least one callback, got {}",
        count
    );

    monitor.stop();
}

#[rstest]
#[cfg(target_os = "windows")]
fn test_parent_monitor_drop_stops_monitoring() {
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = callback_count.clone();

    {
        let _monitor = ParentWindowMonitor::new(
            0,
            move || {
                callback_count_clone.fetch_add(1, Ordering::Relaxed);
            },
            100,
        );

        thread::sleep(Duration::from_millis(250));
    } // Monitor dropped here

    let count_before_drop = callback_count.load(Ordering::Relaxed);

    // Wait a bit more
    thread::sleep(Duration::from_millis(300));

    let count_after_drop = callback_count.load(Ordering::Relaxed);

    // Count should not increase significantly after drop
    assert!(
        count_after_drop <= count_before_drop + 1,
        "Callbacks should stop after drop"
    );
}
