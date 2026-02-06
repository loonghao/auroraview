//! Tests for auroraview-pack metrics module

use auroraview_pack::PackedMetrics;
use std::thread;
use std::time::Duration;

#[test]
fn test_metrics_basic() {
    let mut metrics = PackedMetrics::new();

    thread::sleep(Duration::from_millis(10));
    metrics.mark_overlay_read();

    thread::sleep(Duration::from_millis(5));
    metrics.mark_config_decompress();

    assert!(metrics.overlay_read.is_some());
    assert!(metrics.config_decompress.is_some());
    assert!(metrics.config_decompress.unwrap() > metrics.overlay_read.unwrap());
}

#[test]
fn test_time_phase() {
    let mut metrics = PackedMetrics::new();

    let result = metrics.time_phase("test_phase", || {
        thread::sleep(Duration::from_millis(5));
        42
    });

    assert_eq!(result, 42);
    // Phases are private, but we can check elapsed time
    assert!(metrics.elapsed() >= Duration::from_millis(5));
}

#[test]
fn test_report_format() {
    let mut metrics = PackedMetrics::new();
    metrics.mark_overlay_read();
    metrics.mark_config_decompress();

    let report = metrics.report();
    assert!(report.contains("Packed App Startup Performance"));
    assert!(report.contains("Overlay read"));
}
