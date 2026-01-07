//! JSON-RPC 2.0 Protocol Implementation
//!
//! This module implements the JSON-RPC 2.0 specification for IPC communication
//! between the MCP Sidecar and the main AuroraView process.
//!
//! Reference: <https://www.jsonrpc.org/specification>

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// Error codes for JSON-RPC protocol.
///
/// Standard JSON-RPC error codes are in the range -32768 to -32000.
/// Application-specific error codes are >= 1000.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    // Standard JSON-RPC errors
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,

    // Application-specific errors (1000+)
    ToolNotFound = 1001,
    InvalidArguments = 1002,
    ExecutionError = 1003,
    Timeout = 1004,
    Cancelled = 1005,
    ShuttingDown = 1006,
    AuthenticationFailed = 1007,
    ConnectionError = 1008,
}

impl ErrorCode {
    /// Convert to i32 for serialization.
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// JSON-RPC 2.0 Request object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,

    /// Request ID (can be number, string, or null for notifications).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,

    /// Method name to invoke.
    pub method: String,

    /// Method parameters (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Request ID type (can be number or string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

/// JSON-RPC 2.0 Response object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,

    /// Request ID (must match the request).
    pub id: Option<RequestId>,

    /// Result (present on success, mutually exclusive with error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (present on failure, mutually exclusive with result).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// JSON-RPC 2.0 Error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Error code.
    pub code: i32,

    /// Error message.
    pub message: String,

    /// Additional error data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Request {
    /// Create a new JSON-RPC request.
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id.into()),
            method: method.into(),
            params: None,
        }
    }

    /// Create a new JSON-RPC request with parameters.
    pub fn with_params(id: impl Into<RequestId>, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id.into()),
            method: method.into(),
            params: Some(params),
        }
    }

    /// Create a notification (request without ID, no response expected).
    pub fn notification(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
            method: method.into(),
            params: None,
        }
    }

    /// Create a notification with parameters.
    pub fn notification_with_params(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
            method: method.into(),
            params: Some(params),
        }
    }

    /// Check if this is a notification (no response expected).
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

impl Response {
    /// Create a success response.
    pub fn success(id: Option<RequestId>, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<RequestId>, error: RpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Create an error response from error code.
    pub fn error_with_code(
        id: Option<RequestId>,
        code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self::error(
            id,
            RpcError {
                code: code.as_i32(),
                message: message.into(),
                data: None,
            },
        )
    }

    /// Check if this response is successful.
    pub fn is_success(&self) -> bool {
        self.result.is_some() && self.error.is_none()
    }

    /// Check if this response is an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

impl RpcError {
    /// Create a new RPC error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.as_i32(),
            message: message.into(),
            data: None,
        }
    }

    /// Create a new RPC error with additional data.
    pub fn with_data(code: ErrorCode, message: impl Into<String>, data: Value) -> Self {
        Self {
            code: code.as_i32(),
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a "tool not found" error.
    pub fn tool_not_found(name: &str) -> Self {
        Self::with_data(
            ErrorCode::ToolNotFound,
            format!("Tool not found: {}", name),
            serde_json::json!({ "name": name }),
        )
    }

    /// Create an "execution error" from an exception message.
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ExecutionError, message)
    }

    /// Create a "timeout" error.
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::with_data(
            ErrorCode::Timeout,
            format!("Operation timed out after {}ms", timeout_ms),
            serde_json::json!({ "timeout_ms": timeout_ms }),
        )
    }

    /// Create an "authentication failed" error.
    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::new(ErrorCode::AuthenticationFailed, reason)
    }
}

/// Tool call parameters for `tool.call` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    /// Tool name to call.
    pub name: String,

    /// Tool arguments.
    #[serde(default)]
    pub arguments: Value,

    /// Timeout in milliseconds (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Trace ID for debugging (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Tool definition for `tool.list` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,

    /// Tool description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input JSON Schema.
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,

    /// Output JSON Schema (optional).
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

/// Auth hello parameters for `auth.hello` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHelloParams {
    /// Authentication token.
    pub token: String,
}

/// Lifecycle ready parameters for `lifecycle.ready` notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleReadyParams {
    /// MCP server port.
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request::new(1i64, "tool.call");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tool.call\""));
    }

    #[test]
    fn test_response_success() {
        let resp = Response::success(Some(1i64.into()), serde_json::json!({"ok": true}));
        assert!(resp.is_success());
        assert!(!resp.is_error());
    }

    #[test]
    fn test_response_error() {
        let resp =
            Response::error_with_code(Some(1i64.into()), ErrorCode::ToolNotFound, "Tool not found");
        assert!(!resp.is_success());
        assert!(resp.is_error());
    }
}
