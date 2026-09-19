//! Parent/child IPC — the host-agnostic bridge behind child windows.
//!
//! This module is the Rust half of the contract described in
//! `docs/guide/child-windows.md`. It replaces the Python-only
//! `auroraview.child.ParentBridge` with an implementation any host can drive:
//! Gallery, Unity (C#), Unreal (C++), PowerPoint (VSTO/PowerShell), or a plain
//! terminal.
//!
//! # Roles
//!
//! - **Parent** (host): binds a loopback TCP port, passes
//!   `AURORAVIEW_PARENT_ID` / `AURORAVIEW_PARENT_PORT` to the child process,
//!   accepts the connection and speaks the protocol.
//! - **Child** (AuroraView): reads those variables, connects with
//!   [`ParentBridge`], and exchanges events.
//!
//! # Wire format (summary)
//!
//! - Transport: TCP on `127.0.0.1:<AURORAVIEW_PARENT_PORT>`.
//! - Framing: newline-delimited JSON, one object per line, `\n`-terminated.
//! - Handshake: child sends `{"type":"hello","protocol":1,...}`, a
//!   protocol-aware parent answers `{"type":"hello_ack","accepted":true}`.
//!   Missing the ack is **not** an error — the child downgrades to
//!   [`HandshakeState::Legacy`] so pre-protocol parents keep working.
//! - Errors: `{"type":"error","code":"...","message":"...","fatal":bool}`.
//!
//! See [`protocol`] for the full frame schema and `docs/guide/child-windows.md`
//! for the normative, language-agnostic specification.
//!
//! # Example
//!
//! ```no_run
//! use auroraview_core::parent_ipc::{ChildInfo, ParentBridge};
//!
//! let info = ChildInfo::from_env();
//! if info.can_connect() {
//!     let bridge = ParentBridge::connect(&info)?;
//!     bridge.on_command(|data| println!("command: {}", data));
//!     bridge.send_event("ping", serde_json::json!({"at": "startup"}))?;
//!     bridge.disconnect();
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod bridge;
mod context;
mod protocol;

pub use bridge::{
    BridgeConfig, BridgeError, HandshakeState, ParentBridge, ReconnectPolicy,
};
pub use context::{is_child_mode, parse_hwnd, ChildInfo};
pub use protocol::{
    ErrorCode, FrameError, FrameReader, Message, MessageKind, ENV_CHILD_ID, ENV_EXAMPLE_NAME,
    ENV_PARENT_HWND, ENV_PARENT_ID, ENV_PARENT_PORT, MAX_FRAME_BYTES, PROTOCOL_VERSION,
    READY_EVENT, CLOSING_EVENT, COMMAND_EVENT, DEFAULT_HOST, FRAME_DELIMITER, UTF8_BOM,
};
