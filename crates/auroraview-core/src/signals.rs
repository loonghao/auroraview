//! Signal-Slot System for AuroraView
//!
//! This module re-exports the unified `aurora-signals` crate which provides:
//! - Type-safe signals with generic payloads (`Signal<T>`)
//! - Multiple handlers per signal (multi-receiver support)
//! - Automatic cleanup when ConnectionGuard is dropped (RAII)
//! - Thread-safe operations using parking_lot
//! - Dynamic signal registry (`SignalRegistry`)
//! - Unified event bus with middleware support (`EventBus`)
//!
//! # Migration Guide
//!
//! The API is largely compatible with the previous implementation.
//! Main differences:
//! - `Signal::new()` -> `Signal::new()` (same)
//! - `Signal::named("name")` for named signals (new)
//! - `SignalRegistry` now has a `named()` constructor
//! - New `EventBus` for middleware and bridge support
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

// Re-export core types from aurora-signals
pub use aurora_signals::{
    BridgeError,

    CallbackBridge,
    ChannelBridge,
    ChannelMessage,
    ConnectionGuard,

    ConnectionId,
    // Bridge types for cross-platform events
    EventBridge,
    // Event bus with middleware support
    EventBus,

    FilterMiddleware,
    LogLevel,

    LoggingMiddleware,
    // Middleware types
    Middleware,
    MultiBridge,
    // Core signal types
    Signal,
    // Error types
    SignalError,
    // Registry for dynamic signals
    SignalRegistry,

    TransformMiddleware,
    // WebView bridge for forwarding events to frontend
    WebViewBridge,
    WebViewEventMessage,
    WebViewSender,
};

use std::sync::Arc;

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
