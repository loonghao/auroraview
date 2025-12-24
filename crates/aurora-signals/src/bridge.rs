//! Event Bridge system for cross-platform event forwarding
//!
//! Bridges allow forwarding events to different platforms/systems:
//! - WebView (via message queue)
//! - Python callbacks
//! - IPC channels
//! - Custom bridges

use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur in event bridges
#[derive(Error, Debug)]
pub enum BridgeError {
    /// Failed to send event
    #[error("Failed to send event: {0}")]
    SendFailed(String),

    /// Bridge is disconnected
    #[error("Bridge disconnected: {0}")]
    Disconnected(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Bridge-specific error
    #[error("Bridge error: {0}")]
    Other(String),
}

impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::SerializationError(err.to_string())
    }
}

/// Event bridge trait for cross-platform event forwarding
///
/// Implement this trait to create custom bridges for different platforms.
///
/// # Example
///
/// ```rust
/// use aurora_signals::bridge::{EventBridge, BridgeError};
/// use serde_json::Value;
///
/// struct ConsoleBridge;
///
/// impl EventBridge for ConsoleBridge {
///     fn name(&self) -> &str {
///         "console"
///     }
///
///     fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError> {
///         println!("[{}] {:?}", event, data);
///         Ok(())
///     }
/// }
/// ```
pub trait EventBridge: Send + Sync {
    /// Get the bridge name (for identification)
    fn name(&self) -> &str;

    /// Emit an event through the bridge
    fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError>;

    /// Subscribe to events from this bridge (optional)
    ///
    /// Some bridges support bidirectional communication.
    /// Default implementation does nothing.
    fn subscribe(&self, _event: &str) -> Result<(), BridgeError> {
        Ok(())
    }

    /// Unsubscribe from events (optional)
    fn unsubscribe(&self, _event: &str) -> Result<(), BridgeError> {
        Ok(())
    }

    /// Check if the bridge is connected
    fn is_connected(&self) -> bool {
        true
    }

    /// Disconnect the bridge
    fn disconnect(&self) -> Result<(), BridgeError> {
        Ok(())
    }

    /// Get bridge priority (lower = higher priority for broadcasting)
    fn priority(&self) -> i32 {
        100
    }
}

// ============================================================================
// CallbackBridge - Bridge using a callback function
// ============================================================================

/// Type alias for callback function
pub type BridgeCallback = Arc<dyn Fn(&str, Value) -> Result<(), BridgeError> + Send + Sync>;

/// A bridge that forwards events to a callback function
///
/// This is useful for integrating with Python or other languages
/// where you want to receive events via a callback.
pub struct CallbackBridge {
    name: String,
    callback: BridgeCallback,
    connected: std::sync::atomic::AtomicBool,
}

impl CallbackBridge {
    /// Create a new callback bridge
    pub fn new<F>(name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(&str, Value) -> Result<(), BridgeError> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            callback: Arc::new(callback),
            connected: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl EventBridge for CallbackBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError> {
        if !self.is_connected() {
            return Err(BridgeError::Disconnected(self.name.clone()));
        }
        (self.callback)(event, data)
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn disconnect(&self) -> Result<(), BridgeError> {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

// ============================================================================
// ChannelBridge - Bridge using crossbeam channel
// ============================================================================

/// Message sent through a channel bridge
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    /// Event name
    pub event: String,
    /// Event data
    pub data: Value,
}

/// A bridge that forwards events through a channel
///
/// This is useful for cross-thread or cross-process communication.
pub struct ChannelBridge {
    name: String,
    sender: crossbeam_channel::Sender<ChannelMessage>,
    connected: std::sync::atomic::AtomicBool,
}

impl ChannelBridge {
    /// Create a new channel bridge
    ///
    /// Returns the bridge and a receiver for the channel.
    pub fn new(name: impl Into<String>) -> (Self, crossbeam_channel::Receiver<ChannelMessage>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        (
            Self {
                name: name.into(),
                sender,
                connected: std::sync::atomic::AtomicBool::new(true),
            },
            receiver,
        )
    }

    /// Create a bounded channel bridge
    pub fn bounded(
        name: impl Into<String>,
        capacity: usize,
    ) -> (Self, crossbeam_channel::Receiver<ChannelMessage>) {
        let (sender, receiver) = crossbeam_channel::bounded(capacity);
        (
            Self {
                name: name.into(),
                sender,
                connected: std::sync::atomic::AtomicBool::new(true),
            },
            receiver,
        )
    }
}

impl EventBridge for ChannelBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError> {
        if !self.is_connected() {
            return Err(BridgeError::Disconnected(self.name.clone()));
        }

        self.sender
            .send(ChannelMessage {
                event: event.to_string(),
                data,
            })
            .map_err(|e| BridgeError::SendFailed(e.to_string()))
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn disconnect(&self) -> Result<(), BridgeError> {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

// ============================================================================
// MultiBridge - Broadcast to multiple bridges
// ============================================================================

use parking_lot::RwLock;

/// A bridge that broadcasts to multiple child bridges
pub struct MultiBridge {
    name: String,
    bridges: RwLock<Vec<Arc<dyn EventBridge>>>,
}

impl MultiBridge {
    /// Create a new multi-bridge
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bridges: RwLock::new(Vec::new()),
        }
    }

    /// Add a bridge
    pub fn add<B: EventBridge + 'static>(&self, bridge: B) {
        let mut bridges = self.bridges.write();
        bridges.push(Arc::new(bridge));
        bridges.sort_by_key(|b| b.priority());
    }

    /// Add an Arc-wrapped bridge
    pub fn add_arc(&self, bridge: Arc<dyn EventBridge>) {
        let mut bridges = self.bridges.write();
        bridges.push(bridge);
        bridges.sort_by_key(|b| b.priority());
    }

    /// Remove a bridge by name
    pub fn remove(&self, name: &str) -> bool {
        let mut bridges = self.bridges.write();
        let len_before = bridges.len();
        bridges.retain(|b| b.name() != name);
        bridges.len() < len_before
    }

    /// Get bridge count
    pub fn len(&self) -> usize {
        self.bridges.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bridges.read().is_empty()
    }

    /// Get bridge names
    pub fn bridge_names(&self) -> Vec<String> {
        self.bridges
            .read()
            .iter()
            .map(|b| b.name().to_string())
            .collect()
    }
}

impl EventBridge for MultiBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError> {
        let bridges = self.bridges.read();
        let mut errors = Vec::new();

        for bridge in bridges.iter() {
            if bridge.is_connected() {
                if let Err(e) = bridge.emit(event, data.clone()) {
                    tracing::warn!(
                        bridge_name = bridge.name(),
                        error = %e,
                        "Bridge emit failed"
                    );
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == bridges.len() {
            // All bridges failed
            Err(BridgeError::SendFailed(format!(
                "All {} bridges failed",
                errors.len()
            )))
        } else {
            // Some bridges succeeded
            Ok(())
        }
    }

    fn is_connected(&self) -> bool {
        self.bridges.read().iter().any(|b| b.is_connected())
    }

    fn disconnect(&self) -> Result<(), BridgeError> {
        for bridge in self.bridges.read().iter() {
            let _ = bridge.disconnect();
        }
        Ok(())
    }
}

// ============================================================================
// Need crossbeam-channel dependency
// ============================================================================

// Add to Cargo.toml:
// crossbeam-channel = "0.5"

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_callback_bridge() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();

        let bridge = CallbackBridge::new("test", move |event, data| {
            assert_eq!(event, "test:event");
            assert_eq!(data, json!({"key": "value"}));
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        bridge.emit("test:event", json!({"key": "value"})).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_callback_bridge_disconnect() {
        let bridge = CallbackBridge::new("test", |_, _| Ok(()));

        assert!(bridge.is_connected());
        bridge.disconnect().unwrap();
        assert!(!bridge.is_connected());

        let result = bridge.emit("test", json!(null));
        assert!(matches!(result, Err(BridgeError::Disconnected(_))));
    }

    #[test]
    fn test_channel_bridge() {
        let (bridge, receiver) = ChannelBridge::new("test");

        bridge.emit("test:event", json!({"key": "value"})).unwrap();

        let msg = receiver.recv().unwrap();
        assert_eq!(msg.event, "test:event");
        assert_eq!(msg.data, json!({"key": "value"}));
    }

    #[test]
    fn test_multi_bridge() {
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        let c1 = count1.clone();
        let c2 = count2.clone();

        let multi = MultiBridge::new("multi");
        multi.add(CallbackBridge::new("bridge1", move |_, _| {
            c1.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));
        multi.add(CallbackBridge::new("bridge2", move |_, _| {
            c2.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));

        multi.emit("test", json!(null)).unwrap();

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multi_bridge_remove() {
        let multi = MultiBridge::new("multi");
        multi.add(CallbackBridge::new("bridge1", |_, _| Ok(())));
        multi.add(CallbackBridge::new("bridge2", |_, _| Ok(())));

        assert_eq!(multi.len(), 2);

        multi.remove("bridge1");
        assert_eq!(multi.len(), 1);
        assert_eq!(multi.bridge_names(), vec!["bridge2"]);
    }
}
