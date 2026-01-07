//! Python bindings for Sidecar Bridge.
//!
//! This module provides PyO3 bindings for the SidecarBridge, which manages
//! the IPC Server that communicates with the MCP Sidecar process.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::ipc::sidecar_bridge::{SidecarBridge, SidecarBridgeConfig};

/// Python wrapper for SidecarBridge.
///
/// Manages the IPC Server that receives tool call requests from the
/// MCP Sidecar process.
///
/// Example:
///     >>> from auroraview.core import PySidecarBridge
///     >>> bridge = PySidecarBridge()
///     >>> bridge.register_tool("echo", "Echo back input", lambda args: args)
///     >>> bridge.start()
///     >>> print(f"Channel: {bridge.channel_name}")
///     >>> print(f"Token: {bridge.auth_token}")
#[pyclass(name = "SidecarBridge")]
pub struct PySidecarBridge {
    inner: SidecarBridge,
    /// Python tool handlers
    py_handlers: HashMap<String, Py<PyAny>>,
}

#[pymethods]
impl PySidecarBridge {
    /// Create a new Sidecar Bridge.
    ///
    /// Args:
    ///     channel_name: Optional custom channel name.
    ///     auth_token: Optional custom auth token.
    #[new]
    #[pyo3(signature = (channel_name=None, auth_token=None))]
    fn new(channel_name: Option<String>, auth_token: Option<String>) -> Self {
        let config = SidecarBridgeConfig {
            channel_name,
            auth_token,
        };
        Self {
            inner: SidecarBridge::new(config),
            py_handlers: HashMap::new(),
        }
    }

    /// Get the IPC channel name.
    #[getter]
    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    /// Get the authentication token.
    #[getter]
    fn auth_token(&self) -> &str {
        self.inner.auth_token()
    }

    /// Check if the bridge is running.
    #[getter]
    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Register a tool handler.
    ///
    /// Args:
    ///     name: Tool name (unique identifier).
    ///     description: Human-readable description.
    ///     handler: Python callable that handles the tool call.
    fn register_tool(
        &mut self,
        py: Python<'_>,
        name: String,
        description: String,
        handler: Py<PyAny>,
    ) -> PyResult<()> {
        // Verify handler is callable
        if !handler.bind(py).is_callable() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "handler must be callable",
            ));
        }

        // Store the Python handler
        self.py_handlers.insert(name.clone(), handler.clone_ref(py));

        // Register a Rust handler that calls the Python handler
        // Note: This requires GIL access, which we handle via Python::attach
        let handler_clone = handler.clone_ref(py);
        self.inner.register_tool(name, description, move |args| {
            Python::attach(|py| {
                // Convert JSON args to Python dict
                let py_args = match pythonize::pythonize(py, &args) {
                    Ok(obj) => obj,
                    Err(e) => {
                        return Err(format!("Failed to convert args: {}", e));
                    }
                };

                // Call the Python handler
                match handler_clone.call1(py, (py_args,)) {
                    Ok(result) => {
                        // Convert result back to JSON
                        match pythonize::depythonize::<serde_json::Value>(&result.into_bound(py)) {
                            Ok(json) => Ok(json),
                            Err(e) => Err(format!("Failed to convert result: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Handler error: {}", e)),
                }
            })
        });

        Ok(())
    }

    /// Start the IPC Server.
    fn start(&mut self) -> PyResult<()> {
        self.inner
            .start()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    }

    /// Stop the IPC Server.
    fn stop(&mut self) {
        self.inner.stop();
    }

    /// Get environment variables for launching the Sidecar.
    fn get_sidecar_env(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let env = self.inner.get_sidecar_env();
        let dict = PyDict::new(py);
        for (k, v) in env {
            dict.set_item(k, v)?;
        }
        Ok(dict.into())
    }
}

/// Register sidecar bindings in the module.
pub fn register_sidecar_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySidecarBridge>()?;
    Ok(())
}

