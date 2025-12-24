//! Type-safe Signal implementation
//!
//! A Qt-inspired signal that can have multiple connected handlers.
//! Signals emit values to all connected handlers when `emit()` is called.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::{next_connection_id, ConnectionGuard, ConnectionId};

/// Handler function type - boxed trait object for type erasure
type Handler<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;

/// A type-safe signal that can have multiple connected handlers
///
/// Signals emit values to all connected handlers when `emit()` is called.
/// Handlers can be connected with `connect()` and disconnected with `disconnect()`.
///
/// # Type Safety
///
/// The signal is generic over the payload type `T`, providing compile-time
/// type checking for both emitters and handlers.
///
/// # Thread Safety
///
/// Signals are thread-safe and can be shared across threads using `Arc<Signal<T>>`.
/// All operations use `parking_lot::RwLock` for efficient concurrent access.
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
///
/// // Create a signal that emits strings
/// let signal: Signal<String> = Signal::new();
///
/// // Connect multiple handlers
/// let conn1 = signal.connect(|msg| println!("Handler 1: {}", msg));
/// let conn2 = signal.connect(|msg| println!("Handler 2: {}", msg));
///
/// // Emit a value - both handlers are called
/// signal.emit("Hello".to_string());
///
/// // Disconnect a specific handler
/// signal.disconnect(conn1);
///
/// // Only handler 2 receives this
/// signal.emit("World".to_string());
/// ```
pub struct Signal<T: Clone + Send + 'static> {
    handlers: RwLock<HashMap<ConnectionId, Handler<T>>>,
    name: Option<String>,
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
            name: None,
        }
    }

    /// Create a new named signal
    ///
    /// The name is used for debugging and logging purposes.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            name: Some(name.into()),
        }
    }

    /// Get the signal's name, if any
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Connect a handler to this signal
    ///
    /// Returns a `ConnectionId` that can be used to disconnect the handler.
    /// The handler will be called each time the signal is emitted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    ///
    /// let signal: Signal<i32> = Signal::new();
    /// let conn = signal.connect(|x| println!("Received: {}", x));
    /// ```
    pub fn connect<F>(&self, handler: F) -> ConnectionId
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let id = next_connection_id();
        self.handlers.write().insert(id, Arc::new(handler));
        tracing::trace!(
            signal_name = ?self.name,
            connection_id = %id,
            "Handler connected"
        );
        id
    }

    /// Connect a handler that will only be called once
    ///
    /// After the first emission, the handler is automatically disconnected.
    /// This is useful for one-time events like initialization or cleanup.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    ///
    /// let signal: Signal<String> = Signal::new();
    ///
    /// signal.connect_once(|msg| {
    ///     println!("First message only: {}", msg);
    /// });
    ///
    /// signal.emit("First".to_string());  // Handler called
    /// signal.emit("Second".to_string()); // Handler NOT called
    /// ```
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

        tracing::trace!(
            signal_name = ?self.name,
            connection_id = %id,
            "One-time handler connected"
        );
        id
    }

    /// Connect a handler and return a guard for automatic cleanup
    ///
    /// The returned `ConnectionGuard` will automatically disconnect the handler
    /// when it goes out of scope (RAII pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    /// use std::sync::Arc;
    ///
    /// let signal = Arc::new(Signal::<i32>::new());
    ///
    /// {
    ///     let _guard = signal.connect_guard(|x| println!("{}", x));
    ///     signal.emit(1); // Handler called
    /// } // guard dropped, handler disconnected
    ///
    /// signal.emit(2); // Handler NOT called
    /// ```
    pub fn connect_guard<F>(self: &Arc<Self>, handler: F) -> ConnectionGuard<T>
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let id = self.connect(handler);
        ConnectionGuard::new(self.clone(), id)
    }

    /// Disconnect a handler by its ConnectionId
    ///
    /// Returns `true` if a handler was removed, `false` if the ID was not found.
    pub fn disconnect(&self, id: ConnectionId) -> bool {
        let removed = self.handlers.write().remove(&id).is_some();
        if removed {
            tracing::trace!(
                signal_name = ?self.name,
                connection_id = %id,
                "Handler disconnected"
            );
        }
        removed
    }

    /// Emit a value to all connected handlers
    ///
    /// Each handler receives a clone of the value. Handlers are called
    /// in an unspecified order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aurora_signals::prelude::*;
    ///
    /// let signal: Signal<String> = Signal::new();
    /// signal.connect(|msg| println!("{}", msg));
    /// signal.emit("Hello".to_string());
    /// ```
    pub fn emit(&self, value: T) {
        let handlers = self.handlers.read();
        let count = handlers.len();

        tracing::trace!(
            signal_name = ?self.name,
            handler_count = count,
            "Emitting signal"
        );

        for handler in handlers.values() {
            handler(value.clone());
        }
    }

    /// Emit a value and return the number of handlers that received it
    pub fn emit_count(&self, value: T) -> usize {
        let handlers = self.handlers.read();
        let count = handlers.len();

        for handler in handlers.values() {
            handler(value.clone());
        }

        count
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
        let count = self.handlers.write().len();
        self.handlers.write().clear();
        tracing::trace!(
            signal_name = ?self.name,
            disconnected_count = count,
            "All handlers disconnected"
        );
    }

    /// Get all connection IDs
    pub fn connections(&self) -> Vec<ConnectionId> {
        self.handlers.read().keys().copied().collect()
    }
}

impl<T: Clone + Send + 'static> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("name", &self.name)
            .field("handler_count", &self.handler_count())
            .finish()
    }
}

// Signal is Send + Sync because handlers are Arc<dyn Fn + Send + Sync>
// and we use RwLock for synchronization
unsafe impl<T: Clone + Send + 'static> Send for Signal<T> {}
unsafe impl<T: Clone + Send + 'static> Sync for Signal<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_signal_basic() {
        let signal: Signal<i32> = Signal::new();
        let received = Arc::new(AtomicUsize::new(0));

        let r = received.clone();
        signal.connect(move |x| {
            r.fetch_add(x as usize, Ordering::SeqCst);
        });

        signal.emit(5);
        assert_eq!(received.load(Ordering::SeqCst), 5);

        signal.emit(3);
        assert_eq!(received.load(Ordering::SeqCst), 8);
    }

    #[test]
    fn test_signal_multiple_handlers() {
        let signal: Signal<i32> = Signal::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c1 = count.clone();
        signal.connect(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = count.clone();
        signal.connect(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_signal_disconnect() {
        let signal: Signal<i32> = Signal::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        let conn = signal.connect(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        signal.disconnect(conn);

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_signal_connect_once() {
        let signal: Signal<i32> = Signal::new();
        let count = Arc::new(AtomicUsize::new(0));

        let c = count.clone();
        signal.connect_once(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1); // Still 1, not called again
    }

    #[test]
    fn test_signal_named() {
        let signal: Signal<i32> = Signal::named("test:signal");
        assert_eq!(signal.name(), Some("test:signal"));

        let unnamed: Signal<i32> = Signal::new();
        assert_eq!(unnamed.name(), None);
    }

    #[test]
    fn test_signal_handler_count() {
        let signal: Signal<i32> = Signal::new();
        assert_eq!(signal.handler_count(), 0);
        assert!(!signal.is_connected());

        let conn1 = signal.connect(|_| {});
        assert_eq!(signal.handler_count(), 1);
        assert!(signal.is_connected());

        let conn2 = signal.connect(|_| {});
        assert_eq!(signal.handler_count(), 2);

        signal.disconnect(conn1);
        assert_eq!(signal.handler_count(), 1);

        signal.disconnect(conn2);
        assert_eq!(signal.handler_count(), 0);
        assert!(!signal.is_connected());
    }

    #[test]
    fn test_signal_disconnect_all() {
        let signal: Signal<i32> = Signal::new();

        signal.connect(|_| {});
        signal.connect(|_| {});
        signal.connect(|_| {});

        assert_eq!(signal.handler_count(), 3);

        signal.disconnect_all();
        assert_eq!(signal.handler_count(), 0);
    }

    #[test]
    fn test_signal_emit_count() {
        let signal: Signal<i32> = Signal::new();

        assert_eq!(signal.emit_count(0), 0);

        signal.connect(|_| {});
        signal.connect(|_| {});

        assert_eq!(signal.emit_count(0), 2);
    }

    #[test]
    fn test_signal_thread_safety() {
        use std::thread;

        let signal = Arc::new(Signal::<i32>::new());
        let count = Arc::new(AtomicUsize::new(0));

        // Connect from main thread
        let c = count.clone();
        signal.connect(move |x| {
            c.fetch_add(x as usize, Ordering::SeqCst);
        });

        // Emit from multiple threads
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let s = signal.clone();
                thread::spawn(move || {
                    s.emit(i);
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
