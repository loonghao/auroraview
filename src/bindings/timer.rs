//! Python bindings for the Timer module

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use pyo3::Py;
use std::sync::{Arc, Mutex};

use crate::webview::timer::{Timer as RustTimer, TimerBackend as RustTimerBackend};

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
    callback: Arc<Mutex<Option<Py<PyAny>>>>,
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
        let mut timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        timer
            .start_windows(hwnd)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)
    }

    /// Stop the timer
    fn stop(&mut self) {
        let mut timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        timer.stop();
    }

    /// Set the callback function to be called on each tick
    ///
    /// Args:
    ///     callback: Python callable to invoke on each tick
    fn set_callback(&mut self, callback: Py<PyAny>) {
        let mut cb = match self.callback.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
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
        let mut timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let count = timer.process_messages(|| {
            let cb = match callback.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(ref callback) = *cb {
                // Call the Python callback
                Python::attach(|py| {
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
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        timer.tick_count()
    }

    /// Check if the timer is running
    ///
    /// Returns:
    ///     True if the timer is running, False otherwise
    fn is_running(&self) -> bool {
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        timer.is_running()
    }

    /// Get the timer backend type
    ///
    /// Returns:
    ///     TimerBackend enum value
    fn backend(&self) -> PyTimerBackend {
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        PyTimerBackend {
            backend: timer.backend(),
        }
    }

    /// Get the timer interval in milliseconds
    ///
    /// Returns:
    ///     Timer interval in milliseconds
    fn interval_ms(&self) -> u32 {
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
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
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        format!(
            "NativeTimer(interval_ms={}, running={}, backend={:?}, ticks={})",
            timer.interval_ms(),
            timer.is_running(),
            timer.backend(),
            timer.tick_count()
        )
    }

    /// Get timer properties as a dictionary
    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let timer = match self.timer.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let dict = PyDict::new(py);
        dict.set_item("interval_ms", timer.interval_ms())?;
        dict.set_item("running", timer.is_running())?;
        dict.set_item("backend", format!("{:?}", timer.backend()))?;
        dict.set_item("tick_count", timer.tick_count())?;
        Ok(dict.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_timer_new() {
        Python::attach(|_py| {
            let timer = PyTimer::new(100);
            assert_eq!(timer.interval_ms(), 100);
            assert!(!timer.is_running());
            assert_eq!(timer.tick_count(), 0);
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_repr() {
        Python::attach(|_py| {
            let timer = PyTimer::new(50);
            let repr = timer.__repr__();

            assert!(repr.contains("NativeTimer"));
            assert!(repr.contains("interval_ms=50"));
            assert!(repr.contains("running=false"));
            assert!(repr.contains("ticks=0"));
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_backend() {
        Python::attach(|_py| {
            let timer = PyTimer::new(100);
            let backend = timer.backend();

            // Backend should be ThreadBased by default (not started)
            let backend_str = backend.__str__();
            assert!(backend_str == "ThreadBased" || backend_str == "WindowsSetTimer");
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_backend_repr() {
        Python::attach(|_py| {
            let timer = PyTimer::new(100);
            let backend = timer.backend();
            let repr = backend.__repr__();

            assert!(repr.contains("ThreadBased") || repr.contains("WindowsSetTimer"));
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_to_dict() {
        Python::attach(|py| {
            let timer = PyTimer::new(200);
            let dict_obj = timer.to_dict(py).unwrap();
            let dict = dict_obj.bind(py).cast::<PyDict>().unwrap();

            let interval: u32 = dict
                .get_item("interval_ms")
                .unwrap()
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(interval, 200);

            let running: bool = dict
                .get_item("running")
                .unwrap()
                .unwrap()
                .extract()
                .unwrap();
            assert!(!running);

            let tick_count: u64 = dict
                .get_item("tick_count")
                .unwrap()
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(tick_count, 0);

            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_stop() {
        Python::attach(|_py| {
            let mut timer = PyTimer::new(100);
            // Stop should work even if timer is not running
            timer.stop();
            assert!(!timer.is_running());
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_set_callback() {
        Python::attach(|py| {
            let mut timer = PyTimer::new(100);

            // Create a simple Python lambda
            let callback = py.eval(c"lambda: None", None, None).unwrap();
            timer.set_callback(callback.unbind());

            // Verify callback was set
            let cb = timer.callback.lock().unwrap();
            assert!(cb.is_some());

            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_py_timer_backend_clone() {
        Python::attach(|_py| {
            let timer = PyTimer::new(100);
            let backend = timer.backend();
            let cloned = backend.clone();

            assert_eq!(backend.__str__(), cloned.__str__());
            Ok::<(), pyo3::PyErr>(())
        })
        .unwrap();
    }
}
