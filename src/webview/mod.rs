//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations - Python bindings
#[cfg(feature = "python-bindings")]
mod core;
#[cfg(feature = "python-bindings")]
mod webview_inner;
#[cfg(feature = "python-bindings")]
mod proxy;

// Core modules (always available)
pub mod backend;
pub mod config; // Public for testing
pub(crate) mod event_loop;
pub mod js_assets; // JavaScript assets management
#[cfg(feature = "templates")]
pub mod js_templates; // Type-safe JS templates using Askama
pub mod lifecycle; // Public for testing
mod message_pump;
pub mod protocol;
pub mod protocol_handlers; // Custom protocol handlers
#[cfg(feature = "python-bindings")]
pub(crate) mod standalone;
pub mod timer;
pub mod window_manager; // Multi-window support

// Public exports
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
#[cfg(feature = "python-bindings")]
pub use core::AuroraView;
#[cfg(feature = "python-bindings")]
pub use proxy::WebViewProxy;
pub use window_manager::{WindowInfo, WindowManager};
