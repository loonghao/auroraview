//! IPC handler and router

use super::message::{IpcMessage, IpcResponse};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Callback type for IPC handlers
pub type IpcCallback = Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

/// IPC router for handling messages from JavaScript
///
/// Thread-safe router that dispatches IPC messages to registered handlers.
/// Uses DashMap for fine-grained concurrent access without global read/write locks.
pub struct IpcRouter {
    /// Registered handlers by method name
    handlers: DashMap<String, IpcCallback>,

    /// Event listeners
    #[allow(clippy::type_complexity)]
    event_listeners: DashMap<String, Vec<Arc<dyn Fn(serde_json::Value) + Send + Sync>>>,
}

impl IpcRouter {
    /// Create new IPC router
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
            event_listeners: DashMap::new(),
        }
    }

    /// Register a handler for a method
    pub fn register<F>(&self, method: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        self.handlers.insert(method.to_string(), Box::new(handler));
        debug!("[IPC] Registered handler: {}", method);
    }

    /// Unregister a handler
    pub fn unregister(&self, method: &str) -> bool {
        self.handlers.remove(method).is_some()
    }

    /// Check if a handler is registered
    pub fn has_handler(&self, method: &str) -> bool {
        self.handlers.contains_key(method)
    }

    /// Get all registered method names
    pub fn methods(&self) -> Vec<String> {
        self.handlers.iter().map(|e| e.key().clone()).collect()
    }

    /// Subscribe to an event
    pub fn on<F>(&self, event: &str, handler: F)
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        self.event_listeners
            .entry(event.to_string())
            .or_default()
            .push(Arc::new(handler));
        debug!("[IPC] Subscribed to event: {}", event);
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

        if let Some(handlers) = self.event_listeners.get(event_name) {
            let listeners = handlers.value();
            if let Some((last, rest)) = listeners.split_last() {
                for handler in rest {
                    handler(detail.clone());
                }
                last(detail);
            }
        }
    }

    /// Handle call message (returns response)
    fn handle_call(&self, message: &IpcMessage) -> Option<String> {
        let method = message.method.as_ref()?;
        let id = message.id.as_ref()?.clone();
        let params = message.params.clone().unwrap_or(serde_json::Value::Null);

        let result = if let Some(handler) = self.handlers.get(method) {
            handler.value()(params)
        } else {
            return serde_json::to_string(&IpcResponse::err(
                id,
                "NotFound",
                &format!("Method not found: {}", method),
            ))
            .ok();
        };

        serde_json::to_string(&IpcResponse::ok(id, result)).ok()
    }

    /// Handle invoke message (plugin commands)
    fn handle_invoke(&self, message: &IpcMessage) -> Option<String> {
        let cmd = message.cmd.as_ref()?;
        let id = message.id.as_ref()?.clone();
        let args = message
            .args
            .clone()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let result = if let Some(handler) = self.handlers.get(cmd) {
            handler.value()(args)
        } else {
            return serde_json::to_string(&IpcResponse::err(
                id,
                "NotFound",
                &format!("Command not found: {}", cmd),
            ))
            .ok();
        };

        serde_json::to_string(&IpcResponse::ok(id, result)).ok()
    }
}

impl Default for IpcRouter {
    fn default() -> Self {
        Self::new()
    }
}
