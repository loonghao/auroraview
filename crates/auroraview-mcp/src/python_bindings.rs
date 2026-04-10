//! PyO3 Python bindings for AuroraView MCP Server.
//!
//! Exposes `McpServer` and `McpConfig` to Python so DCC plugins can
//! start/stop the MCP server without managing tokio runtimes manually.
//!
//! # Usage (Python)
//!
//! ```python
//! from auroraview import McpServer
//!
//! server = McpServer(port=7890)
//! server.start()               # non-blocking, runs in background thread
//! server.emit_run_started("run-1", "thread-1")
//! server.stop()
//! ```
//!
//! # Feature gate
//!
//! This module is only compiled when the `python-bindings` Cargo feature is
//! enabled (e.g. via `maturin build --features python-bindings`).

use crate::{
    agui::AguiEvent,
    runner::McpRunner,
    types::McpServerConfig,
};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

// ---------------------------------------------------------------------------
// PyMcpConfig
// ---------------------------------------------------------------------------

/// Python-facing configuration for the AuroraView MCP server.
///
/// Mirrors [`McpServerConfig`] with Python-friendly defaults.
pub struct PyMcpConfig {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub enable_mdns: bool,
    /// Maximum concurrent WebViews (`None` = unlimited).
    pub max_webviews: Option<usize>,
}

impl PyMcpConfig {
    pub fn new(
        host: String,
        port: u16,
        service_name: String,
        enable_mdns: bool,
        max_webviews: Option<usize>,
    ) -> Self {
        Self {
            host,
            port,
            service_name,
            enable_mdns,
            max_webviews,
        }
    }
}

impl Default for PyMcpConfig {
    fn default() -> Self {
        let cfg = McpServerConfig::default();
        Self {
            host: cfg.host,
            port: cfg.port,
            service_name: cfg.service_name,
            enable_mdns: cfg.enable_mdns,
            max_webviews: cfg.max_webviews,
        }
    }
}

impl From<PyMcpConfig> for McpServerConfig {
    fn from(py: PyMcpConfig) -> Self {
        Self {
            host: py.host,
            port: py.port,
            service_name: py.service_name,
            enable_mdns: py.enable_mdns,
            max_webviews: py.max_webviews,
        }
    }
}

// ---------------------------------------------------------------------------
// PyMcpServer — the primary Python-facing type
// ---------------------------------------------------------------------------

/// State shared between Python references after `start()`.
struct RunnerState {
    runtime: Runtime,
    runner: McpRunner,
}

/// AuroraView MCP Server — Python binding.
///
/// Manages its own tokio [`Runtime`] so it runs without conflicting with the
/// DCC application's existing event loop (Qt, Maya, Blender, etc.).
pub struct PyMcpServer {
    config: McpServerConfig,
    state: Arc<Mutex<Option<RunnerState>>>,
}

impl PyMcpServer {
    /// Create a new server with the given port (other settings at defaults).
    pub fn new(port: u16) -> Self {
        let config = McpServerConfig {
            port,
            ..McpServerConfig::default()
        };
        Self {
            config,
            state: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a server from explicit configuration.
    pub fn from_config(config: McpServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(None)),
        }
    }

    /// Port this server listens on.
    pub fn port(&self) -> u16 {
        self.config.port
    }

    /// Host address this server binds to.
    pub fn host(&self) -> &str {
        &self.config.host
    }

    /// Start the MCP server in a background thread (non-blocking).
    ///
    /// Returns an error string if the server is already running or the port
    /// is in use.
    pub fn start(&self) -> Result<(), String> {
        let mut lock = self.state.lock().map_err(|e| e.to_string())?;
        if lock.is_some() {
            return Err(format!(
                "MCP server already running on port {}",
                self.config.port
            ));
        }

        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let runner = McpRunner::new(self.config.clone());
        runtime.block_on(runner.start()).map_err(|e| e.to_string())?;
        *lock = Some(RunnerState { runtime, runner });
        Ok(())
    }

    /// Stop the running server (no-op if not running).
    pub fn stop(&self) -> Result<(), String> {
        let mut lock = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(state) = lock.take() {
            state.runtime.block_on(state.runner.stop());
        }
        Ok(())
    }

    /// Return `true` if the server is currently running.
    pub fn is_running(&self) -> bool {
        let lock = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        if let Some(state) = lock.as_ref() {
            state.runtime.block_on(state.runner.is_running())
        } else {
            false
        }
    }

    /// Emit an AG-UI `RunStarted` event.
    pub fn emit_run_started(&self, run_id: &str, thread_id: &str) -> Result<(), String> {
        self.emit_event(AguiEvent::RunStarted {
            run_id: run_id.to_string(),
            thread_id: thread_id.to_string(),
        })
    }

    /// Emit an AG-UI `RunFinished` event.
    pub fn emit_run_finished(&self, run_id: &str, thread_id: &str) -> Result<(), String> {
        self.emit_event(AguiEvent::RunFinished {
            run_id: run_id.to_string(),
            thread_id: thread_id.to_string(),
        })
    }

    /// Emit an AG-UI `ToolCallStart` event.
    pub fn emit_tool_call_start(
        &self,
        run_id: &str,
        tool_call_id: &str,
        tool_name: &str,
    ) -> Result<(), String> {
        self.emit_event(AguiEvent::ToolCallStart {
            run_id: run_id.to_string(),
            tool_call_id: tool_call_id.to_string(),
            tool_name: tool_name.to_string(),
        })
    }

    /// Emit an AG-UI `ToolCallEnd` event.
    pub fn emit_tool_call_end(&self, run_id: &str, tool_call_id: &str) -> Result<(), String> {
        self.emit_event(AguiEvent::ToolCallEnd {
            run_id: run_id.to_string(),
            tool_call_id: tool_call_id.to_string(),
        })
    }

    /// Emit an arbitrary AG-UI `Custom` event.
    pub fn emit_custom(
        &self,
        run_id: &str,
        name: &str,
        data: serde_json::Value,
    ) -> Result<(), String> {
        self.emit_event(AguiEvent::Custom {
            run_id: run_id.to_string(),
            name: name.to_string(),
            data,
        })
    }

    /// Low-level: emit any `AguiEvent` through the bus.
    pub fn emit_event(&self, event: AguiEvent) -> Result<(), String> {
        let lock = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(state) = lock.as_ref() {
            state.runner.emit_agui(event);
            Ok(())
        } else {
            Err("MCP server is not running".to_string())
        }
    }

    /// Return the MCP endpoint URL (e.g. `http://127.0.0.1:7890/mcp`).
    pub fn mcp_url(&self) -> String {
        format!("http://{}:{}/mcp", self.config.host, self.config.port)
    }

    /// Return the AG-UI SSE endpoint URL.
    pub fn agui_url(&self) -> String {
        format!(
            "http://{}:{}/agui/events",
            self.config.host, self.config.port
        )
    }
}

impl Drop for PyMcpServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ---------------------------------------------------------------------------
// pyo3 feature gate — compiled only with `--features python-bindings`
// ---------------------------------------------------------------------------

#[cfg(feature = "python-bindings")]
mod pyo3_impl {
    use super::*;
    use pyo3::prelude::*;

    /// Python class: `McpConfig`
    ///
    /// ```python
    /// from auroraview import McpConfig
    /// cfg = McpConfig(port=7891, host="0.0.0.0", enable_mdns=False)
    /// cfg_limited = McpConfig(port=7892, max_webviews=5)
    /// ```
    #[pyclass(name = "McpConfig")]
    pub struct PyMcpConfigWrapper {
        inner: McpServerConfig,
    }

    #[pymethods]
    impl PyMcpConfigWrapper {
        #[new]
        #[pyo3(signature = (port=7890, host="127.0.0.1", service_name="auroraview-mcp", enable_mdns=true, max_webviews=None))]
        fn new(
            port: u16,
            host: &str,
            service_name: &str,
            enable_mdns: bool,
            max_webviews: Option<usize>,
        ) -> Self {
            Self {
                inner: McpServerConfig {
                    host: host.to_string(),
                    port,
                    service_name: service_name.to_string(),
                    enable_mdns,
                    max_webviews,
                },
            }
        }

        #[getter]
        fn port(&self) -> u16 {
            self.inner.port
        }

        #[getter]
        fn host(&self) -> &str {
            &self.inner.host
        }

        #[getter]
        fn service_name(&self) -> &str {
            &self.inner.service_name
        }

        #[getter]
        fn enable_mdns(&self) -> bool {
            self.inner.enable_mdns
        }

        #[getter]
        fn max_webviews(&self) -> Option<usize> {
            self.inner.max_webviews
        }

        fn __repr__(&self) -> String {
            format!(
                "McpConfig(host={}, port={}, service_name={}, enable_mdns={}, max_webviews={:?})",
                self.inner.host,
                self.inner.port,
                self.inner.service_name,
                self.inner.enable_mdns,
                self.inner.max_webviews,
            )
        }
    }

    /// Python class: `McpServer`
    ///
    /// ```python
    /// from auroraview import McpServer
    ///
    /// server = McpServer(port=7890)
    /// server.start()
    /// server.emit_run_started("run-1", "thread-1")
    /// server.stop()
    /// ```
    #[pyclass(name = "McpServer")]
    pub struct PyMcpServerWrapper {
        inner: Arc<PyMcpServer>,
    }

    #[pymethods]
    impl PyMcpServerWrapper {
        #[new]
        #[pyo3(signature = (port=7890, config=None))]
        fn new(port: u16, config: Option<&PyMcpConfigWrapper>) -> Self {
            let server = if let Some(cfg) = config {
                PyMcpServer::from_config(cfg.inner.clone())
            } else {
                PyMcpServer::new(port)
            };
            Self {
                inner: Arc::new(server),
            }
        }

        /// Start the server (non-blocking).
        fn start(&self) -> PyResult<()> {
            self.inner
                .start()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        /// Stop the server.
        fn stop(&self) -> PyResult<()> {
            self.inner
                .stop()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        /// Return `True` if the server is running.
        fn is_running(&self) -> bool {
            self.inner.is_running()
        }

        /// MCP endpoint URL.
        fn mcp_url(&self) -> String {
            self.inner.mcp_url()
        }

        /// AG-UI SSE endpoint URL.
        fn agui_url(&self) -> String {
            self.inner.agui_url()
        }

        #[getter]
        fn port(&self) -> u16 {
            self.inner.port()
        }

        #[getter]
        fn host(&self) -> &str {
            self.inner.host()
        }

        /// Emit a RunStarted AG-UI event.
        fn emit_run_started(&self, run_id: &str, thread_id: &str) -> PyResult<()> {
            self.inner
                .emit_run_started(run_id, thread_id)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        /// Emit a RunFinished AG-UI event.
        fn emit_run_finished(&self, run_id: &str, thread_id: &str) -> PyResult<()> {
            self.inner
                .emit_run_finished(run_id, thread_id)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        /// Emit a ToolCallStart AG-UI event.
        fn emit_tool_call_start(
            &self,
            run_id: &str,
            tool_call_id: &str,
            tool_name: &str,
        ) -> PyResult<()> {
            self.inner
                .emit_tool_call_start(run_id, tool_call_id, tool_name)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        /// Emit a ToolCallEnd AG-UI event.
        fn emit_tool_call_end(&self, run_id: &str, tool_call_id: &str) -> PyResult<()> {
            self.inner
                .emit_tool_call_end(run_id, tool_call_id)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
        }

        fn __repr__(&self) -> String {
            format!(
                "McpServer(host={}, port={}, running={})",
                self.inner.host(),
                self.inner.port(),
                self.inner.is_running(),
            )
        }
    }

    /// Register MCP types into a PyO3 module.
    ///
    /// Call this from your `#[pymodule]` init function:
    /// ```rust,ignore
    /// use auroraview_mcp::python_bindings::pyo3_impl::register;
    /// register(m)?;
    /// ```
    pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyMcpConfigWrapper>()?;
        m.add_class::<PyMcpServerWrapper>()?;
        Ok(())
    }
}

#[cfg(feature = "python-bindings")]
pub use pyo3_impl::register;
