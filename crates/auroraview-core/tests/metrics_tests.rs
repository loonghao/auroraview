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
