//! Unit tests for plugin types
//!
//! These tests verify the plugin type definitions and error handling.

use auroraview_plugins::{PluginCommand, PluginError, PluginErrorCode};

// =============================================================================
// PluginCommand tests
// =============================================================================

#[test]
fn test_plugin_command_new() {
    let cmd = PluginCommand::new("test_cmd", "A test command");
    assert_eq!(cmd.name, "test_cmd");
    assert_eq!(cmd.description, "A test command");
    assert!(cmd.required_args.is_empty());
    assert!(cmd.optional_args.is_empty());
}

#[test]
fn test_plugin_command_with_required() {
    let cmd = PluginCommand::new("read_file", "Read a file").with_required(&["path", "encoding"]);
    assert_eq!(cmd.required_args, vec!["path", "encoding"]);
}

#[test]
fn test_plugin_command_with_optional() {
    let cmd =
        PluginCommand::new("write_file", "Write a file").with_optional(&["append", "create_dirs"]);
    assert_eq!(cmd.optional_args, vec!["append", "create_dirs"]);
}

#[test]
fn test_plugin_command_builder_chain() {
    let cmd = PluginCommand::new("copy", "Copy files")
        .with_required(&["from", "to"])
        .with_optional(&["overwrite"]);

    assert_eq!(cmd.name, "copy");
    assert_eq!(cmd.required_args, vec!["from", "to"]);
    assert_eq!(cmd.optional_args, vec!["overwrite"]);
}

#[test]
fn test_plugin_command_clone() {
    let cmd = PluginCommand::new("test", "Test command").with_required(&["arg1"]);
    let cloned = cmd.clone();
    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.required_args, vec!["arg1"]);
}

#[test]
fn test_plugin_command_serialize() {
    let cmd = PluginCommand::new("test", "Test").with_required(&["path"]);
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("\"name\":\"test\""));
    assert!(json.contains("\"required_args\":[\"path\"]"));
}

// =============================================================================
// PluginErrorCode tests
// =============================================================================

#[test]
fn test_plugin_error_code_as_str() {
    assert_eq!(PluginErrorCode::PluginNotFound.as_str(), "PLUGIN_NOT_FOUND");
    assert_eq!(
        PluginErrorCode::CommandNotFound.as_str(),
        "COMMAND_NOT_FOUND"
    );
    assert_eq!(PluginErrorCode::InvalidArgs.as_str(), "INVALID_ARGS");
    assert_eq!(
        PluginErrorCode::PermissionDenied.as_str(),
        "PERMISSION_DENIED"
    );
    assert_eq!(PluginErrorCode::ScopeViolation.as_str(), "SCOPE_VIOLATION");
    assert_eq!(PluginErrorCode::FileNotFound.as_str(), "FILE_NOT_FOUND");
    assert_eq!(PluginErrorCode::IoError.as_str(), "IO_ERROR");
    assert_eq!(PluginErrorCode::EncodingError.as_str(), "ENCODING_ERROR");
    assert_eq!(PluginErrorCode::ClipboardError.as_str(), "CLIPBOARD_ERROR");
    assert_eq!(PluginErrorCode::ShellError.as_str(), "SHELL_ERROR");
    assert_eq!(
        PluginErrorCode::DialogCancelled.as_str(),
        "DIALOG_CANCELLED"
    );
    assert_eq!(PluginErrorCode::Unknown.as_str(), "UNKNOWN");
}

#[test]
fn test_plugin_error_code_display() {
    assert_eq!(
        format!("{}", PluginErrorCode::FileNotFound),
        "FILE_NOT_FOUND"
    );
    assert_eq!(format!("{}", PluginErrorCode::IoError), "IO_ERROR");
}

#[test]
fn test_plugin_error_code_clone() {
    let code = PluginErrorCode::PermissionDenied;
    let cloned = code;
    assert_eq!(code, cloned);
}

#[test]
fn test_plugin_error_code_eq() {
    assert_eq!(PluginErrorCode::FileNotFound, PluginErrorCode::FileNotFound);
    assert_ne!(PluginErrorCode::FileNotFound, PluginErrorCode::IoError);
}

// =============================================================================
// PluginError tests
// =============================================================================

#[test]
fn test_plugin_error_new() {
    let err = PluginError::new(PluginErrorCode::FileNotFound, "File not found: test.txt");
    assert_eq!(err.code(), "FILE_NOT_FOUND");
    assert_eq!(err.message(), "File not found: test.txt");
    assert_eq!(err.error_code(), PluginErrorCode::FileNotFound);
}

#[test]
fn test_plugin_error_command_not_found() {
    let err = PluginError::command_not_found("unknown_cmd");
    assert_eq!(err.error_code(), PluginErrorCode::CommandNotFound);
    assert!(err.message().contains("unknown_cmd"));
}

#[test]
fn test_plugin_error_invalid_args() {
    let err = PluginError::invalid_args("Missing required parameter 'path'");
    assert_eq!(err.error_code(), PluginErrorCode::InvalidArgs);
    assert!(err.message().contains("path"));
}

#[test]
fn test_plugin_error_scope_violation() {
    let err = PluginError::scope_violation("/etc/passwd");
    assert_eq!(err.error_code(), PluginErrorCode::ScopeViolation);
    assert!(err.message().contains("/etc/passwd"));
}

#[test]
fn test_plugin_error_file_not_found() {
    let err = PluginError::file_not_found("/path/to/missing.txt");
    assert_eq!(err.error_code(), PluginErrorCode::FileNotFound);
    assert!(err.message().contains("/path/to/missing.txt"));
}

#[test]
fn test_plugin_error_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let err = PluginError::io_error(io_err);
    assert_eq!(err.error_code(), PluginErrorCode::IoError);
    assert!(err.message().contains("Access denied"));
}

#[test]
fn test_plugin_error_clipboard_error() {
    let err = PluginError::clipboard_error("Clipboard not available");
    assert_eq!(err.error_code(), PluginErrorCode::ClipboardError);
}

#[test]
fn test_plugin_error_shell_error() {
    let err = PluginError::shell_error("Command failed with exit code 1");
    assert_eq!(err.error_code(), PluginErrorCode::ShellError);
}

#[test]
fn test_plugin_error_dialog_cancelled() {
    let err = PluginError::dialog_cancelled();
    assert_eq!(err.error_code(), PluginErrorCode::DialogCancelled);
    assert!(err.message().contains("cancelled"));
}

#[test]
fn test_plugin_error_display() {
    let err = PluginError::new(PluginErrorCode::FileNotFound, "test.txt not found");
    let display = format!("{}", err);
    assert!(display.contains("FILE_NOT_FOUND"));
    assert!(display.contains("test.txt not found"));
}

#[test]
fn test_plugin_error_debug() {
    let err = PluginError::new(PluginErrorCode::IoError, "Read failed");
    let debug = format!("{:?}", err);
    assert!(debug.contains("PluginError"));
}
