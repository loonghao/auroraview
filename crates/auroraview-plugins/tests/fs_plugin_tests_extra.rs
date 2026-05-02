//! Additional edge-case tests for file system plugin
//!
//! Tests for error message verification and edge cases.

use std::sync::Arc;
use std::thread;

use auroraview_plugins::fs::FsPlugin;
use auroraview_plugins::{PluginHandler, ScopeConfig};

// ---------------------------------------------------------------------------
// Error message content verification (following clipboard/dialog plugin pattern)
// ---------------------------------------------------------------------------

#[test]
fn unknown_command_error_message_contains_command() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("totally_unknown", serde_json::json!({}), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("totally_unknown"), "Error should mention the unknown command: {}", msg);
}

#[test]
fn unknown_command_error_code_is_command_not_found() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("nope", serde_json::json!({}), &scope).unwrap_err();
    assert_eq!(err.code(), "COMMAND_NOT_FOUND");
}

#[test]
fn read_file_missing_path_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("read_file", serde_json::json!({ "invalid": "args" }), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("path") || msg.contains("missing"), "Error should mention missing field: {}", msg);
}

#[test]
fn write_file_missing_contents_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("write_file", serde_json::json!({ "path": "/test" }), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("contents") || msg.contains("missing"), "Error should mention missing field: {}", msg);
}

#[test]
fn read_file_null_args_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("read_file", serde_json::json!(null), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}

#[test]
fn write_file_null_args_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("write_file", serde_json::json!(null), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}

#[test]
fn copy_missing_from_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("copy", serde_json::json!({ "to": "/dst" }), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("from") || msg.contains("missing"), "Error should mention missing field: {}", msg);
}

#[test]
fn rename_missing_from_error_message() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("rename", serde_json::json!({ "to": "/dst" }), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("from") || msg.contains("missing"), "Error should mention missing field: {}", msg);
}

// ---------------------------------------------------------------------------
// Additional edge-case tests
// ---------------------------------------------------------------------------

#[test]
fn plugin_handle_empty_command_string() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn plugin_handle_whitespace_only_command() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("   ", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn unknown_command_multiple_threads_same_error_code() {
    let plugin = Arc::new(FsPlugin::new());
    let scope = ScopeConfig::permissive();
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            thread::spawn(move || {
                let cmd = format!("unknown_{}", i);
                p.handle(&cmd, serde_json::json!({}), &s).unwrap_err().code()
            })
        })
        .collect();

    let codes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    for code in &codes {
        assert_eq!(code, &"COMMAND_NOT_FOUND");
    }
}

#[test]
fn read_file_empty_path_error() {
    let plugin = FsPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin.handle("read_file", serde_json::json!({ "path": "" }), &scope).unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}
