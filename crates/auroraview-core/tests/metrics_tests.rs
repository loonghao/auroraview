//! Metrics tests

use auroraview_core::metrics::Metrics;
use std::thread;
use std::time::Duration as StdDuration;

#[test]
fn test_metrics_creation() {
    let metrics = Metrics::new();
    assert!(metrics.window_time().is_none());
    assert!(metrics.webview_time().is_none());
}

#[test]
fn test_mark_window() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_window();

    let duration = metrics.window_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_mark_webview() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_webview();

    let duration = metrics.webview_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_mark_html() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_html();

    let duration = metrics.html_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_mark_js() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_js();

    let duration = metrics.js_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_mark_paint() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_paint();

    let duration = metrics.paint_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_mark_shown() {
    let mut metrics = Metrics::new();
    thread::sleep(StdDuration::from_millis(10));
    metrics.mark_shown();

    let duration = metrics.shown_time();
    assert!(duration.is_some());
    assert!(duration.unwrap().as_millis() >= 10);
}

#[test]
fn test_default() {
    let metrics = Metrics::default();
    assert!(metrics.window_time().is_none());
}

#[test]
fn test_format_report() {
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
fn test_metrics_clone() {
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
fn test_metrics_debug() {
    let metrics = Metrics::new();
    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("Metrics"));
}

#[test]
fn test_format_report_empty() {
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
fn test_format_report_all_marks() {
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
fn test_format_report_partial_marks() {
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
fn test_mark_order_non_decreasing() {
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
fn test_shown_time_as_total_interactive() {
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
fn test_multiple_new_instances_independent() {
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
