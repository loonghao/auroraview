//! Tests for auroraview-pack metrics module

use std::thread;
use std::time::Duration;

use auroraview_pack::PackedMetrics;

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

// ============================================================================
// Extended coverage tests
// ============================================================================

#[test]
fn test_debug_format() {
    let m = PackedMetrics::new();
    let s = format!("{m:?}");
    assert!(s.contains("PackedMetrics") || s.contains("overlay_read"));
}

#[test]
fn test_report_contains_header_line() {
    let m = PackedMetrics::new();
    let report = m.report();
    // Header should be first non-empty line
    let first = report.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    assert!(!first.is_empty());
}

#[test]
fn test_mark_total_records_duration() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(5));
    m.mark_total();
    assert!(m.total.is_some());
    assert!(m.total.unwrap() >= Duration::from_millis(4));
}

#[test]
fn test_time_phase_returns_value_correctly() {
    let mut m = PackedMetrics::new();
    let result = m.time_phase("check_val", || 99u32);
    assert_eq!(result, 99u32);
}

#[test]
fn test_time_phase_string_return() {
    let mut m = PackedMetrics::new();
    let s = m.time_phase("gen_string", || "hello".to_string());
    assert_eq!(s, "hello");
}

#[test]
fn test_multiple_time_phases_all_in_report() {
    let mut m = PackedMetrics::new();
    m.time_phase("alpha", || thread::sleep(Duration::from_millis(1)));
    m.time_phase("beta", || thread::sleep(Duration::from_millis(1)));
    m.time_phase("gamma", || thread::sleep(Duration::from_millis(1)));

    let report = m.report();
    assert!(report.contains("alpha"));
    assert!(report.contains("beta"));
    assert!(report.contains("gamma"));
}

#[test]
fn test_window_and_webview_ordering() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(2));
    m.mark_window_created();
    thread::sleep(Duration::from_millis(2));
    m.mark_webview_created();
    assert!(m.webview_created.unwrap() >= m.window_created.unwrap());
}

#[test]
fn test_python_start_before_window() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(2));
    m.mark_python_start();
    thread::sleep(Duration::from_millis(2));
    m.mark_window_created();
    assert!(m.window_created.unwrap() >= m.python_start.unwrap());
}

#[test]
fn test_add_custom_phase_with_zero_duration() {
    let mut m = PackedMetrics::new();
    m.add_phase("instant_phase", Duration::ZERO);
    let report = m.report();
    assert!(report.contains("instant_phase"));
}

#[test]
fn test_add_large_custom_phase_duration() {
    let mut m = PackedMetrics::new();
    m.add_phase("long_phase", Duration::from_secs(3600));
    let report = m.report();
    assert!(report.contains("long_phase"));
}

#[test]
fn test_elapsed_is_non_zero_after_sleep() {
    let m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(5));
    assert!(m.elapsed() >= Duration::from_millis(4));
}

#[test]
fn test_mark_resources_and_python_files() {
    let mut m = PackedMetrics::new();
    m.mark_resources_extract();
    m.mark_python_files_extract();
    assert!(m.resources_extract.is_some());
    assert!(m.python_files_extract.is_some());
}

#[test]
fn test_report_total_line() {
    let mut m = PackedMetrics::new();
    m.mark_total();
    let report = m.report();
    assert!(report.contains("Total") || report.contains("total"));
}

#[test]
fn test_mark_tar_extract() {
    let mut m = PackedMetrics::new();
    m.mark_tar_extract();
    assert!(m.tar_extract.is_some());
}

#[test]
fn test_mark_python_runtime_extract() {
    let mut m = PackedMetrics::new();
    m.mark_python_runtime_extract();
    assert!(m.python_runtime_extract.is_some());
}
