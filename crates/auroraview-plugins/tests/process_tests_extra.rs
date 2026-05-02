//! Additional edge-case tests for process plugin
//!
//! Tests for error message verification and edge cases.

use std::sync::Arc;
use std::thread;

use auroraview_plugins::process::ProcessPlugin;
use auroraview_plugins::{PluginHandler, ScopeConfig};

// ---------------------------------------------------------------------------
// Error message content verification (following clipboard/dialog/fs pattern)
// ---------------------------------------------------------------------------

#[test]
fn unknown_command_error_message_contains_command() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("totally_unknown", serde_json::json!({}), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("totally_unknown"),
        "Error should mention the unknown command: {}",
        msg
    );
}

#[test]
fn unknown_command_error_code_is_command_not_found() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("nope", serde_json::json!({}), &scope)
        .unwrap_err();
    assert_eq!(err.code(), "COMMAND_NOT_FOUND");
}

#[test]
fn send_missing_pid_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("send", serde_json::json!({ "data": "test" }), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("pid") || msg.contains("missing"),
        "Error should mention missing field: {}",
        msg
    );
}

#[test]
fn send_json_missing_pid_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("send_json", serde_json::json!({ "data": {"a": 1} }), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("pid") || msg.contains("missing"),
        "Error should mention missing field: {}",
        msg
    );
}

#[test]
fn spawn_ipc_missing_command_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle(
            "spawn_ipc",
            serde_json::json!({ "args": ["hello"] }),
            &scope,
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("command") || msg.contains("missing"),
        "Error should mention missing field: {}",
        msg
    );
}

#[test]
fn kill_missing_pid_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("kill", serde_json::json!({}), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("pid") || msg.contains("missing"),
        "Error should mention missing field: {}",
        msg
    );
}

#[test]
fn send_null_args_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("send", serde_json::json!(null), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}

#[test]
fn spawn_ipc_null_args_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("spawn_ipc", serde_json::json!(null), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}

// ---------------------------------------------------------------------------
// Additional edge-case tests
// ---------------------------------------------------------------------------

#[test]
fn plugin_handle_empty_command_string() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn plugin_handle_whitespace_only_command() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("   ", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn unknown_command_multiple_threads_same_error_code() {
    let plugin = Arc::new(ProcessPlugin::new());
    let scope = ScopeConfig::permissive();
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            thread::spawn(move || {
                let cmd = format!("unknown_{}", i);
                p.handle(&cmd, serde_json::json!({}), &s)
                    .unwrap_err()
                    .code()
            })
        })
        .collect();

    let codes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    for code in &codes {
        assert_eq!(code, &"COMMAND_NOT_FOUND");
    }
}

#[test]
fn kill_null_args_error_message() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let err = plugin
        .handle("kill", serde_json::json!(null), &scope)
        .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
}

#[test]
fn list_null_args_succeeds() {
    // list command should accept null args (uses default options)
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("list", serde_json::json!(null), &scope);
    assert!(result.is_ok());
}

#[test]
fn kill_all_null_args_succeeds() {
    // kill_all should accept null args (uses default options)
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("kill_all", serde_json::json!(null), &scope);
    assert!(result.is_ok());
}
