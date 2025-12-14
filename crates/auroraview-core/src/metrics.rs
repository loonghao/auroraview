//! Timing metrics for WebView initialization and lifecycle
//!
//! This module provides timing metrics for tracking WebView initialization
//! and lifecycle events. It can be used by both the main library and CLI tools.

use std::time::{Duration, Instant};

/// Timing metrics for WebView initialization
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Time when WebView creation started
    pub creation_start: Instant,

    /// Time when window was created
    pub window_created: Option<Instant>,

    /// Time when WebView was created
    pub webview_created: Option<Instant>,

    /// Time when HTML was loaded
    pub html_loaded: Option<Instant>,

    /// Time when JavaScript initialized
    pub js_initialized: Option<Instant>,

    /// Time when first paint occurred
    pub first_paint: Option<Instant>,

    /// Time when window was shown
    pub window_shown: Option<Instant>,
}

impl Metrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            creation_start: Instant::now(),
            window_created: None,
            webview_created: None,
            html_loaded: None,
            js_initialized: None,
            first_paint: None,
            window_shown: None,
        }
    }

    /// Mark window as created
    pub fn mark_window(&mut self) {
        self.window_created = Some(Instant::now());
    }

    /// Mark WebView as created
    pub fn mark_webview(&mut self) {
        self.webview_created = Some(Instant::now());
    }

    /// Mark HTML as loaded
    pub fn mark_html(&mut self) {
        self.html_loaded = Some(Instant::now());
    }

    /// Mark JavaScript as initialized
    pub fn mark_js(&mut self) {
        self.js_initialized = Some(Instant::now());
    }

    /// Mark first paint
    pub fn mark_paint(&mut self) {
        self.first_paint = Some(Instant::now());
    }

    /// Mark window as shown
    pub fn mark_shown(&mut self) {
        self.window_shown = Some(Instant::now());
    }

    /// Get time to window creation
    pub fn window_time(&self) -> Option<Duration> {
        self.window_created
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get time to WebView creation
    pub fn webview_time(&self) -> Option<Duration> {
        self.webview_created
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get time to HTML load
    pub fn html_time(&self) -> Option<Duration> {
        self.html_loaded
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get time to JavaScript initialization
    pub fn js_time(&self) -> Option<Duration> {
        self.js_initialized
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get time to first paint
    pub fn paint_time(&self) -> Option<Duration> {
        self.first_paint
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get time to window shown
    pub fn shown_time(&self) -> Option<Duration> {
        self.window_shown
            .map(|t| t.duration_since(self.creation_start))
    }

    /// Get timing report as formatted string
    pub fn format_report(&self) -> String {
        let mut report = String::from("=== Timing Report ===\n");

        if let Some(d) = self.window_time() {
            report.push_str(&format!("[TIMER] Window created: {:?}\n", d));
        }

        if let Some(d) = self.webview_time() {
            report.push_str(&format!("[TIMER] WebView created: {:?}\n", d));
        }

        if let Some(d) = self.html_time() {
            report.push_str(&format!("[TIMER] HTML loaded: {:?}\n", d));
        }

        if let Some(d) = self.js_time() {
            report.push_str(&format!("[TIMER] JavaScript initialized: {:?}\n", d));
        }

        if let Some(d) = self.paint_time() {
            report.push_str(&format!("[TIMER] First paint: {:?}\n", d));
        }

        if let Some(d) = self.shown_time() {
            report.push_str(&format!("[TIMER] Window shown: {:?}\n", d));
            report.push_str(&format!("[OK] Total time to interactive: {:?}\n", d));
        }

        report.push_str("====================");
        report
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
