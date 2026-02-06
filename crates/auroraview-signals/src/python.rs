//! Python bindings for Aurora Signals via PyO3
//!
//! This module provides Python-compatible wrappers for the signal system.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use serde_json::Value;
use std::sync::Arc;

use crate::bridge::{BridgeError, CallbackBridge};
use crate::bus::EventBus;
use crate::connection::ConnectionId;
use crate::middleware::{FilterMiddleware, LogLevel, LoggingMiddleware};
use crate::registry::SignalRegistry;
use crate::signal::Signal;

// ============================================================================
// PyConnectionId - Python wrapper for ConnectionId
// ============================================================================

/// Python wrapper for ConnectionId
#[pyclass(name = "ConnectionId")]
#[derive(Clone)]
pub struct PyConnectionId {
    inner: ConnectionId,
}

#[pymethods]
impl PyConnectionId {
    /// Get the raw ID value
    #[getter]
    fn id(&self) -> u64 {
        self.inner.id()
    }

    fn __repr__(&self) -> String {
        format!("ConnectionId({})", self.inner.id())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner.id())
    }

    fn __eq__(&self, other: &PyConnectionId) -> bool {
        self.inner == other.inner
    }

    fn __hash__(&self) -> u64 {
        self.inner.id()
    }
}

impl From<ConnectionId> for PyConnectionId {
    fn from(id: ConnectionId) -> Self {
        Self { inner: id }
    }
}

impl From<PyConnectionId> for ConnectionId {
    fn from(py_id: PyConnectionId) -> Self {
        py_id.inner
    }
}

// ============================================================================
// PySignal - Python wrapper for Signal<Value>
// ============================================================================

/// Python wrapper for Signal
///
/// A type-safe signal that can have multiple connected handlers.
///
/// Example:
///     signal = Signal(name="my_signal")
///     conn = signal.connect(lambda data: print(data))
///     signal.emit({"key": "value"})
///     signal.disconnect(conn)
#[pyclass(name = "Signal")]
pub struct PySignal {
    inner: Arc<Signal<Value>>,
}

#[pymethods]
impl PySignal {
    /// Create a new signal
    ///
    /// Args:
    ///     name: Optional name for the signal (for debugging)
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<String>) -> Self {
        let signal = match name {
            Some(n) => Signal::named(n),
            None => Signal::new(),
        };
        Self {
            inner: Arc::new(signal),
        }
    }

    /// Get the signal's name
    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.name().map(|s| s.to_string())
    }

    /// Connect a handler to this signal
    ///
    /// Args:
    ///     handler: A callable that takes one argument (the emitted data)
    ///
    /// Returns:
    ///     ConnectionId that can be used to disconnect the handler
    fn connect(&self, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(handler);

        let id = self.inner.connect(move |data| {
            Python::attach(|py| {
                let py_data = json_to_pyobject(py, &data);
                if let Err(e) = handler.call1(py, (py_data,)) {
                    tracing::error!("Python handler error: {}", e);
                }
            });
        });

        Ok(PyConnectionId::from(id))
    }

    /// Connect a handler that will only be called once
    ///
    /// Args:
    ///     handler: A callable that takes one argument
    ///
    /// Returns:
    ///     ConnectionId that can be used to disconnect the handler
    fn connect_once(&self, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(parking_lot::Mutex::new(Some(handler)));

        let id = self.inner.connect(move |data| {
            if let Some(h) = handler.lock().take() {
                Python::attach(|py| {
                    let py_data = json_to_pyobject(py, &data);
                    if let Err(e) = h.call1(py, (py_data,)) {
                        tracing::error!("Python handler error: {}", e);
                    }
                });
            }
        });

        Ok(PyConnectionId::from(id))
    }

    /// Disconnect a handler by its ConnectionId
    ///
    /// Returns:
    ///     True if the handler was disconnected, False if not found
    fn disconnect(&self, conn_id: PyConnectionId) -> bool {
        self.inner.disconnect(conn_id.inner)
    }

    /// Emit a value to all connected handlers
    ///
    /// Args:
    ///     data: The data to emit (will be converted to JSON)
    fn emit(&self, py: Python<'_>, data: Py<PyAny>) -> PyResult<()> {
        let json_data = pyobject_to_json(py, &data)?;
        self.inner.emit(json_data);
        Ok(())
    }

    /// Get the number of connected handlers
    #[getter]
    fn handler_count(&self) -> usize {
        self.inner.handler_count()
    }

    /// Check if any handlers are connected
    #[getter]
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Disconnect all handlers
    fn disconnect_all(&self) {
        self.inner.disconnect_all();
    }

    fn __repr__(&self) -> String {
        format!(
            "Signal(name={:?}, handlers={})",
            self.inner.name(),
            self.inner.handler_count()
        )
    }
}

// ============================================================================
// PySignalRegistry - Python wrapper for SignalRegistry
// ============================================================================

/// Python wrapper for SignalRegistry
///
/// A registry for dynamically named signals.
///
/// Example:
///     registry = SignalRegistry()
///     conn = registry.connect("my_event", lambda data: print(data))
///     registry.emit("my_event", {"key": "value"})
#[pyclass(name = "SignalRegistry")]
pub struct PySignalRegistry {
    inner: Arc<SignalRegistry>,
}

#[pymethods]
impl PySignalRegistry {
    /// Create a new signal registry
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<String>) -> Self {
        let registry = match name {
            Some(n) => SignalRegistry::named(n),
            None => SignalRegistry::new(),
        };
        Self {
            inner: Arc::new(registry),
        }
    }

    /// Connect a handler to a named signal
    ///
    /// Creates the signal if it doesn't exist.
    fn connect(&self, name: &str, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(handler);

        let id = self.inner.connect(name, move |data| {
            Python::attach(|py| {
                let py_data = json_to_pyobject(py, &data);
                if let Err(e) = handler.call1(py, (py_data,)) {
                    tracing::error!("Python handler error: {}", e);
                }
            });
        });

        Ok(PyConnectionId::from(id))
    }

    /// Connect a one-time handler to a named signal
    fn connect_once(&self, name: &str, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(parking_lot::Mutex::new(Some(handler)));

        let id = self.inner.connect(name, move |data| {
            if let Some(h) = handler.lock().take() {
                Python::attach(|py| {
                    let py_data = json_to_pyobject(py, &data);
                    if let Err(e) = h.call1(py, (py_data,)) {
                        tracing::error!("Python handler error: {}", e);
                    }
                });
            }
        });

        Ok(PyConnectionId::from(id))
    }

    /// Emit a value to a named signal
    fn emit(&self, py: Python<'_>, name: &str, data: Py<PyAny>) -> PyResult<usize> {
        let json_data = pyobject_to_json(py, &data)?;
        Ok(self.inner.emit(name, json_data))
    }

    /// Disconnect a handler from a named signal
    fn disconnect(&self, name: &str, conn_id: PyConnectionId) -> bool {
        self.inner.disconnect(name, conn_id.inner)
    }

    /// Check if a signal exists
    fn contains(&self, name: &str) -> bool {
        self.inner.contains(name)
    }

    /// Remove a signal
    fn remove(&self, name: &str) -> bool {
        self.inner.remove(name)
    }

    /// Get all signal names
    fn names(&self) -> Vec<String> {
        self.inner.names()
    }

    /// Get the number of signals
    #[getter]
    fn signal_count(&self) -> usize {
        self.inner.signal_count()
    }

    /// Clear all signals
    fn clear(&self) {
        self.inner.clear();
    }

    fn __repr__(&self) -> String {
        format!(
            "SignalRegistry(signals={}, handlers={})",
            self.inner.signal_count(),
            self.inner.total_handler_count()
        )
    }

    fn __contains__(&self, name: &str) -> bool {
        self.inner.contains(name)
    }
}

// ============================================================================
// PyEventBus - Python wrapper for EventBus
// ============================================================================

/// Python wrapper for EventBus
///
/// Unified event bus with middleware and bridge support.
///
/// Example:
///     bus = EventBus()
///     bus.use_logging_middleware("debug")
///     conn = bus.on("app:ready", lambda data: print(data))
///     bus.emit("app:ready", {"version": "1.0"})
#[pyclass(name = "EventBus")]
pub struct PyEventBus {
    inner: Arc<EventBus>,
}

#[pymethods]
impl PyEventBus {
    /// Create a new event bus
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<String>) -> Self {
        let bus = match name {
            Some(n) => EventBus::named(n),
            None => EventBus::new(),
        };
        Self {
            inner: Arc::new(bus),
        }
    }

    /// Subscribe to an event
    ///
    /// Args:
    ///     event: Event name
    ///     handler: Callable that receives event data
    ///
    /// Returns:
    ///     ConnectionId for unsubscribing
    fn on(&self, event: &str, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(handler);

        let id = self.inner.on(event, move |data| {
            Python::attach(|py| {
                let py_data = json_to_pyobject(py, &data);
                if let Err(e) = handler.call1(py, (py_data,)) {
                    tracing::error!("Python handler error: {}", e);
                }
            });
        });

        Ok(PyConnectionId::from(id))
    }

    /// Subscribe to an event once
    fn once(&self, event: &str, handler: Py<PyAny>) -> PyResult<PyConnectionId> {
        let handler = Arc::new(parking_lot::Mutex::new(Some(handler)));

        let id = self.inner.on(event, move |data| {
            if let Some(h) = handler.lock().take() {
                Python::attach(|py| {
                    let py_data = json_to_pyobject(py, &data);
                    if let Err(e) = h.call1(py, (py_data,)) {
                        tracing::error!("Python handler error: {}", e);
                    }
                });
            }
        });

        Ok(PyConnectionId::from(id))
    }

    /// Unsubscribe from an event
    fn off(&self, event: &str, conn_id: PyConnectionId) -> bool {
        self.inner.off(event, conn_id.inner)
    }

    /// Emit an event
    ///
    /// Args:
    ///     event: Event name
    ///     data: Event data (will be converted to JSON)
    ///
    /// Returns:
    ///     Number of handlers that received the event
    fn emit(&self, py: Python<'_>, event: &str, data: Py<PyAny>) -> PyResult<usize> {
        let json_data = pyobject_to_json(py, &data)?;
        Ok(self.inner.emit(event, json_data))
    }

    /// Emit an event only to local handlers (skip bridges)
    fn emit_local(&self, py: Python<'_>, event: &str, data: Py<PyAny>) -> PyResult<usize> {
        let json_data = pyobject_to_json(py, &data)?;
        Ok(self.inner.emit_local(event, json_data))
    }

    /// Add logging middleware
    ///
    /// Args:
    ///     level: Log level ("trace", "debug", "info", "warn", "error")
    ///     prefix: Optional prefix for log messages
    #[pyo3(signature = (level="debug", prefix=None))]
    fn use_logging_middleware(&self, level: &str, prefix: Option<String>) {
        let log_level = match level.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Debug,
        };

        let mut middleware = LoggingMiddleware::new(log_level);
        if let Some(p) = prefix {
            middleware = middleware.with_prefix(p);
        }

        self.inner.use_middleware(middleware);
    }

    /// Add filter middleware with deny patterns
    ///
    /// Args:
    ///     deny_patterns: List of regex patterns to deny
    #[pyo3(signature = (deny_patterns=None, allow_patterns=None))]
    fn use_filter_middleware(
        &self,
        deny_patterns: Option<Vec<String>>,
        allow_patterns: Option<Vec<String>>,
    ) -> PyResult<()> {
        let mut filter = if allow_patterns.is_some() && deny_patterns.is_none() {
            FilterMiddleware::deny_by_default()
        } else {
            FilterMiddleware::new()
        };

        if let Some(patterns) = deny_patterns {
            for pattern in patterns {
                filter = filter
                    .deny_pattern(&pattern)
                    .map_err(|e| PyRuntimeError::new_err(format!("Invalid pattern: {}", e)))?;
            }
        }

        if let Some(patterns) = allow_patterns {
            for pattern in patterns {
                filter = filter
                    .allow_pattern(&pattern)
                    .map_err(|e| PyRuntimeError::new_err(format!("Invalid pattern: {}", e)))?;
            }
        }

        self.inner.use_middleware(filter);
        Ok(())
    }

    /// Add a Python callback bridge
    ///
    /// Args:
    ///     name: Bridge name
    ///     callback: Callable that receives (event, data)
    fn add_callback_bridge(&self, name: &str, callback: Py<PyAny>) -> PyResult<()> {
        let callback = Arc::new(callback);
        let name_owned = name.to_string();

        let bridge = CallbackBridge::new(name_owned, move |event, data| {
            Python::attach(|py| {
                let py_data = json_to_pyobject(py, &data);
                callback
                    .call1(py, (event, py_data))
                    .map_err(|e| BridgeError::SendFailed(e.to_string()))?;
                Ok(())
            })
        });

        self.inner.add_bridge(bridge);
        Ok(())
    }

    /// Remove a bridge by name
    fn remove_bridge(&self, name: &str) -> bool {
        self.inner.remove_bridge(name)
    }

    /// Check if an event has handlers
    fn has_handlers(&self, event: &str) -> bool {
        self.inner.has_handlers(event)
    }

    /// Get handler count for an event
    fn handler_count(&self, event: &str) -> usize {
        self.inner.handler_count(event)
    }

    /// Get all event names
    fn event_names(&self) -> Vec<String> {
        self.inner.event_names()
    }

    /// Clear all handlers
    fn clear(&self) {
        self.inner.clear();
    }

    fn __repr__(&self) -> String {
        format!(
            "EventBus(events={}, handlers={}, bridges={})",
            self.inner.event_count(),
            self.inner.total_handler_count(),
            self.inner.bridge_count()
        )
    }
}

// ============================================================================
// Helper functions for Python <-> JSON conversion
// ============================================================================

/// Convert serde_json::Value to Py<PyAny>
fn json_to_pyobject(py: Python<'_>, value: &Value) -> Py<PyAny> {
    use pyo3::IntoPyObjectExt;

    match value {
        Value::Null => py.None(),
        Value::Bool(b) => (*b).into_py_any(py).unwrap(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py).unwrap()
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py).unwrap()
            } else {
                py.None()
            }
        }
        Value::String(s) => s.clone().into_py_any(py).unwrap(),
        Value::Array(arr) => {
            let items: Vec<Py<PyAny>> = arr.iter().map(|v| json_to_pyobject(py, v)).collect();
            items.into_py_any(py).unwrap()
        }
        Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_pyobject(py, v)).unwrap();
            }
            dict.unbind().into_any()
        }
    }
}

/// Convert Py<PyAny> to serde_json::Value
fn pyobject_to_json(py: Python<'_>, obj: &Py<PyAny>) -> PyResult<Value> {
    let bound = obj.bind(py);

    if bound.is_none() {
        return Ok(Value::Null);
    }

    if let Ok(b) = bound.extract::<bool>() {
        return Ok(Value::Bool(b));
    }

    if let Ok(i) = bound.extract::<i64>() {
        return Ok(Value::Number(i.into()));
    }

    if let Ok(f) = bound.extract::<f64>() {
        return Ok(serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null));
    }

    if let Ok(s) = bound.extract::<String>() {
        return Ok(Value::String(s));
    }

    if let Ok(list) = bound.extract::<Vec<Py<PyAny>>>() {
        let arr: Result<Vec<Value>, _> =
            list.iter().map(|item| pyobject_to_json(py, item)).collect();
        return Ok(Value::Array(arr?));
    }

    if let Ok(dict) = bound.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            let value = pyobject_to_json(py, &v.unbind())?;
            map.insert(key, value);
        }
        return Ok(Value::Object(map));
    }

    // Fallback: try to convert to string
    if let Ok(s) = bound.str() {
        return Ok(Value::String(s.to_string()));
    }

    Ok(Value::Null)
}

// ============================================================================
// Python module registration
// ============================================================================

/// Register the aurora_signals Python module
pub fn register_module(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent.py(), "signals")?;

    m.add_class::<PyConnectionId>()?;
    m.add_class::<PySignal>()?;
    m.add_class::<PySignalRegistry>()?;
    m.add_class::<PyEventBus>()?;

    parent.add_submodule(&m)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_conversion() {
        Python::initialize();

        Python::attach(|py| {
            // Test null
            let null = Value::Null;
            let py_null = json_to_pyobject(py, &null);
            assert!(py_null.bind(py).is_none());

            // Test bool
            let bool_val = Value::Bool(true);
            let py_bool = json_to_pyobject(py, &bool_val);
            assert!(py_bool.extract::<bool>(py).unwrap());

            // Test number
            let num_val = Value::Number(42.into());
            let py_num = json_to_pyobject(py, &num_val);
            assert_eq!(py_num.extract::<i64>(py).unwrap(), 42);

            // Test string
            let str_val = Value::String("hello".to_string());
            let py_str = json_to_pyobject(py, &str_val);
            assert_eq!(py_str.extract::<String>(py).unwrap(), "hello");
        });
    }
}
