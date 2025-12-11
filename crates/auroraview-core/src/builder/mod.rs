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
//! - Background color, protocol registration, initialization scripts
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auroraview_core::builder::{create_drag_drop_handler, create_ipc_handler};
//!
//! // Create drag-drop handler
//! let drag_handler = create_drag_drop_handler(|event_name, data| {
//!     println!("Drag event: {} {:?}", event_name, data);
//! });
//!
//! // Create IPC handler
//! let ipc_handler = create_ipc_handler(
//!     |name, data| println!("Event: {}", name),
//!     |method, params, id| println!("Call: {}", method),
//!     |cmd, args, id| println!("Invoke: {}", cmd),
//!     |callback_id, data| println!("Callback: {}", callback_id),
//! );
//!
//! let builder = WryWebViewBuilder::new()
//!     .with_drag_drop_handler(drag_handler)
//!     .with_ipc_handler(ipc_handler);
//! ```

#[cfg(feature = "wry-builder")]
mod drag_drop;
#[cfg(feature = "wry-builder")]
mod helpers;
#[cfg(feature = "wry-builder")]
mod ipc;
#[cfg(feature = "wry-builder")]
mod protocol;

#[cfg(feature = "wry-builder")]
pub use drag_drop::{DragDropCallback, DragDropEventData, DragDropEventType, DragDropHandler};
#[cfg(feature = "wry-builder")]
pub use helpers::{create_drag_drop_handler, create_ipc_handler, create_simple_ipc_handler};
#[cfg(feature = "wry-builder")]
pub use ipc::{IpcCallback, IpcMessageHandler, IpcMessageType, ParsedIpcMessage};
#[cfg(feature = "wry-builder")]
pub use protocol::ProtocolConfig;

/// Dark background color (Tailwind slate-950: #020617)
/// Used to prevent white flash during WebView initialization
pub const DARK_BACKGROUND: (u8, u8, u8, u8) = (2, 6, 23, 255);
