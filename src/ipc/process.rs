//! Process-based IPC Backend (Optional)
//!
//! This module implements the IpcBackend trait using ipc-channel
//! for cross-process communication. This is intended for standalone
//! mode where the WebView runs in a separate process.
//!
//! **Note**: This is currently a placeholder and requires the `process-ipc`
//! feature to be enabled.

#![allow(dead_code)]

use pyo3::prelude::*;
use serde_json::Value;

use super::backend::IpcBackend;

/// Process-based IPC backend using ipc-channel
///
/// **TODO**: Implement this backend using ipc-channel for cross-process communication.
pub struct ProcessBackend {
    // Placeholder fields
}

impl ProcessBackend {
    /// Create a new process-based backend
    pub fn new() -> Self {
        unimplemented!("ProcessBackend is not yet implemented. Use ThreadedBackend instead.");
    }
}

impl IpcBackend for ProcessBackend {
    fn send_message(&self, _event: &str, _data: Value) -> Result<(), String> {
        unimplemented!("ProcessBackend is not yet implemented")
    }

    fn register_callback(&self, _event: &str, _callback: PyObject) -> Result<(), String> {
        unimplemented!("ProcessBackend is not yet implemented")
    }

    fn process_pending(&self) -> Result<usize, String> {
        unimplemented!("ProcessBackend is not yet implemented")
    }

    fn pending_count(&self) -> usize {
        unimplemented!("ProcessBackend is not yet implemented")
    }

    fn clear_callbacks(&self) -> Result<(), String> {
        unimplemented!("ProcessBackend is not yet implemented")
    }

    fn remove_callbacks(&self, _event: &str) -> Result<(), String> {
        unimplemented!("ProcessBackend is not yet implemented")
    }
}

impl Default for ProcessBackend {
    fn default() -> Self {
        Self::new()
    }
}
