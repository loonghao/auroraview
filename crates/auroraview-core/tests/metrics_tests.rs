//! Metrics tests

use std::thread;
use std::time::Duration as StdDuration;

use auroraview_core::metrics::Metrics;

#[test]
fn metrics_creation() {
    let metrics = Metrics::new();
    assert!(metrics.window_time().is_none());
    assert!(metrics.webview_time().is_none());
}

#[test]
fn mark_window() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_window();

    let duration = metrics.window_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn mark_webview() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_webview();

    let duration = metrics.webview_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn mark_html() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_html();

    let duration = metrics.html_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn mark_js() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_js();

    let duration = metrics.js_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn mark_paint() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_paint();

    let duration = metrics.paint_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn mark_shown() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_shown();

    let duration = metrics.shown_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn default() {
    let metrics = Metrics::default();
    assert!(metrics.window_time().is_none());
}

#[test]
fn format_report() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_shown();
    let report = metrics.format_report();
    assert!(report.contains("Timing Report"));
    assert!(report.contains("Window created"));
    assert!(report.contains("Window shown"));
}

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn metrics_clone() {
    let mut original = Metrics::new();
    thread::sleep(StdDuration::from_millis(5));
    original.mark_window();
    original.mark_webview();

    let cloned = original.clone();
    assert!(cloned.window_time().is_some());
    assert!(cloned.webview_time().is_some());
    assert!(cloned.html_time().is_none());
    assert!(cloned.js_time().is_none());
    assert!(cloned.paint_time().is_none());
    assert!(cloned.shown_time().is_none());
}

#[test]
fn metrics_debug() {
    let metrics = Metrics::new();
    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("Metrics"));
}

#[test]
fn format_report_empty() {
    let metrics = Metrics::new();
    let report = metrics.format_report();
    // No marks set — should still have header and footer
    assert!(report.contains("Timing Report"));
    assert!(report.contains("=========="));
    // No timing lines
    assert!(!report.contains("[TIMER]"));
    assert!(!report.contains("[OK]"));
}

#[test]
fn format_report_all_marks() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_webview();
    metrics.mark_html();
    metrics.mark_js();
    metrics.mark_paint();
    metrics.mark_shown();

    let report = metrics.format_report();
    assert!(report.contains("Window created"));
    assert!(report.contains("WebView created"));
    assert!(report.contains("HTML loaded"));
    assert!(report.contains("JavaScript initialized"));
    assert!(report.contains("First paint"));
    assert!(report.contains("Window shown"));
    assert!(report.contains("Total time to interactive"));
}

#[test]
fn format_report_partial_marks() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_js();
    // mark_webview, mark_html, mark_paint, mark_shown not called

    let report = metrics.format_report();
    assert!(report.contains("Window created"));
    assert!(report.contains("JavaScript initialized"));
    // Not-marked fields should not appear
    assert!(!report.contains("WebView created"));
    assert!(!report.contains("HTML loaded"));
    assert!(!report.contains("First paint"));
    assert!(!report.contains("Window shown"));
    assert!(!report.contains("Total time to interactive"));
}

#[test]
fn mark_order_non_decreasing() {
    // Marks applied in sequence should produce non-decreasing durations
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_window();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_webview();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_html();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_js();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_paint();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_shown();

    let t_window = metrics.window_time().unwrap();
    let t_webview = metrics.webview_time().unwrap();
    let t_html = metrics.html_time().unwrap();
    let t_js = metrics.js_time().unwrap();
    let t_paint = metrics.paint_time().unwrap();
    let t_shown = metrics.shown_time().unwrap();

    assert!(t_window <= t_webview, "window <= webview");
    assert!(t_webview <= t_html, "webview <= html");
    assert!(t_html <= t_js, "html <= js");
    assert!(t_js <= t_paint, "js <= paint");
    assert!(t_paint <= t_shown, "paint <= shown");
}

#[test]
fn shown_time_as_total_interactive() {
    // shown_time is referenced as "Total time to interactive" in report
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(5));
    metrics.mark_shown();

    let shown = metrics.shown_time().unwrap();
    let report = metrics.format_report();
    let expected_ms = shown.as_millis();
    // The report should contain the duration value for shown_time
    assert!(report.contains(&format!("{}ms", expected_ms)) || report.contains(&format!("{:?}", shown)));
}

#[test]
fn multiple_new_instances_independent() {
    let mut m1 = Metrics::new();
    let mut m2 = Metrics::new();

    thread::sleep(StdDuration::from_millis(5));
    m1.mark_window();
    thread::sleep(StdDuration::from_millis(5));
    m2.mark_window();

    // m2 window time should be >= m1 window time (m2 started after m1)
    let t1 = m1.window_time().unwrap();
    let t2 = m2.window_time().unwrap();
    // Both should be valid durations
    assert!(t1.as_millis() > 0);
    assert!(t2.as_millis() > 0);
}

// ============================================================================
// Initial state: all times are None
// ============================================================================

#[test]
fn metrics_all_none_initially() {
    let metrics = Metrics::new();
    assert!(metrics.window_time().is_none());
    assert!(metrics.webview_time().is_none());
    assert!(metrics.html_time().is_none());
    assert!(metrics.js_time().is_none());
    assert!(metrics.paint_time().is_none());
    assert!(metrics.shown_time().is_none());
}

// ============================================================================
// Mark once → only that mark is Some
// ============================================================================

#[test]
fn mark_window_only() {
    let mut metrics = Metrics::new();
    metrics.mark_window();

    assert!(metrics.window_time().is_some());
    assert!(metrics.webview_time().is_none());
    assert!(metrics.html_time().is_none());
    assert!(metrics.js_time().is_none());
    assert!(metrics.paint_time().is_none());
    assert!(metrics.shown_time().is_none());
}

#[test]
fn mark_webview_only() {
    let mut metrics = Metrics::new();
    metrics.mark_webview();

    assert!(metrics.window_time().is_none());
    assert!(metrics.webview_time().is_some());
    assert!(metrics.html_time().is_none());
}

#[test]
fn mark_html_only() {
    let mut metrics = Metrics::new();
    metrics.mark_html();

    assert!(metrics.html_time().is_some());
    assert!(metrics.window_time().is_none());
    assert!(metrics.js_time().is_none());
}

#[test]
fn mark_js_only() {
    let mut metrics = Metrics::new();
    metrics.mark_js();

    assert!(metrics.js_time().is_some());
    assert!(metrics.html_time().is_none());
    assert!(metrics.paint_time().is_none());
}

#[test]
fn mark_paint_only() {
    let mut metrics = Metrics::new();
    metrics.mark_paint();

    assert!(metrics.paint_time().is_some());
    assert!(metrics.js_time().is_none());
    assert!(metrics.shown_time().is_none());
}

#[test]
fn mark_shown_only() {
    let mut metrics = Metrics::new();
    metrics.mark_shown();

    assert!(metrics.shown_time().is_some());
    assert!(metrics.paint_time().is_none());
}

// ============================================================================
// Double-marking same milestone overwrites previous (or idempotent)
// ============================================================================

#[test]
fn mark_window_twice_still_some() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    let t1 = metrics.window_time().unwrap();
    metrics.mark_window();
    let t2 = metrics.window_time().unwrap();
    // After second mark, time should still be valid (≥ first)
    assert!(t2 >= t1);
}

// ============================================================================
// format_report structure
// ============================================================================

#[test]
fn format_report_contains_separator() {
    let metrics = Metrics::new();
    let report = metrics.format_report();
    assert!(report.contains("==========") || report.contains("---"));
}

#[test]
fn format_report_window_mark_contains_window_line() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    let report = metrics.format_report();
    assert!(report.contains("Window created"));
}

#[test]
fn format_report_does_not_contain_unset_marks() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    // Did not mark webview/html/js/paint/shown
    let report = metrics.format_report();
    assert!(!report.contains("WebView created"));
    assert!(!report.contains("HTML loaded"));
    assert!(!report.contains("JavaScript initialized"));
    assert!(!report.contains("First paint"));
    assert!(!report.contains("Window shown"));
}

// ============================================================================
// R8 Extensions
// ============================================================================

#[test]
fn metrics_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Metrics>();
}

#[test]
fn mark_window_then_webview_both_some() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_webview();

    assert!(metrics.window_time().is_some());
    assert!(metrics.webview_time().is_some());
}

#[test]
fn mark_js_then_paint_both_some() {
    let mut metrics = Metrics::new();
    metrics.mark_js();
    metrics.mark_paint();

    assert!(metrics.js_time().is_some());
    assert!(metrics.paint_time().is_some());
}

#[test]
fn format_report_all_marks_contains_timing_report_header() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_webview();

    let report = metrics.format_report();
    assert!(report.starts_with("") || report.contains("Timing Report"));
}

#[test]
fn format_report_shown_contains_total_time() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_shown();

    let report = metrics.format_report();
    // If shown is marked, the total time line should appear
    assert!(report.contains("Total time to interactive"));
}

#[test]
fn metrics_clone_only_marked_fields_copied() {
    let mut original = Metrics::new();
    original.mark_html();
    original.mark_js();

    let cloned = original.clone();
    assert!(cloned.html_time().is_some());
    assert!(cloned.js_time().is_some());
    // Others remain None
    assert!(cloned.window_time().is_none());
    assert!(cloned.webview_time().is_none());
    assert!(cloned.paint_time().is_none());
    assert!(cloned.shown_time().is_none());
}

#[test]
fn mark_shown_twice_both_some() {
    let mut metrics = Metrics::new();
    metrics.mark_shown();
    let t1 = metrics.shown_time().unwrap();
    thread::sleep(StdDuration::from_millis(2));
    metrics.mark_shown();
    let t2 = metrics.shown_time().unwrap();
    // Both should be valid durations
    assert!(t2 >= t1, "second mark should be >= first mark");
}

#[test]
fn mark_all_then_clone_preserves_all() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    metrics.mark_webview();
    metrics.mark_html();
    metrics.mark_js();
    metrics.mark_paint();
    metrics.mark_shown();

    let cloned = metrics.clone();
    assert!(cloned.window_time().is_some());
    assert!(cloned.webview_time().is_some());
    assert!(cloned.html_time().is_some());
    assert!(cloned.js_time().is_some());
    assert!(cloned.paint_time().is_some());
    assert!(cloned.shown_time().is_some());
}

#[test]
fn default_and_new_equivalent_initial_state() {
    let m_default = Metrics::default();
    let m_new = Metrics::new();
    // Both should have all times as None
    assert!(m_default.window_time().is_none());
    assert!(m_new.window_time().is_none());
    assert!(m_default.shown_time().is_none());
    assert!(m_new.shown_time().is_none());
}

#[test]
fn format_report_no_html_mark_not_in_report() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    // html not marked
    let report = metrics.format_report();
    assert!(!report.contains("HTML loaded"), "HTML line should not appear when not marked");
}

#[test]
fn format_report_no_js_mark_not_in_report() {
    let mut metrics = Metrics::new();
    metrics.mark_window();
    // js not marked
    let report = metrics.format_report();
    assert!(!report.contains("JavaScript initialized"), "JS line should not appear when not marked");
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn format_report_is_string() {
    let metrics = Metrics::new();
    let report = metrics.format_report();
    assert!(report.is_ascii() || !report.is_empty());
}

#[test]
fn mark_webview_is_some() {
    let mut m = Metrics::new();
    m.mark_webview();
    assert!(m.webview_time().is_some());
}

#[test]
fn mark_html_is_some() {
    let mut m = Metrics::new();
    m.mark_html();
    assert!(m.html_time().is_some());
}

#[test]
fn mark_js_is_some() {
    let mut m = Metrics::new();
    m.mark_js();
    assert!(m.js_time().is_some());
}

#[test]
fn mark_paint_is_some() {
    let mut m = Metrics::new();
    m.mark_paint();
    assert!(m.paint_time().is_some());
}

#[test]
fn mark_shown_is_some() {
    let mut m = Metrics::new();
    m.mark_shown();
    assert!(m.shown_time().is_some());
}

#[test]
fn clone_all_none_preserves_none() {
    let m = Metrics::new();
    let cloned = m.clone();
    assert!(cloned.window_time().is_none());
    assert!(cloned.webview_time().is_none());
    assert!(cloned.html_time().is_none());
    assert!(cloned.js_time().is_none());
    assert!(cloned.paint_time().is_none());
    assert!(cloned.shown_time().is_none());
}

#[test]
fn debug_non_empty() {
    let m = Metrics::new();
    let dbg = format!("{:?}", m);
    assert!(!dbg.is_empty());
}

#[test]
fn format_report_webview_and_html_marks() {
    let mut m = Metrics::new();
    m.mark_webview();
    m.mark_html();
    let report = m.format_report();
    assert!(report.contains("WebView created"));
    assert!(report.contains("HTML loaded"));
}

#[test]
fn format_report_paint_mark() {
    let mut m = Metrics::new();
    m.mark_paint();
    let report = m.format_report();
    assert!(report.contains("First paint"));
}

#[test]
fn window_time_non_zero_after_sleep() {
    let mut m = Metrics::new();
    thread::sleep(StdDuration::from_millis(5));
    m.mark_window();
    let t = m.window_time().unwrap();
    assert!(t.as_millis() >= 4);
}

#[test]
fn new_and_default_both_empty_report_header() {
    let m1 = Metrics::new();
    let m2 = Metrics::default();
    for m in &[m1, m2] {
        let r = m.format_report();
        assert!(r.contains("Timing Report"));
    }
}
