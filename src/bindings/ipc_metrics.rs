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

impl From<crate::ipc::IpcMetricsSnapshot> for PyIpcMetrics {
    fn from(snapshot: crate::ipc::IpcMetricsSnapshot) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::IpcMetricsSnapshot;

    #[test]
    fn test_py_ipc_metrics_from_snapshot() {
        let snapshot = IpcMetricsSnapshot {
            messages_sent: 100,
            messages_failed: 5,
            messages_dropped: 2,
            retry_attempts: 10,
            avg_latency_us: 150,
            peak_queue_length: 50,
            messages_received: 95,
            success_rate: 95.24,
        };

        let py_metrics: PyIpcMetrics = snapshot.into();

        assert_eq!(py_metrics.messages_sent, 100);
        assert_eq!(py_metrics.messages_failed, 5);
        assert_eq!(py_metrics.messages_dropped, 2);
        assert_eq!(py_metrics.retry_attempts, 10);
        assert_eq!(py_metrics.avg_latency_us, 150);
        assert_eq!(py_metrics.peak_queue_length, 50);
        assert_eq!(py_metrics.messages_received, 95);
        assert!((py_metrics.success_rate - 95.24).abs() < 0.01);
    }

    #[test]
    fn test_py_ipc_metrics_repr() {
        let metrics = PyIpcMetrics {
            messages_sent: 100,
            messages_failed: 5,
            messages_dropped: 2,
            retry_attempts: 10,
            avg_latency_us: 150,
            peak_queue_length: 50,
            messages_received: 95,
            success_rate: 95.24,
        };

        let repr = metrics.__repr__();
        assert!(repr.contains("sent=100"));
        assert!(repr.contains("failed=5"));
        assert!(repr.contains("dropped=2"));
        assert!(repr.contains("95.24%"));
        assert!(repr.contains("150μs"));
    }

    #[test]
    fn test_py_ipc_metrics_format() {
        let metrics = PyIpcMetrics {
            messages_sent: 100,
            messages_failed: 5,
            messages_dropped: 2,
            retry_attempts: 10,
            avg_latency_us: 150,
            peak_queue_length: 50,
            messages_received: 95,
            success_rate: 95.24,
        };

        let format = metrics.format();
        assert!(format.contains("IPC Metrics:"));
        assert!(format.contains("Messages Sent: 100"));
        assert!(format.contains("Messages Failed: 5"));
        assert!(format.contains("Messages Dropped: 2"));
        assert!(format.contains("Retry Attempts: 10"));
        assert!(format.contains("Success Rate: 95.24%"));
        assert!(format.contains("Avg Latency: 150μs"));
        assert!(format.contains("Peak Queue Length: 50"));
        assert!(format.contains("Messages Received: 95"));
    }

    #[test]
    fn test_register_ipc_metrics_module() {
        pyo3::Python::attach(|py| {
            let m = pyo3::types::PyModule::new(py, "ipc_test").unwrap();
            register_ipc_metrics(&m).expect("register should succeed");
            assert!(m.getattr("IpcMetrics").is_ok());
        });
    }

    #[test]
    fn test_py_ipc_metrics_zero_values() {
        let metrics = PyIpcMetrics {
            messages_sent: 0,
            messages_failed: 0,
            messages_dropped: 0,
            retry_attempts: 0,
            avg_latency_us: 0,
            peak_queue_length: 0,
            messages_received: 0,
            success_rate: 100.0,
        };

        let repr = metrics.__repr__();
        assert!(repr.contains("sent=0"));
        assert!(repr.contains("100.00%"));
    }
}
