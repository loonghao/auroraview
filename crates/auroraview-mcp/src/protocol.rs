//! MCP JSON-RPC protocol types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request ID
    pub id: Option<Value>,

    /// Method name
    pub method: String,

    /// Method parameters
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request ID (matches request)
    pub id: Option<Value>,

    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Create a parse error response
    pub fn parse_error() -> Self {
        Self::error(None, -32700, "Parse error")
    }

    /// Create a method not found error
    pub fn method_not_found(id: Option<Value>, method: &str) -> Self {
        Self::error(id, -32601, format!("Method not found: {}", method))
    }

    /// Create an invalid params error
    pub fn invalid_params(id: Option<Value>, message: impl Into<String>) -> Self {
        Self::error(id, -32602, message)
    }

    /// Create an internal error
    pub fn internal_error(id: Option<Value>, message: impl Into<String>) -> Self {
        Self::error(id, -32603, message)
    }
}

/// JSON-RPC notification (no id, no response expected)
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Parameters
    pub params: Value,
}

impl JsonRpcNotification {
    /// Create a new notification
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// MCP Initialize result
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// Protocol version
    pub protocol_version: String,

    /// Server information
    pub server_info: ServerInfo,

    /// Server capabilities
    pub capabilities: ServerCapabilities,
}

/// Server information
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,

    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,

    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether tool list can change
    pub list_changed: bool,
}

/// Resources capability
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    /// Whether resource list can change
    pub list_changed: bool,

    /// Whether subscriptions are supported
    pub subscribe: bool,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether prompt list can change
    pub list_changed: bool,
}

/// MCP Prompt definition
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptDefinition {
    /// Prompt name
    pub name: String,

    /// Prompt description
    pub description: String,

    /// Optional prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptArgument {
    /// Argument name
    pub name: String,

    /// Argument description
    pub description: String,

    /// Whether argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Get prompt result
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPromptResult {
    /// The prompt definition
    pub prompt: PromptDefinition,

    /// Prompt messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<PromptMessage>>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "role")]
pub enum PromptMessage {
    #[serde(rename = "user")]
    User { content: Content },
    #[serde(rename = "assistant")]
    Assistant { content: Content },
    #[serde(rename = "system")]
    System { content: Content },
}

impl GetPromptResult {
    /// Create a prompt result with default messages
    pub fn new(prompt: PromptDefinition) -> Self {
        Self {
            prompt,
            messages: None,
        }
    }

    /// Create a prompt result with user message
    pub fn with_user_message(mut self, content: impl Into<String>) -> Self {
        self.messages
            .get_or_insert_with(Vec::new)
            .push(PromptMessage::User {
                content: Content::text(content),
            });
        self
    }

    /// Create a prompt result with system message
    pub fn with_system_message(mut self, content: impl Into<String>) -> Self {
        self.messages
            .get_or_insert_with(Vec::new)
            .push(PromptMessage::System {
                content: Content::text(content),
            });
        self
    }
}

/// Content for prompt messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, text: String },
}

impl Content {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: Value,

    /// Output schema (JSON Schema) for structured responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,

    /// Hint that the tool does not modify its environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// Hint that the tool may perform destructive updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    /// Hint that repeated calls with same args have no additional effect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    /// Hint that the tool interacts with external entities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// MCP Tool call result
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResult {
    /// Content items
    pub content: Vec<ContentItem>,

    /// Whether the tool call produced an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content item in tool result
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image content (base64)
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },

    /// Resource content
    #[serde(rename = "resource")]
    Resource { uri: String, text: String },
}

impl ToolCallResult {
    /// Create a text result
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text { text: text.into() }],
            is_error: None,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text {
                text: message.into(),
            }],
            is_error: Some(true),
        }
    }

    /// Create a JSON result
    pub fn json(value: &Value) -> Self {
        Self::text(serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
    }
}
