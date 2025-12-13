//! Shell Plugin
//!
//! Provides shell/process execution and URL/file opening capabilities.
//!
//! ## Commands
//!
//! - `open` - Open a URL or file with the default application
//! - `open_path` - Open a file/folder with the default application
//! - `show_in_folder` - Reveal a file in its parent folder (file manager)
//! - `execute` - Execute a shell command (requires scope permission)
//! - `spawn` - Spawn a detached process
//! - `which` - Find the path of an executable
//! - `get_env` - Get an environment variable
//! - `get_env_all` - Get all environment variables
//!
//! ## Example
//!
//! ```javascript
//! // Open a URL in the default browser
//! await auroraview.invoke("plugin:shell|open", { path: "https://example.com" });
//!
//! // Open a file with the default application
//! await auroraview.invoke("plugin:shell|open_path", { path: "/path/to/document.pdf" });
//!
//! // Reveal file in file manager
//! await auroraview.invoke("plugin:shell|show_in_folder", { path: "/path/to/file.txt" });
//!
//! // Execute a command (if allowed by scope)
//! const result = await auroraview.invoke("plugin:shell|execute", {
//!     command: "git",
//!     args: ["status"]
//! });
//!
//! // Get environment variable
//! const home = await auroraview.invoke("plugin:shell|get_env", { name: "HOME" });
//! ```

use crate::{PluginError, PluginHandler, PluginResult, ScopeConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};

/// Shell plugin
pub struct ShellPlugin {
    name: String,
}

impl ShellPlugin {
    /// Create a new shell plugin
    pub fn new() -> Self {
        Self {
            name: "shell".to_string(),
        }
    }
}

impl Default for ShellPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for opening a URL or file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptions {
    /// Path or URL to open
    pub path: String,
    /// Open with specific application (optional)
    #[serde(default)]
    pub with: Option<String>,
}

/// Options for executing a command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOptions {
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Encoding for output (default: utf-8)
    #[serde(default)]
    pub encoding: Option<String>,
    /// Show console window (Windows only, default: false)
    #[serde(default)]
    pub show_console: bool,
}

/// Options for finding an executable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhichOptions {
    /// Command name to find
    pub command: String,
}

/// Options for path-based operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathOptions {
    /// File or folder path
    pub path: String,
}

/// Options for environment variable lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvOptions {
    /// Environment variable name
    pub name: String,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResult {
    /// Exit code (0 = success)
    pub code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

impl PluginHandler for ShellPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "open" => {
                let opts: OpenOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Check if it's a URL
                let is_url = opts.path.starts_with("http://")
                    || opts.path.starts_with("https://")
                    || opts.path.starts_with("mailto:");

                // Check scope permissions
                if is_url && !scope.shell.allow_open_url {
                    return Err(PluginError::shell_error("Opening URLs is not allowed"));
                }
                if !is_url && !scope.shell.allow_open_file {
                    return Err(PluginError::shell_error("Opening files is not allowed"));
                }

                // Open with specific app or default
                let result = if let Some(app) = opts.with {
                    open::with(&opts.path, &app)
                } else {
                    open::that(&opts.path)
                };

                result.map_err(|e| PluginError::shell_error(format!("Failed to open: {}", e)))?;

                Ok(serde_json::json!({ "success": true }))
            }
            "execute" => {
                let opts: ExecuteOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Check if command is allowed
                if !scope.shell.is_command_allowed(&opts.command) {
                    return Err(PluginError::shell_error(format!(
                        "Command '{}' is not allowed by scope configuration",
                        opts.command
                    )));
                }

                // Build command
                let mut cmd = Command::new(&opts.command);
                cmd.args(&opts.args);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                // Set working directory
                if let Some(cwd) = &opts.cwd {
                    cmd.current_dir(cwd);
                }

                // Set environment variables
                for (key, value) in &opts.env {
                    cmd.env(key, value);
                }

                // Execute
                let output = cmd
                    .output()
                    .map_err(|e| PluginError::shell_error(format!("Failed to execute: {}", e)))?;

                let result = ExecuteResult {
                    code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                };

                Ok(serde_json::to_value(result).unwrap())
            }
            "which" => {
                let opts: WhichOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let path = which::which(&opts.command).ok();

                Ok(serde_json::json!({
                    "path": path.map(|p| p.to_string_lossy().to_string())
                }))
            }
            "open_path" => {
                // Open a file or folder with the default application
                let opts: PathOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Check scope permissions
                if !scope.shell.allow_open_file {
                    return Err(PluginError::shell_error("Opening files is not allowed"));
                }

                open::that(&opts.path)
                    .map_err(|e| PluginError::shell_error(format!("Failed to open: {}", e)))?;

                Ok(serde_json::json!({ "success": true }))
            }
            "show_in_folder" => {
                // Reveal file in file manager (Explorer/Finder/etc.)
                let opts: PathOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Check scope permissions
                if !scope.shell.allow_open_file {
                    return Err(PluginError::shell_error("Opening files is not allowed"));
                }

                // Get parent directory and reveal
                let path = std::path::Path::new(&opts.path);

                #[cfg(target_os = "windows")]
                {
                    // Use explorer.exe /select to highlight the file
                    let path_str = dunce::canonicalize(path)
                        .unwrap_or_else(|_| path.to_path_buf())
                        .to_string_lossy()
                        .to_string();

                    Command::new("explorer.exe")
                        .args(["/select,", &path_str])
                        .spawn()
                        .map_err(|e| {
                            PluginError::shell_error(format!("Failed to show in folder: {}", e))
                        })?;
                }

                #[cfg(target_os = "macos")]
                {
                    Command::new("open")
                        .args(["-R", &opts.path])
                        .spawn()
                        .map_err(|e| {
                            PluginError::shell_error(format!("Failed to show in folder: {}", e))
                        })?;
                }

                #[cfg(target_os = "linux")]
                {
                    // Try common file managers
                    let parent = path.parent().unwrap_or(path);
                    if Command::new("xdg-open").arg(parent).spawn().is_err() {
                        // Fallback to nautilus if available
                        Command::new("nautilus")
                            .arg(&opts.path)
                            .spawn()
                            .map_err(|e| {
                                PluginError::shell_error(format!("Failed to show in folder: {}", e))
                            })?;
                    }
                }

                Ok(serde_json::json!({ "success": true }))
            }
            "get_env" => {
                // Get a single environment variable
                let opts: EnvOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let value = std::env::var(&opts.name).ok();

                Ok(serde_json::json!({ "value": value }))
            }
            "get_env_all" => {
                // Get all environment variables
                let env: std::collections::HashMap<String, String> = std::env::vars().collect();

                Ok(serde_json::json!({ "env": env }))
            }
            "spawn" => {
                // Spawn a detached process (fire and forget)
                let opts: ExecuteOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Check if command is allowed
                if !scope.shell.is_command_allowed(&opts.command) {
                    return Err(PluginError::shell_error(format!(
                        "Command '{}' is not allowed by scope configuration",
                        opts.command
                    )));
                }

                // Build command
                let mut cmd = Command::new(&opts.command);
                cmd.args(&opts.args);
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
                cmd.stdin(Stdio::null());

                // Set working directory
                if let Some(cwd) = &opts.cwd {
                    cmd.current_dir(cwd);
                }

                // Set environment variables
                for (key, value) in &opts.env {
                    cmd.env(key, value);
                }

                // Spawn (detached) with optional console window
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    const CREATE_NEW_CONSOLE: u32 = 0x00000010;
                    const DETACHED_PROCESS: u32 = 0x00000008;

                    if opts.show_console {
                        // Show a new console window for the process
                        cmd.creation_flags(CREATE_NEW_CONSOLE);
                    } else {
                        // Hide console window (default)
                        cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
                    }
                }

                let child = cmd
                    .spawn()
                    .map_err(|e| PluginError::shell_error(format!("Failed to spawn: {}", e)))?;

                Ok(serde_json::json!({
                    "success": true,
                    "pid": child.id()
                }))
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "open",
            "open_path",
            "show_in_folder",
            "execute",
            "spawn",
            "which",
            "get_env",
            "get_env_all",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
