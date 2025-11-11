//! IPC Handler for WebView Communication
//!
//! This module manages communication between Python and JavaScript,
//! handling event callbacks and message routing.

use dashmap::DashMap;
use pyo3::prelude::*;
use std::sync::Arc;

// Re-export IpcMessage from backend module
pub use super::backend::IpcMessage;

/// IPC callback type (Rust closures)
pub type IpcCallback = Arc<dyn Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync>;

/// Python callback wrapper - stores Python callable objects
pub struct PythonCallback {
    /// Python callable object
    pub callback: PyObject,
}

impl PythonCallback {
    /// Create a new Python callback wrapper
    pub fn new(callback: PyObject) -> Self {
        Self { callback }
    }

    /// Call the Python callback with the given data
    pub fn call(&self, data: super::json::Value) -> Result<(), String> {
        Python::with_gil(|py| {
            // Convert JSON value to Python object using the optimized json module
            let py_data = match super::json::json_to_python(py, &data) {
                Ok(obj) => obj,
                Err(e) => {
                    tracing::error!("Failed to convert JSON to Python: {}", e);
                    return Err(format!("Failed to convert JSON to Python: {}", e));
                }
            };

            // Call the Python callback
            match self.callback.call1(py, (py_data,)) {
                Ok(_) => {
                    tracing::debug!("Python callback executed successfully");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Python callback error: {}", e);
                    Err(format!("Python callback error: {}", e))
                }
            }
        })
    }
}

/// IPC handler for managing communication between Python and JavaScript
///
/// Uses DashMap for lock-free concurrent callback storage, improving
/// performance in high-throughput scenarios.
pub struct IpcHandler {
    /// Registered event callbacks (Rust closures) - lock-free concurrent map
    callbacks: Arc<DashMap<String, Vec<IpcCallback>>>,

    /// Registered Python callbacks - lock-free concurrent map
    python_callbacks: Arc<DashMap<String, Vec<PythonCallback>>>,
}

impl IpcHandler {
    /// Create a new IPC handler
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            python_callbacks: Arc::new(DashMap::new()),
        }
    }

    /// Register a Rust callback for an event
    #[allow(dead_code)]
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        self.callbacks
            .entry(event.to_string())
            .or_default()
            .push(Arc::new(callback));
    }

    /// Register a Python callback for an event
    pub fn register_python_callback(&self, event: &str, callback: PyObject) {
        self.python_callbacks
            .entry(event.to_string())
            .or_default()
            .push(PythonCallback::new(callback));
        tracing::info!("Registered Python callback for event: {}", event);
    }

    /// Emit an event to JavaScript
    #[allow(dead_code)]
    pub fn emit(&self, event: &str, data: serde_json::Value) -> Result<(), String> {
        let _message = IpcMessage {
            event: event.to_string(),
            data,
            id: None,
        };

        tracing::debug!("Emitting IPC event: {}", event);

        // TODO: Send message to WebView
        Ok(())
    }

    /// Handle incoming message from JavaScript
    #[allow(dead_code)]
    pub fn handle_message(&self, message: IpcMessage) -> Result<serde_json::Value, String> {
        tracing::debug!("Handling IPC message: {}", message.event);

        // First try Python callbacks
        if let Some(event_callbacks) = self.python_callbacks.get(&message.event) {
            for callback in event_callbacks.value() {
                if let Err(e) = callback.call(message.data.clone()) {
                    tracing::error!("Python callback error: {}", e);
                    return Err(e);
                }
            }
            return Ok(serde_json::json!({"status": "ok"}));
        }

        // Then try Rust callbacks
        if let Some(event_callbacks) = self.callbacks.get(&message.event) {
            if let Some(callback) = event_callbacks.value().first() {
                match callback(message.clone()) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        tracing::error!("IPC callback error: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        // No callback found
        Err(format!(
            "No handler registered for event: {}",
            message.event
        ))
    }

    /// Remove all callbacks for an event
    #[allow(dead_code)]
    pub fn off(&self, event: &str) {
        self.callbacks.remove(event);
        self.python_callbacks.remove(event);
    }

    /// Clear all callbacks
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.callbacks.clear();
        self.python_callbacks.clear();
    }
}

impl Default for IpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyList, PyModule};

    fn py_append_collector() -> (PyObject, PyObject) {
        Python::with_gil(|py| {
            let seen = PyList::empty(py);
            let m = PyModule::from_code(
                py,
                c"def make_cb(seen):\n    def cb(x):\n        seen.append(x)\n    return cb\n",
                c"m.py",
                c"m",
            )
            .unwrap();
            let make_cb = m.getattr("make_cb").unwrap();
            // Keep an owned handle to the list so we can inspect it later
            let seen_obj: PyObject = seen.clone().unbind().into();
            let seen_bound = seen_obj.bind(py).downcast::<PyList>().unwrap();
            let cb = make_cb.call1((seen_bound,)).unwrap().clone().unbind();
            (cb, seen_obj)
        })
    }

    #[test]
    fn test_python_callback_flow() {
        let handler = IpcHandler::new();
        let (cb, seen_obj) = py_append_collector();
        handler.register_python_callback("evt", cb);

        let msg = IpcMessage {
            event: "evt".to_string(),
            data: serde_json::json!({"a":1}),
            id: None,
        };
        let res = handler.handle_message(msg);
        assert!(res.is_ok());

        Python::with_gil(|py| {
            let seen = seen_obj.bind(py).downcast::<PyList>().unwrap();
            assert_eq!(seen.len(), 1);
            let first = seen.get_item(0).unwrap();
            // first is a Python object converted from JSON dict
            let dict = first.downcast::<pyo3::types::PyDict>().unwrap();
            if let Ok(Some(a_val)) = dict.get_item("a") {
                let a: i64 = a_val.extract().unwrap();
                assert_eq!(a, 1);
            } else {
                panic!("missing key a");
            }
        });
    }

    #[test]
    fn test_rust_callback_flow_and_no_handler() {
        let handler = IpcHandler::new();
        handler.on("evt2", |m| {
            assert_eq!(m.event, "evt2");
            Ok(serde_json::json!({"ok": true}))
        });
        let res = handler.handle_message(IpcMessage {
            event: "evt2".to_string(),
            data: serde_json::json!({}),
            id: None,
        });
        assert_eq!(res.unwrap(), serde_json::json!({"ok": true}));

        // No handler case
        let err = handler
            .handle_message(IpcMessage {
                event: "unknown".to_string(),
                data: serde_json::json!({}),
                id: None,
            })
            .unwrap_err();
        assert!(err.contains("No handler registered"));
    }
}
