//! WebView Builder Extensions
//!
//! This module provides shared WebView building logic that can be reused
//! across different modes (standalone, DCC embedded, etc.).
//!
//! ## Architecture
//!
//! The builder extension pattern allows both `standalone.rs` and `native.rs`
//! to share common WebView configuration without code duplication:
//!
//! - `DragDropHandler`: Shared file drag-drop event handling
//! - `IpcMessageHandler`: Shared IPC message parsing and routing
//! - `create_drag_drop_handler`: High-level helper for drag-drop
//! - `create_ipc_handler`: High-level helper for IPC
//! - `apply_child_window_style`: Windows child window style manipulation
//! - `init_com_sta`: COM initialization for WebView2
//! - `create_web_context`: WebContext creation with data directory
//! - Background color, protocol registration, initialization scripts
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auroraview_core::builder::{
//!     create_drag_drop_handler, create_ipc_handler,
//!     init_com_sta, apply_child_window_style, ChildWindowStyleOptions,
//!     get_background_color, create_web_context, WebContextConfig,
//! };
//!
//! // Initialize COM (Windows)
//! init_com_sta();
//!
//! // Create WebContext
//! let ctx_config = WebContextConfig::new()
//!     .with_data_directory(PathBuf::from("/tmp/data"));
//! let web_context = create_web_context(&ctx_config);
//!
//! // Create drag-drop handler
//! let drag_handler = create_drag_drop_handler(|event_name, data| {
//!     println!("Drag event: {} {:?}", event_name, data);
//! });
//!
//! // Apply child window style (Windows)
//! apply_child_window_style(hwnd, parent_hwnd, ChildWindowStyleOptions::for_dcc_embedding());
//! ```

// Platform-independent modules
mod com_init;
mod common_config;
mod web_context;
mod window_style;

// Wry-specific modules (require wry-builder feature)
#[cfg(feature = "wry-builder")]
mod drag_drop;
#[cfg(feature = "wry-builder")]
mod helpers;
#[cfg(feature = "wry-builder")]
mod ipc;
#[cfg(feature = "wry-builder")]
mod protocol;

// Platform-independent exports
pub use com_init::{init_com_sta, ComInitResult};
pub use common_config::{get_background_color, log_background_color, DARK_BACKGROUND};
pub use web_context::WebContextConfig;
pub use window_style::{
    apply_child_window_style, apply_owner_window_style, apply_tool_window_style,
    disable_window_shadow, ChildWindowStyleOptions, ChildWindowStyleResult, OwnerWindowStyleResult,
};

// Wry-specific exports
#[cfg(feature = "wry-builder")]
pub use drag_drop::{DragDropCallback, DragDropEventData, DragDropEventType, DragDropHandler};
#[cfg(feature = "wry-builder")]
pub use helpers::{create_drag_drop_handler, create_ipc_handler, create_simple_ipc_handler};
#[cfg(feature = "wry-builder")]
pub use ipc::{IpcCallback, IpcMessageHandler, IpcMessageType, ParsedIpcMessage};
#[cfg(feature = "wry-builder")]
pub use protocol::ProtocolConfig;
#[cfg(feature = "wry-builder")]
pub use web_context::create_web_context;
