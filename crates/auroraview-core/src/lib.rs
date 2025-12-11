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
//!
//! Used by:
//! - `auroraview-cli` (Command-line interface)
//! - `auroraview` (Python bindings, re-exports)

pub mod assets;
pub mod backend; // WebView backend abstraction (traits, factory, settings)
pub mod bom; // Browser Object Model APIs (navigation, zoom, window control)
pub mod builder; // WebView builder extensions (drag-drop, IPC, protocols)
pub mod cli; // CLI utilities (URL normalization, HTML rewriting)
pub mod config;
pub mod dom; // DOM manipulation primitives (DomOp, DomBatch)
pub mod icon; // Icon utilities (PNG loading, ICO conversion, compression)
pub mod id_generator;
pub mod ipc; // IPC abstractions (message, metrics) - platform-agnostic
pub mod json;
pub mod menu; // Native menu bar support
pub mod metrics; // Timing metrics for WebView lifecycle
pub mod port;
pub mod protocol;
pub mod service_discovery; // Service discovery (port allocation, service info)
pub mod signals; // Qt-inspired signal-slot event system
pub mod templates; // JavaScript templates (Askama)
pub mod utils;
pub mod window; // Window information structures

// Re-export plugin system from auroraview-plugins crate
pub use auroraview_plugins as plugins;
