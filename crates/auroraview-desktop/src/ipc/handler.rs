//! IPC handler and router

use super::message::{IpcMessage, IpcResponse};
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// Per-process "warn once" guards for drag-drop events without a listener.
///
/// Drag-drop events (`file_drop_hover` / `file_drop` / `file_drop_cancelled`)
/// fire per OS-level cursor transition — without rate-limiting a single
/// drag-and-drop session would emit 2-3 identical warnings, and a stuck
/// "no listener" misconfiguration would spam the tracing subscriber.
///
/// The set of event names is closed (RFC 0015 §5), so we keep one
/// `AtomicBool` per known name plus a single catch-all. The `match` in
/// [`should_warn_drag_drop_listener_missing`] is exhaustive on the closed
/// set; adding a fourth `file_drop_*` event in RFC 0015 will surface as
/// a missing arm there rather than silently flowing into the catch-all.
///
/// Scope is process-global rather than per-`IpcRouter`: in production a
/// single process owns at most one router, and "warn once across the
/// whole process" matches operator expectations better than "warn once
/// per router instance" (which would be silently affected by reload /
/// hot-restart paths).
static DRAG_DROP_WARN_HOVER: AtomicBool = AtomicBool::new(false);
static DRAG_DROP_WARN_DROP: AtomicBool = AtomicBool::new(false);
static DRAG_DROP_WARN_CANCELLED: AtomicBool = AtomicBool::new(false);

/// Catch-all guard for any drag-drop event name outside the closed set
/// in [`should_warn_drag_drop_listener_missing`]. See that function's
/// docstring for rationale.
static UNKNOWN_DRAG_DROP_WARNED: AtomicBool = AtomicBool::new(false);

/// Try to flip the warn-once guard for `event_name`.
///
/// Returns `true` exactly once per process per event name, `false` on
/// every subsequent call. Lock-free; no allocation regardless of
/// `event_name`.
///
/// `Ordering::Relaxed` is sufficient: the warn is purely diagnostic and
/// we accept that two threads racing on the same first event could both
/// produce a single warn line — the cost of an extra memory fence on
/// every drag-drop event is not worth ruling that out.
fn should_warn_drag_drop_listener_missing(event_name: &str) -> bool {
    let guard = match event_name {
        "file_drop_hover" => &DRAG_DROP_WARN_HOVER,
        "file_drop" => &DRAG_DROP_WARN_DROP,
        "file_drop_cancelled" => &DRAG_DROP_WARN_CANCELLED,
        _ => &UNKNOWN_DRAG_DROP_WARNED,
    };
    !guard.swap(true, Ordering::Relaxed)
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
        } else if should_warn_drag_drop_listener_missing(event_name) {
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
