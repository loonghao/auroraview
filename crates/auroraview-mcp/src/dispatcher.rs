//! Python-aware MCP Tool Dispatcher
//!
//! This module extends the core McpToolDispatcher trait with Python-specific
//! functionality for passing handlers to the main thread.

use auroraview_core::ipc::{McpToolDispatcher, McpToolFuture};
use pyo3::types::PyAny;
use pyo3::Py;
use serde_json::Value;
use std::sync::Arc;

/// Extended dispatcher trait that supports Python handlers
///
/// This trait extends McpToolDispatcher with the ability to pass
/// Python handlers along with the dispatch request.
pub trait PythonMcpDispatcher: McpToolDispatcher {
    /// Dispatch a tool call with a Python handler
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool to execute
    /// * `args` - JSON arguments for the tool
    /// * `handler` - Python callable to execute on the main thread
    ///
    /// # Returns
    /// A future that resolves to the tool result
    fn dispatch_with_handler(
        &self,
        tool_name: String,
        args: Value,
        handler: Py<PyAny>,
    ) -> McpToolFuture;
}

/// Shared Python dispatcher reference
pub type SharedPythonDispatcher = Arc<dyn PythonMcpDispatcher>;
