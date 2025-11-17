//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations
mod aurora_view;
pub mod backend;
mod config;
pub(crate) mod event_loop;
pub mod js_assets; // JavaScript assets management
mod lifecycle;
pub(crate) mod loading;
mod message_pump;
mod parent_monitor;
mod platform;
mod protocol;
pub(crate) mod protocol_handlers; // Custom protocol handlers
pub(crate) mod standalone;
pub(crate) mod timer;
mod webview_inner;

// Public exports
pub use aurora_view::AuroraView;
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
