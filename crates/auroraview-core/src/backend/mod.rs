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
//! - `AtomicLifecycle`: Lock-free lifecycle state machine
//! - `MessageProcessor`: Unified message processing
//!
//! ## Key Design Decisions
//!
//! ### Lock-Free Lifecycle (P0 Fix)
//!
//! The `AtomicLifecycle` provides a lock-free state machine for WebView lifecycle,
//! eliminating the risk of deadlocks during shutdown identified in the architecture
//! diagnosis.
//!
//! ### Unified Message Processing (P2 Fix)
//!
//! The `MessageProcessor` module provides a single source of truth for message
//! processing across all modes (standalone, embedded, Qt/DCC).
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
pub mod lifecycle;
pub mod message_processor;
mod settings;
mod traits;
mod wry_impl;

pub use error::{WebViewError, WebViewResult};
pub use factory::{BackendConfig, BackendFactory, BackendType};
pub use lifecycle::{
    AtomicLifecycle, LifecycleEvent, LifecycleObserver, LifecycleState, ObservableLifecycle,
    TransitionResult,
};
pub use message_processor::{
    AtomicProcessorStats, MessagePriority, ProcessResult, ProcessingMode, ProcessorConfig,
    ProcessorStats, WakeController,
};
pub use settings::{WebViewSettings, WebViewSettingsImpl};
pub use traits::{
    CookieInfo, EmbeddableBackend, EventLoopBackend, JavaScriptCallback, LoadProgress,
    NavigationEvent, NavigationState, WebViewBackend,
};
pub use wry_impl::WryBackend;
