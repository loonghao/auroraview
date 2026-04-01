//! Type-safe Signal implementation
//!
//! A Qt-inspired signal that can have multiple connected handlers.
//! Signals emit values to all connected handlers when `emit()` is called.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::{next_connection_id, ConnectionGuard, ConnectionId};

/// Handler function type — takes value by move (one clone per non-last handler)
type Handler<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;

/// Ref-handler function type — takes value by shared reference (zero clones)
type RefHandler<T> = Arc<dyn Fn(&T) + Send + Sync + 'static>;

/// A type-safe signal that can have multiple connected handlers
///
/// Signals emit values to all connected handlers when `emit()` is called.
/// Handlers can be connected with `connect()` and disconnected with `disconnect()`.
///
/// # Handler variants
///
/// | Method | Signature | Clone cost |
/// |---|---|---|
/// | `connect` | `Fn(T)` | one clone per non-last handler |
/// | `connect_ref` | `Fn(&T)` | **zero clones** |
///
/// Use `connect_ref` when the handler only needs to inspect the value
/// (e.g., logging, metrics, read-only side effects) and the payload type
/// is expensive to clone.
///
/// # Thread Safety
///
/// Signals are thread-safe and can be shared across threads using `Arc<Signal<T>>`.
/// All operations use `parking_lot::RwLock` for efficient concurrent access.
///
/// # Example
///
/// ```rust
/// use auroraview_signals::prelude::*;
///
/// // Create a signal that emits strings
/// let signal: Signal<String> = Signal::new();
///
/// // Connect a by-value handler (one clone per non-last handler)
/// let conn1 = signal.connect(|msg| println!("Handler 1: {}", msg));
///
/// // Connect a by-reference handler — zero clones
/// let conn2 = signal.connect_ref(|msg| println!("Handler 2 (ref): {}", msg));
///
/// // Emit a value — both handlers are called
/// signal.emit("Hello".to_string());
///
/// // Disconnect a specific handler
/// signal.disconnect(conn1);
///
/// // Only conn2 receives this
/// signal.emit("World".to_string());
/// ```
pub struct Signal<T: Clone + Send + 'static> {
    handlers: RwLock<HashMap<ConnectionId, Handler<T>>>,
    ref_handlers: RwLock<HashMap<ConnectionId, RefHandler<T>>>,
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
            ref_handlers: RwLock::new(HashMap::new()),
            name: None,
        }
    }

    /// Create a new named signal
    ///
    /// The name is used for debugging and logging purposes.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            ref_handlers: RwLock::new(HashMap::new()),
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
    /// use auroraview_signals::prelude::*;
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
    /// use auroraview_signals::prelude::*;
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
    /// use auroraview_signals::prelude::*;
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

    /// Connect a by-reference handler — zero clones per emit
    ///
    /// The handler receives `&T` instead of `T`, so no clone is performed
    /// regardless of the number of connected ref-handlers.
    ///
    /// Use this when the handler only needs to read the value (logging,
    /// metrics, forwarding by reference) and the payload is expensive to clone.
    ///
    /// # Example
    ///
    /// ```rust
    /// use auroraview_signals::prelude::*;
    ///
    /// let signal: Signal<String> = Signal::new();
    ///
    /// // handler receives &String — no clone
    /// let conn = signal.connect_ref(|msg| println!("ref handler: {}", msg));
    ///
    /// signal.emit("hello".to_string());
    /// signal.disconnect(conn);
    /// ```
    pub fn connect_ref<F>(&self, handler: F) -> ConnectionId
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = next_connection_id();
        self.ref_handlers.write().insert(id, Arc::new(handler));
        tracing::trace!(
            signal_name = ?self.name,
            connection_id = %id,
            "Ref-handler connected"
        );
        id
    }

    /// Connect a by-reference handler and return a guard for automatic cleanup
    ///
    /// Combines `connect_ref` with RAII disconnection (see `connect_guard`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use auroraview_signals::prelude::*;
    /// use std::sync::Arc;
    ///
    /// let signal = Arc::new(Signal::<String>::new());
    ///
    /// {
    ///     let _guard = signal.connect_ref_guard(|s| println!("ref: {}", s));
    ///     signal.emit("hello".to_string());
    /// } // guard dropped → disconnected
    ///
    /// signal.emit("ignored".to_string());
    /// ```
    pub fn connect_ref_guard<F>(self: &Arc<Self>, handler: F) -> ConnectionGuard<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = self.connect_ref(handler);
        ConnectionGuard::new(self.clone(), id)
    }

    /// Disconnect a handler by its ConnectionId
    ///
    /// Returns `true` if a handler was removed, `false` if the ID was not found.
    /// Works for both by-value (`connect`) and by-reference (`connect_ref`) handlers.
    pub fn disconnect(&self, id: ConnectionId) -> bool {
        let removed_value = self.handlers.write().remove(&id).is_some();
        let removed_ref = self.ref_handlers.write().remove(&id).is_some();
        let removed = removed_value || removed_ref;
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
    /// - By-reference handlers (`connect_ref`) receive `&value` — zero clones.
    /// - By-value handlers (`connect`) each receive a clone of the value except
    ///   the very last one, which receives the value by move.
    ///
    /// Handlers are called in an unspecified order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use auroraview_signals::prelude::*;
    ///
    /// let signal: Signal<String> = Signal::new();
    /// signal.connect(|msg| println!("{}", msg));
    /// signal.emit("Hello".to_string());
    /// ```
    pub fn emit(&self, value: T) {
        // Snapshot both handler maps before releasing any lock.
        // This prevents deadlocks when a handler calls connect/disconnect
        // on the same signal during emission.
        let (value_handlers, ref_hdls): (Vec<Handler<T>>, Vec<RefHandler<T>>) = {
            let vg = self.handlers.read();
            let rg = self.ref_handlers.read();
            tracing::trace!(
                signal_name = ?self.name,
                value_handler_count = vg.len(),
                ref_handler_count = rg.len(),
                "Emitting signal"
            );
            (
                vg.values().cloned().collect(),
                rg.values().cloned().collect(),
            )
        };
        // Ref-handlers first — zero clones.
        for rh in &ref_hdls {
            rh(&value);
        }
        // Value-handlers: move into the last one to save one clone.
        if let Some((last, rest)) = value_handlers.split_last() {
            for handler in rest {
                handler(value.clone());
            }
            last(value);
        }
    }

    /// Emit a value by shared reference to all connected handlers
    ///
    /// Only by-reference handlers (`connect_ref`) are invoked; by-value
    /// handlers are **not** called. This is a zero-clone, zero-allocation path
    /// and is useful when the caller already holds a reference and does not
    /// need to trigger value-consuming handlers.
    ///
    /// Returns the number of ref-handlers that received the value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use auroraview_signals::prelude::*;
    ///
    /// let signal: Signal<String> = Signal::new();
    /// signal.connect_ref(|msg| println!("ref: {}", msg));
    ///
    /// let value = "hello".to_string();
    /// let count = signal.emit_ref(&value);
    /// assert_eq!(count, 1);
    /// // `value` is still usable here
    /// ```
    pub fn emit_ref(&self, value: &T) -> usize {
        let ref_hdls: Vec<RefHandler<T>> = {
            let rg = self.ref_handlers.read();
            rg.values().cloned().collect()
        };
        let count = ref_hdls.len();
        for rh in &ref_hdls {
            rh(value);
        }
        count
    }

    /// Emit a value and return the number of handlers that received it
    pub fn emit_count(&self, value: T) -> usize {
        let (value_handlers, ref_hdls): (Vec<Handler<T>>, Vec<RefHandler<T>>) = {
            let vg = self.handlers.read();
            let rg = self.ref_handlers.read();
            (
                vg.values().cloned().collect(),
                rg.values().cloned().collect(),
            )
        };
        let count = value_handlers.len() + ref_hdls.len();
        for rh in &ref_hdls {
            rh(&value);
        }
        if let Some((last, rest)) = value_handlers.split_last() {
            for handler in rest {
                handler(value.clone());
            }
            last(value);
        }
        count
    }

    /// Get the number of connected handlers (both by-value and by-reference)
    pub fn handler_count(&self) -> usize {
        self.handlers.read().len() + self.ref_handlers.read().len()
    }

    /// Check if any handlers are connected
    pub fn is_connected(&self) -> bool {
        !self.handlers.read().is_empty() || !self.ref_handlers.read().is_empty()
    }

    /// Disconnect all handlers (both by-value and by-reference)
    pub fn disconnect_all(&self) {
        let mut value_map = self.handlers.write();
        let mut ref_map = self.ref_handlers.write();
        let count = value_map.len() + ref_map.len();
        value_map.clear();
        ref_map.clear();
        tracing::trace!(
            signal_name = ?self.name,
            disconnected_count = count,
            "All handlers disconnected"
        );
    }

    /// Get all connection IDs (both by-value and by-reference)
    pub fn connections(&self) -> Vec<ConnectionId> {
        let mut ids: Vec<ConnectionId> = self.handlers.read().keys().copied().collect();
        ids.extend(self.ref_handlers.read().keys().copied());
        ids
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

    #[test]
    fn test_emit_handler_can_disconnect_without_deadlock() {
        // This test verifies the clone-before-iterate fix:
        // A handler that calls disconnect on the same signal must not deadlock.
        let signal = Arc::new(Signal::<i32>::new());
        let call_count = Arc::new(AtomicUsize::new(0));

        let s_clone = signal.clone();
        let cc = call_count.clone();
        let conn = signal.connect(move |_| {
            cc.fetch_add(1, Ordering::SeqCst);
            // Attempt to disconnect self during emit — this would deadlock
            // with the old implementation that held the read lock.
            s_clone.disconnect_all();
        });

        // This should NOT deadlock
        signal.emit(42);

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // After disconnect_all, no handlers remain
        assert_eq!(signal.handler_count(), 0);
        let _ = conn; // suppress unused warning
    }

    #[test]
    fn test_emit_handler_can_connect_without_deadlock() {
        // A handler that calls connect on the same signal must not deadlock.
        let signal = Arc::new(Signal::<i32>::new());
        let call_count = Arc::new(AtomicUsize::new(0));

        let s_clone = signal.clone();
        let cc = call_count.clone();
        signal.connect(move |_| {
            cc.fetch_add(1, Ordering::SeqCst);
            // Connect a new handler during emit
            s_clone.connect(|_| {});
        });

        signal.emit(1);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // Original handler + newly connected handler
        assert_eq!(signal.handler_count(), 2);
    }

    #[test]
    fn test_disconnect_all_concurrent() {
        use std::thread;

        let signal = Arc::new(Signal::<i32>::new());

        // Add many handlers
        for _ in 0..100 {
            signal.connect(|_| {});
        }
        assert_eq!(signal.handler_count(), 100);

        // Concurrent disconnect_all from multiple threads
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let s = signal.clone();
                thread::spawn(move || {
                    s.disconnect_all();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(signal.handler_count(), 0);
    }

    #[test]
    fn test_connect_ref_receives_value() {
        let signal: Signal<String> = Signal::new();
        let received = Arc::new(parking_lot::Mutex::new(Vec::<String>::new()));
        let r = received.clone();
        signal.connect_ref(move |s| r.lock().push(s.clone()));

        signal.emit("hello".to_string());
        signal.emit("world".to_string());

        let r = received.lock();
        assert_eq!(*r, vec!["hello", "world"]);
    }

    #[test]
    fn test_connect_ref_handler_count() {
        let signal: Signal<i32> = Signal::new();
        assert_eq!(signal.handler_count(), 0);

        let c1 = signal.connect_ref(|_| {});
        assert_eq!(signal.handler_count(), 1);
        assert!(signal.is_connected());

        let _c2 = signal.connect_ref(|_| {});
        assert_eq!(signal.handler_count(), 2);

        signal.disconnect(c1);
        assert_eq!(signal.handler_count(), 1);
    }

    #[test]
    fn test_connect_ref_disconnect_clears_handler() {
        let signal: Signal<i32> = Signal::new();
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let conn = signal.connect_ref(move |_| { c.fetch_add(1, Ordering::SeqCst); });

        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        signal.disconnect(conn);
        signal.emit(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_emit_ref_only_calls_ref_handlers() {
        let signal: Signal<i32> = Signal::new();
        let ref_count = Arc::new(AtomicUsize::new(0));
        let val_count = Arc::new(AtomicUsize::new(0));

        let rc = ref_count.clone();
        signal.connect_ref(move |_| { rc.fetch_add(1, Ordering::SeqCst); });

        let vc = val_count.clone();
        signal.connect(move |_| { vc.fetch_add(1, Ordering::SeqCst); });

        let n = signal.emit_ref(&42);
        assert_eq!(n, 1);
        assert_eq!(ref_count.load(Ordering::SeqCst), 1);
        // Value handler was NOT called by emit_ref
        assert_eq!(val_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_emit_triggers_both_handler_kinds() {
        let signal: Signal<i32> = Signal::new();
        let ref_count = Arc::new(AtomicUsize::new(0));
        let val_count = Arc::new(AtomicUsize::new(0));

        let rc = ref_count.clone();
        signal.connect_ref(move |_| { rc.fetch_add(1, Ordering::SeqCst); });

        let vc = val_count.clone();
        signal.connect(move |_| { vc.fetch_add(1, Ordering::SeqCst); });

        signal.emit(1);
        assert_eq!(ref_count.load(Ordering::SeqCst), 1);
        assert_eq!(val_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_emit_count_includes_ref_handlers() {
        let signal: Signal<i32> = Signal::new();
        signal.connect(|_| {});
        signal.connect_ref(|_| {});
        signal.connect_ref(|_| {});
        assert_eq!(signal.emit_count(0), 3);
    }

    #[test]
    fn test_disconnect_all_clears_ref_handlers() {
        let signal: Signal<i32> = Signal::new();
        signal.connect(|_| {});
        signal.connect_ref(|_| {});
        assert_eq!(signal.handler_count(), 2);

        signal.disconnect_all();
        assert_eq!(signal.handler_count(), 0);
        assert!(!signal.is_connected());
    }

    #[test]
    fn test_connections_includes_ref_handler_ids() {
        let signal: Signal<i32> = Signal::new();
        let c1 = signal.connect(|_| {});
        let c2 = signal.connect_ref(|_| {});

        let ids = signal.connections();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&c1));
        assert!(ids.contains(&c2));
    }
}

