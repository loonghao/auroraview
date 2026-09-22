//! Wire protocol for the AuroraView parent/child IPC channel.
//!
//! This module defines the **host-agnostic** contract that any parent host
//! (Gallery, Unity/C#, Unreal/C++, PowerPoint/VSTO, PowerShell, ...) and any
//! AuroraView child process use to talk to each other.
//!
//! # Transport
//!
//! TCP over loopback (`127.0.0.1:<AURORAVIEW_PARENT_PORT>`). The parent is the
//! **server** (it binds and accepts), the child is the **client** (it connects).
//!
//! # Framing
//!
//! Newline-delimited JSON (NDJSON): every frame is a single JSON object
//! encoded as UTF-8 and terminated by one `\n` (0x0A). A JSON string can never
//! contain a raw 0x0A (it is escaped as `\n`), so `\n` is an unambiguous frame
//! delimiter. Receivers tolerate a trailing `\r` (CRLF) on the wire.
//!
//! # Compatibility
//!
//! The `type` field is optional on inbound frames and defaults to `event`.
//! This keeps the pre-existing Gallery/ParentBridge traffic
//! (`{"event": ..., "data": ...}`) valid, so an older parent and a newer child
//! still interoperate.
//!
//! # Versioning
//!
//! [`PROTOCOL_VERSION`] is negotiated during the handshake: the effective
//! version is `min(child, parent)`. A parent that answers `hello` with
//! `accepted: false` (or a fatal error) makes the child fall back to
//! standalone mode instead of failing to start.

use std::fmt;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

/// Environment variable holding the parent window identifier.
///
/// Its presence is what puts a process into **child mode**.
pub const ENV_PARENT_ID: &str = "AURORAVIEW_PARENT_ID";

/// Environment variable holding the TCP port the parent listens on.
pub const ENV_PARENT_PORT: &str = "AURORAVIEW_PARENT_PORT";

/// Environment variable holding this child's unique identifier.
pub const ENV_CHILD_ID: &str = "AURORAVIEW_CHILD_ID";

/// Environment variable holding the logical name of the running example/app.
pub const ENV_EXAMPLE_NAME: &str = "AURORAVIEW_EXAMPLE_NAME";

/// Environment variable optionally holding the parent window's native handle.
///
/// Windows: the `HWND` value, decimal or `0x`-prefixed hexadecimal.
/// This lets a host pass the embedding target without going through the CLI.
pub const ENV_PARENT_HWND: &str = "AURORAVIEW_PARENT_HWND";

/// Loopback host the child connects to.
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Current revision of the parent/child wire protocol.
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum accepted frame size in bytes (1 MiB).
///
/// Frames larger than this are rejected to keep the receive buffer bounded.
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Event a child emits once its window is up.
pub const READY_EVENT: &str = "child:ready";

/// Event a child emits right before it tears down.
pub const CLOSING_EVENT: &str = "child:closing";

/// Event a parent sends to drive the child (`data.command` selects the action).
pub const COMMAND_EVENT: &str = "parent:command";

/// Terminator written after every frame.
pub const FRAME_DELIMITER: u8 = b'\n';

/// UTF-8 byte order mark, which some hosts emit before the first frame.
pub const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Discriminator of a protocol frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageKind {
    /// Child announces itself; starts the handshake.
    Hello,
    /// Parent accepts (or rejects) a `hello`.
    HelloAck,
    /// Application-level event in either direction.
    Event,
    /// Error notification in either direction.
    Error,
    /// Liveness probe.
    Ping,
    /// Answer to a [`MessageKind::Ping`].
    Pong,
    /// Anything this revision does not know; preserved so it can be forwarded.
    Unknown(String),
}

impl MessageKind {
    /// Canonical wire spelling of this kind.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hello => "hello",
            Self::HelloAck => "hello_ack",
            Self::Event => "event",
            Self::Error => "error",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::Unknown(other) => other.as_str(),
        }
    }

    /// Parse a wire spelling, keeping unrecognised values as
    /// [`MessageKind::Unknown`] so forward compatibility is not lossy.
    pub fn parse(s: &str) -> Self {
        match s {
            "hello" => Self::Hello,
            "hello_ack" => Self::HelloAck,
            "event" => Self::Event,
            "error" => Self::Error,
            "ping" => Self::Ping,
            "pong" => Self::Pong,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for MessageKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for MessageKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse(&s))
    }
}

/// Stable error codes carried by [`MessageKind::Error`] frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Peer speaks a protocol revision this implementation cannot serve.
    UnsupportedProtocol,
    /// Frame was not valid UTF-8, not valid JSON, or not a JSON object.
    BadJson,
    /// Frame exceeded [`MAX_FRAME_BYTES`].
    FrameTooLarge,
    /// Frame carried a `type` the receiver does not implement.
    UnknownType,
    /// A local event handler failed; the connection itself is still usable.
    HandlerError,
    /// Catch-all for failures that do not fit the codes above.
    InternalError,
}

impl ErrorCode {
    /// Wire spelling of this code.
    pub fn as_str(&self) -> &str {
        match self {
            Self::UnsupportedProtocol => "unsupported_protocol",
            Self::BadJson => "bad_json",
            Self::FrameTooLarge => "frame_too_large",
            Self::UnknownType => "unknown_type",
            Self::HandlerError => "handler_error",
            Self::InternalError => "internal_error",
        }
    }

    /// Parse a wire spelling; unknown strings map to [`ErrorCode::InternalError`].
    pub fn parse(s: &str) -> Self {
        match s {
            "unsupported_protocol" => Self::UnsupportedProtocol,
            "bad_json" => Self::BadJson,
            "frame_too_large" => Self::FrameTooLarge,
            "unknown_type" => Self::UnknownType,
            "handler_error" => Self::HandlerError,
            _ => Self::InternalError,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse(&s))
    }
}

/// A fully typed inbound frame.
///
/// `child_id` is only present on frames a child sends. `protocol` is only
/// meaningful on `hello` / `hello_ack`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Message {
    /// Frame discriminator. Defaults to `event` when the peer omitted it.
    #[serde(rename = "type", default = "default_kind")]
    pub kind: MessageKind,

    /// Event name for [`MessageKind::Event`] frames.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,

    /// Payload. `null` when absent.
    #[serde(default, skip_serializing_if = "is_null")]
    pub data: serde_json::Value,

    /// Emitting child; set by children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_id: Option<String>,

    /// Protocol revision claimed by the sender.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<u32>,

    /// `hello_ack` only: whether the parent accepted the child.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted: Option<bool>,

    /// `hello_ack` / `hello` only: logical parent identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// `hello` only: logical example/app name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_name: Option<String>,

    /// `error` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,

    /// `error` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// `error` only: whether the sender is closing the channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fatal: Option<bool>,
}

fn default_kind() -> MessageKind {
    MessageKind::Event
}

fn is_null(v: &serde_json::Value) -> bool {
    v.is_null()
}

impl Message {
    /// Build an outbound event frame.
    pub fn event(event: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            kind: MessageKind::Event,
            event: Some(event.into()),
            data,
            child_id: None,
            protocol: None,
            accepted: None,
            parent_id: None,
            example_name: None,
            code: None,
            message: None,
            fatal: None,
        }
    }

    /// Attach the emitting child id (builder-style, for chaining).
    pub fn with_child_id(mut self, child_id: Option<String>) -> Self {
        self.child_id = child_id;
        self
    }

    /// Build an error frame.
    pub fn error(code: ErrorCode, message: impl Into<String>, fatal: bool) -> Self {
        Self {
            kind: MessageKind::Error,
            event: None,
            data: serde_json::Value::Null,
            child_id: None,
            protocol: None,
            accepted: None,
            parent_id: None,
            example_name: None,
            code: Some(code),
            message: Some(message.into()),
            fatal: Some(fatal),
        }
    }

    /// Serialize to a wire frame, **without** the trailing delimiter.
    pub fn encode(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Incremental NDJSON frame decoder.
///
/// Feed it raw bytes with [`FrameReader::push`] and drain complete frames with
/// [`FrameReader::next_frame`]. Bytes are buffered, so a frame may span any
/// number of `push` calls.
#[derive(Debug, Default)]
pub struct FrameReader {
    buffer: Vec<u8>,
    /// Whether the leading-BOM check has already run.
    bom_checked: bool,
}

/// Reason a [`FrameReader`] rejected its buffered input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// Buffer grew past [`MAX_FRAME_BYTES`] without a delimiter.
    TooLarge,
    /// A complete frame was not valid UTF-8 or not a JSON object.
    BadJson(String),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "frame exceeded {} bytes", MAX_FRAME_BYTES),
            Self::BadJson(detail) => write!(f, "invalid frame: {}", detail),
        }
    }
}

impl FrameReader {
    /// Create an empty decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append freshly received bytes.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Pop the next complete frame, if one is buffered.
    ///
    /// Blank lines are skipped, so a batch containing `\n\n` never stalls the
    /// caller's drain loop. Returns `Err` **and clears the buffer** on a
    /// malformed frame; the caller should surface an error and keep going.
    pub fn next_frame(&mut self) -> Option<Result<Message, FrameError>> {
        // Tolerate a leading UTF-8 BOM. Hosts that build a `StreamWriter` on a
        // default `Encoding.UTF8` (PowerShell, .NET, some C# samples) emit one
        // before the first frame. Senders SHOULD NOT write a BOM; receivers
        // MUST tolerate one, because the alternative is a dead channel.
        if !self.bom_checked && self.buffer.len() >= UTF8_BOM.len() {
            self.bom_checked = true;
            if self.buffer.starts_with(&UTF8_BOM) {
                self.buffer.drain(..UTF8_BOM.len());
            }
        }

        loop {
            let newline = self.buffer.iter().position(|b| *b == FRAME_DELIMITER)?;

            let mut raw: Vec<u8> = self.buffer.drain(..=newline).collect();
            // Drop the delimiter, then any CR from a CRLF sender.
            raw.pop();
            if raw.last() == Some(&b'\r') {
                raw.pop();
            }

            if raw.is_empty() {
                if self.buffer.is_empty() {
                    return None;
                }
                continue;
            }

            if self.buffer.len() > MAX_FRAME_BYTES {
                self.buffer.clear();
                return Some(Err(FrameError::TooLarge));
            }

            let line = match std::str::from_utf8(&raw) {
                Ok(line) => line,
                Err(e) => {
                    self.buffer.clear();
                    return Some(Err(FrameError::BadJson(e.to_string())));
                }
            };

            return match serde_json::from_str::<Message>(line) {
                Ok(message) => Some(Ok(message)),
                Err(e) => {
                    // A JSON array/scalar is not a frame. Anything else is
                    // treated as bad JSON, keeping the error surface small.
                    self.buffer.clear();
                    Some(Err(FrameError::BadJson(e.to_string())))
                }
            };
        }
    }

    /// Guard against an unbounded buffer when a peer never sends `\n`.
    ///
    /// Callers should [`FrameReader::reset`] and surface
    /// [`FrameError::TooLarge`] when this trips.
    pub fn is_overgrown(&self) -> bool {
        self.buffer.len() > MAX_FRAME_BYTES
    }

    /// Drop buffered bytes (used after [`FrameError::TooLarge`]).
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_roundtrips_through_wire_spelling() {
        for kind in [
            MessageKind::Hello,
            MessageKind::HelloAck,
            MessageKind::Event,
            MessageKind::Error,
            MessageKind::Ping,
            MessageKind::Pong,
        ] {
            assert_eq!(MessageKind::parse(kind.as_str()), kind);
        }
        assert_eq!(
            MessageKind::parse("something_new"),
            MessageKind::Unknown("something_new".into())
        );
    }

    #[test]
    fn error_code_roundtrips() {
        assert_eq!(
            ErrorCode::parse(ErrorCode::UnsupportedProtocol.as_str()),
            ErrorCode::UnsupportedProtocol
        );
        assert_eq!(ErrorCode::parse("nope"), ErrorCode::InternalError);
    }

    #[test]
    fn legacy_frames_without_type_default_to_event() {
        let msg: Message = serde_json::from_str(r#"{"event":"parent:command","data":{"a":1}}"#)
            .expect("legacy frame must deserialize");
        assert_eq!(msg.kind, MessageKind::Event);
        assert_eq!(msg.event.as_deref(), Some("parent:command"));
        assert_eq!(msg.data, serde_json::json!({"a": 1}));
    }

    #[test]
    fn event_frame_omits_optional_fields() {
        let encoded = Message::event("child:ready", serde_json::json!({"ok": true}))
            .encode()
            .unwrap();
        assert_eq!(
            encoded,
            r#"{"type":"event","event":"child:ready","data":{"ok":true}}"#
        );
    }

    #[test]
    fn frame_reader_handles_split_and_batched_frames() {
        let mut reader = FrameReader::new();
        reader.push(b"{\"type\":\"ping\"}\n{\"type\":\"po");
        assert!(matches!(
            reader.next_frame(),
            Some(Ok(Message {
                kind: MessageKind::Ping,
                ..
            }))
        ));
        assert!(reader.next_frame().is_none());

        reader.push(b"ng\"}\n");
        assert!(matches!(
            reader.next_frame(),
            Some(Ok(Message {
                kind: MessageKind::Pong,
                ..
            }))
        ));
        assert!(reader.next_frame().is_none());
    }

    #[test]
    fn frame_reader_tolerates_crlf() {
        let mut reader = FrameReader::new();
        reader.push(b"{\"type\":\"hello\",\"protocol\":1}\r\n");
        let frame = reader.next_frame().expect("frame").expect("valid");
        assert_eq!(frame.kind, MessageKind::Hello);
        assert_eq!(frame.protocol, Some(1));
    }

    #[test]
    fn frame_reader_rejects_invalid_json() {
        let mut reader = FrameReader::new();
        reader.push(b"{not json}\n");
        assert!(matches!(
            reader.next_frame(),
            Some(Err(FrameError::BadJson(_)))
        ));
        assert!(reader.buffer.is_empty());
    }

    #[test]
    fn frame_reader_tolerates_a_leading_bom() {
        // Reproduces PowerShell / .NET StreamWriter, which writes a BOM before
        // the first frame. Without BOM handling the two frames would be glued
        // together and rejected as bad JSON.
        let mut reader = FrameReader::new();
        let mut wire = Vec::new();
        wire.extend_from_slice(&UTF8_BOM);
        wire.extend_from_slice(b"{\"type\":\"hello_ack\",\"accepted\":true}\n");
        reader.push(&wire);

        let frame = reader.next_frame().expect("frame").expect("valid");
        assert_eq!(frame.kind, MessageKind::HelloAck);
        assert_eq!(frame.accepted, Some(true));
    }

    #[test]
    fn frame_reader_ignores_a_bom_only_on_the_first_frame() {
        let mut reader = FrameReader::new();
        reader.push(b"{\"type\":\"ping\"}\n");
        let frame = reader.next_frame().expect("frame").expect("valid");
        assert_eq!(frame.kind, MessageKind::Ping);

        // A BOM later in the stream is not a BOM, it is malformed input.
        let mut reader = FrameReader::new();
        reader.push(b"{\"type\":\"ping\"}\n");
        reader.next_frame();
        let mut wire = Vec::new();
        wire.extend_from_slice(&UTF8_BOM);
        wire.extend_from_slice(b"{\"type\":\"pong\"}\n");
        reader.push(&wire);
        assert!(matches!(
            reader.next_frame(),
            Some(Err(FrameError::BadJson(_)))
        ));
    }

    #[test]
    fn frame_reader_skips_blank_lines_without_stalling() {
        let mut reader = FrameReader::new();
        reader.push(b"\n\n{\"type\":\"pong\"}\n");
        let frame = reader.next_frame().expect("frame").expect("valid");
        assert_eq!(frame.kind, MessageKind::Pong);
        assert!(reader.next_frame().is_none());
    }
}
