//! AuroraView DCC Runtime
//!
//! WebView integration for Digital Content Creation applications:
//! - Maya
//! - Houdini
//! - Nuke
//! - Blender
//! - 3ds Max
//! - Unreal Engine
//!
//! # Key Differences from Desktop
//!
//! - **No Event Loop Ownership**: DCC host controls the event loop
//! - **Parent Window**: WebView embeds into host Qt widget
//! - **Threading**: Must respect DCC's threading model
//! - **Non-blocking**: Use `process_events()` in Qt timer
//! - **Multi-window**: Use WindowManager for multiple WebViews
//!
//! # Single Window Usage
//!
//! ```rust,ignore
//! use auroraview_dcc::{DccConfig, DccWebView};
//!
//! let config = DccConfig::new()
//!     .title("My Tool")
//!     .parent_hwnd(qt_widget_hwnd);
//!
//! let webview = DccWebView::new(config)?;
//! webview.init()?;
//! webview.show()?;
//!
//! // In Qt timer:
//! webview.process_events();
//! ```
//!
//! # Multi-window Usage
//!
//! ```rust,ignore
//! use auroraview_dcc::{DccConfig, WindowManager};
//!
//! let manager = WindowManager::new();
//!
//! // Create multiple windows
//! let win1 = manager.create(DccConfig::new().title("Tool 1").parent_hwnd(hwnd1))?;
//! let win2 = manager.create(DccConfig::new().title("Tool 2").parent_hwnd(hwnd2))?;
//!
//! // Initialize and show
//! manager.init(&win1)?;
//! manager.init(&win2)?;
//! manager.show(&win1)?;
//!
//! // Register shared IPC handlers
//! manager.router().register("api.echo", |params| params);
//!
//! // In Qt timer:
//! manager.process_events();
//! ```

/// DCC integration configuration and host type detection.
pub mod config;
/// DCC-specific error types.
pub mod error;
/// IPC message routing for DCC-embedded WebViews.
pub mod ipc;
/// Multi-window management within DCC host applications.
pub mod window_manager;

/// WebView2-based DCC WebView implementation (Windows only).
#[cfg(target_os = "windows")]
pub mod webview;

/// Configuration types for DCC integration and host type enumeration.
pub use config::{DccConfig, DccType};
/// Error and result types for DCC operations.
pub use error::{DccError, Result};
/// IPC types for DCC-embedded WebView communication.
pub use ipc::{IpcError, IpcMessage, IpcResponse, IpcRouter};
/// Window identifier, info, and manager for multi-WebView DCC panels.
pub use window_manager::{WindowId, WindowInfo, WindowManager};

/// DCC WebView message and window types (Windows only).
#[cfg(target_os = "windows")]
pub use webview::{DccMessage, DccWebView};
