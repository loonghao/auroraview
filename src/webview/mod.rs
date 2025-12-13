//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations - Python bindings
#[cfg(feature = "python-bindings")]
mod core;
#[cfg(feature = "python-bindings")]
mod proxy;
#[cfg(feature = "python-bindings")]
mod webview_inner;

// Core modules (always available)
pub mod backend;
pub mod config; // Public for testing
#[cfg(feature = "python-bindings")]
pub(crate) mod desktop;
pub(crate) mod event_loop;
pub mod js_assets; // JavaScript assets management
#[cfg(feature = "templates")]
pub mod js_templates; // Type-safe JS templates using Askama
pub mod lifecycle; // Public for testing
mod message_processor; // Unified message processing
mod message_pump;
pub mod protocol;
pub mod protocol_handlers; // Custom protocol handlers
#[cfg(feature = "python-bindings")]
pub(crate) use desktop as standalone; // Backward compatibility alias
pub mod timer;
pub mod tray; // System tray support
pub mod window_manager; // Multi-window support

// Public exports
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{TrayConfig, TrayMenuItem, TrayMenuItemType, WebViewBuilder, WebViewConfig};
#[cfg(feature = "python-bindings")]
pub use core::AuroraView;
#[cfg(feature = "python-bindings")]
pub use core::EventEmitter;
#[cfg(feature = "python-bindings")]
pub use core::PluginManager;
#[cfg(feature = "python-bindings")]
pub use proxy::WebViewProxy;
pub use window_manager::{WindowInfo, WindowManager};
