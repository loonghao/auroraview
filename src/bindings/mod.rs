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
//! - `tab_browser` - Multi-tab browser (Microsoft WebView2Browser architecture)
//! - `assets` - Static assets (JavaScript, HTML) for testing
//! - `webview2` - Windows WebView2 embedded API (feature-gated)
//! - `runtime_desktop` - Desktop runtime bindings (multi-window, IPC router)
//! - `runtime_dcc` - DCC runtime bindings (Maya, Houdini, Nuke integration)
//! - `cleanup` - WebView2 user data directory cleanup

pub mod assets;
pub mod cleanup;
pub mod cli_utils;
pub mod desktop_runner;
pub mod ipc;
pub mod ipc_metrics;
pub mod service_discovery;
pub mod tab_browser;
pub mod timer;
pub mod warmup;
pub mod webview;
pub mod window_manager;

#[cfg(all(target_os = "windows", feature = "win-webview2"))]
pub mod webview2;

// Runtime crate bindings
#[cfg(feature = "runtime-desktop")]
pub mod runtime_desktop;
#[cfg(feature = "runtime-dcc")]
pub mod runtime_dcc;
