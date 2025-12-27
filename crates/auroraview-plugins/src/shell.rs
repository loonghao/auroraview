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

use auroraview_plugin_core::{PluginError, PluginHandler, PluginResult, ScopeConfig};
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
            "restart_app" => {
                // Restart the current application
                // This spawns a new instance and exits the current one

                // Check for AURORAVIEW_RESTART_CMD environment variable first
                // This allows Python scripts to specify their restart command
                let restart_cmd = std::env::var("AURORAVIEW_RESTART_CMD").ok();

                let mut cmd = if let Some(cmd_str) = restart_cmd {
                    // Use the provided restart command (e.g., "python gallery/main.py")
                    #[cfg(windows)]
                    let shell = "cmd";
                    #[cfg(not(windows))]
                    let shell = "sh";

                    #[cfg(windows)]
                    let shell_arg = "/C";
                    #[cfg(not(windows))]
                    let shell_arg = "-c";

                    let mut command = Command::new(shell);
                    command.arg(shell_arg).arg(&cmd_str);
                    command
                } else {
                    // Default: restart the current executable with same args
                    let exe_path = std::env::current_exe().map_err(|e| {
                        PluginError::shell_error(format!("Failed to get current executable: {}", e))
                    })?;

                    let args: Vec<String> = std::env::args().skip(1).collect();

                    let mut command = Command::new(&exe_path);
                    command.args(&args);
                    command
                };

                // Configure process
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
                cmd.stdin(Stdio::null());

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const DETACHED_PROCESS: u32 = 0x00000008;
                    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
                    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
                }

                cmd.spawn().map_err(|e| {
                    PluginError::shell_error(format!("Failed to restart application: {}", e))
                })?;

                // Exit current instance after a short delay
                // This gives time for the response to be sent
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    std::process::exit(0);
                });

                Ok(serde_json::json!({
                    "success": true,
                    "message": "Application is restarting..."
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
            "restart_app",
        ]
    }
}
