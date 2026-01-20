//! AG-UI (Agent-UI) Protocol implementation
//!
//! AG-UI is a protocol for AI agent and UI communication, defining
//! standard events for text streaming, tool calls, state updates, etc.
//!
//! This implementation is based on the official AG-UI Rust SDK:
//! https://github.com/ag-ui-protocol/ag-ui/tree/main/sdks/community/rust
//!
//! Reference: https://docs.ag-ui.com

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// AG-UI Event types enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    // Text message events
    TextMessageStart,
    TextMessageContent,
    TextMessageEnd,
    TextMessageChunk,

    // Thinking/reasoning events (for models like DeepSeek R1, O1)
    ThinkingTextMessageStart,
    ThinkingTextMessageContent,
    ThinkingTextMessageEnd,

    // Tool call events
    ToolCallStart,
    ToolCallArgs,
    ToolCallEnd,
    ToolCallChunk,
    ToolCallResult,

    // Thinking step events
    ThinkingStart,
    ThinkingEnd,

    // State synchronization
    StateSnapshot,
    StateDelta,
    MessagesSnapshot,

    // Run lifecycle
    RunStarted,
    RunFinished,
    RunError,

    // Step lifecycle
    StepStarted,
    StepFinished,

    // Extension events
    Raw,
    Custom,
}

/// Base event structure containing common fields
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaseEvent {
    /// Event timestamp (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,

    /// Original raw event data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_event: Option<JsonValue>,
}

impl BaseEvent {
    /// Create new base event with current timestamp
    pub fn now() -> Self {
        Self {
            timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs_f64() * 1000.0)
                    .unwrap_or(0.0),
            ),
            raw_event: None,
        }
    }
}

/// AG-UI Event - the main event enum following the protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AGUIEvent {
    // === Run Lifecycle Events ===
    /// Run started event - sent when agent begins processing
    RunStarted {
        /// Unique run ID
        run_id: String,
        /// Thread/conversation ID
        thread_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Run finished event - sent when agent completes processing
    RunFinished {
        /// Run ID
        run_id: String,
        /// Thread ID
        thread_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Run error event - sent when an error occurs
    RunError {
        /// Run ID
        run_id: String,
        /// Error message
        message: String,
        /// Error code
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === Text Message Events ===
    /// Text message start - beginning of a text message
    TextMessageStart {
        /// Message ID
        message_id: String,
        /// Role (assistant, user, etc.)
        role: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Text message content - streaming text content
    TextMessageContent {
        /// Message ID
        message_id: String,
        /// Text delta
        delta: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Text message end - end of a text message
    TextMessageEnd {
        /// Message ID
        message_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Text message chunk - combined start/content/end for efficiency
    TextMessageChunk {
        /// Message ID
        message_id: String,
        /// Role
        role: String,
        /// Content
        content: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === Thinking Events (for reasoning models) ===
    /// Thinking text start
    ThinkingTextMessageStart {
        /// Message ID
        message_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Thinking text content
    ThinkingTextMessageContent {
        /// Message ID
        message_id: String,
        /// Thinking delta
        delta: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Thinking text end
    ThinkingTextMessageEnd {
        /// Message ID
        message_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Thinking step start
    ThinkingStart {
        /// Thinking step ID
        thinking_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Thinking step end
    ThinkingEnd {
        /// Thinking step ID
        thinking_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === Tool Call Events ===
    /// Tool call start - beginning of a tool/function call
    ToolCallStart {
        /// Message ID containing the tool call
        message_id: String,
        /// Unique tool call ID
        tool_call_id: String,
        /// Tool/function name
        tool_name: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Tool call arguments - streaming tool arguments
    ToolCallArgs {
        /// Tool call ID
        tool_call_id: String,
        /// Arguments delta (JSON fragment)
        delta: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Tool call end - end of a tool call
    ToolCallEnd {
        /// Tool call ID
        tool_call_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Tool call chunk - combined for efficiency
    ToolCallChunk {
        /// Message ID
        message_id: String,
        /// Tool call ID
        tool_call_id: String,
        /// Tool name
        tool_name: String,
        /// Arguments
        arguments: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Tool call result - result of tool execution
    ToolCallResult {
        /// Tool call ID
        tool_call_id: String,
        /// Role (usually "tool")
        role: String,
        /// Result content
        content: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === Step Events ===
    /// Step started
    StepStarted {
        /// Step ID
        step_id: String,
        /// Step name/description
        #[serde(skip_serializing_if = "Option::is_none")]
        step_name: Option<String>,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Step finished
    StepFinished {
        /// Step ID
        step_id: String,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === State Synchronization Events ===
    /// State snapshot - complete state update
    StateSnapshot {
        /// State data
        snapshot: JsonValue,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// State delta - incremental state update using JSON Patch (RFC 6902)
    StateDelta {
        /// JSON Patch operations
        delta: Vec<JsonPatchOp>,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Messages snapshot - complete message list
    MessagesSnapshot {
        /// All messages
        messages: Vec<AGUIMessage>,
        #[serde(flatten)]
        base: BaseEvent,
    },

    // === Extension Events ===
    /// Raw event - for external/wrapped events
    Raw {
        /// Event name
        event: String,
        /// Event data
        data: JsonValue,
        #[serde(flatten)]
        base: BaseEvent,
    },

    /// Custom event - for application-specific events
    Custom {
        /// Event name
        name: String,
        /// Event data
        value: JsonValue,
        #[serde(flatten)]
        base: BaseEvent,
    },
}

impl AGUIEvent {
    /// Get the event type
    pub fn event_type(&self) -> EventType {
        match self {
            Self::RunStarted { .. } => EventType::RunStarted,
            Self::RunFinished { .. } => EventType::RunFinished,
            Self::RunError { .. } => EventType::RunError,
            Self::TextMessageStart { .. } => EventType::TextMessageStart,
            Self::TextMessageContent { .. } => EventType::TextMessageContent,
            Self::TextMessageEnd { .. } => EventType::TextMessageEnd,
            Self::TextMessageChunk { .. } => EventType::TextMessageChunk,
            Self::ThinkingTextMessageStart { .. } => EventType::ThinkingTextMessageStart,
            Self::ThinkingTextMessageContent { .. } => EventType::ThinkingTextMessageContent,
            Self::ThinkingTextMessageEnd { .. } => EventType::ThinkingTextMessageEnd,
            Self::ThinkingStart { .. } => EventType::ThinkingStart,
            Self::ThinkingEnd { .. } => EventType::ThinkingEnd,
            Self::ToolCallStart { .. } => EventType::ToolCallStart,
            Self::ToolCallArgs { .. } => EventType::ToolCallArgs,
            Self::ToolCallEnd { .. } => EventType::ToolCallEnd,
            Self::ToolCallChunk { .. } => EventType::ToolCallChunk,
            Self::ToolCallResult { .. } => EventType::ToolCallResult,
            Self::StepStarted { .. } => EventType::StepStarted,
            Self::StepFinished { .. } => EventType::StepFinished,
            Self::StateSnapshot { .. } => EventType::StateSnapshot,
            Self::StateDelta { .. } => EventType::StateDelta,
            Self::MessagesSnapshot { .. } => EventType::MessagesSnapshot,
            Self::Raw { .. } => EventType::Raw,
            Self::Custom { .. } => EventType::Custom,
        }
    }

    /// Get timestamp if present
    pub fn timestamp(&self) -> Option<f64> {
        match self {
            Self::RunStarted { base, .. }
            | Self::RunFinished { base, .. }
            | Self::RunError { base, .. }
            | Self::TextMessageStart { base, .. }
            | Self::TextMessageContent { base, .. }
            | Self::TextMessageEnd { base, .. }
            | Self::TextMessageChunk { base, .. }
            | Self::ThinkingTextMessageStart { base, .. }
            | Self::ThinkingTextMessageContent { base, .. }
            | Self::ThinkingTextMessageEnd { base, .. }
            | Self::ThinkingStart { base, .. }
            | Self::ThinkingEnd { base, .. }
            | Self::ToolCallStart { base, .. }
            | Self::ToolCallArgs { base, .. }
            | Self::ToolCallEnd { base, .. }
            | Self::ToolCallChunk { base, .. }
            | Self::ToolCallResult { base, .. }
            | Self::StepStarted { base, .. }
            | Self::StepFinished { base, .. }
            | Self::StateSnapshot { base, .. }
            | Self::StateDelta { base, .. }
            | Self::MessagesSnapshot { base, .. }
            | Self::Raw { base, .. }
            | Self::Custom { base, .. } => base.timestamp,
        }
    }
}

/// JSON Patch operation (RFC 6902)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum JsonPatchOp {
    Add { path: String, value: JsonValue },
    Remove { path: String },
    Replace { path: String, value: JsonValue },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: JsonValue },
}

/// AG-UI Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGUIMessage {
    /// Message ID
    pub id: String,
    /// Role (system, user, assistant, tool)
    pub role: String,
    /// Message content
    pub content: String,
    /// Tool calls (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AGUIToolCall>>,
    /// Tool call ID (for tool result messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// AG-UI Tool Call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGUIToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Arguments as JSON string
    pub arguments: String,
}

/// Tool definition for AG-UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGUITool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameters schema (JSON Schema)
    pub parameters: JsonValue,
}

/// Context item for AG-UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGUIContext {
    /// Context description
    pub description: String,
    /// Context value
    pub value: JsonValue,
}

/// AG-UI Event emitter trait
pub trait AGUIEmitter: Send + Sync {
    /// Emit an AG-UI event
    fn emit(&self, event: AGUIEvent);

    /// Emit run started
    fn run_started(&self, run_id: &str, thread_id: &str) {
        self.emit(AGUIEvent::RunStarted {
            run_id: run_id.to_string(),
            thread_id: thread_id.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit run finished
    fn run_finished(&self, run_id: &str, thread_id: &str) {
        self.emit(AGUIEvent::RunFinished {
            run_id: run_id.to_string(),
            thread_id: thread_id.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit run error
    fn run_error(&self, run_id: &str, message: &str) {
        self.emit(AGUIEvent::RunError {
            run_id: run_id.to_string(),
            message: message.to_string(),
            code: None,
            base: BaseEvent::now(),
        });
    }

    /// Emit text message start
    fn text_start(&self, message_id: &str, role: &str) {
        self.emit(AGUIEvent::TextMessageStart {
            message_id: message_id.to_string(),
            role: role.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit text message content
    fn text_delta(&self, message_id: &str, delta: &str) {
        self.emit(AGUIEvent::TextMessageContent {
            message_id: message_id.to_string(),
            delta: delta.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit text message end
    fn text_end(&self, message_id: &str) {
        self.emit(AGUIEvent::TextMessageEnd {
            message_id: message_id.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit tool call start
    fn tool_call_start(&self, message_id: &str, tool_call_id: &str, tool_name: &str) {
        self.emit(AGUIEvent::ToolCallStart {
            message_id: message_id.to_string(),
            tool_call_id: tool_call_id.to_string(),
            tool_name: tool_name.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit tool call arguments
    fn tool_call_args(&self, tool_call_id: &str, delta: &str) {
        self.emit(AGUIEvent::ToolCallArgs {
            tool_call_id: tool_call_id.to_string(),
            delta: delta.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit tool call end
    fn tool_call_end(&self, tool_call_id: &str) {
        self.emit(AGUIEvent::ToolCallEnd {
            tool_call_id: tool_call_id.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit tool call result
    fn tool_call_result(&self, tool_call_id: &str, content: &str) {
        self.emit(AGUIEvent::ToolCallResult {
            tool_call_id: tool_call_id.to_string(),
            role: "tool".to_string(),
            content: content.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit state snapshot
    fn state_snapshot(&self, state: JsonValue) {
        self.emit(AGUIEvent::StateSnapshot {
            snapshot: state,
            base: BaseEvent::now(),
        });
    }

    /// Emit thinking start (for reasoning models)
    fn thinking_start(&self, message_id: &str) {
        self.emit(AGUIEvent::ThinkingTextMessageStart {
            message_id: message_id.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit thinking delta
    fn thinking_delta(&self, message_id: &str, delta: &str) {
        self.emit(AGUIEvent::ThinkingTextMessageContent {
            message_id: message_id.to_string(),
            delta: delta.to_string(),
            base: BaseEvent::now(),
        });
    }

    /// Emit thinking end
    fn thinking_end(&self, message_id: &str) {
        self.emit(AGUIEvent::ThinkingTextMessageEnd {
            message_id: message_id.to_string(),
            base: BaseEvent::now(),
        });
    }
}

/// Callback-based AG-UI emitter
pub struct CallbackEmitter<F>
where
    F: Fn(AGUIEvent) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackEmitter<F>
where
    F: Fn(AGUIEvent) + Send + Sync,
{
    /// Create new callback emitter
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> AGUIEmitter for CallbackEmitter<F>
where
    F: Fn(AGUIEvent) + Send + Sync,
{
    fn emit(&self, event: AGUIEvent) {
        (self.callback)(event);
    }
}

/// No-op emitter for testing or when events are not needed
pub struct NoOpEmitter;

impl AGUIEmitter for NoOpEmitter {
    fn emit(&self, _event: AGUIEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agui_event_serialization() {
        let event = AGUIEvent::TextMessageContent {
            message_id: "msg_123".to_string(),
            delta: "Hello".to_string(),
            base: BaseEvent::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TEXT_MESSAGE_CONTENT"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_event_type() {
        let event = AGUIEvent::RunStarted {
            run_id: "run_1".to_string(),
            thread_id: "thread_1".to_string(),
            base: BaseEvent::now(),
        };

        assert_eq!(event.event_type(), EventType::RunStarted);
    }

    #[test]
    fn test_callback_emitter() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let emitter = CallbackEmitter::new(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        emitter.text_start("msg_1", "assistant");
        emitter.text_delta("msg_1", "Hello");
        emitter.text_end("msg_1");

        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_json_patch_op() {
        let op = JsonPatchOp::Replace {
            path: "/status".to_string(),
            value: serde_json::json!("completed"),
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("replace"));
        assert!(json.contains("/status"));
    }
}
