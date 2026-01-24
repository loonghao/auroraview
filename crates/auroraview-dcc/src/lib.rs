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

pub mod config;
pub mod error;
pub mod ipc;
pub mod window_manager;

#[cfg(target_os = "windows")]
pub mod webview;

pub use config::{DccConfig, DccType};
pub use error::{DccError, Result};
pub use ipc::{IpcError, IpcMessage, IpcResponse, IpcRouter};
pub use window_manager::{WindowId, WindowInfo, WindowManager};

#[cfg(target_os = "windows")]
pub use webview::{DccMessage, DccWebView};
