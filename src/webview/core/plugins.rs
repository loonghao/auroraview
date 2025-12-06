//! Plugin system integration for AuroraView
//!
//! This module provides Python bindings for the plugin system,
//! allowing JavaScript to invoke plugin commands like file system operations.

use auroraview_core::plugins::{
    PathScope, PluginRequest, PluginResponse, PluginRouter, ScopeConfig,
};
use pyo3::prelude::*;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Thread-safe plugin router wrapper
#[pyclass]
pub struct PluginManager {
    router: Arc<RwLock<PluginRouter>>,
}

#[pymethods]
impl PluginManager {
    /// Create a new plugin manager with default configuration
    #[new]
    pub fn new() -> Self {
        Self {
            router: Arc::new(RwLock::new(PluginRouter::new())),
        }
    }

    /// Create a permissive plugin manager (allows all file system access)
    #[staticmethod]
    pub fn permissive() -> Self {
        Self {
            router: Arc::new(RwLock::new(PluginRouter::with_scope(
                ScopeConfig::permissive(),
            ))),
        }
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
        }
    }
}
