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

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn test_metrics_default() {
    let metrics = PackedMetrics::default();
    assert!(metrics.overlay_read.is_none());
    assert!(metrics.config_decompress.is_none());
    assert!(metrics.total.is_none());
}

#[test]
fn test_mark_all_phases() {
    let mut m = PackedMetrics::new();
    m.mark_overlay_read();
    m.mark_config_decompress();
    m.mark_assets_decompress();
    m.mark_tar_extract();
    m.mark_python_runtime_extract();
    m.mark_python_files_extract();
    m.mark_resources_extract();
    m.mark_python_start();
    m.mark_window_created();
    m.mark_webview_created();
    m.mark_total();

    assert!(m.overlay_read.is_some());
    assert!(m.config_decompress.is_some());
    assert!(m.assets_decompress.is_some());
    assert!(m.tar_extract.is_some());
    assert!(m.python_runtime_extract.is_some());
    assert!(m.python_files_extract.is_some());
    assert!(m.resources_extract.is_some());
    assert!(m.python_start.is_some());
    assert!(m.window_created.is_some());
    assert!(m.webview_created.is_some());
    assert!(m.total.is_some());
}

#[test]
fn test_report_contains_all_phases() {
    let mut m = PackedMetrics::new();
    m.mark_overlay_read();
    m.mark_config_decompress();
    m.mark_assets_decompress();
    m.mark_tar_extract();
    m.mark_python_runtime_extract();
    m.mark_python_files_extract();
    m.mark_resources_extract();
    m.mark_python_start();
    m.mark_window_created();
    m.mark_webview_created();

    let report = m.report();
    assert!(report.contains("Overlay read"));
    assert!(report.contains("Config decompress"));
    assert!(report.contains("Assets decompress"));
    assert!(report.contains("Tar extract"));
    assert!(report.contains("Python runtime"));
    assert!(report.contains("Python files"));
    assert!(report.contains("Resources extract"));
    assert!(report.contains("Python start"));
    assert!(report.contains("Window created"));
    assert!(report.contains("WebView created"));
}

#[test]
fn test_report_empty_metrics() {
    let m = PackedMetrics::new();
    let report = m.report();
    assert!(report.contains("Packed App Startup Performance"));
    // No phase lines should appear
    assert!(!report.contains("Overlay read"));
}

#[test]
fn test_add_custom_phase() {
    let mut m = PackedMetrics::new();
    m.add_phase("my_custom_phase", Duration::from_millis(50));

    let report = m.report();
    assert!(report.contains("Detailed Phases"));
    assert!(report.contains("my_custom_phase"));
}

#[test]
fn test_time_phase_records_in_report() {
    let mut m = PackedMetrics::new();
    m.time_phase("process_assets", || {
        thread::sleep(Duration::from_millis(2));
    });

    let report = m.report();
    assert!(report.contains("process_assets"));
}

#[test]
fn test_elapsed_increases() {
    let m = PackedMetrics::new();
    let t1 = m.elapsed();
    thread::sleep(Duration::from_millis(5));
    let t2 = m.elapsed();
    assert!(t2 > t1);
}

#[test]
fn test_report_has_separator_lines() {
    let m = PackedMetrics::new();
    let report = m.report();
    // Should have separator "==..." at the end
    assert!(report.contains("===="));
}

#[test]
fn test_multiple_custom_phases() {
    let mut m = PackedMetrics::new();
    m.add_phase("phase_a", Duration::from_millis(10));
    m.add_phase("phase_b", Duration::from_millis(20));
    m.add_phase("phase_c", Duration::from_millis(30));

    let report = m.report();
    assert!(report.contains("phase_a"));
    assert!(report.contains("phase_b"));
    assert!(report.contains("phase_c"));
}

#[test]
fn test_phases_ordering_non_decreasing() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(2));
    m.mark_overlay_read();
    thread::sleep(Duration::from_millis(2));
    m.mark_config_decompress();
    thread::sleep(Duration::from_millis(2));
    m.mark_assets_decompress();

    assert!(m.config_decompress.unwrap() >= m.overlay_read.unwrap());
    assert!(m.assets_decompress.unwrap() >= m.config_decompress.unwrap());
}
