//! Process Plugin with IPC Support
//!
//! Provides process spawning with bidirectional IPC communication.
//! Child processes can send logs and messages back to the parent.
//!
//! ## Commands
//!
//! - `spawn_ipc` - Spawn a process with IPC support (stdout/stderr capture)
//! - `kill` - Kill a managed process by PID
//! - `send` - Send a message to a managed process via stdin
//! - `list` - List all managed processes
//!
//! ## Events (emitted to frontend)
//!
//! - `process:stdout` - { pid, data } - stdout output from child
//! - `process:stderr` - { pid, data } - stderr output from child
//! - `process:exit` - { pid, code } - child process exited
//!
//! ## Integration with PluginRouter
//!
//! When created via `PluginRouter::new()`, the ProcessPlugin automatically
//! shares the router's event callback. Events are emitted to the frontend
//! when the callback is set via `PluginRouter::set_event_callback()`.
//!
//! ## Example
//!
//! ```javascript
//! // Spawn with IPC
//! const { pid } = await auroraview.invoke("plugin:process|spawn_ipc", {
//!     command: "python",
//!     args: ["script.py"],
//!     cwd: "/path/to/dir"
//! });
//!
//! // Listen for output
//! auroraview.on("process:stdout", ({ pid, data }) => {
//!     console.log(`[${pid}] ${data}`);
//! });
//!
//! // Send input to process
//! await auroraview.invoke("plugin:process|send", { pid, data: "hello\n" });
//!
//! // Kill process
//! await auroraview.invoke("plugin:process|kill", { pid });
//! ```

use crate::{PluginError, PluginEventCallback, PluginHandler, PluginResult, ScopeConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Callback type for process events (deprecated, use PluginEventCallback)
pub type ProcessEventCallback = PluginEventCallback;

/// Type alias for the process registry to reduce type complexity
type ProcessRegistry = Arc<RwLock<HashMap<u32, Arc<Mutex<ManagedProcess>>>>>;

/// Managed process info
struct ManagedProcess {
    /// Child process handle
    child: Child,
    /// Stdin writer (if available)
    stdin: Option<std::process::ChildStdin>,
}

/// Process plugin with IPC support
pub struct ProcessPlugin {
    name: String,
    /// Managed processes by PID
    processes: ProcessRegistry,
    /// Event callback for emitting events to frontend (shared with PluginRouter)
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
    /// Shutdown flag to stop background threads from emitting events
    shutdown: Arc<AtomicBool>,
}

impl ProcessPlugin {
    /// Create a new process plugin
    pub fn new() -> Self {
        Self {
            name: "process".to_string(),
            processes: Arc::new(RwLock::new(HashMap::new())),
            event_callback: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create a new process plugin with a shared event callback
    ///
    /// This is used by PluginRouter to share its event callback with ProcessPlugin.
    pub fn with_event_callback(callback: Arc<RwLock<Option<PluginEventCallback>>>) -> Self {
        Self {
            name: "process".to_string(),
            processes: Arc::new(RwLock::new(HashMap::new())),
            event_callback: callback,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the event callback for emitting events to frontend
    pub fn set_event_callback(&self, callback: PluginEventCallback) {
        let mut cb = self.event_callback.write().unwrap();
        *cb = Some(callback);
    }

    /// Spawn a process with IPC support
    fn spawn_ipc(&self, opts: SpawnIpcOptions, scope: &ScopeConfig) -> PluginResult<Value> {
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
        cmd.stdin(Stdio::piped());

        // Set working directory
        if let Some(cwd) = &opts.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &opts.env {
            cmd.env(key, value);
        }

        // Force unbuffered output for Python processes
        // This ensures stdout/stderr are flushed immediately, not line-buffered
        // when connected to a pipe. Critical for real-time IPC.
        cmd.env("PYTHONUNBUFFERED", "1");

        // In packed mode, inherit AURORAVIEW_PYTHON_PATH to PYTHONPATH
        // This allows spawned Python processes to find bundled modules
        if let Ok(python_path) = std::env::var("AURORAVIEW_PYTHON_PATH") {
            // Merge with existing PYTHONPATH if any
            let separator = if cfg!(windows) { ";" } else { ":" };
            let existing = std::env::var("PYTHONPATH").unwrap_or_default();
            let merged = if existing.is_empty() {
                python_path
            } else {
                format!("{}{}{}", python_path, separator, existing)
            };
            cmd.env("PYTHONPATH", merged);
        }

        // Windows: hide console window unless requested
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const CREATE_NEW_CONSOLE: u32 = 0x00000010;

            if opts.show_console {
                cmd.creation_flags(CREATE_NEW_CONSOLE);
            } else {
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
        }

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| PluginError::shell_error(format!("Failed to spawn: {}", e)))?;

        let pid = child.id();

        // Take stdout/stderr for async reading
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        // Store managed process
        let managed = Arc::new(Mutex::new(ManagedProcess { child, stdin }));
        {
            let mut processes = self.processes.write().unwrap();
            processes.insert(pid, managed.clone());
        }

        // Spawn stdout reader thread
        if let Some(stdout) = stdout {
            let event_cb = self.event_callback.clone();
            let processes = self.processes.clone();
            let shutdown = self.shutdown.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    // Check shutdown flag before emitting
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    match line {
                        Ok(data) => {
                            if let Some(cb) = event_cb.read().unwrap().as_ref() {
                                cb(
                                    "process:stdout",
                                    serde_json::json!({
                                        "pid": pid,
                                        "data": data
                                    }),
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }
                // Process stdout closed, check if process exited (only if not shutting down)
                if !shutdown.load(Ordering::Relaxed) {
                    Self::check_exit(&processes, &event_cb, pid);
                }
            });
        }

        // Spawn stderr reader thread
        if let Some(stderr) = stderr {
            let event_cb = self.event_callback.clone();
            let shutdown = self.shutdown.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    // Check shutdown flag before emitting
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    match line {
                        Ok(data) => {
                            if let Some(cb) = event_cb.read().unwrap().as_ref() {
                                cb(
                                    "process:stderr",
                                    serde_json::json!({
                                        "pid": pid,
                                        "data": data
                                    }),
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Ok(serde_json::json!({
            "success": true,
            "pid": pid
        }))
    }

    /// Check if process exited and emit event
    fn check_exit(
        processes: &ProcessRegistry,
        event_cb: &Arc<RwLock<Option<ProcessEventCallback>>>,
        pid: u32,
    ) {
        let exit_code = {
            let procs = processes.read().unwrap();
            if let Some(proc) = procs.get(&pid) {
                let mut p = proc.lock().unwrap();
                p.child.try_wait().ok().flatten().map(|s| s.code())
            } else {
                None
            }
        };

        if let Some(code) = exit_code {
            // Remove from managed processes
            {
                let mut procs = processes.write().unwrap();
                procs.remove(&pid);
            }

            // Emit exit event
            if let Some(cb) = event_cb.read().unwrap().as_ref() {
                cb(
                    "process:exit",
                    serde_json::json!({
                        "pid": pid,
                        "code": code
                    }),
                );
            }
        }
    }

    /// Kill a managed process
    fn kill_process(&self, pid: u32) -> PluginResult<Value> {
        let proc = {
            let processes = self.processes.read().unwrap();
            processes.get(&pid).cloned()
        };

        match proc {
            Some(p) => {
                let mut managed = p.lock().unwrap();

                // Kill the process
                if let Err(e) = managed.child.kill() {
                    // Ignore "process already exited" errors
                    if e.kind() != std::io::ErrorKind::InvalidInput {
                        return Err(PluginError::shell_error(format!("Failed to kill: {}", e)));
                    }
                }

                // Wait for process to fully terminate (with timeout)
                // This ensures stdout/stderr pipes are closed
                let _ = managed.child.wait();

                // Remove from managed
                {
                    let mut processes = self.processes.write().unwrap();
                    processes.remove(&pid);
                }

                Ok(serde_json::json!({ "success": true }))
            }
            None => {
                // Process not found - might already be cleaned up, return success
                Ok(serde_json::json!({ "success": true, "already_exited": true }))
            }
        }
    }

    /// Kill all managed processes (for cleanup on shutdown)
    fn kill_all(&self) -> PluginResult<Value> {
        // Set shutdown flag first to stop background threads from emitting events
        // This prevents "EventLoopClosed" errors when WebView is closing
        self.shutdown.store(true, Ordering::SeqCst);

        let pids: Vec<u32> = {
            let processes = self.processes.read().unwrap();
            processes.keys().copied().collect()
        };

        let mut killed = 0;
        for pid in pids {
            if self.kill_process(pid).is_ok() {
                killed += 1;
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "killed": killed
        }))
    }

    /// Send data to process stdin
    fn send_to_process(&self, pid: u32, data: &str) -> PluginResult<Value> {
        let proc = {
            let processes = self.processes.read().unwrap();
            processes.get(&pid).cloned()
        };

        match proc {
            Some(p) => {
                let mut managed = p.lock().unwrap();
                if let Some(ref mut stdin) = managed.stdin {
                    stdin
                        .write_all(data.as_bytes())
                        .map_err(|e| PluginError::shell_error(format!("Failed to write: {}", e)))?;
                    stdin
                        .flush()
                        .map_err(|e| PluginError::shell_error(format!("Failed to flush: {}", e)))?;
                    Ok(serde_json::json!({ "success": true }))
                } else {
                    Err(PluginError::shell_error("Process stdin not available"))
                }
            }
            None => Err(PluginError::shell_error(format!(
                "Process {} not found",
                pid
            ))),
        }
    }

    /// List all managed processes
    fn list_processes(&self) -> PluginResult<Value> {
        let processes = self.processes.read().unwrap();
        let pids: Vec<u32> = processes.keys().copied().collect();
        Ok(serde_json::json!({
            "processes": pids
        }))
    }
}

impl Default for ProcessPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for spawning a process with IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnIpcOptions {
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
    pub env: HashMap<String, String>,
    /// Show console window (Windows only)
    #[serde(default)]
    pub show_console: bool,
}

/// Options for killing a process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KillOptions {
    /// Process ID
    pub pid: u32,
}

/// Options for sending data to a process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOptions {
    /// Process ID
    pub pid: u32,
    /// Data to send
    pub data: String,
}

impl PluginHandler for ProcessPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "spawn_ipc" => {
                let opts: SpawnIpcOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.spawn_ipc(opts, scope)
            }
            "kill" => {
                let opts: KillOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.kill_process(opts.pid)
            }
            "kill_all" => self.kill_all(),
            "send" => {
                let opts: SendOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.send_to_process(opts.pid, &opts.data)
            }
            "list" => self.list_processes(),
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec!["spawn_ipc", "kill", "kill_all", "send", "list"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
