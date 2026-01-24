//! IPC message types

use serde::{Deserialize, Serialize};

/// IPC message from JavaScript to Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Message type: "event", "call", "invoke"
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Event name
    #[serde(default)]
    pub event: Option<String>,

    /// Method name for calls
    #[serde(default)]
    pub method: Option<String>,

    /// Command for plugin invokes
    #[serde(default)]
    pub cmd: Option<String>,

    /// Parameters
    #[serde(default)]
    pub params: Option<serde_json::Value>,

    /// Arguments for plugin invokes
    #[serde(default)]
    pub args: Option<serde_json::Value>,

    /// Request ID for response matching
    #[serde(default)]
    pub id: Option<String>,

    /// Event detail
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
}

/// IPC response to JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Response type
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Request ID
    pub id: String,

    /// Success flag
    pub ok: bool,

    /// Result data (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error info (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}

/// IPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub name: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl IpcResponse {
    /// Create success response
    pub fn ok(id: String, result: serde_json::Value) -> Self {
        Self {
            msg_type: "call_result".to_string(),
            id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    /// Create error response
    pub fn err(id: String, name: &str, message: &str) -> Self {
        Self {
            msg_type: "call_result".to_string(),
            id,
            ok: false,
            result: None,
            error: Some(IpcError {
                name: name.to_string(),
                message: message.to_string(),
                code: None,
            }),
        }
    }
}
