//! WebView module - Core WebView functionality

#![allow(clippy::useless_conversion)]

// Module declarations
mod aurora_view;
pub mod backend;
mod config;
pub(crate) mod embedded; // TODO: Remove after migration to backend::native
pub(crate) mod event_loop;
mod message_pump;
mod protocol;
mod python_bindings;
pub(crate) mod standalone;
mod webview_inner;

// Public exports
pub use aurora_view::AuroraView;
pub use backend::{BackendType, WebViewBackend};
pub use config::{WebViewBuilder, WebViewConfig};
