//! Connection management for signal-slot system
//!
//! This module provides:
//! - `ConnectionId`: Unique identifier for signal connections
//! - `ConnectionGuard`: RAII-style automatic disconnection

use crate::signal::Signal;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Global counter for generating unique connection IDs
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique connection ID
pub(crate) fn next_connection_id() -> ConnectionId {
    ConnectionId(NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst))
}

/// Unique identifier for a signal connection
///
/// Each connection to a signal receives a unique ID that can be used
/// to disconnect the handler later.
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
///
/// let signal: Signal<i32> = Signal::new();
/// let conn_id = signal.connect(|x| println!("{}", x));
///
/// // Use the ID to disconnect
/// signal.disconnect(conn_id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub(crate) u64);

impl ConnectionId {
    /// Get the raw ID value
    pub fn id(&self) -> u64 {
        self.0
    }

    /// Create a ConnectionId from a raw value (for deserialization)
    pub fn from_raw(id: u64) -> Self {
        ConnectionId(id)
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConnectionId({})", self.0)
    }
}

/// A guard that automatically disconnects a handler when dropped
///
/// This provides RAII-style cleanup for signal connections.
/// When the guard goes out of scope, the handler is automatically disconnected.
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
/// use std::sync::Arc;
///
/// let signal = Arc::new(Signal::<String>::new());
///
/// {
///     let guard = ConnectionGuard::new(
///         signal.clone(),
///         signal.connect(|msg| println!("{}", msg))
///     );
///     
///     signal.emit("Hello".to_string()); // Handler is called
///     
///     // guard goes out of scope here, handler is disconnected
/// }
///
/// signal.emit("World".to_string()); // Handler is NOT called
/// ```
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
    /// the guard is dropped. Returns the connection ID for manual management.
    pub fn detach(mut self) -> ConnectionId {
        self.detached = true;
        self.id
    }

    /// Manually disconnect the handler
    ///
    /// Returns true if the handler was disconnected, false if it was already gone.
    pub fn disconnect(mut self) -> bool {
        self.detached = true; // Prevent double disconnect
        self.signal.disconnect(self.id)
    }

    /// Check if the guard is still attached (will disconnect on drop)
    pub fn is_attached(&self) -> bool {
        !self.detached
    }
}

impl<T: Clone + Send + 'static> Drop for ConnectionGuard<T> {
    fn drop(&mut self) {
        if !self.detached {
            self.signal.disconnect(self.id);
        }
    }
}

impl<T: Clone + Send + 'static> std::fmt::Debug for ConnectionGuard<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionGuard")
            .field("id", &self.id)
            .field("detached", &self.detached)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id_uniqueness() {
        let id1 = next_connection_id();
        let id2 = next_connection_id();
        let id3 = next_connection_id();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_connection_id_display() {
        let id = ConnectionId(42);
        assert_eq!(format!("{}", id), "ConnectionId(42)");
    }

    #[test]
    fn test_connection_id_from_raw() {
        let id = ConnectionId::from_raw(100);
        assert_eq!(id.id(), 100);
    }

    #[test]
    fn test_connection_guard_auto_disconnect() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let signal = Arc::new(Signal::<i32>::new());
        let call_count = Arc::new(AtomicUsize::new(0));

        {
            let count = call_count.clone();
            let _guard = ConnectionGuard::new(
                signal.clone(),
                signal.connect(move |_| {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
            );

            signal.emit(1);
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
        }

        // Guard dropped, handler disconnected
        signal.emit(2);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_connection_guard_detach() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let signal = Arc::new(Signal::<i32>::new());
        let call_count = Arc::new(AtomicUsize::new(0));

        let conn_id = {
            let count = call_count.clone();
            let guard = ConnectionGuard::new(
                signal.clone(),
                signal.connect(move |_| {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
            );

            signal.emit(1);
            assert_eq!(call_count.load(Ordering::SeqCst), 1);

            guard.detach() // Detach before drop
        };

        // Guard dropped but handler still connected
        signal.emit(2);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        // Manual disconnect
        signal.disconnect(conn_id);
        signal.emit(3);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
