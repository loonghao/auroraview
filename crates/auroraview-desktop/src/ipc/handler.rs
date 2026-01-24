//! IPC handler and router

use super::message::{IpcMessage, IpcResponse};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

/// Callback type for IPC handlers
pub type IpcCallback = Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

/// IPC router for handling messages from JavaScript
pub struct IpcRouter {
    /// Registered handlers by method name
    handlers: RwLock<HashMap<String, IpcCallback>>,

    /// Event listeners
    event_listeners: RwLock<HashMap<String, Vec<Arc<dyn Fn(serde_json::Value) + Send + Sync>>>>,
}

impl IpcRouter {
    /// Create new IPC router
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            event_listeners: RwLock::new(HashMap::new()),
        }
    }

    /// Register a handler for a method
    pub fn register<F>(&self, method: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.insert(method.to_string(), Box::new(handler));
            debug!("[IPC] Registered handler: {}", method);
        }
    }

    /// Subscribe to an event
    pub fn on<F>(&self, event: &str, handler: F)
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        if let Ok(mut listeners) = self.event_listeners.write() {
            listeners
                .entry(event.to_string())
                .or_default()
                .push(Arc::new(handler));
            debug!("[IPC] Subscribed to event: {}", event);
        }
    }

    /// Handle incoming IPC message
    pub fn handle(&self, raw: &str) -> Option<String> {
        let message: IpcMessage = match serde_json::from_str(raw) {
            Ok(m) => m,
            Err(e) => {
                warn!("[IPC] Failed to parse message: {}", e);
                return None;
            }
        };

        match message.msg_type.as_str() {
            "event" => {
                self.handle_event(&message);
                None
            }
            "call" => self.handle_call(&message),
            "invoke" => self.handle_invoke(&message),
            _ => {
                warn!("[IPC] Unknown message type: {}", message.msg_type);
                None
            }
        }
    }

    /// Handle event message
    fn handle_event(&self, message: &IpcMessage) {
        let event_name = match &message.event {
            Some(e) => e,
            None => return,
        };

        let detail = message.detail.clone().unwrap_or(serde_json::Value::Null);

        if let Ok(listeners) = self.event_listeners.read() {
            if let Some(handlers) = listeners.get(event_name) {
                for handler in handlers {
                    handler(detail.clone());
                }
            }
        }
    }

    /// Handle call message (returns response)
    fn handle_call(&self, message: &IpcMessage) -> Option<String> {
        let method = match &message.method {
            Some(m) => m,
            None => return None,
        };

        let id = match &message.id {
            Some(i) => i.clone(),
            None => return None,
        };

        let params = message.params.clone().unwrap_or(serde_json::Value::Null);

        let result = if let Ok(handlers) = self.handlers.read() {
            if let Some(handler) = handlers.get(method) {
                handler(params)
            } else {
                return Some(
                    serde_json::to_string(&IpcResponse::err(
                        id,
                        "NotFound",
                        &format!("Method not found: {}", method),
                    ))
                    .ok()?,
                );
            }
        } else {
            return None;
        };

        Some(serde_json::to_string(&IpcResponse::ok(id, result)).ok()?)
    }

    /// Handle invoke message (plugin commands)
    fn handle_invoke(&self, message: &IpcMessage) -> Option<String> {
        let cmd = match &message.cmd {
            Some(c) => c,
            None => return None,
        };

        let id = match &message.id {
            Some(i) => i.clone(),
            None => return None,
        };

        let args = message
            .args
            .clone()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        // Invoke commands are prefixed with plugin name: "fs:read_file"
        let result = if let Ok(handlers) = self.handlers.read() {
            if let Some(handler) = handlers.get(cmd) {
                handler(args)
            } else {
                return Some(
                    serde_json::to_string(&IpcResponse::err(
                        id,
                        "NotFound",
                        &format!("Command not found: {}", cmd),
                    ))
                    .ok()?,
                );
            }
        } else {
            return None;
        };

        Some(serde_json::to_string(&IpcResponse::ok(id, result)).ok()?)
    }
}

impl Default for IpcRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcRouter {
    /// Check if a handler exists for a method
    pub fn has_handler(&self, method: &str) -> bool {
        if let Ok(handlers) = self.handlers.read() {
            handlers.contains_key(method)
        } else {
            false
        }
    }

    /// Unregister a handler
    pub fn unregister(&self, method: &str) -> bool {
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.remove(method).is_some()
        } else {
            false
        }
    }

    /// Get all registered method names
    pub fn methods(&self) -> Vec<String> {
        if let Ok(handlers) = self.handlers.read() {
            handlers.keys().cloned().collect()
        } else {
            vec![]
        }
    }
}
