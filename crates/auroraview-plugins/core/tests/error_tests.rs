//! Tests for PluginError and PluginErrorCode

use auroraview_plugin_core::{PluginError, PluginErrorCode, PluginResult};
use rstest::rstest;

// ── PluginErrorCode::as_str ─────────────────────────────────────────────────

#[rstest]
#[case(PluginErrorCode::PluginNotFound, "PLUGIN_NOT_FOUND")]
#[case(PluginErrorCode::CommandNotFound, "COMMAND_NOT_FOUND")]
#[case(PluginErrorCode::InvalidArgs, "INVALID_ARGS")]
#[case(PluginErrorCode::PermissionDenied, "PERMISSION_DENIED")]
#[case(PluginErrorCode::ScopeViolation, "SCOPE_VIOLATION")]
#[case(PluginErrorCode::FileNotFound, "FILE_NOT_FOUND")]
#[case(PluginErrorCode::IoError, "IO_ERROR")]
#[case(PluginErrorCode::EncodingError, "ENCODING_ERROR")]
#[case(PluginErrorCode::ClipboardError, "CLIPBOARD_ERROR")]
#[case(PluginErrorCode::ShellError, "SHELL_ERROR")]
#[case(PluginErrorCode::DialogCancelled, "DIALOG_CANCELLED")]
#[case(PluginErrorCode::SerializationError, "SERIALIZATION_ERROR")]
#[case(PluginErrorCode::Unknown, "UNKNOWN")]
fn error_code_as_str(#[case] code: PluginErrorCode, #[case] expected: &str) {
    assert_eq!(code.as_str(), expected);
}

// ── PluginErrorCode::Display ─────────────────────────────────────────────────

#[rstest]
#[case(PluginErrorCode::PluginNotFound, "PLUGIN_NOT_FOUND")]
#[case(PluginErrorCode::CommandNotFound, "COMMAND_NOT_FOUND")]
#[case(PluginErrorCode::Unknown, "UNKNOWN")]
fn error_code_display(#[case] code: PluginErrorCode, #[case] expected: &str) {
    assert_eq!(format!("{}", code), expected);
}

// ── PluginErrorCode Debug ────────────────────────────────────────────────────

#[test]
fn error_code_debug() {
    let s = format!("{:?}", PluginErrorCode::InvalidArgs);
    assert!(s.contains("InvalidArgs"));
}

// ── PluginErrorCode PartialEq / Clone / Copy ─────────────────────────────────

#[test]
fn error_code_eq() {
    assert_eq!(PluginErrorCode::IoError, PluginErrorCode::IoError);
    assert_ne!(PluginErrorCode::IoError, PluginErrorCode::ShellError);
}

#[test]
fn error_code_copy() {
    let a = PluginErrorCode::FileNotFound;
    let b = a; // copy
    assert_eq!(a, b);
}

// ── PluginError constructors ──────────────────────────────────────────────────

#[test]
fn error_new_stores_fields() {
    let err = PluginError::new(PluginErrorCode::InvalidArgs, "bad input");
    assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
    assert_eq!(err.code(), "INVALID_ARGS");
    assert_eq!(err.message(), "bad input");
}

#[test]
fn error_command_not_found() {
    let err = PluginError::command_not_found("my_cmd");
    assert_eq!(err.error_code(), PluginErrorCode::CommandNotFound);
    assert!(err.message().contains("my_cmd"));
}

#[test]
fn error_invalid_args() {
    let err = PluginError::invalid_args("missing field 'path'");
    assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
    assert!(err.message().contains("missing field"));
}

#[test]
fn error_scope_violation() {
    let err = PluginError::scope_violation("/etc/passwd");
    assert_eq!(err.error_code(), PluginErrorCode::ScopeViolation);
    assert!(err.message().contains("/etc/passwd"));
}

#[test]
fn error_file_not_found() {
    let err = PluginError::file_not_found("/nonexistent/file.txt");
    assert_eq!(err.error_code(), PluginErrorCode::FileNotFound);
    assert!(err.message().contains("/nonexistent/file.txt"));
}

#[test]
fn error_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err = PluginError::io_error(io_err);
    assert_eq!(err.error_code(), PluginErrorCode::IoError);
    assert!(err.message().contains("access denied"));
}

#[test]
fn error_clipboard_error() {
    let err = PluginError::clipboard_error("clipboard unavailable");
    assert_eq!(err.error_code(), PluginErrorCode::ClipboardError);
    assert!(err.message().contains("clipboard unavailable"));
}

#[test]
fn error_shell_error() {
    let err = PluginError::shell_error("process failed");
    assert_eq!(err.error_code(), PluginErrorCode::ShellError);
    assert!(err.message().contains("process failed"));
}

#[test]
fn error_dialog_cancelled() {
    let err = PluginError::dialog_cancelled();
    assert_eq!(err.error_code(), PluginErrorCode::DialogCancelled);
    assert!(!err.message().is_empty());
}

#[test]
fn error_serialization_error() {
    let err = PluginError::serialization_error("unexpected token");
    assert_eq!(err.error_code(), PluginErrorCode::SerializationError);
    assert!(err.message().contains("unexpected token"));
}

#[test]
fn error_from_plugin() {
    let err = PluginError::from_plugin("my_plugin", "custom error");
    assert_eq!(err.error_code(), PluginErrorCode::Unknown);
    assert!(err.message().contains("custom error"));
}

// ── PluginError Display ───────────────────────────────────────────────────────

#[test]
fn error_display_contains_code_and_message() {
    let err = PluginError::shell_error("cmd not found");
    let s = format!("{}", err);
    assert!(s.contains("SHELL_ERROR"));
    assert!(s.contains("cmd not found"));
}

// ── PluginError Debug ─────────────────────────────────────────────────────────

#[test]
fn error_debug_non_empty() {
    let err = PluginError::invalid_args("oops");
    let s = format!("{:?}", err);
    assert!(!s.is_empty());
}

// ── PluginError Send + Sync ───────────────────────────────────────────────────

#[test]
fn error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginError>();
}

// ── PluginResult alias ────────────────────────────────────────────────────────

#[test]
fn plugin_result_ok() {
    let r: PluginResult<i32> = Ok(42);
    assert!(r.is_ok());
}

#[test]
fn plugin_result_err() {
    let r: PluginResult<i32> = Err(PluginError::dialog_cancelled());
    assert!(r.is_err());
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn error_empty_message() {
    let err = PluginError::invalid_args("");
    assert_eq!(err.message(), "");
}

#[test]
fn error_long_message() {
    let msg = "x".repeat(10_000);
    let err = PluginError::shell_error(msg.clone());
    assert_eq!(err.message(), msg);
}

#[rstest]
#[case("ls")]
#[case("rm -rf /")]
#[case("")]
#[case("cmd with spaces")]
fn error_command_not_found_various(#[case] cmd: &str) {
    let err = PluginError::command_not_found(cmd);
    assert_eq!(err.error_code(), PluginErrorCode::CommandNotFound);
    if !cmd.is_empty() {
        assert!(err.message().contains(cmd));
    }
}
