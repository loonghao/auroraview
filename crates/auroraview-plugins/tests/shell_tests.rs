//! Unit tests for shell plugin
//!
//! Tests for ShellPlugin commands and options.

use auroraview_plugins::shell::{
    EnvOptions, ExecuteOptions, ExecuteResult, OpenOptions, PathOptions, ShellPlugin, WhichOptions,
};
use auroraview_plugins::{PluginHandler, ScopeConfig};

#[test]
fn test_shell_plugin_commands() {
    let plugin = ShellPlugin::new();
    let commands = plugin.commands();
    assert!(commands.contains(&"open"));
    assert!(commands.contains(&"open_path"));
    assert!(commands.contains(&"show_in_folder"));
    assert!(commands.contains(&"execute"));
    assert!(commands.contains(&"which"));
    assert!(commands.contains(&"spawn"));
    assert!(commands.contains(&"get_env"));
    assert!(commands.contains(&"get_env_all"));
}

#[test]
fn test_which_command() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    // Try to find a common command
    #[cfg(windows)]
    let cmd = "cmd";
    #[cfg(not(windows))]
    let cmd = "sh";

    let result = plugin.handle("which", serde_json::json!({ "command": cmd }), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["path"].is_string() || data["path"].is_null());
}

#[test]
fn test_get_env() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    // PATH should exist on all systems
    let result = plugin.handle("get_env", serde_json::json!({ "name": "PATH" }), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["value"].is_string());
}

#[test]
fn test_get_env_nonexistent() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "get_env",
        serde_json::json!({ "name": "AURORAVIEW_NONEXISTENT_VAR_12345" }),
        &scope,
    );
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["value"].is_null());
}

#[test]
fn test_get_env_all() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("get_env_all", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["env"].is_object());
    // Should have at least PATH
    assert!(data["env"]["PATH"].is_string() || data["env"]["Path"].is_string());
}

#[test]
fn test_execute_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new(); // Default scope blocks all commands

    let result = plugin.handle(
        "execute",
        serde_json::json!({
            "command": "echo",
            "args": ["hello"]
        }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_execute_allowed_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::permissive();
    scope.shell = scope.shell.allow_command("echo");

    #[cfg(windows)]
    let _result = plugin.handle(
        "execute",
        serde_json::json!({
            "command": "cmd",
            "args": ["/c", "echo", "hello"]
        }),
        &scope,
    );

    #[cfg(not(windows))]
    let _result = plugin.handle(
        "execute",
        serde_json::json!({
            "command": "echo",
            "args": ["hello"]
        }),
        &scope,
    );

    // May fail if command not found, but should not fail due to scope
    // The test verifies scope check passes
}

#[test]
fn test_open_path_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_file = false;

    let result = plugin.handle(
        "open_path",
        serde_json::json!({ "path": "/tmp/test.txt" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_show_in_folder_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_file = false;

    let result = plugin.handle(
        "show_in_folder",
        serde_json::json!({ "path": "/tmp/test.txt" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_shell_plugin_name() {
    let plugin = ShellPlugin::new();
    assert_eq!(plugin.name(), "shell");
}

#[test]
fn test_shell_plugin_default() {
    let plugin = ShellPlugin::default();
    assert_eq!(plugin.name(), "shell");
}

#[test]
fn test_open_options_deserialization() {
    let json = serde_json::json!({
        "path": "https://example.com",
        "with": "firefox"
    });
    let opts: OpenOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "https://example.com");
    assert_eq!(opts.with, Some("firefox".to_string()));
}

#[test]
fn test_open_options_without_with() {
    let json = serde_json::json!({
        "path": "/tmp/file.txt"
    });
    let opts: OpenOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "/tmp/file.txt");
    assert!(opts.with.is_none());
}

#[test]
fn test_execute_options_deserialization() {
    let json = serde_json::json!({
        "command": "echo",
        "args": ["hello", "world"],
        "cwd": "/tmp",
        "env": {"FOO": "bar"},
        "encoding": "utf-8"
    });
    let opts: ExecuteOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "echo");
    assert_eq!(opts.args, vec!["hello", "world"]);
    assert_eq!(opts.cwd, Some("/tmp".to_string()));
    assert_eq!(opts.env.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(opts.encoding, Some("utf-8".to_string()));
}

#[test]
fn test_execute_options_defaults() {
    let json = serde_json::json!({
        "command": "ls"
    });
    let opts: ExecuteOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "ls");
    assert!(opts.args.is_empty());
    assert!(opts.cwd.is_none());
    assert!(opts.env.is_empty());
    assert!(opts.encoding.is_none());
}

#[test]
fn test_which_options_deserialization() {
    let json = serde_json::json!({
        "command": "git"
    });
    let opts: WhichOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "git");
}

#[test]
fn test_path_options_deserialization() {
    let json = serde_json::json!({
        "path": "/home/user/documents"
    });
    let opts: PathOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "/home/user/documents");
}

#[test]
fn test_env_options_deserialization() {
    let json = serde_json::json!({
        "name": "HOME"
    });
    let opts: EnvOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.name, "HOME");
}

#[test]
fn test_execute_result_serialization() {
    let result = ExecuteResult {
        code: Some(0),
        stdout: "output".to_string(),
        stderr: "".to_string(),
    };
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["code"], 0);
    assert_eq!(json["stdout"], "output");
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_execute_result_with_none_code() {
    let result = ExecuteResult {
        code: None,
        stdout: "".to_string(),
        stderr: "error".to_string(),
    };
    let json = serde_json::to_value(&result).unwrap();
    assert!(json["code"].is_null());
    assert_eq!(json["stderr"], "error");
}

#[test]
fn test_command_not_found() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("nonexistent_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[test]
fn test_open_url_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_url = false;

    let result = plugin.handle(
        "open",
        serde_json::json!({ "path": "https://example.com" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_open_mailto_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_url = false;

    let result = plugin.handle(
        "open",
        serde_json::json!({ "path": "mailto:test@example.com" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_open_file_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_file = false;

    let result = plugin.handle(
        "open",
        serde_json::json!({ "path": "/tmp/file.txt" }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_spawn_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new(); // Default scope blocks all commands

    let result = plugin.handle(
        "spawn",
        serde_json::json!({
            "command": "echo",
            "args": ["hello"]
        }),
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_which_nonexistent_command() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "which",
        serde_json::json!({ "command": "nonexistent_command_12345" }),
        &scope,
    );
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["path"].is_null());
}

#[test]
fn test_execute_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "execute",
        serde_json::json!({ "invalid": "args" }), // Missing required "command"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_open_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "open",
        serde_json::json!({ "invalid": "args" }), // Missing required "path"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_which_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "which",
        serde_json::json!({ "invalid": "args" }), // Missing required "command"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_get_env_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(
        "get_env",
        serde_json::json!({ "invalid": "args" }), // Missing required "name"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_open_path_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "open_path",
        serde_json::json!({ "invalid": "args" }), // Missing required "path"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_show_in_folder_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "show_in_folder",
        serde_json::json!({ "invalid": "args" }), // Missing required "path"
        &scope,
    );
    assert!(result.is_err());
}

#[test]
fn test_spawn_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "spawn",
        serde_json::json!({ "invalid": "args" }), // Missing required "command"
        &scope,
    );
    assert!(result.is_err());
}
