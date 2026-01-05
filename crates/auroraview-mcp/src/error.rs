//! Error types for MCP Server

use thiserror::Error;

/// MCP Server errors
#[derive(Error, Debug)]
pub enum McpError {
    /// Server is already running
    #[error("MCP Server is already running on port {0}. Use .stop() first or check for existing instances.")]
    AlreadyRunning(u16),

    /// Server failed to start
    #[error("Failed to start MCP Server on port {port}: {reason}. Suggestion: Try a different port or check if the port is already in use.")]
    StartFailed { port: String, reason: String },

    /// Tool not found
    #[error("Tool '{name}' not found. Available tools: {available}. Suggestion: Check the tool name spelling or call tools/list to see all available tools.")]
    ToolNotFound { name: String, available: String },

    /// Invalid tool arguments
    #[error("Invalid arguments for tool '{tool}': {reason}. Suggestion: {suggestion}")]
    InvalidArguments {
        tool: String,
        reason: String,
        suggestion: String,
    },

    /// Tool execution failed
    #[error("Tool '{tool}' execution failed: {reason}. Suggestion: {suggestion}")]
    ToolExecutionFailed {
        tool: String,
        reason: String,
        suggestion: String,
    },

    /// JSON-RPC error
    #[error("JSON-RPC error {code}: {message}")]
    JsonRpc { code: i32, message: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON serialization error: {0}. Suggestion: Check that the data structure is valid and contains only JSON-serializable types.")]
    Json(#[from] serde_json::Error),

    /// Internal error
    #[error("Internal error: {0}. Please report this issue if it persists.")]
    Internal(String),
}

#[cfg(feature = "python")]
impl From<pyo3::PyErr> for McpError {
    fn from(err: pyo3::PyErr) -> Self {
        McpError::ToolExecutionFailed {
            tool: "unknown".to_string(),
            reason: format!("Python error: {}", err),
            suggestion: "Check the Python function implementation and ensure it handles all expected cases. Review the full error message and traceback for details.".to_string(),
        }
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

impl McpError {
    /// Get JSON-RPC error code
    pub fn code(&self) -> i32 {
        match self {
            McpError::ToolNotFound { .. } => -32601, // Method not found
            McpError::InvalidArguments { .. } => -32602, // Invalid params
            McpError::ToolExecutionFailed { .. } => -32000, // Server error
            McpError::JsonRpc { code, .. } => *code,
            _ => -32603, // Internal error
        }
    }

    /// Convert to JSON-RPC error response
    pub fn to_jsonrpc_error(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code(),
            "message": self.to_string()
        })
    }
}
