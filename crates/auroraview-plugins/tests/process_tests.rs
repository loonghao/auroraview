//! Unit tests for process plugin
//!
//! Tests for ProcessPlugin commands and IPC functionality.

use auroraview_plugins::process::{
    IpcMode, KillOptions, ProcessPlugin, SendJsonOptions, SendOptions, SpawnIpcOptions,
};
use auroraview_plugins::{PluginHandler, ScopeConfig};
use rstest::*;

// ─────────────────────────────────────────────────────────────
// Plugin metadata
// ─────────────────────────────────────────────────────────────

#[test]
fn process_plugin_name() {
    let plugin = ProcessPlugin::new();
    assert_eq!(plugin.name(), "process");
}

#[test]
fn process_plugin_default_name() {
    let plugin = ProcessPlugin::default();
    assert_eq!(plugin.name(), "process");
}

#[test]
fn process_plugin_commands_contains_all() {
    let plugin = ProcessPlugin::new();
    let commands = plugin.commands();
    for cmd in &["spawn_ipc", "spawn_ipc_channel", "kill", "kill_all", "send", "send_json", "list"]
    {
        assert!(commands.contains(cmd), "missing command: {cmd}");
    }
}

#[test]
fn process_plugin_commands_count() {
    let plugin = ProcessPlugin::new();
    assert!(plugin.commands().len() >= 7);
}

// ─────────────────────────────────────────────────────────────
// list command
// ─────────────────────────────────────────────────────────────

#[test]
fn list_empty_returns_empty_array() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("list", serde_json::json!({}), &scope).unwrap();
    assert_eq!(result["processes"], serde_json::json!([]));
}

#[test]
fn list_any_valid_json_succeeds() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("list", serde_json::json!(null), &scope).unwrap();
    assert!(result["processes"].is_array());
}

// ─────────────────────────────────────────────────────────────
// kill command
// ─────────────────────────────────────────────────────────────

#[test]
fn kill_nonexistent_pid_succeeds_with_already_exited() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin
        .handle("kill", serde_json::json!({ "pid": 99999 }), &scope)
        .unwrap();
    assert!(result["success"].as_bool().unwrap());
    assert!(result["already_exited"].as_bool().unwrap_or(false));
}

#[rstest]
#[case(0u32)]
#[case(1u32)]
#[case(999999u32)]
fn kill_various_pids_not_found_succeeds(#[case] pid: u32) {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin
        .handle("kill", serde_json::json!({ "pid": pid }), &scope)
        .unwrap();
    assert!(result["success"].as_bool().unwrap());
}

#[test]
fn kill_missing_pid_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("kill", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// kill_all command
// ─────────────────────────────────────────────────────────────

#[test]
fn kill_all_empty_returns_zero_killed() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin
        .handle("kill_all", serde_json::json!({}), &scope)
        .unwrap();
    assert!(result["success"].as_bool().unwrap());
    assert_eq!(result["killed"].as_i64().unwrap(), 0);
}

#[test]
fn kill_all_second_call_still_succeeds() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    plugin
        .handle("kill_all", serde_json::json!({}), &scope)
        .unwrap();
    // After shutdown, kill_all and kill are still accepted
    let result = plugin.handle("kill_all", serde_json::json!({}), &scope);
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────
// send command
// ─────────────────────────────────────────────────────────────

#[test]
fn send_nonexistent_pid_returns_err() {
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
fn send_missing_args_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("send", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn send_missing_data_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("send", serde_json::json!({ "pid": 1 }), &scope);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// send_json command
// ─────────────────────────────────────────────────────────────

#[test]
fn send_json_nonexistent_pid_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle(
        "send_json",
        serde_json::json!({ "pid": 99999, "data": { "action": "ping" } }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn send_json_missing_args_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("send_json", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// spawn_ipc blocked by scope
// ─────────────────────────────────────────────────────────────

#[test]
fn spawn_ipc_blocked_by_default_scope() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(
        "spawn_ipc",
        serde_json::json!({ "command": "echo", "args": ["hello"] }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn spawn_ipc_channel_blocked_by_default_scope() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::new();
    let result = plugin.handle(
        "spawn_ipc_channel",
        serde_json::json!({ "command": "echo", "args": ["hello"] }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn spawn_ipc_missing_command_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("spawn_ipc", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn spawn_ipc_channel_missing_command_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("spawn_ipc_channel", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// Unknown command
// ─────────────────────────────────────────────────────────────

#[test]
fn unknown_command_returns_err() {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    let result = plugin.handle("unknown_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("spawn")]
#[case("exec")]
#[case("start")]
#[case("")]
fn various_unknown_commands_return_err(#[case] cmd: &str) {
    let plugin = ProcessPlugin::new();
    let scope = ScopeConfig::permissive();
    assert!(plugin.handle(cmd, serde_json::json!({}), &scope).is_err());
}

// ─────────────────────────────────────────────────────────────
// SpawnIpcOptions serde
// ─────────────────────────────────────────────────────────────

#[test]
fn spawn_ipc_options_full_deserialization() {
    let json = serde_json::json!({
        "command": "python",
        "args": ["-c", "print('hello')"],
        "cwd": "/tmp",
        "env": {"FOO": "bar", "BAZ": "qux"},
        "showConsole": true
    });
    let opts: SpawnIpcOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "python");
    assert_eq!(opts.args, vec!["-c", "print('hello')"]);
    assert_eq!(opts.cwd, Some("/tmp".to_string()));
    assert_eq!(opts.env.get("FOO").map(|s| s.as_str()), Some("bar"));
    assert_eq!(opts.env.get("BAZ").map(|s| s.as_str()), Some("qux"));
    assert!(opts.show_console);
}

#[test]
fn spawn_ipc_options_defaults() {
    let opts: SpawnIpcOptions =
        serde_json::from_value(serde_json::json!({ "command": "echo" })).unwrap();
    assert_eq!(opts.command, "echo");
    assert!(opts.args.is_empty());
    assert!(opts.cwd.is_none());
    assert!(opts.env.is_empty());
    assert!(!opts.show_console);
}

#[rstest]
#[case("python")]
#[case("node")]
#[case("echo")]
#[case("cargo")]
fn spawn_ipc_options_command_field(#[case] cmd: &str) {
    let opts: SpawnIpcOptions =
        serde_json::from_value(serde_json::json!({ "command": cmd })).unwrap();
    assert_eq!(opts.command, cmd);
}

#[test]
fn spawn_ipc_options_serialization_roundtrip() {
    let original = serde_json::json!({
        "command": "python",
        "args": ["script.py"],
        "cwd": null,
        "env": {},
        "showConsole": false
    });
    let opts: SpawnIpcOptions = serde_json::from_value(original).unwrap();
    let json = serde_json::to_string(&opts).unwrap();
    let restored: SpawnIpcOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.command, "python");
}

// ─────────────────────────────────────────────────────────────
// KillOptions / SendOptions / SendJsonOptions serde
// ─────────────────────────────────────────────────────────────

#[test]
fn kill_options_deserialization() {
    let opts: KillOptions =
        serde_json::from_value(serde_json::json!({ "pid": 1234 })).unwrap();
    assert_eq!(opts.pid, 1234);
}

#[test]
fn send_options_deserialization() {
    let opts: SendOptions =
        serde_json::from_value(serde_json::json!({ "pid": 42, "data": "hello\n" })).unwrap();
    assert_eq!(opts.pid, 42);
    assert_eq!(opts.data, "hello\n");
}

#[test]
fn send_json_options_deserialization() {
    let opts: SendJsonOptions = serde_json::from_value(serde_json::json!({
        "pid": 99,
        "data": { "action": "getData", "key": "value" }
    }))
    .unwrap();
    assert_eq!(opts.pid, 99);
    assert_eq!(opts.data["action"], "getData");
}

// ─────────────────────────────────────────────────────────────
// IpcMode
// ─────────────────────────────────────────────────────────────

#[test]
fn ipc_mode_pipe_eq() {
    assert_eq!(IpcMode::Pipe, IpcMode::Pipe);
}

#[test]
fn ipc_mode_channel_eq() {
    assert_eq!(IpcMode::Channel, IpcMode::Channel);
}

#[test]
fn ipc_mode_pipe_ne_channel() {
    assert_ne!(IpcMode::Pipe, IpcMode::Channel);
}

#[test]
fn ipc_mode_debug_output() {
    assert!(format!("{:?}", IpcMode::Pipe).contains("Pipe"));
    assert!(format!("{:?}", IpcMode::Channel).contains("Channel"));
}

#[test]
fn ipc_mode_clone() {
    let m = IpcMode::Pipe;
    let n = m;
    assert_eq!(m, n);
}

// ─────────────────────────────────────────────────────────────
// Concurrent operations (no panic)
// ─────────────────────────────────────────────────────────────

#[test]
fn concurrent_list_no_panic() {
    use std::sync::Arc;
    use std::thread;

    let plugin = Arc::new(ProcessPlugin::new());
    let scope = ScopeConfig::permissive();

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            thread::spawn(move || {
                let result = p.handle("list", serde_json::json!({}), &s).unwrap();
                assert!(result["processes"].is_array());
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[test]
fn concurrent_kill_nonexistent_no_panic() {
    use std::sync::Arc;
    use std::thread;

    let plugin = Arc::new(ProcessPlugin::new());
    let scope = ScopeConfig::permissive();

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let p = Arc::clone(&plugin);
            let s = scope.clone();
            thread::spawn(move || {
                let pid = 900_000 + i as u32;
                let result = p.handle("kill", serde_json::json!({ "pid": pid }), &s).unwrap();
                assert!(result["success"].as_bool().unwrap());
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

// ─────────────────────────────────────────────────────────────
// KillOptions: edge cases
// ─────────────────────────────────────────────────────────────

#[test]
fn kill_options_max_pid() {
    let opts: KillOptions =
        serde_json::from_value(serde_json::json!({ "pid": u32::MAX })).unwrap();
    assert_eq!(opts.pid, u32::MAX);
}

#[test]
fn kill_options_pid_zero() {
    let opts: KillOptions =
        serde_json::from_value(serde_json::json!({ "pid": 0 })).unwrap();
    assert_eq!(opts.pid, 0);
}

#[test]
fn kill_options_serde_roundtrip() {
    let original = KillOptions { pid: 42 };
    let json = serde_json::to_value(&original).unwrap();
    let restored: KillOptions = serde_json::from_value(json).unwrap();
    assert_eq!(restored.pid, original.pid);
}

#[test]
fn kill_options_debug() {
    let opts = KillOptions { pid: 100 };
    let debug = format!("{:?}", opts);
    assert!(debug.contains("100"));
}

// ─────────────────────────────────────────────────────────────
// SendOptions: edge cases
// ─────────────────────────────────────────────────────────────

#[test]
fn send_options_newline_data() {
    let opts: SendOptions =
        serde_json::from_value(serde_json::json!({ "pid": 10, "data": "\n" })).unwrap();
    assert_eq!(opts.data, "\n");
}

#[test]
fn send_options_empty_data() {
    let opts: SendOptions =
        serde_json::from_value(serde_json::json!({ "pid": 5, "data": "" })).unwrap();
    assert_eq!(opts.data, "");
}

#[test]
fn send_options_unicode_data() {
    let opts: SendOptions =
        serde_json::from_value(serde_json::json!({ "pid": 7, "data": "你好\nこんにちは" })).unwrap();
    assert!(opts.data.contains("你好"));
}

#[test]
fn send_options_serde_roundtrip() {
    let original = SendOptions { pid: 99, data: "hello".to_string() };
    let json = serde_json::to_value(&original).unwrap();
    let restored: SendOptions = serde_json::from_value(json).unwrap();
    assert_eq!(restored.pid, 99);
    assert_eq!(restored.data, "hello");
}

// ─────────────────────────────────────────────────────────────
// SendJsonOptions: edge cases
// ─────────────────────────────────────────────────────────────

#[test]
fn send_json_options_null_data() {
    let opts: SendJsonOptions =
        serde_json::from_value(serde_json::json!({ "pid": 1, "data": null })).unwrap();
    assert_eq!(opts.pid, 1);
    assert!(opts.data.is_null());
}

#[test]
fn send_json_options_array_data() {
    let opts: SendJsonOptions =
        serde_json::from_value(serde_json::json!({ "pid": 2, "data": [1, 2, 3] })).unwrap();
    assert!(opts.data.is_array());
}

#[test]
fn send_json_options_nested_object_data() {
    let opts: SendJsonOptions = serde_json::from_value(serde_json::json!({
        "pid": 3,
        "data": { "level1": { "level2": { "value": 42 } } }
    })).unwrap();
    assert_eq!(opts.data["level1"]["level2"]["value"], 42);
}

// ─────────────────────────────────────────────────────────────
// SpawnIpcOptions: additional coverage
// ─────────────────────────────────────────────────────────────

#[test]
fn spawn_ipc_options_many_env_vars() {
    let env: serde_json::Value = (0..10)
        .map(|i| (format!("VAR_{}", i), serde_json::json!(format!("val_{}", i))))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let json = serde_json::json!({ "command": "test", "env": env });
    let opts: SpawnIpcOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.env.len(), 10);
}

#[test]
fn spawn_ipc_options_show_console_true() {
    let opts: SpawnIpcOptions =
        serde_json::from_value(serde_json::json!({ "command": "cmd", "showConsole": true })).unwrap();
    assert!(opts.show_console);
}

#[test]
fn spawn_ipc_options_clone() {
    let opts: SpawnIpcOptions =
        serde_json::from_value(serde_json::json!({ "command": "test" })).unwrap();
    let cloned = opts.clone();
    assert_eq!(cloned.command, opts.command);
}

#[test]
fn spawn_ipc_options_debug() {
    let opts: SpawnIpcOptions =
        serde_json::from_value(serde_json::json!({ "command": "debug_test" })).unwrap();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("debug_test"));
}

// ─────────────────────────────────────────────────────────────
// IpcMode: serde + all variants
// ─────────────────────────────────────────────────────────────

#[test]
fn ipc_mode_copy_semantics() {
    let m = IpcMode::Channel;
    let n = m; // Copy
    assert_eq!(m, n);
}

#[rstest]
#[case(IpcMode::Pipe)]
#[case(IpcMode::Channel)]
fn ipc_mode_debug_non_empty(#[case] mode: IpcMode) {
    let debug = format!("{:?}", mode);
    assert!(!debug.is_empty());
}

// ─────────────────────────────────────────────────────────────
// Plugin: command consistency
// ─────────────────────────────────────────────────────────────

#[test]
fn process_plugin_commands_no_duplicates() {
    let plugin = ProcessPlugin::new();
    let cmds = plugin.commands();
    let mut unique = cmds.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), cmds.len());
}

#[test]
fn process_plugin_commands_all_non_empty() {
    let plugin = ProcessPlugin::new();
    for cmd in plugin.commands() {
        assert!(!cmd.is_empty());
    }
}
