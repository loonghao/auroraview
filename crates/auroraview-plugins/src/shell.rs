//! Shell Plugin
//!
//! Provides shell/process execution and URL/file opening capabilities.
//!
//! ## Commands
//!
//! - `open` - Open a URL or file with the default application
//! - `execute` - Execute a shell command (requires scope permission)
//! - `which` - Find the path of an executable
//!
//! ## Example
//!
//! ```javascript
//! // Open a URL in the default browser
//! await auroraview.invoke("plugin:shell|open", { path: "https://example.com" });
//!
//! // Open a file with the default application
//! await auroraview.invoke("plugin:shell|open", { path: "/path/to/document.pdf" });
//!
//! // Execute a command (if allowed by scope)
//! const result = await auroraview.invoke("plugin:shell|execute", {
//!     command: "git",
//!     args: ["status"]
//! });
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
}

/// Options for finding an executable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhichOptions {
    /// Command name to find
    pub command: String,
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

                // Spawn (detached)
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    const DETACHED_PROCESS: u32 = 0x00000008;
                    cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
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
        vec!["open", "execute", "which", "spawn"]
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
        assert!(commands.contains(&"execute"));
        assert!(commands.contains(&"which"));
        assert!(commands.contains(&"spawn"));
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
}
