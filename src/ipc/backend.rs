//! IPC Backend Abstraction Layer
//!
//! This module defines the unified IPC backend trait that supports both
//! thread-based communication (for embedded mode) and process-based
//! communication (for standalone mode).

use super::json::Value;
use pyo3::prelude::*;

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
    fn register_callback(&self, event: &str, callback: PyObject) -> Result<(), String>;

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
