//! Plugin system integration for AuroraView
//!
//! This module provides Python bindings for the plugin system,
//! allowing JavaScript to invoke plugin commands like file system operations.

use auroraview_core::plugins::{
    PathScope, PluginEventCallback, PluginRequest, PluginResponse, PluginRouter, ScopeConfig,
};
use pyo3::prelude::*;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::ipc::json::json_to_python;

/// Thread-safe plugin router wrapper
#[pyclass]
pub struct PluginManager {
    router: Arc<RwLock<PluginRouter>>,
    /// Python callback for emitting events (kept alive)
    #[allow(dead_code)]
    py_callback: Arc<RwLock<Option<Py<PyAny>>>>,
}

#[pymethods]
impl PluginManager {
    /// Create a new plugin manager with default configuration
    #[new]
    pub fn new() -> Self {
        Self {
            router: Arc::new(RwLock::new(PluginRouter::new())),
            py_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a permissive plugin manager (allows all file system access)
    #[staticmethod]
    pub fn permissive() -> Self {
        Self {
            router: Arc::new(RwLock::new(PluginRouter::with_scope(
                ScopeConfig::permissive(),
            ))),
            py_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the event callback for plugins to emit events
    ///
    /// The callback should accept two arguments: (event_name: str, data: dict)
    ///
    /// Example:
    ///     def on_plugin_event(event_name, data):
    ///         print(f"Event: {event_name}, Data: {data}")
    ///
    ///     plugins.set_emit_callback(on_plugin_event)
    ///
    /// This enables ProcessPlugin to emit events like:
    ///     - process:stdout - { pid, data }
    ///     - process:stderr - { pid, data }
    ///     - process:exit - { pid, code }
    pub fn set_emit_callback(&self, py: Python<'_>, callback: Py<PyAny>) -> PyResult<()> {
        // Store the Python callback to keep it alive
        {
            let mut py_cb = self.py_callback.write().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
            })?;
            *py_cb = Some(callback.clone_ref(py));
        }

        // Create a Rust callback that calls the Python callback
        let py_callback = callback;
        #[allow(deprecated)]
        let rust_callback: PluginEventCallback = Arc::new(move |event_name: &str, data: Value| {
            Python::with_gil(|py| {
                // Convert Value to Python dict
                let py_data = match json_to_python(py, &data) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("Failed to convert event data to Python: {}", e);
                        return;
                    }
                };

                // Call the Python callback
                if let Err(e) = py_callback.call1(py, (event_name, py_data)) {
                    tracing::error!("Plugin event callback error: {}", e);
                }
            });
        });

        // Set the callback on the router
        let router = self.router.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;
        router.set_event_callback(rust_callback);

        Ok(())
    }

    /// Clear the event callback
    pub fn clear_emit_callback(&self) -> PyResult<()> {
        // Clear Python callback
        {
            let mut py_cb = self.py_callback.write().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
            })?;
            *py_cb = None;
        }

        // Clear router callback
        let router = self.router.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;
        router.clear_event_callback();

        Ok(())
    }

    /// Set allowed file system paths
    #[pyo3(signature = (paths, allow_all=false))]
    pub fn set_fs_scope(&self, paths: Vec<String>, allow_all: bool) -> PyResult<()> {
        let mut router = self.router.write().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        let mut scope = if allow_all {
            PathScope::allow_all()
        } else {
            PathScope::new()
        };

        for path in paths {
            scope = scope.allow(PathBuf::from(path));
        }

        let mut config = router.scope().clone();
        config.fs = scope;
        router.set_scope(config);

        Ok(())
    }

    /// Add denied paths to file system scope
    pub fn deny_fs_paths(&self, paths: Vec<String>) -> PyResult<()> {
        let mut router = self.router.write().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        let mut config = router.scope().clone();
        for path in paths {
            config.fs = config.fs.clone().deny(PathBuf::from(path));
        }
        router.set_scope(config);

        Ok(())
    }

    /// Enable a plugin
    pub fn enable_plugin(&self, name: &str) -> PyResult<()> {
        let mut router = self.router.write().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        let mut config = router.scope().clone();
        config.enable_plugin(name);
        router.set_scope(config);

        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&self, name: &str) -> PyResult<()> {
        let mut router = self.router.write().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        let mut config = router.scope().clone();
        config.disable_plugin(name);
        router.set_scope(config);

        Ok(())
    }

    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, name: &str) -> PyResult<bool> {
        let router = self.router.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        Ok(router.scope().is_plugin_enabled(name))
    }

    /// Get list of enabled plugins
    pub fn enabled_plugins(&self) -> PyResult<Vec<String>> {
        let router = self.router.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        Ok(router.scope().enabled_plugins.iter().cloned().collect())
    }

    /// Handle a plugin command (internal use)
    pub fn handle_command(&self, invoke_cmd: &str, args_json: &str) -> PyResult<String> {
        let router = self.router.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock poisoned: {}", e))
        })?;

        let args: Value = serde_json::from_str(args_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON: {}", e))
        })?;

        let request = match PluginRequest::from_invoke(invoke_cmd, args) {
            Some(req) => req,
            None => {
                let resp = PluginResponse::err("Invalid plugin command format", "INVALID_FORMAT");
                return Ok(serde_json::to_string(&resp).unwrap());
            }
        };

        let response = router.handle(request);
        Ok(serde_json::to_string(&response).unwrap())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PluginManager {
    fn clone(&self) -> Self {
        Self {
            router: Arc::clone(&self.router),
            py_callback: Arc::clone(&self.py_callback),
        }
    }
}
