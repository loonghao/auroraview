//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations - Python bindings
#[cfg(feature = "python-bindings")]
mod aurora_view;
#[cfg(feature = "python-bindings")]
mod webview_inner;

// Core modules (always available)
pub mod backend;
pub mod config; // Public for testing
pub(crate) mod event_loop;
pub mod js_assets; // JavaScript assets management
#[cfg(feature = "templates")]
pub mod js_templates; // Type-safe JS templates using Askama
pub mod lifecycle; // Public for testing
pub(crate) mod loading;
mod message_pump;
pub mod parent_monitor;
mod platform;
pub mod protocol;
pub mod protocol_handlers; // Custom protocol handlers
#[cfg(feature = "python-bindings")]
pub(crate) mod standalone;
pub mod timer;

// Public exports
#[cfg(feature = "python-bindings")]
pub use aurora_view::AuroraView;
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
