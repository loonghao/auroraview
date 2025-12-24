//! Plugin request and response types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Plugin command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// Plugin name (e.g., "fs", "clipboard")
    pub plugin: String,
    /// Command name (e.g., "read_file", "write")
    pub command: String,
    /// Command arguments as JSON
    pub args: Value,
    /// Optional request ID for async response
    pub id: Option<String>,
}

impl PluginRequest {
    /// Parse a command string in format "plugin:<plugin>|<command>"
    pub fn from_invoke(invoke_cmd: &str, args: Value) -> Option<Self> {
        if !invoke_cmd.starts_with("plugin:") {
            return None;
        }

        let rest = &invoke_cmd[7..]; // Skip "plugin:"
        let parts: Vec<&str> = rest.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }

        Some(Self {
            plugin: parts[0].to_string(),
            command: parts[1].to_string(),
            args,
            id: None,
        })
    }

    /// Create a new plugin request
    pub fn new(plugin: impl Into<String>, command: impl Into<String>, args: Value) -> Self {
        Self {
            plugin: plugin.into(),
            command: command.into(),
            args,
            id: None,
        }
    }

    /// Set the request ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Plugin command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// Success flag
    pub success: bool,
    /// Response data (if success)
    pub data: Option<Value>,
    /// Error message (if failure)
    pub error: Option<String>,
    /// Error code (if failure)
    pub code: Option<String>,
    /// Request ID (echoed from request)
    pub id: Option<String>,
}

impl PluginResponse {
    /// Create a success response
    pub fn ok(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            code: None,
            id: None,
        }
    }

    /// Create an error response
    pub fn err(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            code: Some(code.into()),
            id: None,
        }
    }

    /// Set the request ID
    pub fn with_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
    }
}
