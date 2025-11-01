//! Thread-based IPC Backend
//!
//! This module implements the IpcBackend trait using crossbeam-channel
//! for high-performance thread-based communication. This is the default
//! backend for embedded mode (e.g., Maya, Houdini).

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use dashmap::DashMap;
use pyo3::prelude::*;
use serde_json::Value;
use std::sync::Arc;

use super::backend::{IpcBackend, IpcMessage};
use super::handler::PythonCallback;

/// Configuration for threaded backend
#[derive(Debug, Clone)]
pub struct ThreadedConfig {
    /// Maximum number of messages in the queue (backpressure)
    pub capacity: usize,

    /// Whether to block when queue is full (true) or drop messages (false)
    #[allow(dead_code)]
    pub block_on_full: bool,
}

impl Default for ThreadedConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            block_on_full: false,
        }
    }
}

/// Thread-based IPC backend using crossbeam-channel
///
/// This backend provides high-performance lock-free communication
/// between threads using crossbeam-channel. It's optimized for
/// embedded mode where the WebView runs in the same process.
pub struct ThreadedBackend {
    /// Sender for outgoing messages (Python -> JavaScript)
    #[allow(dead_code)]
    tx: Sender<IpcMessage>,

    /// Receiver for incoming messages (JavaScript -> Python)
    rx: Receiver<IpcMessage>,

    /// Registered Python callbacks (lock-free concurrent map)
    #[allow(dead_code)]
    callbacks: Arc<DashMap<String, Vec<PythonCallback>>>,

    /// Configuration
    #[allow(dead_code)]
    config: ThreadedConfig,
}

impl ThreadedBackend {
    /// Create a new threaded backend with default configuration
    pub fn new() -> Self {
        Self::with_config(ThreadedConfig::default())
    }

    /// Create a new threaded backend with custom configuration
    pub fn with_config(config: ThreadedConfig) -> Self {
        let (tx, rx) = bounded(config.capacity);
        Self {
            tx,
            rx,
            callbacks: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Get a receiver for consuming messages
    ///
    /// This is used by the WebView thread to receive messages
    /// that need to be sent to JavaScript.
    #[allow(dead_code)]
    pub fn receiver(&self) -> Receiver<IpcMessage> {
        self.rx.clone()
    }
}

impl IpcBackend for ThreadedBackend {
    fn send_message(&self, event: &str, data: Value) -> Result<(), String> {
        let message = IpcMessage {
            event: event.to_string(),
            data,
            id: None,
        };

        tracing::debug!(
            "[SEND] [ThreadedBackend::send_message] Sending event: {}",
            event
        );

        match self.tx.try_send(message.clone()) {
            Ok(_) => {
                tracing::debug!("[OK] [ThreadedBackend::send_message] Message sent successfully");
                Ok(())
            }
            Err(TrySendError::Full(_)) => {
                if self.config.block_on_full {
                    // Block until space is available
                    tracing::warn!(
                        "[WARNING] [ThreadedBackend::send_message] Queue full, blocking..."
                    );
                    self.tx
                        .send(message)
                        .map_err(|e| format!("Failed to send message: {}", e))
                } else {
                    // Drop the message
                    let err = format!("Queue full, dropping message for event: {}", event);
                    tracing::error!("[ERROR] [ThreadedBackend::send_message] {}", err);
                    Err(err)
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                let err = "Channel disconnected".to_string();
                tracing::error!("[ERROR] [ThreadedBackend::send_message] {}", err);
                Err(err)
            }
        }
    }

    fn register_callback(&self, event: &str, callback: PyObject) -> Result<(), String> {
        self.callbacks
            .entry(event.to_string())
            .or_default()
            .push(PythonCallback::new(callback));

        tracing::info!(
            "[OK] [ThreadedBackend] Registered callback for event: {}",
            event
        );
        Ok(())
    }

    fn process_pending(&self) -> Result<usize, String> {
        let mut count = 0;

        // Process all pending messages
        while let Ok(message) = self.rx.try_recv() {
            tracing::debug!(
                "[PROCESS] [ThreadedBackend::process_pending] Processing event: {}",
                message.event
            );

            // Find and execute callbacks for this event
            if let Some(event_callbacks) = self.callbacks.get(&message.event) {
                for callback in event_callbacks.value() {
                    if let Err(e) = callback.call(message.data.clone()) {
                        tracing::error!(
                            "[ERROR] [ThreadedBackend] Callback error for event {}: {}",
                            message.event,
                            e
                        );
                        return Err(e);
                    }
                }
            } else {
                tracing::warn!(
                    "[WARNING] [ThreadedBackend] No callback registered for event: {}",
                    message.event
                );
            }

            count += 1;
        }

        if count > 0 {
            tracing::debug!(
                "[OK] [ThreadedBackend::process_pending] Processed {} messages",
                count
            );
        }

        Ok(count)
    }

    fn pending_count(&self) -> usize {
        self.rx.len()
    }

    fn clear_callbacks(&self) -> Result<(), String> {
        self.callbacks.clear();
        tracing::info!("[OK] [ThreadedBackend] Cleared all callbacks");
        Ok(())
    }

    fn remove_callbacks(&self, event: &str) -> Result<(), String> {
        self.callbacks.remove(event);
        tracing::info!(
            "[OK] [ThreadedBackend] Removed callbacks for event: {}",
            event
        );
        Ok(())
    }
}

impl Default for ThreadedBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threaded_backend_creation() {
        let backend = ThreadedBackend::new();
        assert_eq!(backend.pending_count(), 0);
    }

    #[test]
    fn test_send_message() {
        let backend = ThreadedBackend::new();
        let result = backend.send_message("test_event", serde_json::json!({"key": "value"}));
        assert!(result.is_ok());
        assert_eq!(backend.pending_count(), 1);
    }

    #[test]
    fn test_backpressure() {
        let config = ThreadedConfig {
            capacity: 2,
            block_on_full: false,
        };
        let backend = ThreadedBackend::with_config(config);

        // Fill the queue
        assert!(backend
            .send_message("event1", serde_json::json!({}))
            .is_ok());
        assert!(backend
            .send_message("event2", serde_json::json!({}))
            .is_ok());

        // This should fail due to backpressure
        let result = backend.send_message("event3", serde_json::json!({}));
        assert!(result.is_err());
    }
}
