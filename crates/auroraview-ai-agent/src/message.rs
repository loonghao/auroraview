//! Message types for AI conversations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (sets context/behavior)
    System,
    /// User message
    User,
    /// Assistant (AI) message
    Assistant,
    /// Tool result message
    Tool,
}

/// Content of a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Multi-part content (text + images)
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// Create text content
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create multi-part content
    pub fn parts(parts: Vec<ContentPart>) -> Self {
        Self::Parts(parts)
    }

    /// Get as text (concatenating parts if multi-part)
    pub fn as_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// A part of multi-modal content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

impl ContentPart {
    /// Create text part
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { text: s.into() }
    }

    /// Create image URL part
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }

    /// Create base64 image part
    pub fn image_base64(data: impl Into<String>, media_type: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: format!("data:{};base64,{}", media_type.into(), data.into()),
                detail: None,
            },
        }
    }
}

/// Image URL specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// URL of the image (can be data URL)
    pub url: String,
    /// Detail level for processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Image detail level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    #[serde(default = "generate_message_id")]
    pub id: String,

    /// Role of the message sender
    pub role: MessageRole,

    /// Message content
    pub content: MessageContent,

    /// Optional name (for multi-user scenarios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tool calls made by assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Tool call ID (for tool response messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

fn generate_message_id() -> String {
    Uuid::new_v4().to_string()
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: impl Into<MessageContent>) -> Self {
        Self {
            id: generate_message_id(),
            role,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content.into())
    }

    /// Create a user message
    pub fn user(content: impl Into<MessageContent>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content.into())
    }

    /// Create a tool result message
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        let mut msg = Self::new(MessageRole::Tool, content.into());
        msg.tool_call_id = Some(tool_call_id.into());
        msg
    }

    /// Set message name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set tool calls
    pub fn with_tool_calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(calls);
        self
    }
}

/// A tool (function) call from the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,

    /// Name of the tool/function
    pub name: String,

    /// Arguments as JSON string
    pub arguments: String,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    /// Parse arguments as JSON
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.arguments)
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Parameters schema (JSON Schema)
    pub parameters: serde_json::Value,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Tool choice for function calling
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Let the model decide
    #[default]
    Auto,
    /// Don't use tools
    None,
    /// Must use a specific tool
    Required { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.as_text(), "Hello, world!");
    }

    #[test]
    fn test_tool_call_parsing() {
        let call = ToolCall::new("navigate", r#"{"url": "https://example.com"}"#);

        #[derive(Deserialize)]
        struct Args {
            url: String,
        }

        let args: Args = call.parse_arguments().unwrap();
        assert_eq!(args.url, "https://example.com");
    }
}
