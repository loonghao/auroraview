//! WebView Backend Abstraction Layer
//!
//! This module provides a unified interface for different WebView backends,
//! inspired by Qt WebView's cross-platform abstraction design.
//!
//! ## Architecture
//!
//! - `WebViewBackend` trait: Platform-agnostic interface for WebView operations
//! - `WebViewSettings` trait: Unified settings management
//! - `BackendFactory`: Factory pattern for backend instantiation
//! - `BackendType`: Enum representing available backend types
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auroraview_core::backend::{BackendFactory, BackendConfig};
//!
//! let config = BackendConfig::default();
//! let backend = BackendFactory::create(&config)?;
//! backend.navigate("https://example.com")?;
//! ```

mod error;
mod factory;
mod settings;
mod traits;
mod wry_impl;

pub use error::{WebViewError, WebViewResult};
pub use factory::{BackendConfig, BackendFactory, BackendType};
pub use settings::{WebViewSettings, WebViewSettingsImpl};
pub use traits::{
    CookieInfo, JavaScriptCallback, LoadProgress, NavigationEvent, NavigationState, WebViewBackend,
};
pub use wry_impl::WryBackend;
