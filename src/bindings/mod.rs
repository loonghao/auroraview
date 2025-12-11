//! Python bindings module
//!
//! This module contains all PyO3 Python bindings for the AuroraView library.
//! All bindings are organized by functionality:
//!
//! - `webview` - Main WebView class and related functionality
//! - `timer` - Timer utilities for event loop integration
//! - `ipc` - IPC message handling and JSON serialization
//! - `ipc_metrics` - IPC performance metrics
//! - `service_discovery` - Service discovery and port allocation
//! - `cli_utils` - CLI utility functions (URL normalization, HTML rewriting)
//! - `desktop_runner` - Desktop WebView runner (uses event_loop.run())
//! - `webview2` - Windows WebView2 embedded API (feature-gated)

pub mod cli_utils;
pub mod desktop_runner;
pub mod ipc;
pub mod ipc_metrics;
pub mod service_discovery;
pub mod timer;
pub mod warmup;
pub mod webview;
pub mod window_manager;

#[cfg(all(target_os = "windows", feature = "win-webview2"))]
pub mod webview2;
