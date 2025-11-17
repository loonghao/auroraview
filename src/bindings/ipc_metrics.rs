//! Python bindings for IPC metrics
//!
//! Exposes IPC performance metrics to Python for monitoring and debugging.

use pyo3::prelude::*;

/// IPC performance metrics snapshot (Python-facing)
///
/// This class provides read-only access to IPC performance metrics.
/// All metrics are captured at the time of the snapshot.
///
/// Example:
/// ```python
/// from auroraview import WebView
///
/// webview = WebView(...)
/// metrics = webview.get_ipc_metrics()
///
/// print(f"Messages sent: {metrics.messages_sent}")
/// print(f"Success rate: {metrics.success_rate}%")
/// print(f"Avg latency: {metrics.avg_latency_us}μs")
/// ```
#[pyclass(name = "IpcMetrics")]
#[derive(Debug, Clone)]
pub struct PyIpcMetrics {
    /// Total messages sent successfully
    #[pyo3(get)]
    pub messages_sent: u64,

    /// Total messages that failed
    #[pyo3(get)]
    pub messages_failed: u64,

    /// Total messages dropped
    #[pyo3(get)]
    pub messages_dropped: u64,

    /// Total retry attempts
    #[pyo3(get)]
    pub retry_attempts: u64,

    /// Average latency in microseconds
    #[pyo3(get)]
    pub avg_latency_us: u64,

    /// Peak queue length
    #[pyo3(get)]
    pub peak_queue_length: u64,

    /// Total messages received
    #[pyo3(get)]
    pub messages_received: u64,

    /// Success rate (percentage)
    #[pyo3(get)]
    pub success_rate: f64,
}

#[pymethods]
impl PyIpcMetrics {
    /// String representation of metrics
    fn __repr__(&self) -> String {
        format!(
            "IpcMetrics(sent={}, failed={}, dropped={}, success_rate={:.2}%, avg_latency={}μs)",
            self.messages_sent,
            self.messages_failed,
            self.messages_dropped,
            self.success_rate,
            self.avg_latency_us
        )
    }

    /// Human-readable format
    fn format(&self) -> String {
        format!(
            "IPC Metrics:\n\
             - Messages Sent: {}\n\
             - Messages Failed: {}\n\
             - Messages Dropped: {}\n\
             - Retry Attempts: {}\n\
             - Success Rate: {:.2}%\n\
             - Avg Latency: {}μs\n\
             - Peak Queue Length: {}\n\
             - Messages Received: {}",
            self.messages_sent,
            self.messages_failed,
            self.messages_dropped,
            self.retry_attempts,
            self.success_rate,
            self.avg_latency_us,
            self.peak_queue_length,
            self.messages_received
        )
    }
}

impl From<crate::ipc::metrics::IpcMetricsSnapshot> for PyIpcMetrics {
    fn from(snapshot: crate::ipc::metrics::IpcMetricsSnapshot) -> Self {
        Self {
            messages_sent: snapshot.messages_sent,
            messages_failed: snapshot.messages_failed,
            messages_dropped: snapshot.messages_dropped,
            retry_attempts: snapshot.retry_attempts,
            avg_latency_us: snapshot.avg_latency_us,
            peak_queue_length: snapshot.peak_queue_length,
            messages_received: snapshot.messages_received,
            success_rate: snapshot.success_rate,
        }
    }
}

/// Register IPC metrics class to Python module
pub fn register_ipc_metrics(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIpcMetrics>()?;
    Ok(())
}
