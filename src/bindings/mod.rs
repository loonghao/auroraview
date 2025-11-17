//! Python bindings module
//!
//! This module contains all PyO3 Python bindings for the AuroraView library.
//! All bindings are organized by functionality:
//!
//! - `webview` - Main WebView class and related functionality
//! - `timer` - Timer utilities for event loop integration
//! - `ipc` - IPC message handling and JSON serialization
//! - `service_discovery` - Service discovery and port allocation

pub mod ipc;
pub mod service_discovery;
pub mod timer;
pub mod webview;
