//! Unit tests for process plugin
//!
//! Tests for ProcessPlugin commands and IPC functionality.

use auroraview_plugins::process::{ProcessPlugin, SpawnIpcOptions};
use auroraview_plugins::{PluginHandler, ScopeConfig};

#[test]
fn test_process_plugin_commands() {
    let plugin = ProcessPlugin::new();
    let commands = plugin.commands();
    assert!(commands.contains(&"spawn_ipc"));
    assert!(commands.contains(&"kill"));
    assert!(commands.contains(&"kill_all"));
    assert!(commands.contains(&"send"));
    assert!(commands.contains(&"list"));
}

#[test]
fn test_process_plugin_name() {
    let plugin = ProcessPlugin::new();
    assert_eq!(plugin.name(), "process");
}

#[test]
fn test_process_plugin_default() {
    let plugin = ProcessPlugin::default();
    assert_eq!(plugin.name(), "process");
}

#[test]
fn test_list_empty() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("list", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data["processes"], serde_json::json!([]));
}

#[test]
fn test_spawn_blocked_by_scope() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::new(); // Default blocks all

    let result = plugin.handle(
        "spawn_ipc",
        serde_json::json!({
            "command": "echo",
            "args": ["hello"]
        }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_kill_nonexistent() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    // Kill nonexistent process should succeed (already exited)
    let result = plugin.handle("kill", serde_json::json!({ "pid": 99999 }), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["success"].as_bool().unwrap());
    assert!(data["already_exited"].as_bool().unwrap_or(false));
}

#[test]
fn test_kill_all_empty() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("kill_all", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["success"].as_bool().unwrap());
    assert_eq!(data["killed"].as_i64().unwrap(), 0);
}

#[test]
fn test_send_nonexistent() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "send",
        serde_json::json!({ "pid": 99999, "data": "test" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_spawn_ipc_options_deserialization() {
    let json = serde_json::json!({
        "command": "python",
        "args": ["-c", "print('hello')"],
        "cwd": "/tmp",
        "env": {"FOO": "bar"},
        "showConsole": true
    });
    let opts: SpawnIpcOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "python");
    assert_eq!(opts.args, vec!["-c", "print('hello')"]);
    assert_eq!(opts.cwd, Some("/tmp".to_string()));
    assert!(opts.show_console);
}

#[test]
fn test_spawn_ipc_options_defaults() {
    let json = serde_json::json!({
        "command": "echo"
    });
    let opts: SpawnIpcOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "echo");
    assert!(opts.args.is_empty());
    assert!(opts.cwd.is_none());
    assert!(opts.env.is_empty());
    assert!(!opts.show_console);
}

#[test]
fn test_invalid_args() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("spawn_ipc", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_kill_invalid_args() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("kill", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_send_invalid_args() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("send", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_command_not_found() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}
