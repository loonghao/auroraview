//! AuroraView Core - Shared utilities and assets
//!
//! This crate provides reusable components for AuroraView:
//! - Static assets (HTML, JavaScript)
//! - Protocol handling utilities
//! - URL/path utilities
//! - High-performance JSON operations
//! - WebView configuration structures
//! - Port allocation utilities
//! - ID generation
//! - Browser Object Model (BOM) APIs
//! - DOM manipulation primitives
//! - Timing metrics for WebView lifecycle
//! - **IPC abstraction layer** (platform-agnostic messaging)
//! - **Backend abstraction layer** (unified WebView interface)
//! - **Builder extensions** (shared WebView building logic)
//! - **Plugin system** (native desktop capabilities)
//! - **Thread safety utilities** (lock ordering, deadlock prevention)
//! - **Events** (unified user event types)
//!
//! Used by:
//! - `auroraview-cli` (Command-line interface)
//! - `auroraview` (Python bindings, re-exports)

/// Static assets (HTML, JavaScript) embedded at compile time.
pub mod assets;
/// WebView backend abstraction (traits, factory, settings).
pub mod backend;
/// Browser Object Model APIs (navigation, zoom, window control).
pub mod bom;
/// WebView builder extensions (drag-drop, IPC, protocols).
pub mod builder;
/// WebView user data directory cleanup (cross-platform).
pub mod cleanup;
/// CLI utilities (URL normalization, HTML rewriting).
pub mod cli;
/// WebView configuration structures.
pub mod config;
/// DOM manipulation primitives (DomOp, DomBatch).
pub mod dom;
/// Unified user event types (CoreUserEvent, ExtendedUserEvent).
pub mod events;
/// Icon utilities (PNG loading, ICO conversion, compression).
pub mod icon;
/// Unique ID generation utilities.
pub mod id_generator;
/// IPC abstractions (message, metrics, WebViewMessage) - platform-agnostic.
pub mod ipc;
/// High-performance JSON operations.
pub mod json;
/// Native menu bar support.
pub mod menu;
/// Timing metrics for WebView lifecycle.
pub mod metrics;
/// Dynamic port allocation utilities.
pub mod port;
/// Protocol handling utilities.
pub mod protocol;
/// Service discovery (port allocation, mDNS, HTTP discovery).
pub mod service_discovery;
/// Qt-inspired signal-slot event system.
pub mod signals;
/// JavaScript templates (Askama).
pub mod templates;
/// Thread safety utilities (lock ordering, deadlock prevention).
pub mod thread_safety;
/// Common utility functions.
pub mod utils;
/// Window information structures.
pub mod window;

/// Plugin system for native desktop capabilities (re-exported from `auroraview-plugins`).
pub use auroraview_plugins as plugins;
