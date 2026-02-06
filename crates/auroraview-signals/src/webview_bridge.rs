//! WebView Bridge for Aurora Signals
//!
//! This module provides a bridge to forward aurora-signals events to WebView
//! via a message queue. This enables seamless integration between the signal
//! system and the WebView frontend.

use crate::bridge::{BridgeError, EventBridge};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Message type for WebView events
#[derive(Debug, Clone)]
pub struct WebViewEventMessage {
    /// Event name
    pub event_name: String,
    /// Event data as JSON
    pub data: Value,
}

/// Callback type for WebView message sending
pub type WebViewSender = Arc<dyn Fn(WebViewEventMessage) -> Result<(), String> + Send + Sync>;

/// Bridge that forwards events to a WebView via a callback
///
/// This bridge is designed to integrate with AuroraView's message queue system.
/// It converts aurora-signals events into WebView-compatible messages.
///
/// # Example
///
/// ```ignore
/// use aurora_signals::{EventBus, WebViewBridge};
///
/// // Create a bridge with a sender callback
/// let bridge = WebViewBridge::new("webview", |msg| {
///     // Forward to WebView message queue
///     message_queue.push(WebViewMessage::EmitEvent {
///         event_name: msg.event_name,
///         data: msg.data,
///     });
///     Ok(())
/// });
///
/// // Add to event bus
/// let bus = EventBus::new();
/// bus.add_bridge(bridge);
///
/// // Events will now be forwarded to WebView
/// bus.emit("app:ready", json!({"version": "1.0"}));
/// ```
pub struct WebViewBridge {
    name: String,
    sender: WebViewSender,
    connected: AtomicBool,
    /// Optional event prefix filter (only forward events with this prefix)
    prefix_filter: Option<String>,
}

impl WebViewBridge {
    /// Create a new WebView bridge
    pub fn new<F>(name: impl Into<String>, sender: F) -> Self
    where
        F: Fn(WebViewEventMessage) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            sender: Arc::new(sender),
            connected: AtomicBool::new(true),
            prefix_filter: None,
        }
    }

    /// Create a bridge with an event prefix filter
    ///
    /// Only events starting with the given prefix will be forwarded.
    pub fn with_prefix_filter<F>(
        name: impl Into<String>,
        prefix: impl Into<String>,
        sender: F,
    ) -> Self
    where
        F: Fn(WebViewEventMessage) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            sender: Arc::new(sender),
            connected: AtomicBool::new(true),
            prefix_filter: Some(prefix.into()),
        }
    }

    /// Create a bridge from an Arc-wrapped sender
    pub fn from_arc(name: impl Into<String>, sender: WebViewSender) -> Self {
        Self {
            name: name.into(),
            sender,
            connected: AtomicBool::new(true),
            prefix_filter: None,
        }
    }

    /// Check if event should be forwarded based on prefix filter
    fn should_forward(&self, event: &str) -> bool {
        match &self.prefix_filter {
            Some(prefix) => event.starts_with(prefix),
            None => true,
        }
    }
}

impl EventBridge for WebViewBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn emit(&self, event: &str, data: Value) -> Result<(), BridgeError> {
        if !self.is_connected() {
            return Err(BridgeError::Disconnected(self.name.clone()));
        }

        // Check prefix filter
        if !self.should_forward(event) {
            return Ok(());
        }

        let message = WebViewEventMessage {
            event_name: event.to_string(),
            data,
        };

        (self.sender)(message).map_err(BridgeError::SendFailed)
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    fn disconnect(&self) -> Result<(), BridgeError> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn priority(&self) -> i32 {
        // WebView bridge has high priority
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_webview_bridge_basic() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();

        let bridge = WebViewBridge::new("test", move |msg| {
            assert_eq!(msg.event_name, "test:event");
            assert_eq!(msg.data, json!({"key": "value"}));
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        bridge.emit("test:event", json!({"key": "value"})).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_webview_bridge_prefix_filter() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();

        let bridge = WebViewBridge::with_prefix_filter("test", "webview:", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        // Should forward
        bridge.emit("webview:ready", json!(null)).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Should not forward (different prefix)
        bridge.emit("app:ready", json!(null)).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_webview_bridge_disconnect() {
        let bridge = WebViewBridge::new("test", |_| Ok(()));

        assert!(bridge.is_connected());
        bridge.disconnect().unwrap();
        assert!(!bridge.is_connected());

        let result = bridge.emit("test", json!(null));
        assert!(matches!(result, Err(BridgeError::Disconnected(_))));
    }
}
