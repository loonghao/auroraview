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

/// Desktop window and tray configuration.
pub mod config;
/// Desktop-specific error types.
pub mod error;
/// Event loop management and application entry points.
pub mod event_loop;
/// Inter-process communication message routing.
pub mod ipc;
/// System tray icon and menu support.
pub mod tray;
/// Window creation and management.
pub mod window;
/// Multi-window lifecycle and coordination.
pub mod window_manager;

/// Configuration types for desktop windows and system tray.
pub use config::{DesktopConfig, TrayConfig, TrayMenuItem};
/// Error and result types for desktop operations.
pub use error::{DesktopError, Result};
/// Application entry points and event loop types.
pub use event_loop::{run, run_with_router, UserEvent};
/// IPC message and routing types.
pub use ipc::{IpcMessage, IpcResponse, IpcRouter};
/// Window creation helpers and desktop window type.
pub use window::{create_window, create_window_with_router, DesktopWindow};
/// Window identifier, info, and manager for multi-window support.
pub use window_manager::{WindowId, WindowInfo, WindowManager};
