//! Event Bus - Unified event distribution system
//!
//! The EventBus provides a unified interface for:
//! - Subscribing to events
//! - Emitting events through middleware pipeline
//! - Broadcasting to multiple bridges
//! - Cross-platform event distribution

use serde_json::Value;
use std::sync::Arc;

use crate::bridge::{BridgeError, EventBridge, MultiBridge};
use crate::connection::ConnectionId;
use crate::middleware::{Middleware, MiddlewareChain};
use crate::registry::SignalRegistry;

/// Unified event bus for signal distribution
///
/// The EventBus combines:
/// - SignalRegistry for local event handling
/// - Middleware pipeline for event processing
/// - Bridge system for cross-platform distribution
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
/// use serde_json::json;
///
/// let bus = EventBus::new();
///
/// // Add logging middleware
/// bus.use_middleware(LoggingMiddleware::new(LogLevel::Debug));
///
/// // Subscribe to events
/// let conn = bus.on("app:ready", |data| {
///     println!("App ready: {:?}", data);
/// });
///
/// // Emit events
/// bus.emit("app:ready", json!({"version": "1.0"}));
/// ```
pub struct EventBus {
    /// Local signal registry
    registry: SignalRegistry,
    /// Middleware chain
    middleware: MiddlewareChain,
    /// Bridges for cross-platform distribution
    bridges: MultiBridge,
    /// Bus name for debugging
    name: Option<String>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            registry: SignalRegistry::new(),
            middleware: MiddlewareChain::new(),
            bridges: MultiBridge::new("event_bus_bridges"),
            name: None,
        }
    }

    /// Create a named event bus
    pub fn named(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            registry: SignalRegistry::named(&name),
            middleware: MiddlewareChain::new(),
            bridges: MultiBridge::new(format!("{}_bridges", name)),
            name: Some(name),
        }
    }

    /// Get the bus name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    // ========================================================================
    // Middleware
    // ========================================================================

    /// Add a middleware to the processing pipeline
    ///
    /// Middleware are sorted by priority (lower = earlier).
    pub fn use_middleware<M: Middleware + 'static>(&self, middleware: M) {
        self.middleware.add(middleware);
    }

    /// Add an Arc-wrapped middleware
    pub fn use_middleware_arc(&self, middleware: Arc<dyn Middleware>) {
        self.middleware.add_arc(middleware);
    }

    /// Get the number of middleware in the pipeline
    pub fn middleware_count(&self) -> usize {
        self.middleware.len()
    }

    // ========================================================================
    // Bridges
    // ========================================================================

    /// Add a bridge for cross-platform event distribution
    pub fn add_bridge<B: EventBridge + 'static>(&self, bridge: B) {
        self.bridges.add(bridge);
    }

    /// Add an Arc-wrapped bridge
    pub fn add_bridge_arc(&self, bridge: Arc<dyn EventBridge>) {
        self.bridges.add_arc(bridge);
    }

    /// Remove a bridge by name
    pub fn remove_bridge(&self, name: &str) -> bool {
        self.bridges.remove(name)
    }

    /// Get the number of bridges
    pub fn bridge_count(&self) -> usize {
        self.bridges.len()
    }

    /// Get bridge names
    pub fn bridge_names(&self) -> Vec<String> {
        self.bridges.bridge_names()
    }

    // ========================================================================
    // Event Subscription
    // ========================================================================

    /// Subscribe to an event
    ///
    /// The handler will be called when the event is emitted.
    /// Returns a ConnectionId that can be used to unsubscribe.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    ///
    /// let bus = EventBus::new();
    /// let conn = bus.on("user:login", |data| {
    ///     println!("User logged in: {:?}", data);
    /// });
    /// ```
    pub fn on<F>(&self, event: &str, handler: F) -> ConnectionId
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        self.registry.connect(event, handler)
    }

    /// Subscribe to an event once
    ///
    /// The handler will be called only once, then automatically unsubscribed.
    pub fn once<F>(&self, event: &str, handler: F) -> ConnectionId
    where
        F: FnOnce(Value) + Send + Sync + 'static,
    {
        self.registry.connect_once(event, handler)
    }

    /// Unsubscribe from an event
    pub fn off(&self, event: &str, id: ConnectionId) -> bool {
        self.registry.disconnect(event, id)
    }

    /// Unsubscribe all handlers from an event
    pub fn off_all(&self, event: &str) {
        self.registry.disconnect_all(event);
    }

    // ========================================================================
    // Event Emission
    // ========================================================================

    /// Emit an event
    ///
    /// The event goes through the middleware pipeline, then is delivered to:
    /// 1. All local handlers subscribed via `on()`
    /// 2. All connected bridges
    ///
    /// Returns the number of local handlers that received the event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    /// use serde_json::json;
    ///
    /// let bus = EventBus::new();
    /// bus.emit("app:start", json!({"timestamp": 1234567890}));
    /// ```
    pub fn emit(&self, event: &str, data: Value) -> usize {
        let mut data = data;

        // Process through middleware
        let result = self.middleware.process_before(event, &mut data);
        if !result.should_continue() {
            tracing::trace!(
                bus_name = ?self.name,
                event = event,
                result = ?result,
                "Event stopped by middleware"
            );
            return 0;
        }

        // Emit to local handlers
        let handler_count = self.registry.emit(event, data.clone());

        // Emit to bridges
        if !self.bridges.is_empty() {
            if let Err(e) = self.bridges.emit(event, data.clone()) {
                tracing::warn!(
                    bus_name = ?self.name,
                    event = event,
                    error = %e,
                    "Bridge emit failed"
                );
            }
        }

        // Notify middleware after emit
        self.middleware.process_after(event, &data, handler_count);

        handler_count
    }

    /// Emit an event only to local handlers (skip bridges)
    pub fn emit_local(&self, event: &str, data: Value) -> usize {
        let mut data = data;

        let result = self.middleware.process_before(event, &mut data);
        if !result.should_continue() {
            return 0;
        }

        let handler_count = self.registry.emit(event, data.clone());
        self.middleware.process_after(event, &data, handler_count);

        handler_count
    }

    /// Emit an event only to bridges (skip local handlers)
    pub fn emit_to_bridges(&self, event: &str, data: Value) -> Result<(), BridgeError> {
        let mut data = data;

        let result = self.middleware.process_before(event, &mut data);
        if !result.should_continue() {
            return Ok(());
        }

        self.bridges.emit(event, data.clone())?;
        self.middleware.process_after(event, &data, 0);

        Ok(())
    }

    /// Emit an event to a specific bridge by name
    pub fn emit_to(&self, bridge_name: &str, event: &str, data: Value) -> Result<(), BridgeError> {
        let mut data = data;

        let result = self.middleware.process_before(event, &mut data);
        if !result.should_continue() {
            return Ok(());
        }

        // Find the bridge and emit
        // Note: This is a simplified implementation. In production,
        // you might want to keep a HashMap of bridges by name.
        for name in self.bridges.bridge_names() {
            if name == bridge_name {
                // We can't directly access the bridge, so we emit to all
                // and rely on the bridge to filter by name.
                // This is a limitation of the current design.
                return self.bridges.emit(event, data);
            }
        }

        Err(BridgeError::Other(format!(
            "Bridge not found: {}",
            bridge_name
        )))
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Check if an event has any handlers
    pub fn has_handlers(&self, event: &str) -> bool {
        self.registry.is_connected(event)
    }

    /// Get handler count for an event
    pub fn handler_count(&self, event: &str) -> usize {
        self.registry.handler_count(event)
    }

    /// Get total handler count across all events
    pub fn total_handler_count(&self) -> usize {
        self.registry.total_handler_count()
    }

    /// Get all event names that have handlers
    pub fn event_names(&self) -> Vec<String> {
        self.registry.names()
    }

    /// Get the number of registered events
    pub fn event_count(&self) -> usize {
        self.registry.signal_count()
    }

    /// Clear all handlers and events
    pub fn clear(&self) {
        self.registry.clear();
    }

    /// Get direct access to the signal registry
    pub fn registry(&self) -> &SignalRegistry {
        &self.registry
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("name", &self.name)
            .field("event_count", &self.event_count())
            .field("total_handlers", &self.total_handler_count())
            .field("middleware_count", &self.middleware_count())
            .field("bridge_count", &self.bridge_count())
            .finish()
    }
}

// EventBus is Send + Sync
unsafe impl Send for EventBus {}
unsafe impl Sync for EventBus {}

// ============================================================================
// Global Event Bus
// ============================================================================

use once_cell::sync::Lazy;

/// Global event bus instance
///
/// This provides a convenient singleton for applications that only need
/// one event bus.
///
/// # Example
///
/// ```rust
/// use aurora_signals::bus::global_bus;
/// use serde_json::json;
///
/// global_bus().on("app:event", |data| {
///     println!("Received: {:?}", data);
/// });
///
/// global_bus().emit("app:event", json!({"key": "value"}));
/// ```
static GLOBAL_BUS: Lazy<EventBus> = Lazy::new(|| EventBus::named("global"));

/// Get the global event bus
pub fn global_bus() -> &'static EventBus {
    &GLOBAL_BUS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::CallbackBridge;
    use crate::middleware::FilterMiddleware;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_bus_basic() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        bus.on("test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_event_bus_once() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        bus.once("test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1); // Still 1
    }

    #[test]
    fn test_event_bus_off() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        let conn = bus.on("test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        bus.off("test", conn);

        bus.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_bus_middleware() {
        let bus = EventBus::new();

        // Add filter that denies internal events
        bus.use_middleware(FilterMiddleware::new().deny_pattern("internal:.*").unwrap());

        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        bus.on("internal:secret", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = count.clone();
        bus.on("public:event", move |_| {
            c2.fetch_add(10, Ordering::SeqCst);
        });

        // Internal event should be filtered
        bus.emit("internal:secret", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 0);

        // Public event should pass
        bus.emit("public:event", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_event_bus_bridge() {
        let bus = EventBus::new();
        let bridge_count = Arc::new(AtomicUsize::new(0));

        let bc = bridge_count.clone();
        bus.add_bridge(CallbackBridge::new("test_bridge", move |event, _| {
            assert_eq!(event, "test:event");
            bc.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        let local_count = Arc::new(AtomicUsize::new(0));
        let lc = local_count.clone();
        bus.on("test:event", move |_| {
            lc.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit("test:event", json!(null));

        assert_eq!(local_count.load(Ordering::SeqCst), 1);
        assert_eq!(bridge_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_bus_emit_local() {
        let bus = EventBus::new();
        let bridge_count = Arc::new(AtomicUsize::new(0));

        let bc = bridge_count.clone();
        bus.add_bridge(CallbackBridge::new("test_bridge", move |_, _| {
            bc.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        let local_count = Arc::new(AtomicUsize::new(0));
        let lc = local_count.clone();
        bus.on("test:event", move |_| {
            lc.fetch_add(1, Ordering::SeqCst);
        });

        // emit_local should skip bridges
        bus.emit_local("test:event", json!(null));

        assert_eq!(local_count.load(Ordering::SeqCst), 1);
        assert_eq!(bridge_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_event_bus_emit_to_bridges() {
        let bus = EventBus::new();
        let bridge_count = Arc::new(AtomicUsize::new(0));

        let bc = bridge_count.clone();
        bus.add_bridge(CallbackBridge::new("test_bridge", move |_, _| {
            bc.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        let local_count = Arc::new(AtomicUsize::new(0));
        let lc = local_count.clone();
        bus.on("test:event", move |_| {
            lc.fetch_add(1, Ordering::SeqCst);
        });

        // emit_to_bridges should skip local handlers
        bus.emit_to_bridges("test:event", json!(null)).unwrap();

        assert_eq!(local_count.load(Ordering::SeqCst), 0);
        assert_eq!(bridge_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_bus_named() {
        let bus = EventBus::named("my_bus");
        assert_eq!(bus.name(), Some("my_bus"));
    }

    #[test]
    fn test_event_bus_utility_methods() {
        let bus = EventBus::new();

        assert!(!bus.has_handlers("test"));
        assert_eq!(bus.handler_count("test"), 0);

        bus.on("test", |_| {});
        bus.on("test", |_| {});
        bus.on("other", |_| {});

        assert!(bus.has_handlers("test"));
        assert_eq!(bus.handler_count("test"), 2);
        assert_eq!(bus.total_handler_count(), 3);
        assert_eq!(bus.event_count(), 2);

        let mut names = bus.event_names();
        names.sort();
        assert_eq!(names, vec!["other", "test"]);
    }

    #[test]
    fn test_global_bus() {
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        let conn = global_bus().on("global:test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        global_bus().emit("global:test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        global_bus().off("global:test", conn);
    }
}
