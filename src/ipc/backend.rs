//! IPC Backend Abstraction Layer
//!
//! This module defines the unified IPC backend trait that supports both
//! thread-based communication (for embedded mode) and process-based
//! communication (for standalone mode).

use super::json::Value;
use pyo3::{Py, PyAny};

/// IPC message structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpcMessage {
    /// Event name
    pub event: String,

    /// Message data (JSON)
    pub data: Value,

    /// Message ID for request-response pattern
    pub id: Option<String>,
}

/// Unified IPC backend trait
///
/// This trait provides a common interface for different IPC implementations:
/// - ThreadedBackend: Thread-based communication using crossbeam-channel
/// - ProcessBackend: Process-based communication using ipc-channel (optional)
#[allow(dead_code)]
pub trait IpcBackend: Send + Sync {
    /// Send a message to the WebView
    ///
    /// # Arguments
    /// * `event` - Event name
    /// * `data` - Event data as JSON value
    ///
    /// # Returns
    /// * `Ok(())` if the message was sent successfully
    /// * `Err(String)` if the send failed
    fn send_message(&self, event: &str, data: Value) -> Result<(), String>;

    /// Register a Python callback for an event
    ///
    /// # Arguments
    /// * `event` - Event name to listen for
    /// * `callback` - Python callable object
    ///
    /// # Returns
    /// * `Ok(())` if the callback was registered successfully
    /// * `Err(String)` if registration failed
    fn register_callback(&self, event: &str, callback: Py<PyAny>) -> Result<(), String>;

    /// Process pending messages
    ///
    /// This should be called from the WebView thread to process
    /// all pending messages in the queue.
    ///
    /// # Returns
    /// * `Ok(count)` - Number of messages processed
    /// * `Err(String)` - Error message if processing failed
    fn process_pending(&self) -> Result<usize, String>;

    /// Get the number of pending messages
    fn pending_count(&self) -> usize;

    /// Clear all registered callbacks
    fn clear_callbacks(&self) -> Result<(), String>;

    /// Remove callbacks for a specific event
    fn remove_callbacks(&self, event: &str) -> Result<(), String>;
}

/// IPC mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum IpcMode {
    /// Thread-based communication (default for embedded mode)
    #[default]
    Threaded,

    /// Process-based communication (for standalone mode)
    #[allow(dead_code)]
    Process,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

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
