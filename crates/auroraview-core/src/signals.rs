//! Signal-Slot System for AuroraView
//!
//! A Qt-inspired signal-slot pattern for event handling with:
//! - Type-safe signals with generic payloads
//! - Multiple handlers per signal (multi-receiver support)
//! - Automatic cleanup when ConnectionId is dropped
//! - Thread-safe operations using parking_lot
//!
//! # Example
//!
//! ```rust
//! use auroraview_core::signals::{Signal, ConnectionId};
//!
//! let signal: Signal<String> = Signal::new();
//!
//! // Connect handler
//! let conn = signal.connect(|msg| {
//!     println!("Received: {}", msg);
//! });
//!
//! // Emit signal
//! signal.emit("Hello".to_string());
//!
//! // Disconnect (or let conn drop)
//! signal.disconnect(conn);
//! ```

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Unique identifier for a signal connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Get the raw ID value
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Global counter for generating unique connection IDs
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique connection ID
fn next_connection_id() -> ConnectionId {
    ConnectionId(NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst))
}

/// Handler function type
type Handler<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;

/// A type-safe signal that can have multiple connected handlers
///
/// Signals emit values to all connected handlers when `emit()` is called.
/// Handlers can be connected with `connect()` and disconnected with `disconnect()`.
pub struct Signal<T: Clone + Send + 'static> {
    handlers: RwLock<HashMap<ConnectionId, Handler<T>>>,
}

impl<T: Clone + Send + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + 'static> Signal<T> {
    /// Create a new signal with no connected handlers
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Connect a handler to this signal
    ///
    /// Returns a ConnectionId that can be used to disconnect the handler.
    /// The handler will be called each time the signal is emitted.
    pub fn connect<F>(&self, handler: F) -> ConnectionId
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let id = next_connection_id();
        self.handlers.write().insert(id, Arc::new(handler));
        id
    }

    /// Connect a handler that will only be called once
    ///
    /// After the first emission, the handler is automatically disconnected.
    pub fn connect_once<F>(&self, handler: F) -> ConnectionId
    where
        F: FnOnce(T) + Send + Sync + 'static,
    {
        let id = next_connection_id();
        let handler_cell = Arc::new(parking_lot::Mutex::new(Some(handler)));
        let handler_clone = handler_cell.clone();

        self.handlers.write().insert(
            id,
            Arc::new(move |value: T| {
                if let Some(h) = handler_clone.lock().take() {
                    h(value);
                }
            }),
        );
        id
    }

    /// Disconnect a handler by its ConnectionId
    ///
    /// Returns true if a handler was removed, false if the ID was not found.
    pub fn disconnect(&self, id: ConnectionId) -> bool {
        self.handlers.write().remove(&id).is_some()
    }

    /// Emit a value to all connected handlers
    ///
    /// Each handler receives a clone of the value.
    pub fn emit(&self, value: T) {
        let handlers = self.handlers.read();
        for handler in handlers.values() {
            handler(value.clone());
        }
    }

    /// Get the number of connected handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.read().len()
    }

    /// Check if any handlers are connected
    pub fn is_connected(&self) -> bool {
        !self.handlers.read().is_empty()
    }

    /// Disconnect all handlers
    pub fn disconnect_all(&self) {
        self.handlers.write().clear();
    }
}

// ============================================================================
// ConnectionGuard - RAII-style automatic disconnection
// ============================================================================

/// A guard that automatically disconnects a handler when dropped
///
/// This provides RAII-style cleanup for signal connections.
/// When the guard goes out of scope, the handler is automatically disconnected.
pub struct ConnectionGuard<T: Clone + Send + 'static> {
    signal: Arc<Signal<T>>,
    id: ConnectionId,
    detached: bool,
}

impl<T: Clone + Send + 'static> ConnectionGuard<T> {
    /// Create a new connection guard
    pub fn new(signal: Arc<Signal<T>>, id: ConnectionId) -> Self {
        Self {
            signal,
            id,
            detached: false,
        }
    }

    /// Get the connection ID
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Detach the guard, preventing automatic disconnection on drop
    ///
    /// After calling this, the handler will remain connected even after
    /// the guard is dropped.
    pub fn detach(mut self) {
        self.detached = true;
    }

    /// Manually disconnect the handler
    pub fn disconnect(mut self) -> bool {
        self.detached = true; // Prevent double disconnect
        self.signal.disconnect(self.id)
    }
}

impl<T: Clone + Send + 'static> Drop for ConnectionGuard<T> {
    fn drop(&mut self) {
        if !self.detached {
            self.signal.disconnect(self.id);
        }
    }
}

// ============================================================================
// SignalRegistry - Dynamic signal management
// ============================================================================

/// A registry for dynamically named signals with JSON values
///
/// This allows creating and accessing signals by name at runtime,
/// useful for event-driven systems where signal names are not known at compile time.
///
/// # Simple API
///
/// ```ignore
/// let registry = SignalRegistry::new();
///
/// // Connect handler (creates signal if needed)
/// let conn = registry.connect("my_event", |data| {
///     println!("Received: {:?}", data);
/// });
///
/// // Emit to signal
/// registry.emit("my_event", json!({"key": "value"}));
///
/// // Disconnect
/// registry.disconnect("my_event", conn);
/// ```
pub struct SignalRegistry {
    /// Dynamic signals registry
    signals: RwLock<HashMap<String, Arc<Signal<serde_json::Value>>>>,
}

impl Default for SignalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            signals: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a signal by name
    ///
    /// If the signal doesn't exist, a new one is created.
    pub fn get_or_create(&self, name: &str) -> Arc<Signal<serde_json::Value>> {
        // Fast path: try to get existing signal with read lock
        {
            let signals = self.signals.read();
            if let Some(signal) = signals.get(name) {
                return signal.clone();
            }
        }

        // Slow path: create new signal with write lock
        let mut signals = self.signals.write();
        signals
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Signal::new()))
            .clone()
    }

    /// Get a signal by name, returns None if it doesn't exist
    pub fn get(&self, name: &str) -> Option<Arc<Signal<serde_json::Value>>> {
        self.signals.read().get(name).cloned()
    }

    /// Check if a signal exists
    pub fn contains(&self, name: &str) -> bool {
        self.signals.read().contains_key(name)
    }

    /// Remove a signal by name
    ///
    /// Returns true if the signal was removed.
    pub fn remove(&self, name: &str) -> bool {
        self.signals.write().remove(name).is_some()
    }

    /// Get all signal names
    pub fn names(&self) -> Vec<String> {
        self.signals.read().keys().cloned().collect()
    }

    /// Connect a handler to a named signal
    ///
    /// This is the recommended API - creates the signal if it doesn't exist.
    ///
    /// # Example
    /// ```ignore
    /// let conn = registry.connect("my_event", |data| {
    ///     println!("Received: {:?}", data);
    /// });
    /// ```
    pub fn connect<F>(&self, name: &str, handler: F) -> ConnectionId
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        self.get_or_create(name).connect(handler)
    }

    /// Connect a one-time handler to a named signal
    ///
    /// The handler will be automatically disconnected after first emission.
    pub fn connect_once<F>(&self, name: &str, handler: F) -> ConnectionId
    where
        F: FnOnce(serde_json::Value) + Send + Sync + 'static,
    {
        self.get_or_create(name).connect_once(handler)
    }

    /// Emit a value to a named signal
    ///
    /// Does nothing if the signal doesn't exist.
    pub fn emit(&self, name: &str, value: serde_json::Value) {
        if let Some(signal) = self.get(name) {
            signal.emit(value);
        }
    }

    /// Disconnect a handler from a named signal
    pub fn disconnect(&self, name: &str, id: ConnectionId) -> bool {
        if let Some(signal) = self.get(name) {
            signal.disconnect(id)
        } else {
            false
        }
    }
}

// ============================================================================
// WebViewSignals - Pre-defined signals for WebView lifecycle
// ============================================================================

/// Pre-defined signals for WebView lifecycle and events
///
/// These signals are emitted automatically by the WebView during its lifecycle.
/// Applications can connect handlers to respond to these events.
pub struct WebViewSignals {
    /// Emitted when the page has finished loading
    pub page_loaded: Signal<()>,

    /// Emitted when the WebView is about to close
    pub closing: Signal<()>,

    /// Emitted when the WebView has closed
    pub closed: Signal<()>,

    /// Emitted when the WebView receives focus
    pub focused: Signal<()>,

    /// Emitted when the WebView loses focus
    pub blurred: Signal<()>,

    /// Emitted when the WebView is resized (width, height)
    pub resized: Signal<(u32, u32)>,

    /// Emitted when the WebView is moved (x, y)
    pub moved: Signal<(i32, i32)>,

    /// Emitted when the WebView is minimized
    pub minimized: Signal<()>,

    /// Emitted when the WebView is maximized
    pub maximized: Signal<()>,

    /// Emitted when the WebView is restored from minimized/maximized state
    pub restored: Signal<()>,

    /// Dynamic signal registry for custom events
    pub custom: SignalRegistry,
}

impl Default for WebViewSignals {
    fn default() -> Self {
        Self::new()
    }
}

impl WebViewSignals {
    /// Create a new set of WebView signals
    pub fn new() -> Self {
        Self {
            page_loaded: Signal::new(),
            closing: Signal::new(),
            closed: Signal::new(),
            focused: Signal::new(),
            blurred: Signal::new(),
            resized: Signal::new(),
            moved: Signal::new(),
            minimized: Signal::new(),
            maximized: Signal::new(),
            restored: Signal::new(),
            custom: SignalRegistry::new(),
        }
    }

    /// Get or create a custom signal by name
    pub fn get_custom(&self, name: &str) -> Arc<Signal<serde_json::Value>> {
        self.custom.get_or_create(name)
    }

    /// Connect a handler to a custom signal
    pub fn on<F>(&self, event_name: &str, handler: F) -> ConnectionId
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        self.custom.connect(event_name, handler)
    }

    /// Emit a custom event
    pub fn emit_custom(&self, event_name: &str, data: serde_json::Value) {
        self.custom.emit(event_name, data);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_signal_connect_emit() {
        let signal: Signal<i32> = Signal::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _conn = signal.connect(move |value| {
            counter_clone.fetch_add(value as usize, Ordering::SeqCst);
        });

        signal.emit(5);
        signal.emit(3);

        assert_eq!(counter.load(Ordering::SeqCst), 8);
    }

    #[test]
    fn test_signal_disconnect() {
        let signal: Signal<i32> = Signal::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let conn = signal.connect(move |value| {
            counter_clone.fetch_add(value as usize, Ordering::SeqCst);
        });

        signal.emit(5);
        assert_eq!(counter.load(Ordering::SeqCst), 5);

        signal.disconnect(conn);
        signal.emit(3);
        assert_eq!(counter.load(Ordering::SeqCst), 5); // Still 5, handler disconnected
    }

    #[test]
    fn test_signal_multiple_handlers() {
        let signal: Signal<i32> = Signal::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let c1 = counter1.clone();
        let c2 = counter2.clone();

        let _conn1 = signal.connect(move |v| {
            c1.fetch_add(v as usize, Ordering::SeqCst);
        });
        let _conn2 = signal.connect(move |v| {
            c2.fetch_add(v as usize * 2, Ordering::SeqCst);
        });

        signal.emit(5);

        assert_eq!(counter1.load(Ordering::SeqCst), 5);
        assert_eq!(counter2.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_connect_once() {
        let signal: Signal<i32> = Signal::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _conn = signal.connect_once(move |value| {
            counter_clone.fetch_add(value as usize, Ordering::SeqCst);
        });

        signal.emit(5);
        signal.emit(3); // Should not trigger again

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_signal_registry() {
        let registry = SignalRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _conn = registry.connect("test_event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        registry.emit("test_event", serde_json::json!({"key": "value"}));
        registry.emit("test_event", serde_json::json!(null));

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_webview_signals() {
        let signals = WebViewSignals::new();
        let loaded = Arc::new(AtomicUsize::new(0));
        let loaded_clone = loaded.clone();

        signals.page_loaded.connect(move |_| {
            loaded_clone.fetch_add(1, Ordering::SeqCst);
        });

        signals.page_loaded.emit(());

        assert_eq!(loaded.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_webview_custom_signals() {
        let signals = WebViewSignals::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        signals.on("custom_event", move |data| {
            if let Some(n) = data.get("count").and_then(|v| v.as_u64()) {
                counter_clone.fetch_add(n as usize, Ordering::SeqCst);
            }
        });

        signals.emit_custom("custom_event", serde_json::json!({"count": 42}));

        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_registry_connect_creates_signal() {
        let registry = SignalRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // connect() creates the signal automatically
        let _conn = registry.connect("new_event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Signal now exists
        assert!(registry.contains("new_event"));

        registry.emit("new_event", serde_json::json!({}));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_registry_connect_once() {
        let registry = SignalRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _conn = registry.connect_once("one_time", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        registry.emit("one_time", serde_json::json!(1));
        registry.emit("one_time", serde_json::json!(2)); // Won't trigger

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_registry_disconnect() {
        let registry = SignalRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let conn = registry.connect("my_event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        registry.emit("my_event", serde_json::json!({}));
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Disconnect
        assert!(registry.disconnect("my_event", conn));

        registry.emit("my_event", serde_json::json!({}));
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Still 1
    }

    #[test]
    fn test_registry_remove_signal() {
        let registry = SignalRegistry::new();

        registry.connect("temp_signal", |_| {});
        assert!(registry.contains("temp_signal"));

        assert!(registry.remove("temp_signal"));
        assert!(!registry.contains("temp_signal"));

        // Remove non-existent signal returns false
        assert!(!registry.remove("non_existent"));
    }

    #[test]
    fn test_registry_names() {
        let registry = SignalRegistry::new();

        registry.connect("event_a", |_| {});
        registry.connect("event_b", |_| {});

        let names = registry.names();
        assert!(names.contains(&"event_a".to_string()));
        assert!(names.contains(&"event_b".to_string()));
    }
}
