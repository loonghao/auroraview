//! IPC handler and router

use super::message::{IpcMessage, IpcResponse};
use dashmap::{DashMap, DashSet};
use std::sync::Arc;
use tracing::{debug, warn};

/// Callback type for IPC handlers
pub type IpcCallback = Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

/// IPC router for handling messages from JavaScript
///
/// Uses DashMap for fine-grained concurrent access without global read/write locks.
pub struct IpcRouter {
    /// Registered handlers by method name
    handlers: DashMap<String, IpcCallback>,

    /// Event listeners
    #[allow(clippy::type_complexity)]
    event_listeners: DashMap<String, Vec<Arc<dyn Fn(serde_json::Value) + Send + Sync>>>,

    /// Drag-drop event names already warned about. Drag-drop events
    /// (`file_drop_hover` / `file_drop` / `file_drop_cancelled`) are
    /// dispatched per OS-level cursor transition — without rate-limiting
    /// a single drag-and-drop session would emit 2-3 identical warnings,
    /// and a stuck "no listener" misconfiguration would spam the log.
    /// We warn at most once per event name per `IpcRouter` lifetime.
    drag_drop_warned: DashSet<String>,
}

impl IpcRouter {
    /// Create new IPC router
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
            event_listeners: DashMap::new(),
            drag_drop_warned: DashSet::new(),
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
            for handler in handlers.value() {
                handler(detail.clone());
            }
        }
    }

    /// Handle call message (returns response)
    fn handle_call(&self, message: &IpcMessage) -> Option<String> {
        let method = message.method.as_ref()?;

        let id = message.id.clone()?;

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

        let id = message.id.clone()?;

        let args = message
            .args
            .clone()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        // Invoke commands are prefixed with plugin name: "fs:read_file"
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

    /// Check if a handler exists for a method
    pub fn has_handler(&self, method: &str) -> bool {
        self.handlers.contains_key(method)
    }

    /// Unregister a handler
    pub fn unregister(&self, method: &str) -> bool {
        self.handlers.remove(method).is_some()
    }

    /// Get all registered method names
    pub fn methods(&self) -> Vec<String> {
        self.handlers.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for IpcRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl auroraview_core::builder::DragDropIpcSink for IpcRouter {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), auroraview_core::builder::DispatchError> {
        // Forward to every subscribed listener. If none are registered we
        // emit a single `tracing::warn!` per event name and then silently
        // drop subsequent occurrences — drag-drop events fire per OS
        // cursor transition, so an unconditional log would flood the
        // tracing subscriber. Returning `Ok(())` (instead of `Err`) keeps
        // the helper from logging an additional `error!` on every event:
        // the warning here is the canonical "you forgot router.on(...)"
        // breadcrumb.
        if let Some(handlers) = self.event_listeners.get(event_name) {
            for handler in handlers.value() {
                handler(data.clone());
            }
        } else if self.drag_drop_warned.insert(event_name.to_string()) {
            warn!(
                target: "auroraview::drag_drop",
                "[IpcRouter] no listener registered for `{}`; \
                 dropping payload (subsequent occurrences will be silent). \
                 Did you call `router.on(\"{}\", ...)`?",
                event_name,
                event_name
            );
        }
        Ok(())
    }
}
