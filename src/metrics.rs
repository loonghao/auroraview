//! Timing metrics for WebView initialization and lifecycle
//!
//! This module re-exports the Metrics struct from auroraview-core and adds
//! thread-safe tracker utilities and a logging-based report function.

use parking_lot::Mutex;
use std::sync::Arc;

// Re-export core Metrics
pub use auroraview_core::metrics::Metrics;

/// Thread-safe metrics tracker
#[allow(dead_code)]
pub type Tracker = Arc<Mutex<Metrics>>;

/// Create a new metrics tracker
#[allow(dead_code)]
pub fn create() -> Tracker {
    Arc::new(Mutex::new(Metrics::new()))
}

/// Extension trait for Metrics to add logging-based report
pub trait MetricsReport {
    /// Print timing report using tracing
    fn report(&self);
}

impl MetricsReport for Metrics {
    fn report(&self) {
        tracing::info!("{}", self.format_report());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_reexport() {
        let metrics = Metrics::new();
        assert!(metrics.window_time().is_none());
        assert!(metrics.webview_time().is_none());
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = create();
        let metrics = tracker.lock();
        assert!(metrics.window_time().is_none());
    }

    #[test]
    fn test_metrics_report_trait() {
        let mut metrics = Metrics::new();
        metrics.mark_window();
        // Just verify it doesn't panic
        metrics.report();
    }
}
