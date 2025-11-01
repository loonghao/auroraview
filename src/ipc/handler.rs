//! IPC Handler for WebView Communication
//!
//! This module manages communication between Python and JavaScript,
//! handling event callbacks and message routing.

use dashmap::DashMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
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
    pub fn call(&self, data: serde_json::Value) -> Result<(), String> {
        Python::with_gil(|py| {
            // Convert JSON value to Python object
            let py_data = match json_to_python(py, &data) {
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

/// Convert JSON value to Python object
#[allow(deprecated)]
fn json_to_python(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(n.to_string().into_py(py))
            }
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = json_to_python(py, item)?;
                py_list.append(py_item.bind(py))?;
            }
            Ok(py_list.into_py(py))
        }
        serde_json::Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                let py_val = json_to_python(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_py(py))
        }
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
