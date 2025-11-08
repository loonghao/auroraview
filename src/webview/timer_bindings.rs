//! Python bindings for the Timer module

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use std::sync::{Arc, Mutex};

use super::timer::{Timer as RustTimer, TimerBackend as RustTimerBackend};

/// Python-exposed timer backend type
#[pyclass(name = "TimerBackend")]
#[derive(Clone)]
pub struct PyTimerBackend {
    backend: RustTimerBackend,
}

#[pymethods]
impl PyTimerBackend {
    fn __repr__(&self) -> String {
        format!("{:?}", self.backend)
    }

    fn __str__(&self) -> String {
        match self.backend {
            #[cfg(target_os = "windows")]
            RustTimerBackend::WindowsSetTimer => "WindowsSetTimer".to_string(),
            RustTimerBackend::ThreadBased => "ThreadBased".to_string(),
        }
    }
}

/// Python-exposed Timer class
#[pyclass(name = "NativeTimer", unsendable)]
pub struct PyTimer {
    timer: Arc<Mutex<RustTimer>>,
    callback: Arc<Mutex<Option<PyObject>>>,
}

#[pymethods]
impl PyTimer {
    /// Create a new timer
    ///
    /// Args:
    ///     interval_ms: Timer interval in milliseconds
    ///
    /// Returns:
    ///     A new NativeTimer instance
    #[new]
    fn new(interval_ms: u32) -> Self {
        #[allow(clippy::arc_with_non_send_sync)]
        Self {
            timer: Arc::new(Mutex::new(RustTimer::new(interval_ms))),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the timer with Windows SetTimer backend
    ///
    /// Args:
    ///     hwnd: Window handle (as integer)
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     RuntimeError: If the timer fails to start
    #[cfg(target_os = "windows")]
    fn start_windows(&mut self, hwnd: isize) -> PyResult<()> {
        let mut timer = self.timer.lock().unwrap();
        timer
            .start_windows(hwnd)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)
    }

    /// Stop the timer
    fn stop(&mut self) {
        let mut timer = self.timer.lock().unwrap();
        timer.stop();
    }

    /// Set the callback function to be called on each tick
    ///
    /// Args:
    ///     callback: Python callable to invoke on each tick
    fn set_callback(&mut self, callback: PyObject) {
        let mut cb = self.callback.lock().unwrap();
        *cb = Some(callback);
    }

    /// Process pending timer messages (Windows only)
    ///
    /// This should be called periodically in the application's main loop.
    ///
    /// Returns:
    ///     Number of timer messages processed
    #[cfg(target_os = "windows")]
    fn process_messages(&mut self, _py: Python) -> PyResult<u32> {
        let callback = self.callback.clone();
        let mut timer = self.timer.lock().unwrap();

        let count = timer.process_messages(|| {
            let cb = callback.lock().unwrap();
            if let Some(ref callback) = *cb {
                // Call the Python callback
                Python::with_gil(|py| {
                    if let Err(e) = callback.call0(py) {
                        eprintln!("Error calling timer callback: {:?}", e);
                    }
                });
            }
        });

        Ok(count)
    }

    /// Get the current tick count
    ///
    /// Returns:
    ///     Number of ticks since the timer started
    fn tick_count(&self) -> u64 {
        let timer = self.timer.lock().unwrap();
        timer.tick_count()
    }

    /// Check if the timer is running
    ///
    /// Returns:
    ///     True if the timer is running, False otherwise
    fn is_running(&self) -> bool {
        let timer = self.timer.lock().unwrap();
        timer.is_running()
    }

    /// Get the timer backend type
    ///
    /// Returns:
    ///     TimerBackend enum value
    fn backend(&self) -> PyTimerBackend {
        let timer = self.timer.lock().unwrap();
        PyTimerBackend {
            backend: timer.backend(),
        }
    }

    /// Get the timer interval in milliseconds
    ///
    /// Returns:
    ///     Timer interval in milliseconds
    fn interval_ms(&self) -> u32 {
        let timer = self.timer.lock().unwrap();
        timer.interval_ms()
    }

    /// Context manager support: __enter__
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Context manager support: __exit__
    fn __exit__(
        &mut self,
        _exc_type: Option<Bound<'_, PyAny>>,
        _exc_val: Option<Bound<'_, PyAny>>,
        _exc_tb: Option<Bound<'_, PyAny>>,
    ) {
        self.stop();
    }

    /// String representation
    fn __repr__(&self) -> String {
        let timer = self.timer.lock().unwrap();
        format!(
            "NativeTimer(interval_ms={}, running={}, backend={:?}, ticks={})",
            timer.interval_ms(),
            timer.is_running(),
            timer.backend(),
            timer.tick_count()
        )
    }

    /// Get timer properties as a dictionary
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let timer = self.timer.lock().unwrap();
        let dict = PyDict::new(py);
        dict.set_item("interval_ms", timer.interval_ms())?;
        dict.set_item("running", timer.is_running())?;
        dict.set_item("backend", format!("{:?}", timer.backend()))?;
        dict.set_item("tick_count", timer.tick_count())?;
        Ok(dict.into())
    }
}
