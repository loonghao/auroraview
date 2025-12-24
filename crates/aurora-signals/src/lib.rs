//! # Aurora Signals
//!
//! A cross-language signal-slot system with middleware and event bus support.
//!
//! ## Features
//!
//! - **Type-safe signals**: `Signal<T>` with compile-time type checking
//! - **Dynamic signals**: `SignalRegistry` for runtime-named signals
//! - **Event bus**: Unified event distribution with middleware pipeline
//! - **Middleware**: Logging, filtering, and transformation middleware
//! - **Event bridges**: Cross-platform event forwarding (WebView, Python, IPC)
//! - **Python bindings**: PyO3-based Python API with identical interface
//!
//! ## Quick Start
//!
//! ```rust
//! use aurora_signals::prelude::*;
//!
//! // Create a typed signal
//! let signal: Signal<String> = Signal::new();
//!
//! // Connect a handler
//! let conn = signal.connect(|msg| {
//!     println!("Received: {}", msg);
//! });
//!
//! // Emit a value
//! signal.emit("Hello, World!".to_string());
//!
//! // Disconnect when done
//! signal.disconnect(conn);
//! ```
//!
//! ## Dynamic Signals with Registry
//!
//! ```rust
//! use aurora_signals::prelude::*;
//! use serde_json::json;
//!
//! let registry = SignalRegistry::new();
//!
//! // Connect to a named signal (auto-created if not exists)
//! let conn = registry.connect("process:stdout", |data| {
//!     println!("stdout: {:?}", data);
//! });
//!
//! // Emit to the signal
//! registry.emit("process:stdout", json!({"pid": 123, "data": "hello"}));
//! ```
//!
//! ## Event Bus with Middleware
//!
//! ```rust
//! use aurora_signals::prelude::*;
//!
//! let bus = EventBus::new();
//!
//! // Add logging middleware
//! bus.use_middleware(LoggingMiddleware::new(LogLevel::Debug));
//!
//! // Subscribe to events
//! let conn = bus.on("app:ready", |data| {
//!     println!("App ready: {:?}", data);
//! });
//!
//! // Emit events (goes through middleware pipeline)
//! bus.emit("app:ready", serde_json::json!({"version": "1.0"}));
//! ```

// Core modules
pub mod bridge;
pub mod bus;
pub mod connection;
pub mod error;
pub mod middleware;
pub mod registry;
pub mod signal;
pub mod webview_bridge;

// Python bindings (optional)
#[cfg(feature = "python")]
pub mod python;

// Re-export commonly used types
pub mod prelude {
    pub use crate::bridge::{
        BridgeError, CallbackBridge, ChannelBridge, ChannelMessage, EventBridge, MultiBridge,
    };
    pub use crate::bus::EventBus;
    pub use crate::connection::{ConnectionGuard, ConnectionId};
    pub use crate::error::SignalError;
    pub use crate::middleware::{
        FilterMiddleware, LogLevel, LoggingMiddleware, Middleware, MiddlewareResult,
        TransformMiddleware,
    };
    pub use crate::registry::SignalRegistry;
    pub use crate::signal::Signal;
    pub use crate::webview_bridge::{WebViewBridge, WebViewEventMessage, WebViewSender};
}

// Re-export at crate root for convenience
pub use prelude::*;
