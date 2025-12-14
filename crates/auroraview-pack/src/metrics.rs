//! Performance metrics for packed application startup
//!
//! This module provides detailed timing information for debugging
//! and optimizing the packed application startup process.

use std::time::{Duration, Instant};

/// Performance metrics for packed application startup
#[derive(Debug)]
pub struct PackedMetrics {
    /// When measurement started
    start: Instant,
    /// Overlay read completed
    pub overlay_read: Option<Duration>,
    /// Config decompression completed
    pub config_decompress: Option<Duration>,
    /// Assets decompression completed
    pub assets_decompress: Option<Duration>,
    /// Tar extraction completed
    pub tar_extract: Option<Duration>,
    /// Python runtime extraction completed
    pub python_runtime_extract: Option<Duration>,
    /// Python files extraction completed
    pub python_files_extract: Option<Duration>,
    /// Resources extraction completed
    pub resources_extract: Option<Duration>,
    /// Python process started
    pub python_start: Option<Duration>,
    /// Window created
    pub window_created: Option<Duration>,
    /// WebView created
    pub webview_created: Option<Duration>,
    /// Total startup completed
    pub total: Option<Duration>,
    /// Individual phase timings for detailed analysis
    phases: Vec<(String, Duration)>,
}

impl Default for PackedMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PackedMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            overlay_read: None,
            config_decompress: None,
            assets_decompress: None,
            tar_extract: None,
            python_runtime_extract: None,
            python_files_extract: None,
            resources_extract: None,
            python_start: None,
            window_created: None,
            webview_created: None,
            total: None,
            phases: Vec::new(),
        }
    }

    /// Mark overlay read completion
    pub fn mark_overlay_read(&mut self) {
        self.overlay_read = Some(self.start.elapsed());
    }

    /// Mark config decompression completion
    pub fn mark_config_decompress(&mut self) {
        self.config_decompress = Some(self.start.elapsed());
    }

    /// Mark assets decompression completion
    pub fn mark_assets_decompress(&mut self) {
        self.assets_decompress = Some(self.start.elapsed());
    }

    /// Mark tar extraction completion
    pub fn mark_tar_extract(&mut self) {
        self.tar_extract = Some(self.start.elapsed());
    }

    /// Mark Python runtime extraction completion
    pub fn mark_python_runtime_extract(&mut self) {
        self.python_runtime_extract = Some(self.start.elapsed());
    }

    /// Mark Python files extraction completion
    pub fn mark_python_files_extract(&mut self) {
        self.python_files_extract = Some(self.start.elapsed());
    }

    /// Mark resources extraction completion
    pub fn mark_resources_extract(&mut self) {
        self.resources_extract = Some(self.start.elapsed());
    }

    /// Mark Python process start
    pub fn mark_python_start(&mut self) {
        self.python_start = Some(self.start.elapsed());
    }

    /// Mark window creation
    pub fn mark_window_created(&mut self) {
        self.window_created = Some(self.start.elapsed());
    }

    /// Mark WebView creation
    pub fn mark_webview_created(&mut self) {
        self.webview_created = Some(self.start.elapsed());
    }

    /// Mark total completion
    pub fn mark_total(&mut self) {
        self.total = Some(self.start.elapsed());
    }

    /// Add a custom phase timing
    pub fn add_phase(&mut self, name: impl Into<String>, duration: Duration) {
        self.phases.push((name.into(), duration));
    }

    /// Time a closure and record it as a phase
    pub fn time_phase<F, R>(&mut self, name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let phase_start = Instant::now();
        let result = f();
        self.phases.push((name.into(), phase_start.elapsed()));
        result
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Format a duration for display
    fn format_duration(d: Duration) -> String {
        let ms = d.as_secs_f64() * 1000.0;
        if ms < 1.0 {
            format!("{:.2}µs", d.as_micros())
        } else if ms < 1000.0 {
            format!("{:.2}ms", ms)
        } else {
            format!("{:.2}s", d.as_secs_f64())
        }
    }

    /// Format delta between two durations
    fn format_delta(prev: Option<Duration>, curr: Option<Duration>) -> String {
        match (prev, curr) {
            (Some(p), Some(c)) if c > p => {
                let delta = c - p;
                format!("+{}", Self::format_duration(delta))
            }
            (None, Some(c)) => format!("+{}", Self::format_duration(c)),
            _ => String::new(),
        }
    }

    /// Generate a formatted performance report
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== Packed App Startup Performance ===".to_string());
        lines.push(format!(
            "Total elapsed: {}",
            Self::format_duration(self.elapsed())
        ));
        lines.push(String::new());

        // Main phases
        lines.push("--- Main Phases ---".to_string());

        let mut prev: Option<Duration> = None;

        if let Some(d) = self.overlay_read {
            lines.push(format!(
                "  Overlay read:        {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.config_decompress {
            lines.push(format!(
                "  Config decompress:   {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.assets_decompress {
            lines.push(format!(
                "  Assets decompress:   {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.tar_extract {
            lines.push(format!(
                "  Tar extract:         {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.python_runtime_extract {
            lines.push(format!(
                "  Python runtime:      {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.python_files_extract {
            lines.push(format!(
                "  Python files:        {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.resources_extract {
            lines.push(format!(
                "  Resources extract:   {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.python_start {
            lines.push(format!(
                "  Python start:        {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.window_created {
            lines.push(format!(
                "  Window created:      {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
            prev = Some(d);
        }

        if let Some(d) = self.webview_created {
            lines.push(format!(
                "  WebView created:     {:>10} ({})",
                Self::format_duration(d),
                Self::format_delta(prev, Some(d))
            ));
        }

        // Custom phases
        if !self.phases.is_empty() {
            lines.push(String::new());
            lines.push("--- Detailed Phases ---".to_string());
            for (name, duration) in &self.phases {
                lines.push(format!(
                    "  {:<20} {:>10}",
                    name,
                    Self::format_duration(*duration)
                ));
            }
        }

        lines.push("=".repeat(40));
        lines.join("\n")
    }

    /// Log the report using tracing
    pub fn log_report(&self) {
        for line in self.report().lines() {
            if line.is_empty() {
                continue;
            }
            tracing::info!("{}", line);
        }
    }
}

/// A scoped timer that records duration when dropped
#[allow(dead_code)]
pub struct ScopedTimer<'a> {
    metrics: &'a mut PackedMetrics,
    name: String,
    start: Instant,
}

#[allow(dead_code)]
impl<'a> ScopedTimer<'a> {
    /// Create a new scoped timer
    pub fn new(metrics: &'a mut PackedMetrics, name: impl Into<String>) -> Self {
        Self {
            metrics,
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        self.metrics
            .phases
            .push((self.name.clone(), self.start.elapsed()));
    }
}

