//! Python bindings for auroraview-testing

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tokio::runtime::Runtime;

use crate::inspector::{Inspector, InspectorConfig};
use crate::snapshot::{ActionResult, RefId, RefInfo, ScrollDirection, Snapshot};

/// Create a new Tokio runtime
fn get_runtime() -> Arc<Runtime> {
    Arc::new(Runtime::new().expect("Failed to create Tokio runtime"))
}

/// Python Inspector class
#[pyclass(name = "Inspector")]
pub struct PyInspector {
    inner: Arc<Mutex<Option<Inspector>>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyInspector {
    /// Connect to CDP endpoint
    ///
    /// Args:
    ///     endpoint: CDP HTTP endpoint (e.g., "http://localhost:9222")
    ///
    /// Returns:
    ///     Inspector instance
    #[classmethod]
    fn connect(_cls: &Bound<'_, pyo3::types::PyType>, endpoint: &str) -> PyResult<Self> {
        let runtime = get_runtime();
        let inspector = runtime
            .block_on(Inspector::connect(endpoint))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(inspector))),
            runtime,
        })
    }

    /// Connect with custom configuration
    #[classmethod]
    fn connect_with_config(
        _cls: &Bound<'_, pyo3::types::PyType>,
        endpoint: &str,
        timeout_secs: Option<f64>,
        capture_screenshots: Option<bool>,
        detect_changes: Option<bool>,
    ) -> PyResult<Self> {
        let config = InspectorConfig {
            timeout: Duration::from_secs_f64(timeout_secs.unwrap_or(30.0)),
            capture_screenshots: capture_screenshots.unwrap_or(false),
            detect_changes: detect_changes.unwrap_or(true),
        };

        let runtime = get_runtime();
        let inspector = runtime
            .block_on(Inspector::connect_with_config(endpoint, config))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(inspector))),
            runtime,
        })
    }

    /// Get page snapshot
    ///
    /// Returns:
    ///     Snapshot with page info, refs, and structure
    fn snapshot(&self) -> PyResult<PySnapshot> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let snapshot = self
            .runtime
            .block_on(inspector.snapshot())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PySnapshot(snapshot))
    }

    /// Take screenshot
    ///
    /// Args:
    ///     path: Optional file path to save screenshot
    ///
    /// Returns:
    ///     PNG bytes
    fn screenshot(&self, path: Option<&str>) -> PyResult<Vec<u8>> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let bytes = self
            .runtime
            .block_on(inspector.screenshot())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        // Optionally save to file
        if let Some(path) = path {
            std::fs::write(path, &bytes)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        }

        Ok(bytes)
    }

    /// Click element by ref
    ///
    /// Args:
    ///     ref_id: Ref ID (e.g., "@3", "3", or 3)
    fn click(&self, ref_id: PyRefId) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.click(ref_id.0))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Fill input by ref
    ///
    /// Args:
    ///     ref_id: Ref ID of input element
    ///     text: Text to fill
    fn fill(&self, ref_id: PyRefId, text: &str) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.fill(ref_id.0, text))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Press a key
    ///
    /// Args:
    ///     key: Key to press (e.g., "Enter", "Tab", "Escape")
    fn press(&self, key: &str) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.press(key))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Scroll page
    ///
    /// Args:
    ///     direction: "up", "down", "left", or "right"
    ///     amount: Scroll amount in pixels (default: 300)
    fn scroll(&self, direction: &str, amount: Option<i32>) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let dir = ScrollDirection::parse(direction).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid direction: {}",
                direction
            ))
        })?;

        let result = self
            .runtime
            .block_on(inspector.scroll(dir, amount.unwrap_or(300)))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Navigate to URL
    fn goto(&self, url: &str) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.goto(url))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Go back
    fn back(&self) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.back())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Go forward
    fn forward(&self) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.forward())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Reload page
    fn reload(&self) -> PyResult<PyActionResult> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.reload())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyActionResult(result))
    }

    /// Get element text
    fn text(&self, ref_id: PyRefId) -> PyResult<String> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        self.runtime
            .block_on(inspector.text(ref_id.0))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get input value
    fn value(&self, ref_id: PyRefId) -> PyResult<String> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        self.runtime
            .block_on(inspector.value(ref_id.0))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Execute JavaScript
    fn eval(&self, py: Python<'_>, script: &str) -> PyResult<PyObject> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let result = self
            .runtime
            .block_on(inspector.eval(script))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        json_to_py(py, &result)
    }

    /// Wait for condition
    ///
    /// Args:
    ///     condition: Condition string (e.g., "text:Welcome", "ref:@5", "idle")
    ///     timeout: Optional timeout in seconds (default: 30)
    ///
    /// Returns:
    ///     True if condition met, False if timeout
    fn wait(&self, condition: &str, timeout: Option<f64>) -> PyResult<bool> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        let timeout_duration = timeout.map(Duration::from_secs_f64);

        self.runtime
            .block_on(inspector.wait(condition, timeout_duration))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Current URL
    #[getter]
    fn url(&self) -> PyResult<String> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        self.runtime
            .block_on(inspector.url())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Current title
    #[getter]
    fn title(&self) -> PyResult<String> {
        let guard = self.inner.lock();
        let inspector = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Inspector closed"))?;

        self.runtime
            .block_on(inspector.title())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Close connection
    fn close(&self) -> PyResult<()> {
        let mut guard = self.inner.lock();
        if let Some(inspector) = guard.take() {
            self.runtime
                .block_on(inspector.close())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }
        Ok(())
    }

    /// Context manager enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager exit
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, pyo3::types::PyAny>>,
        _exc_val: Option<&Bound<'_, pyo3::types::PyAny>>,
        _exc_tb: Option<&Bound<'_, pyo3::types::PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }
}

/// Python wrapper for RefId
#[derive(Clone)]
pub struct PyRefId(RefId);

impl<'py> FromPyObject<'py> for PyRefId {
    fn extract_bound(ob: &Bound<'py, pyo3::PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.extract::<String>() {
            Ok(PyRefId(RefId::from(s)))
        } else if let Ok(n) = ob.extract::<i32>() {
            Ok(PyRefId(RefId::from(n)))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected string or integer ref ID",
            ))
        }
    }
}

/// Python Snapshot class
#[pyclass(name = "Snapshot")]
pub struct PySnapshot(Snapshot);

#[pymethods]
impl PySnapshot {
    /// Page title
    #[getter]
    fn title(&self) -> &str {
        &self.0.title
    }

    /// Page URL
    #[getter]
    fn url(&self) -> &str {
        &self.0.url
    }

    /// Viewport dimensions
    #[getter]
    fn viewport(&self) -> (u32, u32) {
        self.0.viewport
    }

    /// Ref count
    fn ref_count(&self) -> usize {
        self.0.ref_count()
    }

    /// Get refs as dict
    #[getter]
    fn refs(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        for (k, v) in &self.0.refs {
            dict.set_item(k, PyRefInfo(v.clone()).into_pyobject(py)?)?;
        }
        Ok(dict.into())
    }

    /// Get accessibility tree
    #[getter]
    fn tree(&self) -> &str {
        &self.0.tree
    }

    /// Find refs containing text
    fn find(&self, text: &str, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let results = self.0.find(text);
        results
            .into_iter()
            .map(|r| PyRefInfo(r.clone()).into_pyobject(py).map(|o| o.into()))
            .collect()
    }

    /// Get ref by ID
    fn get_ref(&self, id: &str, py: Python<'_>) -> PyResult<Option<PyObject>> {
        self.0
            .get_ref(id)
            .map(|r| PyRefInfo(r.clone()).into_pyobject(py).map(|o| o.into()))
            .transpose()
    }

    /// Format as text
    fn to_text(&self) -> String {
        self.0.to_text()
    }

    /// Format as JSON
    fn to_json(&self) -> String {
        self.0.to_json()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "Snapshot(title='{}', url='{}', refs={})",
            self.0.title,
            self.0.url,
            self.0.ref_count()
        )
    }
}

/// Python RefInfo class
#[pyclass(name = "RefInfo")]
#[derive(Clone)]
pub struct PyRefInfo(RefInfo);

#[pymethods]
impl PyRefInfo {
    /// Ref ID
    #[getter]
    fn ref_id(&self) -> &str {
        &self.0.ref_id
    }

    /// ARIA role
    #[getter]
    fn role(&self) -> &str {
        &self.0.role
    }

    /// Accessible name
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// Description
    #[getter]
    fn description(&self) -> &str {
        &self.0.description
    }

    /// CSS selector
    #[getter]
    fn selector(&self) -> &str {
        &self.0.selector
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "RefInfo(ref_id='{}', role='{}', name='{}')",
            self.0.ref_id, self.0.role, self.0.name
        )
    }
}

/// Python ActionResult class
#[pyclass(name = "ActionResult")]
pub struct PyActionResult(ActionResult);

#[pymethods]
impl PyActionResult {
    /// Whether action succeeded
    #[getter]
    fn success(&self) -> bool {
        self.0.success
    }

    /// Action description
    #[getter]
    fn action(&self) -> &str {
        &self.0.action
    }

    /// State before action
    #[getter]
    fn before(&self) -> &str {
        &self.0.before
    }

    /// State after action
    #[getter]
    fn after(&self) -> &str {
        &self.0.after
    }

    /// Detected changes
    #[getter]
    fn changes(&self) -> Vec<String> {
        self.0.changes.clone()
    }

    /// Error message
    #[getter]
    fn error(&self) -> Option<&str> {
        self.0.error.as_deref()
    }

    /// Duration in milliseconds
    #[getter]
    fn duration_ms(&self) -> u64 {
        self.0.duration_ms
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        if self.0.success {
            format!("ActionResult(success=True, action='{}')", self.0.action)
        } else {
            format!(
                "ActionResult(success=False, error='{}')",
                self.0.error.as_deref().unwrap_or("unknown")
            )
        }
    }

    fn __bool__(&self) -> bool {
        self.0.success
    }
}

/// Convert JSON value to Python object
fn json_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_pyobject(py)?.into()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into()),
        serde_json::Value::Array(arr) => {
            let list: Vec<PyObject> = arr
                .iter()
                .map(|v| json_to_py(py, v))
                .collect::<PyResult<_>>()?;
            Ok(list.into_pyobject(py)?.into())
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into())
        }
    }
}

/// Register Python module
pub fn register_module(parent: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    let m = pyo3::types::PyModule::new(parent.py(), "testing")?;
    m.add_class::<PyInspector>()?;
    m.add_class::<PySnapshot>()?;
    m.add_class::<PyRefInfo>()?;
    m.add_class::<PyActionResult>()?;
    parent.add_submodule(&m)?;
    Ok(())
}
