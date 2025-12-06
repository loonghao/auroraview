//! IPC Message Types
//!
//! Core message structures for IPC communication, independent of any
//! specific language bindings.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// IPC message structure
///
/// This is the fundamental message type used for all IPC communication.
/// It is serializable and can be sent between threads or processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Event name (e.g., "click", "state_changed", "invoke")
    pub event: String,

    /// Message data as JSON value
    pub data: Value,

    /// Optional message ID for request-response pattern
    pub id: Option<String>,
}

impl IpcMessage {
    /// Create a new IPC message
    pub fn new(event: impl Into<String>, data: Value) -> Self {
        Self {
            event: event.into(),
            data,
            id: None,
        }
    }

    /// Create a new IPC message with an ID
    pub fn with_id(event: impl Into<String>, data: Value, id: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            data,
            id: Some(id.into()),
        }
    }
}

/// IPC mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IpcMode {
    /// Thread-based communication (default for embedded mode)
    #[default]
    Threaded,

    /// Process-based communication (for standalone mode)
    Process,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_new() {
        let msg = IpcMessage::new("test_event", serde_json::json!({"key": "value"}));
        assert_eq!(msg.event, "test_event");
        assert!(msg.id.is_none());
    }

    #[test]
    fn test_ipc_message_with_id() {
        let msg = IpcMessage::with_id("test", serde_json::json!(null), "msg_123");
        assert_eq!(msg.event, "test");
        assert_eq!(msg.id, Some("msg_123".to_string()));
    }

    #[test]
    fn test_ipc_message_serialize() {
        let msg = IpcMessage::new("serialize_test", serde_json::json!({"a": 1}));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("serialize_test"));
    }

    #[test]
    fn test_ipc_message_deserialize() {
        let json = r#"{"event":"deser_test","data":{"key":"value"},"id":"id_1"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.event, "deser_test");
        assert_eq!(msg.id, Some("id_1".to_string()));
    }

    #[test]
    fn test_ipc_mode_default() {
        let mode = IpcMode::default();
        assert_eq!(mode, IpcMode::Threaded);
    }

    #[test]
    fn test_ipc_mode_equality() {
        assert_eq!(IpcMode::Threaded, IpcMode::Threaded);
        assert_ne!(IpcMode::Threaded, IpcMode::Process);
    }
}
