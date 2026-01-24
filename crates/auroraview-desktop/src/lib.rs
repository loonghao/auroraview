//! AuroraView Desktop Runtime
//!
//! Standalone desktop WebView applications, similar to Tauri/Electron.
//!
//! # Features
//!
//! - Independent window and event loop management
//! - System tray support
//! - Multi-window support via WindowManager
//! - Shared IPC routing
//!
//! # Single Window Usage
//!
//! ```rust,ignore
//! use auroraview_desktop::{DesktopConfig, run};
//!
//! let config = DesktopConfig::new()
//!     .title("My App")
//!     .url("https://example.com");
//!
//! run(config)?;
//! ```
//!
//! # Multi-window Usage
//!
//! ```rust,ignore
//! use auroraview_desktop::{DesktopConfig, WindowManager};
//!
//! let manager = WindowManager::new();
//!
//! // Create multiple windows
//! let main = manager.create(DesktopConfig::new().title("Main"))?;
//! let settings = manager.create(DesktopConfig::new().title("Settings"))?;
//!
//! // Register shared IPC handlers
//! manager.router().register("api.echo", |params| params);
//!
//! // Show windows
//! manager.show(&main)?;
//! ```

pub mod config;
pub mod error;
pub mod event_loop;
pub mod ipc;
pub mod tray;
pub mod window;
pub mod window_manager;

pub use config::{DesktopConfig, TrayConfig, TrayMenuItem};
pub use error::{DesktopError, Result};
pub use event_loop::{run, run_with_router, UserEvent};
pub use ipc::{IpcMessage, IpcResponse, IpcRouter};
pub use window::{create_window, create_window_with_router, DesktopWindow};
pub use window_manager::{WindowId, WindowInfo, WindowManager};
