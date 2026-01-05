//! Python bindings for MCP Server
//!
//! This module provides PyO3 bindings for the embedded MCP Server.

use std::collections::HashMap;
use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule, PyString};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::config::McpConfig;
use crate::server::McpServer;
use crate::tool::Tool;

/// Python MCP Server configuration
#[pyclass(name = "McpConfig")]
#[derive(Clone)]
pub struct PyMcpConfig {
    /// Inner Rust config (public for access from webview crate)
    pub inner: McpConfig,
}

#[pymethods]
impl PyMcpConfig {
    /// Create a new MCP config
    #[new]
    #[pyo3(signature = (
        name = None,
        version = None,
        host = None,
        port = None,
        auto_expose_api = None,
        expose_events = None,
        expose_dom = None,
        expose_debug = None,
        require_auth = None,
        auth_token = None,
        max_connections = None,
        heartbeat_interval = None,
        timeout = None,
        direct_execution = None
    ))]
    fn new(
        name: Option<String>,
        version: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        auto_expose_api: Option<bool>,
        expose_events: Option<bool>,
        expose_dom: Option<bool>,
        expose_debug: Option<bool>,
        require_auth: Option<bool>,
        auth_token: Option<String>,
        max_connections: Option<usize>,
        heartbeat_interval: Option<u64>,
        timeout: Option<u64>,
        direct_execution: Option<bool>,
    ) -> Self {
        let mut config = McpConfig::default();

        if let Some(v) = name {
            config.name = v;
        }
        if let Some(v) = version {
            config.version = v;
        }
        if let Some(v) = host {
            config.host = v;
        }
        if let Some(v) = port {
            config.port = v;
        }
        if let Some(v) = auto_expose_api {
            config.auto_expose_api = v;
        }
        if let Some(v) = expose_events {
            config.expose_events = v;
        }
        if let Some(v) = expose_dom {
            config.expose_dom = v;
        }
        if let Some(v) = expose_debug {
            config.expose_debug = v;
        }
        if let Some(v) = require_auth {
            config.require_auth = v;
        }
        if let Some(v) = auth_token {
            config.auth_token = Some(v);
        }
        if let Some(v) = max_connections {
            config.max_connections = v;
        }
        if let Some(v) = heartbeat_interval {
            config.heartbeat_interval = v;
        }
        if let Some(v) = timeout {
            config.timeout = v;
        }
        if let Some(v) = direct_execution {
            config.direct_execution = v;
        }

        Self { inner: config }
    }

    /// Get server name
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Set server name
    #[setter(name)]
    fn set_name(&mut self, name: String) {
        self.inner.name = name;
    }

    /// Get server port
    #[getter]
    fn port(&self) -> u16 {
        self.inner.port
    }

    /// Set server port
    #[setter(port)]
    fn set_port(&mut self, port: u16) {
        self.inner.port = port;
    }

    /// Get server host
    #[getter]
    fn host(&self) -> &str {
        &self.inner.host
    }

    /// Set server host
    #[setter(host)]
    fn set_host(&mut self, host: String) {
        self.inner.host = host;
    }

    /// Get auto_expose_api flag
    #[getter]
    fn auto_expose_api(&self) -> bool {
        self.inner.auto_expose_api
    }

    /// Set auto_expose_api flag
    #[setter(auto_expose_api)]
    fn set_auto_expose_api(&mut self, value: bool) {
        self.inner.auto_expose_api = value;
    }

    /// Get max_connections
    #[getter]
    fn max_connections(&self) -> usize {
        self.inner.max_connections
    }

    /// Set max_connections
    #[setter(max_connections)]
    fn set_max_connections(&mut self, value: usize) {
        self.inner.max_connections = value;
    }

    /// Get heartbeat_interval
    #[getter]
    fn heartbeat_interval(&self) -> u64 {
        self.inner.heartbeat_interval
    }

    /// Set heartbeat_interval
    #[setter(heartbeat_interval)]
    fn set_heartbeat_interval(&mut self, value: u64) {
        self.inner.heartbeat_interval = value;
    }

    /// Get direct_execution flag
    #[getter]
    fn direct_execution(&self) -> bool {
        self.inner.direct_execution
    }

    /// Set direct_execution flag
    #[setter(direct_execution)]
    fn set_direct_execution(&mut self, value: bool) {
        self.inner.direct_execution = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "McpConfig(name='{}', host='{}', port={})",
            self.inner.name, self.inner.host, self.inner.port
        )
    }
}

/// Python MCP Server
#[pyclass(name = "McpServer")]
pub struct PyMcpServer {
    server: Arc<McpServer>,
    runtime: Arc<Runtime>,
    handlers: Arc<parking_lot::RwLock<HashMap<String, Py<PyAny>>>>,
    /// Optional dispatcher for routing tool calls to main thread
    dispatcher: Arc<parking_lot::RwLock<Option<crate::dispatcher::SharedPythonDispatcher>>>,
}

#[pymethods]
impl PyMcpServer {
    /// Create a new MCP Server
    #[new]
    #[pyo3(signature = (config = None))]
    fn new(config: Option<PyMcpConfig>) -> PyResult<Self> {
        let config = config.map(|c| c.inner).unwrap_or_default();

        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            server: Arc::new(McpServer::new(config)),
            runtime: Arc::new(runtime),
            handlers: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            dispatcher: Arc::new(parking_lot::RwLock::new(None)),
        })
    }

    /// Start MCP Server
    fn start(&self) -> PyResult<u16> {
        let server = Arc::clone(&self.server);
        self.runtime
            .block_on(async move { server.start().await })
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to start server: {}", e)))
    }

    /// Stop MCP Server
    fn stop(&self) {
        let server = Arc::clone(&self.server);
        self.runtime.block_on(async move {
            server.stop().await;
        });
    }

    /// Check if server is running
    #[getter]
    fn is_running(&self) -> bool {
        self.server.is_running()
    }

    /// Get server port
    #[getter]
    fn port(&self) -> u16 {
        self.server.port()
    }

    /// Register a tool with a Python handler
    #[pyo3(signature = (name, handler, description = None))]
    fn register_tool(
        &self,
        py: Python<'_>,
        name: String,
        handler: Py<PyAny>,
        description: Option<String>,
    ) -> PyResult<()> {
        if !handler.bind(py).is_callable() {
            return Err(PyValueError::new_err("handler must be callable"));
        }

        let description = description.unwrap_or_else(|| {
            // Try to get docstring from function
            handler
                .bind(py)
                .getattr("__doc__")
                .ok()
                .and_then(|doc| doc.extract::<String>().ok())
                .unwrap_or_else(|| format!("Tool: {}", name))
        });

        // Extract parameter information from Python function signature
        let params = extract_function_params(py, &handler)?;

        tracing::info!(
            "[MCP:Python] Registering tool: {} with {} params",
            name,
            params.len()
        );

        // Store handler for later invocation
        self.handlers
            .write()
            .insert(name.clone(), handler.clone_ref(py));

        let mut tool = Tool::new(&name, &description);

        // Add parameters to tool
        for param in params {
            tracing::debug!(
                "[MCP:Python] Tool {} param: {} ({})",
                name,
                param.name,
                param.param_type
            );
            if param.required {
                tool = tool.with_param(&param.name, &param.param_type, &param.description);
            } else {
                tool = tool.with_optional_param(
                    &param.name,
                    &param.param_type,
                    &param.description,
                    param.default,
                );
            }
        }

        // Check if we have a dispatcher for main thread routing
        let has_dispatcher = self.dispatcher.read().is_some();

        if has_dispatcher {
            // Use async handler that routes through dispatcher to main thread
            // We need to pass the handler along with the dispatch request
            let dispatcher = Arc::clone(&self.dispatcher);
            let handlers = Arc::clone(&self.handlers);
            let tool_name = name.clone();

            tracing::info!(
                "[MCP:Python] Tool {} will use dispatcher for main thread execution",
                name
            );

            tool = tool.with_async_handler(move |args: Value| {
                let dispatcher = Arc::clone(&dispatcher);
                let handlers = Arc::clone(&handlers);
                let tool_name = tool_name.clone();

                async move {
                    tracing::info!(
                        "[MCP:Python] Dispatching tool: {} to main thread",
                        tool_name
                    );

                    // Get the handler for this tool
                    let handler = {
                        let handlers_guard = handlers.read();
                        let handler_ref = handlers_guard.get(&tool_name).ok_or_else(|| {
                            crate::error::McpError::Internal(format!(
                                "Handler not found for tool: {}",
                                tool_name
                            ))
                        })?;
                        // Clone the Py<PyAny> - this is safe and increments the reference count
                        Python::attach(|py| handler_ref.clone_ref(py))
                    };

                    // Get the dispatcher and clone it to avoid holding the guard across await
                    let dispatcher_arc = {
                        let dispatcher_guard = dispatcher.read();
                        dispatcher_guard.as_ref().cloned().ok_or_else(|| {
                            crate::error::McpError::Internal(
                                "Dispatcher was removed after tool registration".to_string(),
                            )
                        })?
                    };

                    // Dispatch with handler (guard is now released)
                    dispatcher_arc
                        .dispatch_with_handler(tool_name.clone(), args, handler)
                        .await
                        .map_err(|e| crate::error::McpError::ToolExecutionFailed {
                            tool: tool_name,
                            reason: e,
                            suggestion: "Check the main thread event loop is running".to_string(),
                        })
                }
            });
        } else {
            // Use sync handler that calls Python directly (legacy behavior)
            let handlers: Arc<parking_lot::RwLock<HashMap<String, Py<PyAny>>>> =
                Arc::clone(&self.handlers);
            let tool_name = name.clone();
            let tool_name_for_log = name.clone();

            tracing::info!(
                "[MCP:Python] Tool {} will use direct Python execution (no dispatcher)",
                name
            );

            tool = tool.with_handler(move |args: Value| {
                tracing::info!(
                    "[MCP:Python] Calling tool: {} with args: {}",
                    tool_name_for_log,
                    args
                );

                #[allow(deprecated)]
                Python::with_gil(|py_gil_ref| {
                    let handlers = handlers.read();
                    let handler = handlers.get(&tool_name).ok_or_else(|| {
                        crate::error::McpError::Internal(format!(
                            "Handler not found for tool: {}. Please restart the MCP server.",
                            tool_name
                        ))
                    })?;

                    // Check if args is an empty object - if so, call without arguments
                    let result = match &args {
                        Value::Object(map) if map.is_empty() => {
                            // No arguments - call function without parameters
                            handler.call0(py_gil_ref).map_err(|e| {
                                tracing::error!(
                                    "[MCP:Python] Tool {} error (no args): {}",
                                    tool_name,
                                    e
                                );
                                crate::error::McpError::ToolExecutionFailed {
                                    tool: tool_name.clone(),
                                    reason: format!("Python execution error: {}", e),
                                    suggestion: "Check the Python function implementation"
                                        .to_string(),
                                }
                            })?
                        }
                        Value::Null => {
                            // Null args - call function without parameters
                            handler.call0(py_gil_ref).map_err(|e| {
                                tracing::error!(
                                    "[MCP:Python] Tool {} error (null args): {}",
                                    tool_name,
                                    e
                                );
                                crate::error::McpError::ToolExecutionFailed {
                                    tool: tool_name.clone(),
                                    reason: format!("Python execution error: {}", e),
                                    suggestion: "Check the Python function implementation"
                                        .to_string(),
                                }
                            })?
                        }
                        _ => {
                            // Has arguments - convert and pass as kwargs
                            let py_kwargs = json_to_py(py_gil_ref, &args)?;
                            // Use call with kwargs - Python functions expect keyword arguments
                            let kwargs_dict = py_kwargs.downcast_bound::<pyo3::types::PyDict>(py_gil_ref)
                                .map_err(|_| {
                                    crate::error::McpError::ToolExecutionFailed {
                                        tool: tool_name.clone(),
                                        reason: "Arguments must be a JSON object".to_string(),
                                        suggestion: "Ensure MCP tool arguments are passed as an object".to_string(),
                                    }
                                })?;
                            // PyO3 0.27: Use bind().call() and unbind() to get Py<PyAny>
                            handler.bind(py_gil_ref).call((), Some(kwargs_dict)).map_err(|e| {
                                tracing::error!("[MCP:Python] Tool {} error: {}", tool_name, e);
                                crate::error::McpError::ToolExecutionFailed {
                                    tool: tool_name.clone(),
                                    reason: format!("Python execution error: {}", e),
                                    suggestion: "Check the Python function implementation"
                                        .to_string(),
                                }
                            })?.unbind()
                        }
                    };

                    // Convert result back to JSON
                    let json_result = py_to_json(py_gil_ref, result.bind(py_gil_ref))?;

                    tracing::info!("[MCP:Python] Tool {} returned successfully", tool_name);

                    Ok(json_result)
                })
            });
        }

        self.server.register_tool(tool);
        Ok(())
    }

    /// Register a tool using a decorator pattern
    #[pyo3(signature = (name = None, description = None))]
    fn tool<'py>(
        &self,
        py: Python<'py>,
        name: Option<String>,
        description: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let server = self.clone_ref();

        // Create a decorator function
        let code = r#"
from typing import Callable

def make_decorator(server, name, description):
    def decorator(func: Callable):
        tool_name = name or func.__name__
        tool_desc = description or func.__doc__ or f"Tool: {tool_name}"
        server._register_tool_internal(tool_name, func, tool_desc)
        return func
    return decorator
"#;
        use std::ffi::CString;
        let code_cstr = CString::new(code).unwrap();
        let filename_cstr = CString::new("decorator.py").unwrap();
        let modulename_cstr = CString::new("decorator").unwrap();
        let decorator = PyModule::from_code(
            py,
            code_cstr.as_c_str(),
            filename_cstr.as_c_str(),
            modulename_cstr.as_c_str(),
        )?;

        decorator
            .getattr("make_decorator")?
            .call1((server.into_pyobject(py)?, name, description))
    }

    /// Internal method for decorator registration
    fn _register_tool_internal(
        &self,
        py: Python<'_>,
        name: String,
        handler: Py<PyAny>,
        description: String,
    ) -> PyResult<()> {
        self.register_tool(py, name, handler, Some(description))
    }

    /// Register a prompt with Python handler
    #[pyo3(signature = (name, handler, description))]
    fn register_prompt(
        &self,
        py: Python<'_>,
        name: String,
        handler: Py<PyAny>,
        description: String,
    ) -> PyResult<()> {
        if !handler.bind(py).is_callable() {
            return Err(PyValueError::new_err("handler must be callable"));
        }

        // Try to get docstring from function
        let prompt_desc = if description.is_empty() {
            handler
                .bind(py)
                .getattr("__doc__")
                .ok()
                .and_then(|doc| doc.extract::<String>().ok())
                .unwrap_or_else(|| format!("Prompt: {}", name))
        } else {
            description
        };

        let prompt = crate::tool::Prompt::new(&name, &prompt_desc);

        // Store handler for later invocation
        let handler_key = format!("prompt_{}", name);
        self.handlers
            .write()
            .insert(handler_key.clone(), handler.clone_ref(py));

        // Create handler for prompt execution
        let handlers_arc = Arc::clone(&self.handlers);
        let prompt_name = name.clone();
        let prompt_desc_clone = prompt_desc.clone();
        let prompt = prompt.with_handler(Arc::new(move |args: serde_json::Map<String, Value>| {
            #[allow(deprecated)]
            Python::with_gil(|py_gil_ref| {
                let handlers = handlers_arc.read();
                let handler = handlers.get(&handler_key).ok_or_else(|| {
                    crate::error::McpError::Internal(
                        format!("Handler not found for prompt: {}. This may indicate an internal inconsistency. Please restart the MCP server.", prompt_name)
                    )
                })?;

                // Convert JSON args to Python dict
                let py_args = json_to_py(py_gil_ref, &serde_json::Value::Object(args))?;

                // Call Python function
                let result = handler.call1(py_gil_ref, (py_args,)).map_err(|e| {
                    tracing::error!("[MCP:Python] Prompt {} error: {}", prompt_name, e);
                    crate::error::McpError::ToolExecutionFailed {
                        tool: prompt_name.clone(),
                        reason: format!("Python execution error: {}", e),
                        suggestion: "Check the Python function implementation and ensure it returns valid JSON-serializable data. Review Python traceback for details.".to_string(),
                    }
                })?;

                // Convert Python result to JSON
                let json_result = py_to_json(py_gil_ref, result.bind(py_gil_ref))
                    .map_err(|e| {
                        crate::error::McpError::ToolExecutionFailed {
                            tool: prompt_name.clone(),
                            reason: format!("Failed to convert Python result to JSON: {}", e),
                            suggestion: "Ensure the Python function returns a JSON-serializable value.".to_string(),
                        }
                    })?;

                // Create GetPromptResult
                let prompt_definition = crate::protocol::PromptDefinition {
                    name: prompt_name.clone(),
                    description: prompt_desc_clone.clone(),
                    arguments: None,
                };

                let message_text = serde_json::to_string_pretty(&json_result)
                    .unwrap_or_else(|_| json_result.to_string());

                let prompt_result = crate::protocol::GetPromptResult::new(prompt_definition)
                    .with_user_message(message_text);

                Ok(prompt_result)
            })
        }));

        let server = Arc::clone(&self.server);
        server
            .register_prompt(prompt)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to register prompt: {}", e)))?;

        Ok(())
    }

    /// Internal method for prompt decorator
    #[pyo3(signature = (name, handler))]
    fn _register_prompt_internal(
        &self,
        py: Python<'_>,
        name: String,
        handler: Py<PyAny>,
    ) -> PyResult<()> {
        // Get docstring from handler
        let description = handler
            .bind(py)
            .getattr("__doc__")
            .ok()
            .and_then(|doc| doc.extract::<String>().ok())
            .unwrap_or_else(|| format!("Prompt: {}", name));
        self.register_prompt(py, name, handler, description)
    }

    /// List all registered tools
    fn list_tools(&self) -> Vec<String> {
        self.server.tools().list()
    }

    /// Register a prompt using a decorator pattern
    #[pyo3(signature = (name = None, description = None))]
    fn prompt<'py>(
        &self,
        py: Python<'py>,
        name: Option<String>,
        description: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let server = self.clone_ref();

        // Create a decorator function
        let code = r#"
from typing import Callable, Any, Dict

def make_prompt_decorator(server, name, description):
    def decorator(func: Callable):
        prompt_name = name or func.__name__
        prompt_desc = description or func.__doc__ or f"Prompt: {prompt_name}"
        server._register_prompt_internal(prompt_name, func, prompt_desc)
        return func
    return decorator
"#;

        use std::ffi::CString;
        let code_cstr = CString::new(code).unwrap();
        let filename_cstr = CString::new("prompt_decorator.py").unwrap();
        let modulename_cstr = CString::new("prompt_decorator").unwrap();
        let decorator = PyModule::from_code(
            py,
            code_cstr.as_c_str(),
            filename_cstr.as_c_str(),
            modulename_cstr.as_c_str(),
        )?;

        decorator.getattr("make_prompt_decorator")?.call1((
            server.into_pyobject(py)?,
            name,
            description,
        ))
    }

    /// Get tool count
    fn __len__(&self) -> usize {
        self.server.tools().len()
    }

    fn __repr__(&self) -> String {
        format!(
            "McpServer(running={}, port={}, tools={}, prompts={})",
            self.is_running(),
            self.port(),
            self.server.tools().len(),
            self.server.prompts().read().len()
        )
    }

    /// Clean up resources when the Python object is garbage collected
    fn __del__(&self) {
        self.shutdown();
    }

    /// Explicitly shutdown the server and cleanup resources
    fn shutdown(&self) {
        if self.server.is_running() {
            tracing::info!("[MCP:Python] Shutting down MCP server...");
            let server = Arc::clone(&self.server);
            self.runtime.block_on(async move {
                server.stop().await;
            });
        }
        // Clear handlers to release Python references
        self.handlers.write().clear();
    }
}

impl Drop for PyMcpServer {
    fn drop(&mut self) {
        // Stop the server if still running
        if self.server.is_running() {
            tracing::debug!("[MCP:Python] Drop: stopping MCP server");
            let server = Arc::clone(&self.server);
            // Use block_on to ensure server is stopped before runtime is dropped
            self.runtime.block_on(async move {
                server.stop().await;
            });
        }
    }
}

impl PyMcpServer {
    fn clone_ref(&self) -> Self {
        Self {
            server: Arc::clone(&self.server),
            runtime: Arc::clone(&self.runtime),
            handlers: Arc::clone(&self.handlers),
            dispatcher: Arc::clone(&self.dispatcher),
        }
    }

    /// Create a new MCP Server from Rust code
    /// This is the public constructor for use from other Rust crates
    pub fn new_from_rust(config: Option<PyMcpConfig>) -> pyo3::PyResult<Self> {
        use pyo3::exceptions::PyRuntimeError;

        let config = config.map(|c| c.inner).unwrap_or_default();

        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            server: Arc::new(McpServer::new(config)),
            runtime: Arc::new(runtime),
            handlers: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            dispatcher: Arc::new(parking_lot::RwLock::new(None)),
        })
    }

    /// Set the dispatcher for routing tool calls to main thread
    /// This is called from Rust code, not from Python
    pub fn set_dispatcher(&self, dispatcher: crate::dispatcher::SharedPythonDispatcher) {
        *self.dispatcher.write() = Some(dispatcher);
        tracing::info!("[MCP:Python] Dispatcher set for main thread routing");
    }

    /// Check if a dispatcher is configured
    pub fn has_dispatcher(&self) -> bool {
        self.dispatcher.read().is_some()
    }
}

/// Parameter information extracted from Python function signature
struct ParamInfo {
    name: String,
    param_type: String,
    description: String,
    required: bool,
    default: Option<Value>,
}

/// Extract parameter information from a Python function using inspect module
fn extract_function_params(py: Python<'_>, handler: &Py<PyAny>) -> PyResult<Vec<ParamInfo>> {
    let inspect = py.import("inspect")?;
    let signature = inspect.call_method1("signature", (handler,))?;
    let parameters = signature.getattr("parameters")?;

    let mut params = Vec::new();

    // Iterate over parameters
    for item in parameters.call_method0("items")?.try_iter()? {
        let item = item?;
        let tuple = item.cast::<pyo3::types::PyTuple>()?;
        let name: String = tuple.get_item(0)?.extract()?;
        let param = tuple.get_item(1)?;

        // Get parameter kind
        let kind = param.getattr("kind")?;
        let kind_name: String = kind.getattr("name")?.extract()?;

        // Skip *args and **kwargs
        if kind_name == "VAR_POSITIONAL" || kind_name == "VAR_KEYWORD" {
            continue;
        }

        // Get annotation (type hint)
        let annotation = param.getattr("annotation")?;
        let param_type = if annotation.is(&inspect.getattr("Parameter")?.getattr("empty")?) {
            "string".to_string() // Default to string if no type hint
        } else {
            python_type_to_json_type(py, &annotation)
        };

        // Get default value
        let default_attr = param.getattr("default")?;
        let empty = inspect.getattr("Parameter")?.getattr("empty")?;
        let (required, default) = if default_attr.is(&empty) {
            (true, None)
        } else {
            let default_value = py_to_json(py, &default_attr).ok();
            (false, default_value)
        };

        params.push(ParamInfo {
            name,
            param_type,
            description: String::new(), // Could be extracted from docstring in the future
            required,
            default,
        });
    }

    Ok(params)
}

/// Convert Python type annotation to JSON Schema type
fn python_type_to_json_type(py: Python<'_>, annotation: &Bound<'_, PyAny>) -> String {
    // Get the type name
    let type_name = annotation
        .getattr("__name__")
        .or_else(|_| annotation.call_method0("__str__"))
        .and_then(|n| n.extract::<String>())
        .unwrap_or_else(|_| "string".to_string());

    match type_name.as_str() {
        "str" => "string",
        "int" => "integer",
        "float" => "number",
        "bool" => "boolean",
        "list" | "List" => "array",
        "dict" | "Dict" => "object",
        "NoneType" => "null",
        // Handle Optional[T] and Union types
        _ if type_name.starts_with("Optional") => {
            // Try to extract the inner type
            if let Ok(args) = annotation.getattr("__args__") {
                if let Ok(first) = args.get_item(0) {
                    return python_type_to_json_type(py, &first);
                }
            }
            "string"
        }
        _ => "string", // Default to string for unknown types
    }
    .to_string()
}

/// Convert JSON Value to Python object
fn json_to_py(py: Python<'_>, value: &Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().unbind().into()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().unbind().into())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().unbind().into())
            } else {
                Err(PyValueError::new_err("Invalid number"))
            }
        }
        Value::String(s) => Ok(PyString::new(py, s).into()),
        Value::Array(arr) => {
            let list: Vec<Py<PyAny>> = arr
                .iter()
                .map(|v| json_to_py(py, v))
                .collect::<PyResult<_>>()?;
            let py_list = PyList::new(py, list)?;
            Ok(py_list.unbind().into())
        }
        Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.unbind().into())
        }
    }
}

/// Convert Python object to JSON Value
fn py_to_json(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }

    if let Ok(b) = obj.extract::<bool>() {
        return Ok(Value::Bool(b));
    }

    if let Ok(i) = obj.extract::<i64>() {
        return Ok(Value::Number(i.into()));
    }

    if let Ok(f) = obj.extract::<f64>() {
        return Ok(serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null));
    }

    if let Ok(s) = obj.extract::<String>() {
        return Ok(Value::String(s));
    }

    if let Ok(list) = obj.cast::<PyList>() {
        let arr: Vec<Value> = list
            .iter()
            .map(|item| py_to_json(py, &item))
            .collect::<PyResult<_>>()?;
        return Ok(Value::Array(arr));
    }

    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key = k.extract::<String>()?;
            let value = py_to_json(py, &v)?;
            map.insert(key, value);
        }
        return Ok(Value::Object(map));
    }

    // Fallback: try to convert to string
    let s = obj.str()?.to_string();
    Ok(Value::String(s))
}

/// Register MCP module functions
pub fn register_mcp_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMcpConfig>()?;
    m.add_class::<PyMcpServer>()?;
    Ok(())
}
