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
