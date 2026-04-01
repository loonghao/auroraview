//! IPC handler and router

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use dashmap::DashMap;
use tracing::{debug, warn};

use super::message::{IpcMessage, IpcResponse};

/// Unique ID returned by [`IpcRouter::on`] and consumed by [`IpcRouter::off`].
pub type ListenerId = u64;

/// Callback type for IPC handlers
pub type IpcCallback = Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

/// Entry stored per event listener
struct ListenerEntry {
    id: ListenerId,
    handler: Arc<dyn Fn(serde_json::Value) + Send + Sync>,
}

/// Global counter for listener IDs
static NEXT_LISTENER_ID: AtomicU64 = AtomicU64::new(1);

fn next_listener_id() -> ListenerId {
    NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed)
}

/// IPC router for handling messages from JavaScript
///
/// Thread-safe router that dispatches IPC messages to registered handlers.
/// Uses DashMap for fine-grained concurrent access without global read/write locks.
pub struct IpcRouter {
    /// Registered handlers by method name
    handlers: DashMap<String, IpcCallback>,

    /// Event listeners — each entry carries an ID for targeted removal via `off()`
    event_listeners: DashMap<String, Vec<ListenerEntry>>,
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
    ///
    /// Returns a [`ListenerId`] that can be passed to [`off`](IpcRouter::off)
    /// to remove this specific listener.
    pub fn on<F>(&self, event: &str, handler: F) -> ListenerId
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        let id = next_listener_id();
        self.event_listeners
            .entry(event.to_string())
            .or_default()
            .push(ListenerEntry { id, handler: Arc::new(handler) });
        debug!("[IPC] Subscribed to event: {} (id={})", event, id);
        id
    }

    /// Unsubscribe a specific event listener by its [`ListenerId`]
    ///
    /// Returns `true` if the listener was found and removed, `false` otherwise.
    pub fn off(&self, event: &str, id: ListenerId) -> bool {
        if let Some(mut entry) = self.event_listeners.get_mut(event) {
            let before = entry.len();
            entry.retain(|e| e.id != id);
            let removed = entry.len() < before;
            if removed {
                debug!("[IPC] Unsubscribed from event: {} (id={})", event, id);
            }
            return removed;
        }
        false
    }

    /// Unsubscribe all listeners for an event
    pub fn off_all(&self, event: &str) -> usize {
        if let Some(mut entry) = self.event_listeners.get_mut(event) {
            let count = entry.len();
            entry.clear();
            debug!("[IPC] Cleared all listeners for event: {} ({})", event, count);
            count
        } else {
            0
        }
    }

    /// Get the number of listeners registered for an event
    pub fn listener_count(&self, event: &str) -> usize {
        self.event_listeners
            .get(event)
            .map(|e| e.len())
            .unwrap_or(0)
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
            let listeners: Vec<Arc<dyn Fn(serde_json::Value) + Send + Sync>> =
                handlers.iter().map(|e| e.handler.clone()).collect();
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
