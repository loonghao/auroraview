//! Unit tests for plugin types
//!
//! Tests for PluginCommand, PluginError, PluginErrorCode, PluginRequest, PluginResponse.

use auroraview_plugins::{PluginCommand, PluginError, PluginErrorCode, PluginRequest, PluginResponse};
use rstest::rstest;
use std::sync::Arc;

// =============================================================================
// PluginCommand tests
// =============================================================================

#[test]
fn plugin_command_new() {
    let cmd = PluginCommand::new("test_cmd", "A test command");
    assert_eq!(cmd.name, "test_cmd");
    assert_eq!(cmd.description, "A test command");
    assert!(cmd.required_args.is_empty());
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn plugin_command_with_required() {
    let cmd = PluginCommand::new("read_file", "Read a file").with_required(&["path", "encoding"]);
    assert_eq!(cmd.required_args, vec!["path", "encoding"]);
}

#[test]
fn plugin_command_with_optional() {
    let cmd =
        PluginCommand::new("write_file", "Write a file").with_optional(&["append", "create_dirs"]);
    assert_eq!(cmd.optional_args, vec!["append", "create_dirs"]);
}

#[test]
fn plugin_command_builder_chain() {
    let cmd = PluginCommand::new("copy", "Copy files")
        .with_required(&["from", "to"])
        .with_optional(&["overwrite"]);
    assert_eq!(cmd.name, "copy");
    assert_eq!(cmd.required_args, vec!["from", "to"]);
    assert_eq!(cmd.optional_args, vec!["overwrite"]);
}

#[test]
fn plugin_command_clone() {
    let cmd = PluginCommand::new("test", "Test command").with_required(&["arg1"]);
    let cloned = cmd.clone();
    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.required_args, vec!["arg1"]);
}

#[test]
fn plugin_command_serialize() {
    let cmd = PluginCommand::new("test", "Test").with_required(&["path"]);
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"name\":\"test\""));
    assert!(json.contains("\"required_args\":[\"path\"]"));
}

#[test]
fn plugin_command_empty_args() {
    let cmd = PluginCommand::new("list", "List items").with_required(&[]).with_optional(&[]);
    assert!(cmd.required_args.is_empty());
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn plugin_command_many_args() {
    let required: Vec<&str> = (0..10).map(|_| "arg").collect();
    let optional: Vec<&str> = (0..10).map(|_| "opt").collect();
    let cmd = PluginCommand::new("complex", "Complex cmd")
        .with_required(&required)
        .with_optional(&optional);
    assert_eq!(cmd.required_args.len(), 10);
    assert_eq!(cmd.optional_args.len(), 10);
}

#[test]
fn plugin_command_debug() {
    let cmd = PluginCommand::new("debug_cmd", "Debug test");
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("debug_cmd") || debug.contains("PluginCommand"));
}

#[test]
fn plugin_command_serde_roundtrip() {
    let original = PluginCommand::new("rt_cmd", "Roundtrip")
        .with_required(&["path"])
        .with_optional(&["encoding"]);
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: PluginCommand = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.description, original.description);
    assert_eq!(deserialized.required_args, original.required_args);
    assert_eq!(deserialized.optional_args, original.optional_args);
}

// =============================================================================
// PluginErrorCode tests
// =============================================================================

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
#[case(PluginErrorCode::Unknown, "UNKNOWN")]
fn error_code_as_str(#[case] code: PluginErrorCode, #[case] expected: &str) {
    assert_eq!(code.as_str(), expected);
}

#[rstest]
#[case(PluginErrorCode::PluginNotFound, "PLUGIN_NOT_FOUND")]
#[case(PluginErrorCode::CommandNotFound, "COMMAND_NOT_FOUND")]
#[case(PluginErrorCode::InvalidArgs, "INVALID_ARGS")]
#[case(PluginErrorCode::FileNotFound, "FILE_NOT_FOUND")]
fn error_code_display(#[case] code: PluginErrorCode, #[case] expected: &str) {
    assert_eq!(format!("{}", code), expected);
}

#[test]
fn error_code_clone_eq() {
    let code = PluginErrorCode::PermissionDenied;
    let cloned = code;
    assert_eq!(code, cloned);
}

#[test]
fn error_code_eq_same() {
    assert_eq!(PluginErrorCode::FileNotFound, PluginErrorCode::FileNotFound);
    assert_ne!(PluginErrorCode::FileNotFound, PluginErrorCode::IoError);
}

#[test]
fn error_code_debug() {
    let code = PluginErrorCode::ClipboardError;
    let debug = format!("{:?}", code);
    assert!(debug.contains("ClipboardError") || !debug.is_empty());
}

// =============================================================================
// PluginError tests
// =============================================================================

#[test]
fn plugin_error_new() {
    let err = PluginError::new(PluginErrorCode::FileNotFound, "File not found: test.txt");
    assert_eq!(err.code(), "FILE_NOT_FOUND");
    assert_eq!(err.message(), "File not found: test.txt");
    assert_eq!(err.error_code(), PluginErrorCode::FileNotFound);
}

#[test]
fn plugin_error_command_not_found() {
    let err = PluginError::command_not_found("unknown_cmd");
    assert_eq!(err.error_code(), PluginErrorCode::CommandNotFound);
    assert!(err.message().contains("unknown_cmd"));
}

#[test]
fn plugin_error_invalid_args() {
    let err = PluginError::invalid_args("Missing required parameter 'path'");
    assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
    assert!(err.message().contains("path"));
}

#[test]
fn plugin_error_scope_violation() {
    let err = PluginError::scope_violation("/etc/passwd");
    assert_eq!(err.error_code(), PluginErrorCode::ScopeViolation);
    assert!(err.message().contains("/etc/passwd"));
}

#[test]
fn plugin_error_file_not_found() {
    let err = PluginError::file_not_found("/path/to/missing.txt");
    assert_eq!(err.error_code(), PluginErrorCode::FileNotFound);
    assert!(err.message().contains("/path/to/missing.txt"));
}

#[test]
fn plugin_error_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let err = PluginError::io_error(io_err);
    assert_eq!(err.error_code(), PluginErrorCode::IoError);
    assert!(err.message().contains("Access denied"));
}

#[test]
fn plugin_error_clipboard_error() {
    let err = PluginError::clipboard_error("Clipboard not available");
    assert_eq!(err.error_code(), PluginErrorCode::ClipboardError);
}

#[test]
fn plugin_error_shell_error() {
    let err = PluginError::shell_error("Command failed with exit code 1");
    assert_eq!(err.error_code(), PluginErrorCode::ShellError);
}

#[test]
fn plugin_error_dialog_cancelled() {
    let err = PluginError::dialog_cancelled();
    assert_eq!(err.error_code(), PluginErrorCode::DialogCancelled);
    assert!(err.message().contains("cancelled"));
}

#[test]
fn plugin_error_display() {
    let err = PluginError::new(PluginErrorCode::FileNotFound, "test.txt not found");
    let display = format!("{}", err);
    assert!(display.contains("FILE_NOT_FOUND"));
    assert!(display.contains("test.txt not found"));
}

#[test]
fn plugin_error_debug() {
    let err = PluginError::new(PluginErrorCode::IoError, "Read failed");
    let debug = format!("{:?}", err);
    assert!(debug.contains("PluginError"));
}

#[test]
fn plugin_error_as_std_error() {
    let err = PluginError::new(PluginErrorCode::IoError, "io failure");
    // PluginError should implement std::error::Error
    let std_err: &dyn std::error::Error = &err;
    assert!(std_err.to_string().contains("io failure"));
}

#[test]
fn plugin_error_empty_message() {
    let err = PluginError::invalid_args("");
    assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
    assert_eq!(err.message(), "");
}

#[test]
fn plugin_error_unicode_message() {
    let err = PluginError::file_not_found("/path/to/日本語/ファイル.txt");
    assert!(err.message().contains("日本語"));
}

#[test]
fn plugin_error_long_message() {
    let msg = "x".repeat(4096);
    let err = PluginError::invalid_args(msg.clone());
    assert_eq!(err.message(), msg);
}

#[rstest]
#[case("unknown_command")]
#[case("")]
#[case("COMMAND")]
#[case("cmd.sub")]
fn command_not_found_various(#[case] cmd: &str) {
    let err = PluginError::command_not_found(cmd);
    assert_eq!(err.error_code(), PluginErrorCode::CommandNotFound);
}

#[test]
fn plugin_error_io_various_kinds() {
    for kind in [
        std::io::ErrorKind::NotFound,
        std::io::ErrorKind::PermissionDenied,
        std::io::ErrorKind::TimedOut,
        std::io::ErrorKind::BrokenPipe,
    ] {
        let io_err = std::io::Error::new(kind, format!("{:?}", kind));
        let err = PluginError::io_error(io_err);
        assert_eq!(err.error_code(), PluginErrorCode::IoError);
    }
}

// =============================================================================
// PluginRequest / PluginResponse tests
// =============================================================================

#[test]
fn plugin_request_deserialize() {
    let json = serde_json::json!({
        "plugin": "fs",
        "command": "read_file",
        "args": { "path": "/test.txt" }
    });
    let req: PluginRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.plugin, "fs");
    assert_eq!(req.command, "read_file");
    assert_eq!(req.args["path"], "/test.txt");
}

#[test]
fn plugin_request_serde_roundtrip() {
    let original = PluginRequest {
        plugin: "clipboard".to_string(),
        command: "read_text".to_string(),
        args: serde_json::json!({}),
        id: None,
    };
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: PluginRequest = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.plugin, original.plugin);
    assert_eq!(deserialized.command, original.command);
}

#[test]
fn plugin_request_clone() {
    let req = PluginRequest {
        plugin: "dialog".to_string(),
        command: "open_file".to_string(),
        args: serde_json::json!({ "title": "Open" }),
        id: Some("req-001".to_string()),
    };
    let cloned = req.clone();
    assert_eq!(cloned.plugin, req.plugin);
    assert_eq!(cloned.command, req.command);
    assert_eq!(cloned.id, req.id);
}

#[test]
fn plugin_request_debug() {
    let req = PluginRequest {
        plugin: "shell".to_string(),
        command: "open".to_string(),
        args: serde_json::json!({}),
        id: None,
    };
    let debug = format!("{:?}", req);
    assert!(debug.contains("shell") || debug.contains("PluginRequest"));
}

#[test]
fn plugin_response_ok() {
    let resp = PluginResponse::ok(serde_json::json!({ "result": 42 }));
    assert!(resp.success);
    assert_eq!(resp.data.as_ref().unwrap()["result"], 42);
}

#[test]
fn plugin_response_err() {
    let resp = PluginResponse::err("not found", "FILE_NOT_FOUND");
    assert!(!resp.success);
    assert_eq!(resp.error.as_deref(), Some("not found"));
    assert_eq!(resp.code.as_deref(), Some("FILE_NOT_FOUND"));
}

#[test]
fn plugin_response_serde_roundtrip() {
    let original = PluginResponse::ok(serde_json::json!([1, 2, 3]));
    let serialized = serde_json::to_value(&original).unwrap();
    let deserialized: PluginResponse = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.success, original.success);
    assert_eq!(deserialized.data, original.data);
}

// =============================================================================
// Concurrent PluginError creation
// =============================================================================

#[test]
fn concurrent_plugin_error_creation() {
    let handles: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                let err = PluginError::invalid_args(format!("error_{}", i));
                assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn concurrent_plugin_command_creation() {
    let handles: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                let cmd = PluginCommand::new(
                    format!("cmd_{}", i),
                    format!("Command {}", i),
                );

                assert!(!cmd.name.is_empty());
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// Shared Arc<PluginError> across threads
#[test]
fn arc_plugin_error_shared() {
    let err = Arc::new(PluginError::new(PluginErrorCode::FileNotFound, "shared error"));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let e = Arc::clone(&err);
            std::thread::spawn(move || {
                assert_eq!(e.code(), "FILE_NOT_FOUND");
                assert_eq!(e.message(), "shared error");
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}
