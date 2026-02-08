//! WebView metrics collection.
//!
//! Pre-defined instruments for monitoring WebView performance and health.

use opentelemetry::global;
use opentelemetry::KeyValue;

/// Pre-defined WebView metrics.
pub struct WebViewMetrics {
    webview_count: opentelemetry::metrics::UpDownCounter<i64>,
    load_time: opentelemetry::metrics::Histogram<f64>,
    ipc_latency: opentelemetry::metrics::Histogram<f64>,
    ipc_message_count: opentelemetry::metrics::Counter<u64>,
    js_eval_count: opentelemetry::metrics::Counter<u64>,
    js_eval_duration: opentelemetry::metrics::Histogram<f64>,
    error_count: opentelemetry::metrics::Counter<u64>,
    navigation_count: opentelemetry::metrics::Counter<u64>,
    event_emit_count: opentelemetry::metrics::Counter<u64>,
    memory_bytes: opentelemetry::metrics::Gauge<u64>,
}

impl WebViewMetrics {
    /// Create a new metrics instance from the global meter provider.
    pub fn new() -> Self {
        let meter = global::meter("auroraview");

        Self {
            webview_count: meter
                .i64_up_down_counter("auroraview.webview.count")
                .with_description("Number of active WebView instances")
                .build(),
            load_time: meter
                .f64_histogram("auroraview.webview.load_time")
                .with_description("WebView page load time in milliseconds")
                .with_unit("ms")
                .build(),
            ipc_latency: meter
                .f64_histogram("auroraview.ipc.latency")
                .with_description("IPC round-trip latency in milliseconds")
                .with_unit("ms")
                .build(),
            ipc_message_count: meter
                .u64_counter("auroraview.ipc.messages")
                .with_description("Total IPC messages sent/received")
                .build(),
            js_eval_count: meter
                .u64_counter("auroraview.js.eval_count")
                .with_description("Total JavaScript evaluations")
                .build(),
            js_eval_duration: meter
                .f64_histogram("auroraview.js.eval_duration")
                .with_description("JavaScript evaluation duration in milliseconds")
                .with_unit("ms")
                .build(),
            error_count: meter
                .u64_counter("auroraview.errors")
                .with_description("Total errors encountered")
                .build(),
            navigation_count: meter
                .u64_counter("auroraview.navigation.count")
                .with_description("Total navigation events")
                .build(),
            event_emit_count: meter
                .u64_counter("auroraview.events.emitted")
                .with_description("Total events emitted from Python to JS")
                .build(),
            memory_bytes: meter
                .u64_gauge("auroraview.memory.bytes")
                .with_description("Estimated memory usage in bytes")
                .with_unit("By")
                .build(),
        }
    }

    /// Record a WebView instance creation.
    pub fn webview_created(&self, webview_id: &str) {
        self.webview_count
            .add(1, &[KeyValue::new("webview_id", webview_id.to_string())]);
    }

    /// Record a WebView instance destruction.
    pub fn webview_destroyed(&self, webview_id: &str) {
        self.webview_count
            .add(-1, &[KeyValue::new("webview_id", webview_id.to_string())]);
    }

    /// Record page load time for a WebView.
    pub fn record_load_time(&self, webview_id: &str, duration_ms: f64) {
        self.load_time.record(
            duration_ms,
            &[KeyValue::new("webview_id", webview_id.to_string())],
        );
    }

    /// Record IPC message latency.
    pub fn record_ipc_latency(&self, webview_id: &str, direction: &str, latency_ms: f64) {
        self.ipc_latency.record(
            latency_ms,
            &[
                KeyValue::new("webview_id", webview_id.to_string()),
                KeyValue::new("direction", direction.to_string()),
            ],
        );
    }

    /// Increment IPC message counter.
    pub fn record_ipc_message(&self, webview_id: &str, direction: &str) {
        self.ipc_message_count.add(
            1,
            &[
                KeyValue::new("webview_id", webview_id.to_string()),
                KeyValue::new("direction", direction.to_string()),
            ],
        );
    }

    /// Record a JavaScript evaluation.
    pub fn record_js_eval(&self, webview_id: &str, duration_ms: f64) {
        self.js_eval_count
            .add(1, &[KeyValue::new("webview_id", webview_id.to_string())]);
        self.js_eval_duration.record(
            duration_ms,
            &[KeyValue::new("webview_id", webview_id.to_string())],
        );
    }

    /// Record an error.
    pub fn record_error(&self, webview_id: &str, error_type: &str) {
        self.error_count.add(
            1,
            &[
                KeyValue::new("webview_id", webview_id.to_string()),
                KeyValue::new("error_type", error_type.to_string()),
            ],
        );
    }

    /// Record a navigation event.
    pub fn record_navigation(&self, webview_id: &str, url: &str) {
        self.navigation_count.add(
            1,
            &[
                KeyValue::new("webview_id", webview_id.to_string()),
                KeyValue::new("url", url.to_string()),
            ],
        );
    }

    /// Record an event emission (Python -> JS).
    pub fn record_event_emit(&self, webview_id: &str, event_name: &str) {
        self.event_emit_count.add(
            1,
            &[
                KeyValue::new("webview_id", webview_id.to_string()),
                KeyValue::new("event", event_name.to_string()),
            ],
        );
    }

    /// Record memory usage.
    pub fn record_memory(&self, webview_id: &str, bytes: u64) {
        self.memory_bytes.record(
            bytes,
            &[KeyValue::new("webview_id", webview_id.to_string())],
        );
    }
}

impl Default for WebViewMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions for quick metric recording without holding a WebViewMetrics instance.

/// Record WebView page load time.
pub fn record_webview_load_time(webview_id: &str, duration_ms: f64) {
    let meter = global::meter("auroraview");
    let histogram = meter
        .f64_histogram("auroraview.webview.load_time")
        .with_unit("ms")
        .build();
    histogram.record(
        duration_ms,
        &[KeyValue::new("webview_id", webview_id.to_string())],
    );
}

/// Record an IPC message.
pub fn record_ipc_message(webview_id: &str, direction: &str, latency_ms: f64) {
    let meter = global::meter("auroraview");
    let counter = meter.u64_counter("auroraview.ipc.messages").build();
    counter.add(
        1,
        &[
            KeyValue::new("webview_id", webview_id.to_string()),
            KeyValue::new("direction", direction.to_string()),
        ],
    );
    let histogram = meter
        .f64_histogram("auroraview.ipc.latency")
        .with_unit("ms")
        .build();
    histogram.record(
        latency_ms,
        &[
            KeyValue::new("webview_id", webview_id.to_string()),
            KeyValue::new("direction", direction.to_string()),
        ],
    );
}

/// Record an error.
pub fn record_error(webview_id: &str, error_type: &str) {
    let meter = global::meter("auroraview");
    let counter = meter.u64_counter("auroraview.errors").build();
    counter.add(
        1,
        &[
            KeyValue::new("webview_id", webview_id.to_string()),
            KeyValue::new("error_type", error_type.to_string()),
        ],
    );
}
