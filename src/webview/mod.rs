//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations
mod aurora_view;
pub mod backend;
pub mod config; // Public for testing
pub(crate) mod event_loop;
pub mod js_assets; // JavaScript assets management
pub mod lifecycle; // Public for testing
pub(crate) mod loading;
mod message_pump;
pub mod parent_monitor;
mod platform;
pub mod protocol;
pub mod protocol_handlers; // Custom protocol handlers
pub(crate) mod standalone;
pub mod timer;
mod webview_inner;

// Public exports
pub use aurora_view::AuroraView;
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
