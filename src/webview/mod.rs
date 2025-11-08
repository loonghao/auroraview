//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations
mod aurora_view;
pub mod backend;
mod config;
pub(crate) mod embedded; // TODO: Remove after migration to backend::native
pub(crate) mod event_loop;
mod lifecycle;
pub(crate) mod loading;
mod message_pump;
mod parent_monitor;
mod platform;
mod protocol;
mod python_bindings;
pub(crate) mod standalone;
mod timer;
mod timer_bindings;
mod webview_inner;

// Public exports
pub use aurora_view::AuroraView;
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
