//! IPC Backend Abstraction Layer - Python Bindings
//!
//! This module defines the Python-specific IPC backend trait. Core types
//! like IpcMessage and IpcMode are re-exported from auroraview-core.

#[cfg(feature = "python-bindings")]
use super::json::Value;
#[cfg(feature = "python-bindings")]
use pyo3::{Py, PyAny};

// Re-export core types for backward compatibility
pub use auroraview_core::ipc::{IpcMessage, IpcMode};

/// Python-specific IPC backend trait
///
/// This trait provides a common interface for IPC implementations
/// that need to interact with Python callbacks.
#[cfg(feature = "python-bindings")]
#[allow(dead_code)]
pub trait IpcBackend: Send + Sync {
    /// Send a message to the WebView
    fn send_message(&self, event: &str, data: Value) -> Result<(), String>;

    /// Register a Python callback for an event
    fn register_callback(&self, event: &str, callback: Py<PyAny>) -> Result<(), String>;

    /// Process pending messages
    fn process_pending(&self) -> Result<usize, String>;

    /// Get the number of pending messages
    fn pending_count(&self) -> usize;

    /// Clear all registered callbacks
    fn clear_callbacks(&self) -> Result<(), String>;

    /// Remove callbacks for a specific event
    fn remove_callbacks(&self, event: &str) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde_json::Value;

    #[test]
    fn test_ipc_message_new() {
        let msg = IpcMessage {
            event: "test_event".to_string(),
            data: Value::String("test_data".to_string()),
            id: Some("msg_123".to_string()),
        };

        assert_eq!(msg.event, "test_event");
        assert_eq!(msg.id, Some("msg_123".to_string()));
    }

    #[test]
    fn test_ipc_message_without_id() {
        let msg = IpcMessage {
            event: "event".to_string(),
            data: Value::Null,
            id: None,
        };

        assert_eq!(msg.event, "event");
        assert!(msg.id.is_none());
    }

    #[test]
    fn test_ipc_message_clone() {
        let msg = IpcMessage {
            event: "test".to_string(),
            data: Value::Bool(true),
            id: Some("id".to_string()),
        };

        let cloned = msg.clone();
        assert_eq!(msg.event, cloned.event);
        assert_eq!(msg.id, cloned.id);
    }

    #[test]
    fn test_ipc_message_debug() {
        let msg = IpcMessage {
            event: "debug_test".to_string(),
            data: Value::Number(serde_json::Number::from(42)),
            id: None,
        };

        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("IpcMessage"));
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_ipc_message_serialize() {
        let msg = IpcMessage {
            event: "serialize_test".to_string(),
            data: Value::Object(serde_json::Map::new()),
            id: Some("ser_id".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("serialize_test"));
        assert!(json.contains("ser_id"));
    }

    #[test]
    fn test_ipc_message_deserialize() {
        let json = r#"{"event":"deser_test","data":{"key":"value"},"id":"deser_id"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.event, "deser_test");
        assert_eq!(msg.id, Some("deser_id".to_string()));
    }

    #[rstest]
    fn test_ipc_mode_default() {
        let mode = IpcMode::default();
        assert_eq!(mode, IpcMode::Threaded);
    }

    #[rstest]
    fn test_ipc_mode_clone() {
        let mode = IpcMode::Process;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[rstest]
    fn test_ipc_mode_debug() {
        let mode = IpcMode::Threaded;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Threaded"));
    }

    #[rstest]
    fn test_ipc_mode_equality() {
        assert_eq!(IpcMode::Threaded, IpcMode::Threaded);
        assert_eq!(IpcMode::Process, IpcMode::Process);
        assert_ne!(IpcMode::Threaded, IpcMode::Process);
    }
}
