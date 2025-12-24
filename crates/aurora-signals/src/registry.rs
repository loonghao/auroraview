//! Dynamic Signal Registry
//!
//! A registry for dynamically named signals with JSON values.
//! This allows creating and accessing signals by name at runtime,
//! useful for event-driven systems where signal names are not known at compile time.

use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::ConnectionId;
use crate::signal::Signal;

/// A registry for dynamically named signals with JSON values
///
/// This allows creating and accessing signals by name at runtime,
/// useful for event-driven systems where signal names are not known at compile time.
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
/// use serde_json::json;
///
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
    signals: RwLock<HashMap<String, Arc<Signal<Value>>>>,
    /// Optional name for this registry (for debugging)
    name: Option<String>,
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
            name: None,
        }
    }

    /// Create a new named registry
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            signals: RwLock::new(HashMap::new()),
            name: Some(name.into()),
        }
    }

    /// Get the registry's name, if any
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get or create a signal by name
    ///
    /// If the signal doesn't exist, a new one is created.
    pub fn get_or_create(&self, name: &str) -> Arc<Signal<Value>> {
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
            .or_insert_with(|| {
                tracing::trace!(
                    registry_name = ?self.name,
                    signal_name = name,
                    "Creating new signal"
                );
                Arc::new(Signal::named(name))
            })
            .clone()
    }

    /// Get a signal by name, returns None if it doesn't exist
    pub fn get(&self, name: &str) -> Option<Arc<Signal<Value>>> {
        self.signals.read().get(name).cloned()
    }

    /// Check if a signal exists
    pub fn contains(&self, name: &str) -> bool {
        self.signals.read().contains_key(name)
    }

    /// Remove a signal by name
    ///
    /// Returns true if the signal was removed.
    /// Note: This will disconnect all handlers for that signal.
    pub fn remove(&self, name: &str) -> bool {
        let removed = self.signals.write().remove(name).is_some();
        if removed {
            tracing::trace!(
                registry_name = ?self.name,
                signal_name = name,
                "Signal removed"
            );
        }
        removed
    }

    /// Get all signal names
    pub fn names(&self) -> Vec<String> {
        self.signals.read().keys().cloned().collect()
    }

    /// Get the number of registered signals
    pub fn signal_count(&self) -> usize {
        self.signals.read().len()
    }

    /// Connect a handler to a named signal
    ///
    /// This is the recommended API - creates the signal if it doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    ///
    /// let registry = SignalRegistry::new();
    /// let conn = registry.connect("my_event", |data| {
    ///     println!("Received: {:?}", data);
    /// });
    /// ```
    pub fn connect<F>(&self, name: &str, handler: F) -> ConnectionId
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        self.get_or_create(name).connect(handler)
    }

    /// Connect a one-time handler to a named signal
    ///
    /// The handler will be automatically disconnected after first emission.
    pub fn connect_once<F>(&self, name: &str, handler: F) -> ConnectionId
    where
        F: FnOnce(Value) + Send + Sync + 'static,
    {
        self.get_or_create(name).connect_once(handler)
    }

    /// Emit a value to a named signal
    ///
    /// Does nothing if the signal doesn't exist.
    /// Returns the number of handlers that received the value.
    pub fn emit(&self, name: &str, value: Value) -> usize {
        if let Some(signal) = self.get(name) {
            tracing::trace!(
                registry_name = ?self.name,
                signal_name = name,
                "Emitting to signal"
            );
            signal.emit_count(value)
        } else {
            tracing::trace!(
                registry_name = ?self.name,
                signal_name = name,
                "Signal not found, skipping emit"
            );
            0
        }
    }

    /// Emit a value to a named signal, creating it if needed
    ///
    /// Unlike `emit()`, this will create the signal if it doesn't exist.
    /// Returns the number of handlers that received the value.
    pub fn emit_or_create(&self, name: &str, value: Value) -> usize {
        self.get_or_create(name).emit_count(value)
    }

    /// Disconnect a handler from a named signal
    pub fn disconnect(&self, name: &str, id: ConnectionId) -> bool {
        if let Some(signal) = self.get(name) {
            signal.disconnect(id)
        } else {
            false
        }
    }

    /// Disconnect all handlers from a named signal
    pub fn disconnect_all(&self, name: &str) {
        if let Some(signal) = self.get(name) {
            signal.disconnect_all();
        }
    }

    /// Clear all signals from the registry
    pub fn clear(&self) {
        self.signals.write().clear();
        tracing::trace!(
            registry_name = ?self.name,
            "Registry cleared"
        );
    }

    /// Get handler count for a specific signal
    pub fn handler_count(&self, name: &str) -> usize {
        self.get(name).map(|s| s.handler_count()).unwrap_or(0)
    }

    /// Get total handler count across all signals
    pub fn total_handler_count(&self) -> usize {
        self.signals
            .read()
            .values()
            .map(|s| s.handler_count())
            .sum()
    }

    /// Check if a signal has any handlers
    pub fn is_connected(&self, name: &str) -> bool {
        self.get(name).map(|s| s.is_connected()).unwrap_or(false)
    }
}

impl std::fmt::Debug for SignalRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalRegistry")
            .field("name", &self.name)
            .field("signal_count", &self.signal_count())
            .field("total_handlers", &self.total_handler_count())
            .finish()
    }
}

// SignalRegistry is Send + Sync
unsafe impl Send for SignalRegistry {}
unsafe impl Sync for SignalRegistry {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_registry_basic() {
        let registry = SignalRegistry::new();
        let received = Arc::new(AtomicUsize::new(0));

        let r = received.clone();
        registry.connect("test", move |data| {
            if let Some(n) = data.as_u64() {
                r.fetch_add(n as usize, Ordering::SeqCst);
            }
        });

        registry.emit("test", json!(5));
        assert_eq!(received.load(Ordering::SeqCst), 5);

        registry.emit("test", json!(3));
        assert_eq!(received.load(Ordering::SeqCst), 8);
    }

    #[test]
    fn test_registry_auto_create() {
        let registry = SignalRegistry::new();

        assert!(!registry.contains("new_signal"));

        registry.connect("new_signal", |_| {});

        assert!(registry.contains("new_signal"));
    }

    #[test]
    fn test_registry_emit_nonexistent() {
        let registry = SignalRegistry::new();

        // Should not panic, just return 0
        let count = registry.emit("nonexistent", json!(null));
        assert_eq!(count, 0);
    }

    #[test]
    fn test_registry_disconnect() {
        let registry = SignalRegistry::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        let conn = registry.connect("test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        registry.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        registry.disconnect("test", conn);

        registry.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_registry_remove_signal() {
        let registry = SignalRegistry::new();

        registry.connect("test", |_| {});
        assert!(registry.contains("test"));

        registry.remove("test");
        assert!(!registry.contains("test"));
    }

    #[test]
    fn test_registry_names() {
        let registry = SignalRegistry::new();

        registry.connect("signal_a", |_| {});
        registry.connect("signal_b", |_| {});
        registry.connect("signal_c", |_| {});

        let mut names = registry.names();
        names.sort();

        assert_eq!(names, vec!["signal_a", "signal_b", "signal_c"]);
    }

    #[test]
    fn test_registry_handler_counts() {
        let registry = SignalRegistry::new();

        registry.connect("signal_a", |_| {});
        registry.connect("signal_a", |_| {});
        registry.connect("signal_b", |_| {});

        assert_eq!(registry.handler_count("signal_a"), 2);
        assert_eq!(registry.handler_count("signal_b"), 1);
        assert_eq!(registry.handler_count("nonexistent"), 0);
        assert_eq!(registry.total_handler_count(), 3);
    }

    #[test]
    fn test_registry_clear() {
        let registry = SignalRegistry::new();

        registry.connect("signal_a", |_| {});
        registry.connect("signal_b", |_| {});

        assert_eq!(registry.signal_count(), 2);

        registry.clear();

        assert_eq!(registry.signal_count(), 0);
    }

    #[test]
    fn test_registry_connect_once() {
        let registry = SignalRegistry::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        registry.connect_once("test", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        registry.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        registry.emit("test", json!(null));
        assert_eq!(count.load(Ordering::SeqCst), 1); // Still 1
    }

    #[test]
    fn test_registry_named() {
        let registry = SignalRegistry::named("my_registry");
        assert_eq!(registry.name(), Some("my_registry"));

        let unnamed = SignalRegistry::new();
        assert_eq!(unnamed.name(), None);
    }

    #[test]
    fn test_registry_thread_safety() {
        use std::thread;

        let registry = Arc::new(SignalRegistry::new());
        let count = Arc::new(AtomicUsize::new(0));

        // Connect from main thread
        let c = count.clone();
        registry.connect("test", move |data| {
            if let Some(n) = data.as_u64() {
                c.fetch_add(n as usize, Ordering::SeqCst);
            }
        });

        // Emit from multiple threads
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let r = registry.clone();
                thread::spawn(move || {
                    r.emit("test", json!(i));
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // Sum of 0..10 = 45
        assert_eq!(count.load(Ordering::SeqCst), 45);
    }
}
