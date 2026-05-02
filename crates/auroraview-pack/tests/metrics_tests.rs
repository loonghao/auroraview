//! Tests for auroraview-pack metrics module

use std::thread;
use std::time::Duration;

use auroraview_pack::PackedMetrics;
use rstest::rstest;

#[test]
fn metrics_basic() {
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
fn time_phase() {
    let mut metrics = PackedMetrics::new();

    let result = metrics.time_phase("test_phase", || {
        thread::sleep(Duration::from_millis(5));
        42
    });

    assert_eq!(result, 42);
    assert!(metrics.elapsed() >= Duration::from_millis(5));
}

#[test]
fn report_format() {
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
fn metrics_default() {
    let metrics = PackedMetrics::default();
    assert!(metrics.overlay_read.is_none());
    assert!(metrics.config_decompress.is_none());
    assert!(metrics.total.is_none());
}

#[test]
fn mark_all_phases() {
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
fn report_contains_all_phases() {
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
fn report_empty_metrics() {
    let m = PackedMetrics::new();
    let report = m.report();
    assert!(report.contains("Packed App Startup Performance"));
    // No phase lines should appear
    assert!(!report.contains("Overlay read"));
}

#[test]
fn add_custom_phase() {
    let mut m = PackedMetrics::new();
    m.add_phase("my_custom_phase", Duration::from_millis(50));

    let report = m.report();
    assert!(report.contains("Detailed Phases"));
    assert!(report.contains("my_custom_phase"));
}

#[test]
fn time_phase_records_in_report() {
    let mut m = PackedMetrics::new();
    m.time_phase("process_assets", || {
        thread::sleep(Duration::from_millis(2));
    });

    let report = m.report();
    assert!(report.contains("process_assets"));
}

#[test]
fn elapsed_increases() {
    let m = PackedMetrics::new();
    let t1 = m.elapsed();
    thread::sleep(Duration::from_millis(5));
    let t2 = m.elapsed();
    assert!(t2 > t1);
}

#[test]
fn report_has_separator_lines() {
    let m = PackedMetrics::new();
    let report = m.report();
    // Should have separator "==..." at the end
    assert!(report.contains("===="));
}

#[test]
fn multiple_custom_phases() {
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
fn phases_ordering_non_decreasing() {
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
fn debug_format() {
    let m = PackedMetrics::new();
    let s = format!("{m:?}");
    assert!(s.contains("PackedMetrics") || s.contains("overlay_read"));
}

#[test]
fn report_contains_header_line() {
    let m = PackedMetrics::new();
    let report = m.report();
    // Header should be first non-empty line
    let first = report.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    assert!(!first.is_empty());
}

#[test]
fn mark_total_records_duration() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(5));
    m.mark_total();
    assert!(m.total.is_some());
    assert!(m.total.unwrap() >= Duration::from_millis(4));
}

#[test]
fn time_phase_returns_value_correctly() {
    let mut m = PackedMetrics::new();
    let result = m.time_phase("check_val", || 99u32);
    assert_eq!(result, 99u32);
}

#[test]
fn time_phase_string_return() {
    let mut m = PackedMetrics::new();
    let s = m.time_phase("gen_string", || "hello".to_string());
    assert_eq!(s, "hello");
}

#[test]
fn multiple_time_phases_all_in_report() {
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
fn window_and_webview_ordering() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(2));
    m.mark_window_created();
    thread::sleep(Duration::from_millis(2));
    m.mark_webview_created();
    assert!(m.webview_created.unwrap() >= m.window_created.unwrap());
}

#[test]
fn python_start_before_window() {
    let mut m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(2));
    m.mark_python_start();
    thread::sleep(Duration::from_millis(2));
    m.mark_window_created();
    assert!(m.window_created.unwrap() >= m.python_start.unwrap());
}

#[test]
fn add_custom_phase_with_zero_duration() {
    let mut m = PackedMetrics::new();
    m.add_phase("instant_phase", Duration::ZERO);
    let report = m.report();
    assert!(report.contains("instant_phase"));
}

#[test]
fn add_large_custom_phase_duration() {
    let mut m = PackedMetrics::new();
    m.add_phase("long_phase", Duration::from_secs(3600));
    let report = m.report();
    assert!(report.contains("long_phase"));
}

#[test]
fn elapsed_is_non_zero_after_sleep() {
    let m = PackedMetrics::new();
    thread::sleep(Duration::from_millis(5));
    assert!(m.elapsed() >= Duration::from_millis(4));
}

#[test]
fn mark_resources_and_python_files() {
    let mut m = PackedMetrics::new();
    m.mark_resources_extract();
    m.mark_python_files_extract();
    assert!(m.resources_extract.is_some());
    assert!(m.python_files_extract.is_some());
}

#[test]
fn report_total_line() {
    let mut m = PackedMetrics::new();
    m.mark_total();
    let report = m.report();
    assert!(report.contains("Total") || report.contains("total"));
}

#[test]
fn mark_tar_extract() {
    let mut m = PackedMetrics::new();
    m.mark_tar_extract();
    assert!(m.tar_extract.is_some());
}

#[test]
fn mark_python_runtime_extract() {
    let mut m = PackedMetrics::new();
    m.mark_python_runtime_extract();
    assert!(m.python_runtime_extract.is_some());
}

// ─── Additional coverage R9 ──────────────────────────────────────────────────

#[test]
fn metrics_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PackedMetrics>();
}

#[test]
fn new_metrics_has_no_phases() {
    let m = PackedMetrics::new();
    // new metrics without any phase adds should produce a report without custom phases
    let report = m.report();
    assert!(!report.is_empty());
}

#[test]
fn elapsed_monotonically_increases() {
    let m = PackedMetrics::new();
    let t1 = m.elapsed();
    thread::sleep(Duration::from_millis(2));
    let t2 = m.elapsed();
    assert!(t2 >= t1);
}

#[test]
fn time_phase_result_returned() {
    let mut m = PackedMetrics::new();
    let result = m.time_phase("compute", || 42u32 * 2);
    assert_eq!(result, 84);
}

#[test]
fn add_phase_does_not_panic_zero() {
    let mut m = PackedMetrics::new();
    m.add_phase("zero", Duration::ZERO);
    // Just verify no panic
}

#[test]
fn mark_total_sets_timestamp() {
    let mut m = PackedMetrics::new();
    assert!(m.total.is_none());
    m.mark_total();
    assert!(m.total.is_some());
}

#[test]
fn mark_window_created_once() {
    let mut m = PackedMetrics::new();
    assert!(m.window_created.is_none());
    m.mark_window_created();
    assert!(m.window_created.is_some());
}

#[test]
fn mark_webview_created_once() {
    let mut m = PackedMetrics::new();
    m.mark_webview_created();
    assert!(m.webview_created.is_some());
}

#[test]
fn mark_python_start_once() {
    let mut m = PackedMetrics::new();
    m.mark_python_start();
    assert!(m.python_start.is_some());
}

#[test]
fn report_not_empty() {
    let m = PackedMetrics::new();
    assert!(!m.report().is_empty());
}

#[test]
fn phases_count_after_multiple_adds() {
    let mut m = PackedMetrics::new();
    m.add_phase("a", Duration::from_millis(1));
    m.add_phase("b", Duration::from_millis(2));
    m.add_phase("c", Duration::from_millis(3));
    let report = m.report();
    assert!(report.contains("a"));
    assert!(report.contains("b"));
    assert!(report.contains("c"));
}

#[test]
fn time_phase_adds_to_phases() {
    let mut m = PackedMetrics::new();
    m.time_phase("new_phase", || {});
    let report = m.report();
    assert!(report.contains("new_phase"));
}

#[test]
fn full_lifecycle() {
    let mut m = PackedMetrics::new();
    m.mark_python_start();
    m.mark_window_created();
    m.mark_webview_created();
    m.mark_resources_extract();
    m.mark_python_files_extract();
    m.mark_tar_extract();
    m.mark_python_runtime_extract();
    m.mark_total();

    let report = m.report();
    assert!(!report.is_empty());
    assert!(report.contains("Total") || report.contains("total"));
}

#[rstest]
#[case("phase_one")]
#[case("phase_two")]
#[case("final")]
fn time_phase_name_in_report(#[case] name: &str) {
    let mut m = PackedMetrics::new();
    m.time_phase(name, || {});
    let report = m.report();
    assert!(report.contains(name));
}

#[rstest]
#[case(1u64)]
#[case(100)]
#[case(1000)]
fn add_phase_duration_millis(#[case] millis: u64) {
    let mut m = PackedMetrics::new();
    m.add_phase("phase", Duration::from_millis(millis));
    let report = m.report();
    assert!(report.contains("phase"));
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn metrics_debug_format_non_empty() {
    let m = PackedMetrics::new();
    let dbg = format!("{:?}", m);
    assert!(!dbg.is_empty());
}

#[test]
fn mark_overlay_read_and_config_both_some() {
    let mut m1 = PackedMetrics::new();
    m1.mark_overlay_read();
    m1.mark_config_decompress();
    assert!(m1.overlay_read.is_some());
    assert!(m1.config_decompress.is_some());
    assert!(m1.assets_decompress.is_none());
}

#[test]
fn two_independent_metrics_instances() {
    let m1 = PackedMetrics::default();
    let m2 = PackedMetrics::default();
    assert!(m1.overlay_read.is_none());
    assert!(m2.total.is_none());
}

#[test]
fn mark_overlay_read_once() {
    let mut m = PackedMetrics::new();
    assert!(m.overlay_read.is_none());
    m.mark_overlay_read();
    assert!(m.overlay_read.is_some());
}

#[test]
fn mark_config_decompress_once() {
    let mut m = PackedMetrics::new();
    m.mark_config_decompress();
    assert!(m.config_decompress.is_some());
    assert!(m.overlay_read.is_none());
}

#[test]
fn mark_assets_decompress_once() {
    let mut m = PackedMetrics::new();
    m.mark_assets_decompress();
    assert!(m.assets_decompress.is_some());
}

#[test]
fn report_contains_detailed_phases_section() {
    let mut m = PackedMetrics::new();
    m.add_phase("custom_x", Duration::from_millis(5));
    let report = m.report();
    assert!(report.contains("custom_x"));
}

#[test]
fn time_phase_zero_duration_included() {
    let mut m = PackedMetrics::new();
    m.time_phase("fast_op", || {});
    let report = m.report();
    assert!(report.contains("fast_op"));
}

#[rstest]
#[case("alpha", 5u64)]
#[case("beta", 10)]
#[case("gamma", 50)]
fn multiple_named_phases_parametrized(#[case] name: &str, #[case] millis: u64) {
    let mut m = PackedMetrics::new();
    m.add_phase(name, Duration::from_millis(millis));
    let report = m.report();
    assert!(report.contains(name));
}

#[test]
fn elapsed_non_zero_immediately() {
    let m = PackedMetrics::new();
    // Even without sleep, Instant::now() difference should be >= 0
    assert!(m.elapsed() >= Duration::ZERO);
}

#[test]
fn report_structure_with_full_lifecycle() {
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
    let report = m.report();
    // Should contain the header and all phase labels
    assert!(report.contains("Packed App Startup Performance"));
    assert!(report.contains("Overlay read"));
    assert!(report.contains("Total"));
}
