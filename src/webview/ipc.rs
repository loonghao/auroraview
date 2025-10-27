//! IPC (Inter-Process Communication) handler for WebView

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// IPC message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Event name
    pub event: String,

    /// Message data (JSON)
    pub data: serde_json::Value,

    /// Message ID for request-response pattern
    pub id: Option<String>,
}

/// IPC callback type
pub type IpcCallback = Arc<dyn Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync>;

/// IPC handler for managing communication between Python and JavaScript
pub struct IpcHandler {
    /// Registered event callbacks
    callbacks: Arc<Mutex<HashMap<String, Vec<IpcCallback>>>>,
}

impl IpcHandler {
    /// Create a new IPC handler
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a callback for an event
    #[allow(dead_code)]
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(IpcMessage) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(callback));
    }

    /// Emit an event to JavaScript
    #[allow(dead_code)]
    pub fn emit(&self, event: &str, data: serde_json::Value) -> Result<(), String> {
        let _message = IpcMessage {
            event: event.to_string(),
            data,
            id: None,
        };

        tracing::debug!("Emitting IPC event: {}", event);

        // TODO: Send message to WebView
        Ok(())
    }

    /// Handle incoming message from JavaScript
    #[allow(dead_code)]
    pub fn handle_message(&self, message: IpcMessage) -> Result<serde_json::Value, String> {
        tracing::debug!("Handling IPC message: {}", message.event);

        let callbacks = self.callbacks.lock().unwrap();

        if let Some(event_callbacks) = callbacks.get(&message.event) {
            // Execute all registered callbacks for this event
            for callback in event_callbacks {
                match callback(message.clone()) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        tracing::error!("IPC callback error: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        // No callback found
        Err(format!(
            "No handler registered for event: {}",
            message.event
        ))
    }

    /// Remove all callbacks for an event
    #[allow(dead_code)]
    pub fn off(&self, event: &str) {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.remove(event);
    }

    /// Clear all callbacks
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.clear();
    }
}

impl Default for IpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_handler() {
        let handler = IpcHandler::new();

        // Register a callback
        handler.on("test_event", |msg| {
            Ok(serde_json::json!({"received": msg.event}))
        });

        // Handle a message
        let message = IpcMessage {
            event: "test_event".to_string(),
            data: serde_json::json!({"key": "value"}),
            id: None,
        };

        let result = handler.handle_message(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit() {
        let handler = IpcHandler::new();
        let result = handler.emit("test", serde_json::json!({"data": "test"}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_off() {
        let handler = IpcHandler::new();
        handler.on("test", |_| Ok(serde_json::json!({})));
        handler.off("test");

        let message = IpcMessage {
            event: "test".to_string(),
            data: serde_json::json!({}),
            id: None,
        };

        let result = handler.handle_message(message);
        assert!(result.is_err());
    }
}
