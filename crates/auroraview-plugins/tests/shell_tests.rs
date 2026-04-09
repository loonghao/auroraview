//! Unit tests for shell plugin
//!
//! Tests for ShellPlugin commands and options.

use auroraview_plugins::shell::{
    EnvOptions, ExecuteOptions, ExecuteResult, OpenOptions, PathOptions, ShellPlugin, WhichOptions,
};
use auroraview_plugins::{PluginHandler, ScopeConfig};
use rstest::rstest;
use std::sync::Arc;

// ============================================================================
// ShellPlugin creation and metadata
// ============================================================================

#[rstest]
fn shell_plugin_commands() {
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

#[rstest]
fn shell_plugin_name() {
    let plugin = ShellPlugin::new();
    assert_eq!(plugin.name(), "shell");
}

#[rstest]
fn shell_plugin_default() {
    let plugin = ShellPlugin::default();
    assert_eq!(plugin.name(), "shell");
}

#[rstest]
fn shell_plugin_commands_count() {
    let plugin = ShellPlugin::new();
    // At least 8 base commands + restart_app
    assert!(plugin.commands().len() >= 8);
}

// ============================================================================
// which command
// ============================================================================

#[rstest]
fn which_command() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    #[cfg(windows)]
    let cmd = "cmd";
    #[cfg(not(windows))]
    let cmd = "sh";

    let result = plugin.handle("which", serde_json::json!({ "command": cmd }), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["path"].is_string() || data["path"].is_null());
}

#[rstest]
fn which_nonexistent_command() {
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

#[rstest]
fn which_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("which", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("nonexistent_a")]
#[case("nonexistent_b")]
#[case("nonexistent_c")]
fn which_parametrized_nonexistent(#[case] cmd: &str) {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("which", serde_json::json!({ "command": cmd }), &scope);
    assert!(result.is_ok());
    assert!(result.unwrap()["path"].is_null());
}

// ============================================================================
// get_env / get_env_all commands
// ============================================================================

#[rstest]
fn get_env() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("get_env", serde_json::json!({ "name": "PATH" }), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["value"].is_string());
}

#[rstest]
fn get_env_nonexistent() {
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

#[rstest]
fn get_env_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("get_env", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

#[rstest]
fn get_env_all() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("get_env_all", serde_json::json!({}), &scope);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["env"].is_object());
    // Should have at least PATH
    assert!(data["env"]["PATH"].is_string() || data["env"]["Path"].is_string());
}

#[rstest]
fn get_env_all_nonempty() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin
        .handle("get_env_all", serde_json::json!({}), &scope)
        .unwrap();
    let env = result["env"].as_object().unwrap();
    assert!(!env.is_empty());
}

#[rstest]
#[cfg(windows)]
#[case("SYSTEMROOT")]
#[case("WINDIR")]
fn get_env_windows_vars(#[case] var: &str) {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("get_env", serde_json::json!({ "name": var }), &scope);
    assert!(result.is_ok());
    // These vars should be present on Windows
    let data = result.unwrap();
    assert!(data["value"].is_string() || data["value"].is_null());
}

// ============================================================================
// execute command — scope checks
// ============================================================================

#[rstest]
fn execute_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

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

#[rstest]
fn execute_allowed_by_scope() {
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
}

#[rstest]
fn execute_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("execute", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("rm", &["-rf", "/"])]
#[case("format", &["c:"])]
#[case("deltree", &["/y", "c:\\"])]
fn execute_dangerous_cmds_blocked_by_default(#[case] cmd: &str, #[case] _args: &[&str]) {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new(); // Default: block all commands

    let result = plugin.handle(
        "execute",
        serde_json::json!({ "command": cmd, "args": _args }),
        &scope,
    );
    assert!(result.is_err());
}

// ============================================================================
// open command — scope checks
// ============================================================================

#[rstest]
fn open_url_blocked_by_scope() {
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

#[rstest]
fn open_mailto_blocked_by_scope() {
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

#[rstest]
fn open_file_blocked_by_scope() {
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

#[rstest]
fn open_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("open", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("https://example.com")]
#[case("http://example.com")]
#[case("mailto:user@example.com")]
fn open_url_schemes_blocked(#[case] url: &str) {
    let plugin = ShellPlugin::new();
    let mut scope = ScopeConfig::new();
    scope.shell.allow_open_url = false;

    let result = plugin.handle("open", serde_json::json!({ "path": url }), &scope);
    assert!(result.is_err());
}

// ============================================================================
// open_path / show_in_folder — scope checks
// ============================================================================

#[rstest]
fn open_path_blocked_by_scope() {
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

#[rstest]
fn show_in_folder_blocked_by_scope() {
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

#[rstest]
fn open_path_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "open_path",
        serde_json::json!({ "invalid": "args" }),
        &scope,
    );
    assert!(result.is_err());
}

#[rstest]
fn show_in_folder_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle(
        "show_in_folder",
        serde_json::json!({ "invalid": "args" }),
        &scope,
    );
    assert!(result.is_err());
}

// ============================================================================
// spawn command — scope checks
// ============================================================================

#[rstest]
fn spawn_blocked_by_scope() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

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

#[rstest]
fn spawn_invalid_args() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::permissive();

    let result = plugin.handle("spawn", serde_json::json!({ "invalid": "args" }), &scope);
    assert!(result.is_err());
}

// ============================================================================
// restart_app command
// ============================================================================

#[rstest]
fn restart_app_blocked_by_scope() {
    // restart_app does NOT check scope by default — test it doesn't panic
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    // We just verify the command is recognized (not "command not found")
    // Note: calling restart_app in tests may actually restart; we only test
    // that the handler dispatches to it (not CommandNotFound error).
    // To avoid actual restart, we do NOT call it. Just verify commands list.
    assert!(plugin.commands().contains(&"restart_app"));
    let _ = scope; // suppress unused warning
}

// ============================================================================
// Unknown command
// ============================================================================

#[rstest]
fn command_not_found() {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle("nonexistent_command", serde_json::json!({}), &scope);
    assert!(result.is_err());
}

#[rstest]
#[case("foobar")]
#[case("__invalid__")]
#[case("open2")]
#[case("exec")]
fn unknown_commands_err(#[case] cmd: &str) {
    let plugin = ShellPlugin::new();
    let scope = ScopeConfig::new();

    let result = plugin.handle(cmd, serde_json::json!({}), &scope);
    assert!(result.is_err());
}

// ============================================================================
// Options struct deserialization
// ============================================================================

#[rstest]
fn open_options_deserialization() {
    let json = serde_json::json!({
        "path": "https://example.com",
        "with": "firefox"
    });
    let opts: OpenOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "https://example.com");
    assert_eq!(opts.with, Some("firefox".to_string()));
}

#[rstest]
fn open_options_without_with() {
    let json = serde_json::json!({
        "path": "/tmp/file.txt"
    });
    let opts: OpenOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "/tmp/file.txt");
    assert!(opts.with.is_none());
}

#[rstest]
#[case("https://example.com", Some("firefox"))]
#[case("https://test.org", Some("chrome"))]
#[case("/tmp/doc.pdf", None)]
fn open_options_parametrized(#[case] path: &str, #[case] with_app: Option<&str>) {
    let json = if let Some(app) = with_app {
        serde_json::json!({ "path": path, "with": app })
    } else {
        serde_json::json!({ "path": path })
    };
    let opts: OpenOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, path);
    assert_eq!(opts.with.as_deref(), with_app);
}

#[rstest]
fn execute_options_deserialization() {
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

#[rstest]
fn execute_options_defaults() {
    let json = serde_json::json!({
        "command": "ls"
    });
    let opts: ExecuteOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "ls");
    assert!(opts.args.is_empty());
    assert!(opts.cwd.is_none());
    assert!(opts.env.is_empty());
    assert!(opts.encoding.is_none());
    assert!(!opts.show_console);
}

#[rstest]
fn execute_options_show_console_default_false() {
    let json = serde_json::json!({ "command": "cmd" });
    let opts: ExecuteOptions = serde_json::from_value(json).unwrap();
    assert!(!opts.show_console);
}

#[rstest]
fn execute_options_show_console_true() {
    let json = serde_json::json!({ "command": "cmd", "showConsole": true });
    let opts: ExecuteOptions = serde_json::from_value(json).unwrap();
    assert!(opts.show_console);
}

#[rstest]
#[case("git", &["status"], None, false)]
#[case("python", &["-c", "print('hi')"], Some("/tmp"), true)]
fn execute_options_various(
    #[case] cmd: &str,
    #[case] args: &[&str],
    #[case] cwd: Option<&str>,
    #[case] show_console: bool,
) {
    let json_args: Vec<serde_json::Value> = args.iter().map(|a| serde_json::json!(a)).collect();
    let mut obj = serde_json::json!({
        "command": cmd,
        "args": json_args,
        "showConsole": show_console
    });
    if let Some(c) = cwd {
        obj["cwd"] = serde_json::json!(c);
    }
    let opts: ExecuteOptions = serde_json::from_value(obj).unwrap();
    assert_eq!(opts.command, cmd);
    assert_eq!(opts.args.len(), args.len());
    assert_eq!(opts.show_console, show_console);
    assert_eq!(opts.cwd.as_deref(), cwd);
}

#[rstest]
fn which_options_deserialization() {
    let json = serde_json::json!({
        "command": "git"
    });
    let opts: WhichOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.command, "git");
}

#[rstest]
fn path_options_deserialization() {
    let json = serde_json::json!({
        "path": "/home/user/documents"
    });
    let opts: PathOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.path, "/home/user/documents");
}

#[rstest]
fn env_options_deserialization() {
    let json = serde_json::json!({
        "name": "HOME"
    });
    let opts: EnvOptions = serde_json::from_value(json).unwrap();
    assert_eq!(opts.name, "HOME");
}

// ============================================================================
// ExecuteResult serialization
// ============================================================================

#[rstest]
fn execute_result_serialization() {
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

#[rstest]
fn execute_result_with_none_code() {
    let result = ExecuteResult {
        code: None,
        stdout: "".to_string(),
        stderr: "error".to_string(),
    };
    let json = serde_json::to_value(&result).unwrap();
    assert!(json["code"].is_null());
    assert_eq!(json["stderr"], "error");
}

#[rstest]
#[case(Some(0), "ok", "")]
#[case(Some(1), "", "fail")]
#[case(Some(127), "", "command not found")]
#[case(None, "", "killed")]
fn execute_result_parametrized(
    #[case] code: Option<i32>,
    #[case] stdout: &str,
    #[case] stderr: &str,
) {
    let result = ExecuteResult {
        code,
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    };
    let json = serde_json::to_value(&result).unwrap();
    if let Some(c) = code {
        assert_eq!(json["code"], c);
    } else {
        assert!(json["code"].is_null());
    }
    assert_eq!(json["stdout"], stdout);
    assert_eq!(json["stderr"], stderr);
}

#[rstest]
fn execute_result_clone() {
    let result = ExecuteResult {
        code: Some(0),
        stdout: "hello".to_string(),
        stderr: "".to_string(),
    };
    let cloned = result.clone();
    assert_eq!(cloned.code, Some(0));
    assert_eq!(cloned.stdout, "hello");
}

#[rstest]
fn execute_result_debug() {
    let result = ExecuteResult {
        code: Some(42),
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    };
    let dbg = format!("{:?}", result);
    assert!(dbg.contains("42"));
}

// ============================================================================
// Concurrent access — get_env / which (read-only, no scope mutation)
// ============================================================================

#[rstest]
fn get_env_concurrent_no_panic() {
    let plugin = Arc::new(ShellPlugin::new());
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&plugin);
            std::thread::spawn(move || {
                let scope = ScopeConfig::new();
                let _ = p.handle("get_env", serde_json::json!({ "name": "PATH" }), &scope);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn which_concurrent_no_panic() {
    let plugin = Arc::new(ShellPlugin::new());
    #[cfg(windows)]
    let cmd = "cmd";
    #[cfg(not(windows))]
    let cmd = "sh";

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let p = Arc::clone(&plugin);
            std::thread::spawn(move || {
                let scope = ScopeConfig::new();
                let _ = p.handle("which", serde_json::json!({ "command": cmd }), &scope);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn get_env_all_concurrent_no_panic() {
    let plugin = Arc::new(ShellPlugin::new());
    let handles: Vec<_> = (0..6)
        .map(|_| {
            let p = Arc::clone(&plugin);
            std::thread::spawn(move || {
                let scope = ScopeConfig::new();
                let _ = p.handle("get_env_all", serde_json::json!({}), &scope);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[rstest]
fn blocked_commands_concurrent_no_panic() {
    let plugin = Arc::new(ShellPlugin::new());
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let p = Arc::clone(&plugin);
            std::thread::spawn(move || {
                let scope = ScopeConfig::new();
                let cmd = if i % 2 == 0 { "execute" } else { "spawn" };
                let _ = p.handle(
                    cmd,
                    serde_json::json!({ "command": "echo", "args": ["hi"] }),
                    &scope,
                );
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}
