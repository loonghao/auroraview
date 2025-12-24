//! Process Plugin with IPC Support (powered by ipckit)
//!
//! Provides process spawning with bidirectional IPC communication.
//! Child processes can send logs and messages back to the parent.
//!
//! ## IPC Modes
//!
//! - **Pipe Mode** (`spawn_ipc`): Uses stdout/stderr pipes for communication.
//!   Simple and compatible with any process. Best for log streaming.
//!
//! - **Channel Mode** (`spawn_ipc_channel`): Uses ipckit's LocalSocket for
//!   high-performance bidirectional JSON-RPC communication. Best for
//!   structured data exchange and real-time messaging.
//!
//! ## Commands
//!
//! - `spawn_ipc` - Spawn a process with pipe-based IPC (stdout/stderr capture)
//! - `spawn_ipc_channel` - Spawn a process with ipckit LocalSocket IPC
//! - `kill` - Kill a managed process by PID
//! - `send` - Send a message to a managed process via stdin (pipe mode)
//! - `send_json` - Send JSON message via ipckit channel (channel mode)
//! - `list` - List all managed processes
//!
//! ## Events (emitted to frontend)
//!
//! - `process:stdout` - { pid, data } - stdout output from child
//! - `process:stderr` - { pid, data } - stderr output from child
//! - `process:message` - { pid, data } - JSON message from child (channel mode)
//! - `process:exit` - { pid, code } - child process exited
//!
//! ## Integration with PluginRouter
//!
//! When created via `PluginRouter::new()`, the ProcessPlugin automatically
//! shares the router's event callback. Events are emitted to the frontend
//! when the callback is set via `PluginRouter::set_event_callback()`.
//!
//! ## Graceful Shutdown (powered by ipckit)
//!
//! Uses ipckit's `ShutdownState` for coordinated shutdown across all
//! background threads. This prevents "EventLoopClosed" errors when
//! the WebView is closing.
//!
//! ## Example (Pipe Mode)
//!
//! ```javascript
//! // Spawn with pipe IPC
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
//!
//! ## Example (Channel Mode - High Performance)
//!
//! ```javascript
//! // Spawn with ipckit channel IPC
//! const { pid, channel } = await auroraview.invoke("plugin:process|spawn_ipc_channel", {
//!     command: "python",
//!     args: ["script.py"],
//!     cwd: "/path/to/dir"
//! });
//!
//! // Listen for JSON messages
//! auroraview.on("process:message", ({ pid, data }) => {
//!     console.log(`[${pid}] Message:`, data);
//! });
//!
//! // Send JSON message to process
//! await auroraview.invoke("plugin:process|send_json", {
//!     pid,
//!     data: { action: "getData", params: { key: "value" } }
//! });
//! ```

use auroraview_plugin_core::{
    PluginError, PluginEventCallback, PluginHandler, PluginResult, ScopeConfig,
};
use ipckit::graceful::ShutdownState;
use ipckit::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Callback type for process events (deprecated, use PluginEventCallback)
pub type ProcessEventCallback = PluginEventCallback;

/// Type alias for the process registry to reduce type complexity
type ProcessRegistry = Arc<RwLock<HashMap<u32, Arc<Mutex<ManagedProcess>>>>>;

/// Type alias for the IPC channel registry
type ChannelRegistry = Arc<RwLock<HashMap<u32, Arc<Mutex<IpcChannelHandle>>>>>;

/// IPC mode for spawned processes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcMode {
    /// Pipe-based IPC (stdout/stderr)
    Pipe,
    /// ipckit LocalSocket-based IPC (high performance)
    Channel,
}

/// Managed process info
struct ManagedProcess {
    /// Child process handle
    child: Child,
    /// Stdin writer (if available, for pipe mode)
    stdin: Option<std::process::ChildStdin>,
    /// IPC mode
    ipc_mode: IpcMode,
}

/// IPC channel handle for LocalSocket-based communication
struct IpcChannelHandle {
    /// Socket stream for sending messages
    stream: LocalSocketStream,
    /// Channel name (for cleanup)
    #[allow(dead_code)]
    channel_name: String,
}

/// Process plugin with IPC support
///
/// Uses ipckit's `ShutdownState` for graceful shutdown coordination.
/// Supports two IPC modes:
/// - Pipe mode: Traditional stdout/stderr pipes
/// - Channel mode: ipckit LocalSocket for high-performance JSON messaging
pub struct ProcessPlugin {
    name: String,
    /// Managed processes by PID
    processes: ProcessRegistry,
    /// IPC channels by PID (for channel mode)
    channels: ChannelRegistry,
    /// Event callback for emitting events to frontend (shared with PluginRouter)
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
    /// Shutdown state from ipckit for graceful shutdown coordination
    shutdown_state: Arc<ShutdownState>,
}

impl ProcessPlugin {
    /// Create a new process plugin
    pub fn new() -> Self {
        Self {
            name: "process".to_string(),
            processes: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_callback: Arc::new(RwLock::new(None)),
            shutdown_state: Arc::new(ShutdownState::new()),
        }
    }

    /// Create a new process plugin with a shared event callback
    ///
    /// This is used by PluginRouter to share its event callback with ProcessPlugin.
    pub fn with_event_callback(callback: Arc<RwLock<Option<PluginEventCallback>>>) -> Self {
        Self {
            name: "process".to_string(),
            processes: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_callback: callback,
            shutdown_state: Arc::new(ShutdownState::new()),
        }
    }

    /// Set the event callback for emitting events to frontend
    pub fn set_event_callback(&self, callback: PluginEventCallback) {
        let mut cb = self.event_callback.write().unwrap();
        *cb = Some(callback);
    }

    /// Check if shutdown has been initiated
    fn is_shutting_down(&self) -> bool {
        self.shutdown_state.is_shutdown()
    }

    /// Spawn a process with IPC support
    fn spawn_ipc(&self, opts: SpawnIpcOptions, scope: &ScopeConfig) -> PluginResult<Value> {
        tracing::info!("[Rust:ProcessPlugin] spawn_ipc called");
        tracing::info!("[Rust:ProcessPlugin] command: {}", opts.command);
        tracing::info!("[Rust:ProcessPlugin] args: {:?}", opts.args);
        tracing::info!("[Rust:ProcessPlugin] cwd: {:?}", opts.cwd);
        tracing::info!("[Rust:ProcessPlugin] show_console: {}", opts.show_console);

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

        // IMPORTANT: Clear AURORAVIEW_PACKED so spawned processes run in standalone mode
        // Otherwise, examples would detect packed mode and run as API servers instead of
        // creating their own windows. The parent Gallery is in packed mode, but spawned
        // examples should run independently.
        let current_packed = std::env::var("AURORAVIEW_PACKED").unwrap_or_default();
        tracing::debug!(
            "[Rust:ProcessPlugin] Current AURORAVIEW_PACKED={}, will set to 0 for child",
            current_packed
        );
        cmd.env("AURORAVIEW_PACKED", "0");

        // Apply environment isolation (rez-style) if running in packed mode
        // The isolation context is stored in environment variables by backend.rs
        let isolate_path_env = std::env::var("AURORAVIEW_ISOLATE_PATH").unwrap_or_default();
        let isolate_pythonpath_env =
            std::env::var("AURORAVIEW_ISOLATE_PYTHONPATH").unwrap_or_default();
        let isolate_path = isolate_path_env == "1";
        let isolate_pythonpath = isolate_pythonpath_env == "1";

        tracing::info!(
            "[Rust:ProcessPlugin] Isolation check: AURORAVIEW_ISOLATE_PATH='{}', AURORAVIEW_ISOLATE_PYTHONPATH='{}'",
            isolate_path_env,
            isolate_pythonpath_env
        );

        if isolate_path || isolate_pythonpath {
            tracing::debug!(
                "[Rust:ProcessPlugin] Environment isolation: PATH={}, PYTHONPATH={}",
                if isolate_path {
                    "isolated"
                } else {
                    "inherited"
                },
                if isolate_pythonpath {
                    "isolated"
                } else {
                    "inherited"
                }
            );

            // Use pre-built isolated paths from parent process
            if isolate_path {
                if let Ok(isolated_path) = std::env::var("AURORAVIEW_ISOLATED_PATH") {
                    cmd.env("PATH", &isolated_path);
                    tracing::debug!("[Rust:ProcessPlugin] Using isolated PATH");
                }
            }

            if isolate_pythonpath {
                if let Ok(isolated_pythonpath) = std::env::var("AURORAVIEW_ISOLATED_PYTHONPATH") {
                    cmd.env("PYTHONPATH", &isolated_pythonpath);
                    tracing::debug!("[Rust:ProcessPlugin] Using isolated PYTHONPATH");
                }
            }

            // Set PYTHONHOME for python-build-standalone
            if let Ok(python_home) = std::env::var("AURORAVIEW_PYTHON_HOME") {
                cmd.env("PYTHONHOME", &python_home);
                tracing::debug!("[Rust:ProcessPlugin] Setting PYTHONHOME={}", python_home);
            }
        } else {
            // Legacy behavior: merge PYTHONPATH, inherit PATH
            // In packed mode, inherit AURORAVIEW_PYTHON_PATH to PYTHONPATH
            // This allows spawned Python processes to find bundled modules
            let python_path_result = std::env::var("AURORAVIEW_PYTHON_PATH");
            tracing::debug!(
                "[Rust:ProcessPlugin] AURORAVIEW_PYTHON_PATH env check: {:?}",
                python_path_result
            );
            if let Ok(python_path) = python_path_result {
                // Merge with existing PYTHONPATH if any
                let separator = if cfg!(windows) { ";" } else { ":" };
                let existing = std::env::var("PYTHONPATH").unwrap_or_default();
                let merged = if existing.is_empty() {
                    python_path.clone()
                } else {
                    format!("{}{}{}", python_path, separator, existing)
                };
                tracing::debug!("[Rust:ProcessPlugin] Setting PYTHONPATH={}", merged);
                cmd.env("PYTHONPATH", merged);
            } else {
                tracing::debug!("[Rust:ProcessPlugin] AURORAVIEW_PYTHON_PATH not set - spawned Python may not find modules");
            }

            // In packed mode, inherit PYTHONHOME for python-build-standalone
            // This is required for Python to find its standard library (encodings, etc.)
            if let Ok(python_home) = std::env::var("PYTHONHOME") {
                tracing::debug!("[Rust:ProcessPlugin] Setting PYTHONHOME={}", python_home);
                cmd.env("PYTHONHOME", &python_home);
            }
        }

        // Windows: control console window visibility
        // Note: CREATE_NO_WINDOW prevents ALL windows including GUI windows,
        // so we use DETACHED_PROCESS instead which only hides the console.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_CONSOLE: u32 = 0x00000010;

            if opts.show_console {
                tracing::info!("[Rust:ProcessPlugin] Creating with new console window");
                cmd.creation_flags(CREATE_NEW_CONSOLE);
            } else {
                // DETACHED_PROCESS: Runs without a console window but allows GUI windows
                // This is different from CREATE_NO_WINDOW which blocks all windows
                tracing::debug!(
                    "[Rust:ProcessPlugin] Creating detached process (no console, GUI allowed)"
                );
                cmd.creation_flags(DETACHED_PROCESS);
            }
        }

        // Spawn the process
        tracing::info!("[Rust:ProcessPlugin] Spawning process...");
        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("[Rust:ProcessPlugin] Failed to spawn: {}", e);
            PluginError::shell_error(format!("Failed to spawn: {}", e))
        })?;

        let pid = child.id();
        tracing::info!("[Rust:ProcessPlugin] Process spawned with PID: {}", pid);

        // Brief check to see if process exits immediately (indicates startup error)
        std::thread::sleep(std::time::Duration::from_millis(50));
        match child.try_wait() {
            Ok(Some(status)) => {
                tracing::warn!(
                    "[Rust:ProcessPlugin] Process {} exited immediately with status: {:?}",
                    pid,
                    status
                );
            }
            Ok(None) => {
                tracing::info!(
                    "[Rust:ProcessPlugin] Process {} is still running after 50ms",
                    pid
                );
            }
            Err(e) => {
                tracing::warn!("[Rust:ProcessPlugin] Failed to check process status: {}", e);
            }
        }

        // Take stdout/stderr for async reading
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        // Store managed process
        let managed = Arc::new(Mutex::new(ManagedProcess {
            child,
            stdin,
            ipc_mode: IpcMode::Pipe,
        }));
        {
            let mut processes = self.processes.write().unwrap();
            processes.insert(pid, managed.clone());
        }

        // Spawn stdout reader thread with graceful shutdown support
        if let Some(stdout) = stdout {
            let event_cb = self.event_callback.clone();
            let processes = self.processes.clone();
            let shutdown_state = Arc::clone(&self.shutdown_state);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    // Check shutdown state before emitting (using ipckit)
                    if shutdown_state.is_shutdown() {
                        tracing::debug!(
                            "[Rust:ProcessPlugin] Shutdown detected, stopping stdout reader for PID {}",
                            pid
                        );
                        break;
                    }

                    // Use operation guard to track this emit operation
                    let _guard = shutdown_state.begin_operation();

                    match line {
                        Ok(data) => {
                            tracing::debug!(
                                "[Rust:ProcessPlugin] Process {} stdout: {}",
                                pid,
                                data
                            );
                            let cb_guard = event_cb.read().unwrap();
                            if let Some(cb) = cb_guard.as_ref() {
                                tracing::debug!(
                                    "[Rust:ProcessPlugin] Emitting process:stdout event for PID {}",
                                    pid
                                );
                                cb(
                                    "process:stdout",
                                    serde_json::json!({
                                        "pid": pid,
                                        "data": data
                                    }),
                                );
                            } else {
                                tracing::warn!(
                                    "[Rust:ProcessPlugin] No event callback set, stdout not forwarded for PID {}",
                                    pid
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }
                // Process stdout closed, check if process exited (only if not shutting down)
                if !shutdown_state.is_shutdown() {
                    Self::check_exit(&processes, &event_cb, pid);
                }
                tracing::debug!("[Rust:ProcessPlugin] Process {} stdout reader exiting", pid);
            });
        }

        // Spawn stderr reader thread with graceful shutdown support
        if let Some(stderr) = stderr {
            let event_cb = self.event_callback.clone();
            let shutdown_state = Arc::clone(&self.shutdown_state);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    // Check shutdown state before emitting (using ipckit)
                    if shutdown_state.is_shutdown() {
                        tracing::debug!(
                            "[Rust:ProcessPlugin] Shutdown detected, stopping stderr reader for PID {}",
                            pid
                        );
                        break;
                    }

                    // Use operation guard to track this emit operation
                    let _guard = shutdown_state.begin_operation();

                    match line {
                        Ok(data) => {
                            // Always log stderr for debugging
                            tracing::info!("[Rust:ProcessPlugin] Process {} stderr: {}", pid, data);
                            let cb_guard = event_cb.read().unwrap();
                            if let Some(cb) = cb_guard.as_ref() {
                                tracing::debug!(
                                    "[Rust:ProcessPlugin] Emitting process:stderr event for PID {}",
                                    pid
                                );
                                cb(
                                    "process:stderr",
                                    serde_json::json!({
                                        "pid": pid,
                                        "data": data
                                    }),
                                );
                            } else {
                                tracing::warn!(
                                    "[Rust:ProcessPlugin] No event callback set, stderr not forwarded for PID {}",
                                    pid
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }
                tracing::debug!("[Rust:ProcessPlugin] Process {} stderr reader exiting", pid);
            });
        }

        Ok(serde_json::json!({
            "success": true,
            "pid": pid,
            "mode": "pipe"
        }))
    }

    /// Spawn a process with ipckit LocalSocket-based IPC (high performance)
    ///
    /// This mode uses ipckit's LocalSocket for bidirectional JSON messaging,
    /// which is more efficient than pipe-based IPC for structured data.
    fn spawn_ipc_channel(&self, opts: SpawnIpcOptions, scope: &ScopeConfig) -> PluginResult<Value> {
        tracing::info!("[Rust:ProcessPlugin] spawn_ipc_channel called (ipckit mode)");
        tracing::info!("[Rust:ProcessPlugin] command: {}", opts.command);
        tracing::info!("[Rust:ProcessPlugin] args: {:?}", opts.args);

        // Check if command is allowed
        if !scope.shell.is_command_allowed(&opts.command) {
            return Err(PluginError::shell_error(format!(
                "Command '{}' is not allowed by scope configuration",
                opts.command
            )));
        }

        // Generate unique channel name based on timestamp and random suffix
        let channel_name = format!(
            "auroraview_ipc_{}_{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        tracing::info!(
            "[Rust:ProcessPlugin] Creating LocalSocket listener: {}",
            channel_name
        );

        // Create LocalSocket listener
        let listener = LocalSocketListener::bind(&channel_name).map_err(|e| {
            tracing::error!("[Rust:ProcessPlugin] Failed to create LocalSocket: {}", e);
            PluginError::shell_error(format!("Failed to create IPC channel: {}", e))
        })?;

        // Build command with channel name in environment
        let mut cmd = Command::new(&opts.command);
        cmd.args(&opts.args);
        cmd.stdout(Stdio::piped()); // Still capture stdout for logging
        cmd.stderr(Stdio::piped()); // Still capture stderr for logging
        cmd.stdin(Stdio::null()); // No stdin in channel mode

        // Pass channel name to child process
        cmd.env("AURORAVIEW_IPC_CHANNEL", &channel_name);
        cmd.env("AURORAVIEW_IPC_MODE", "channel");

        // Set working directory
        if let Some(cwd) = &opts.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &opts.env {
            cmd.env(key, value);
        }

        // Force unbuffered output for Python processes
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("AURORAVIEW_PACKED", "0");

        // Apply environment isolation (same as spawn_ipc)
        self.apply_environment_isolation(&mut cmd);

        // Windows: control console window visibility
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_CONSOLE: u32 = 0x00000010;

            if opts.show_console {
                cmd.creation_flags(CREATE_NEW_CONSOLE);
            } else {
                cmd.creation_flags(DETACHED_PROCESS);
            }
        }

        // Spawn the process
        tracing::info!("[Rust:ProcessPlugin] Spawning process with ipckit channel...");
        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("[Rust:ProcessPlugin] Failed to spawn: {}", e);
            PluginError::shell_error(format!("Failed to spawn: {}", e))
        })?;

        let pid = child.id();
        tracing::info!(
            "[Rust:ProcessPlugin] Process spawned with PID: {} (channel mode)",
            pid
        );

        // Take stdout/stderr for logging (not for IPC)
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Store managed process
        let managed = Arc::new(Mutex::new(ManagedProcess {
            child,
            stdin: None,
            ipc_mode: IpcMode::Channel,
        }));
        {
            let mut processes = self.processes.write().unwrap();
            processes.insert(pid, managed.clone());
        }

        // Spawn thread to accept connection from child process
        let event_cb = self.event_callback.clone();
        let channels = self.channels.clone();
        let shutdown_state = Arc::clone(&self.shutdown_state);
        let channel_name_clone = channel_name.clone();

        thread::spawn(move || {
            tracing::info!(
                "[Rust:ProcessPlugin] Waiting for child {} to connect to channel...",
                pid
            );

            // Wait for child to connect (blocking)
            // Note: ipckit LocalSocketListener doesn't have timeout, so we rely on
            // the child process to connect within a reasonable time
            match listener.accept() {
                Ok(stream) => {
                    tracing::info!(
                        "[Rust:ProcessPlugin] Child {} connected to IPC channel",
                        pid
                    );

                    // Store channel handle for sending (stream is used for both read and write)
                    // We'll read in this thread and write via send_json_to_process
                    {
                        let mut chans = channels.write().unwrap();
                        chans.insert(
                            pid,
                            Arc::new(Mutex::new(IpcChannelHandle {
                                stream,
                                channel_name: channel_name_clone.clone(),
                            })),
                        );
                    }

                    // Get the stream back for reading
                    let read_handle = {
                        let chans = channels.read().unwrap();
                        chans.get(&pid).cloned()
                    };

                    if let Some(handle) = read_handle {
                        // Read messages from child using the stored stream
                        let mut line = String::new();

                        loop {
                            if shutdown_state.is_shutdown() {
                                tracing::debug!(
                                    "[Rust:ProcessPlugin] Shutdown detected, stopping channel reader for PID {}",
                                    pid
                                );
                                break;
                            }

                            // Lock the handle to read
                            let read_result = {
                                let mut h = handle.lock().unwrap();
                                let mut reader = BufReader::new(&mut h.stream);
                                line.clear();
                                reader.read_line(&mut line)
                            };

                            match read_result {
                                Ok(0) => {
                                    // EOF - child disconnected
                                    tracing::info!(
                                        "[Rust:ProcessPlugin] Child {} disconnected from channel",
                                        pid
                                    );
                                    break;
                                }
                                Ok(_) => {
                                    let _guard = shutdown_state.begin_operation();

                                    // Try to parse as JSON
                                    let trimmed = line.trim();
                                    if !trimmed.is_empty() {
                                        match serde_json::from_str::<Value>(trimmed) {
                                            Ok(json_data) => {
                                                tracing::debug!(
                                                    "[Rust:ProcessPlugin] Process {} message: {}",
                                                    pid,
                                                    trimmed
                                                );
                                                let cb_guard = event_cb.read().unwrap();
                                                if let Some(cb) = cb_guard.as_ref() {
                                                    cb(
                                                        "process:message",
                                                        serde_json::json!({
                                                            "pid": pid,
                                                            "data": json_data
                                                        }),
                                                    );
                                                }
                                            }
                                            Err(_) => {
                                                // Not valid JSON, emit as raw data
                                                let cb_guard = event_cb.read().unwrap();
                                                if let Some(cb) = cb_guard.as_ref() {
                                                    cb(
                                                        "process:message",
                                                        serde_json::json!({
                                                            "pid": pid,
                                                            "data": trimmed
                                                        }),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!(
                                        "[Rust:ProcessPlugin] Channel read error for PID {}: {}",
                                        pid,
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                    }

                    // Cleanup channel
                    {
                        let mut chans = channels.write().unwrap();
                        chans.remove(&pid);
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "[Rust:ProcessPlugin] Child {} failed to connect: {}",
                        pid,
                        e
                    );
                }
            }
        });

        // Spawn stdout/stderr readers for logging (same as pipe mode)
        if let Some(stdout) = stdout {
            let event_cb = self.event_callback.clone();
            let processes = self.processes.clone();
            let shutdown_state = Arc::clone(&self.shutdown_state);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if shutdown_state.is_shutdown() {
                        break;
                    }
                    let _guard = shutdown_state.begin_operation();
                    tracing::debug!("[Rust:ProcessPlugin] Process {} stdout: {}", pid, line);
                    let cb_guard = event_cb.read().unwrap();
                    if let Some(cb) = cb_guard.as_ref() {
                        cb(
                            "process:stdout",
                            serde_json::json!({
                                "pid": pid,
                                "data": line
                            }),
                        );
                    }
                }
                if !shutdown_state.is_shutdown() {
                    Self::check_exit(&processes, &event_cb, pid);
                }
            });
        }

        if let Some(stderr) = stderr {
            let event_cb = self.event_callback.clone();
            let shutdown_state = Arc::clone(&self.shutdown_state);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    if shutdown_state.is_shutdown() {
                        break;
                    }
                    let _guard = shutdown_state.begin_operation();
                    tracing::info!("[Rust:ProcessPlugin] Process {} stderr: {}", pid, line);
                    let cb_guard = event_cb.read().unwrap();
                    if let Some(cb) = cb_guard.as_ref() {
                        cb(
                            "process:stderr",
                            serde_json::json!({
                                "pid": pid,
                                "data": line
                            }),
                        );
                    }
                }
            });
        }

        Ok(serde_json::json!({
            "success": true,
            "pid": pid,
            "mode": "channel",
            "channel": channel_name
        }))
    }

    /// Apply environment isolation for spawned processes
    fn apply_environment_isolation(&self, cmd: &mut Command) {
        let isolate_path_env = std::env::var("AURORAVIEW_ISOLATE_PATH").unwrap_or_default();
        let isolate_pythonpath_env =
            std::env::var("AURORAVIEW_ISOLATE_PYTHONPATH").unwrap_or_default();
        let isolate_path = isolate_path_env == "1";
        let isolate_pythonpath = isolate_pythonpath_env == "1";

        if isolate_path || isolate_pythonpath {
            if isolate_path {
                if let Ok(isolated_path) = std::env::var("AURORAVIEW_ISOLATED_PATH") {
                    cmd.env("PATH", &isolated_path);
                }
            }
            if isolate_pythonpath {
                if let Ok(isolated_pythonpath) = std::env::var("AURORAVIEW_ISOLATED_PYTHONPATH") {
                    cmd.env("PYTHONPATH", &isolated_pythonpath);
                }
            }
            if let Ok(python_home) = std::env::var("AURORAVIEW_PYTHON_HOME") {
                cmd.env("PYTHONHOME", &python_home);
            }
        } else {
            if let Ok(python_path) = std::env::var("AURORAVIEW_PYTHON_PATH") {
                let separator = if cfg!(windows) { ";" } else { ":" };
                let existing = std::env::var("PYTHONPATH").unwrap_or_default();
                let merged = if existing.is_empty() {
                    python_path
                } else {
                    format!("{}{}{}", python_path, separator, existing)
                };
                cmd.env("PYTHONPATH", merged);
            }
            if let Ok(python_home) = std::env::var("PYTHONHOME") {
                cmd.env("PYTHONHOME", &python_home);
            }
        }
    }

    /// Send JSON message to process via ipckit channel
    fn send_json_to_process(&self, pid: u32, data: &Value) -> PluginResult<Value> {
        let channel = {
            let channels = self.channels.read().unwrap();
            channels.get(&pid).cloned()
        };

        match channel {
            Some(ch) => {
                let mut handle = ch.lock().unwrap();
                let json_str = serde_json::to_string(data)
                    .map_err(|e| PluginError::shell_error(format!("Failed to serialize: {}", e)))?;

                // Write JSON line
                writeln!(handle.stream, "{}", json_str)
                    .map_err(|e| PluginError::shell_error(format!("Failed to write: {}", e)))?;
                handle
                    .stream
                    .flush()
                    .map_err(|e| PluginError::shell_error(format!("Failed to flush: {}", e)))?;

                tracing::debug!("[Rust:ProcessPlugin] Sent JSON to PID {} via channel", pid);
                Ok(serde_json::json!({ "success": true }))
            }
            None => {
                // Check if process exists but in pipe mode
                let proc = {
                    let processes = self.processes.read().unwrap();
                    processes.get(&pid).cloned()
                };

                match proc {
                    Some(p) => {
                        let managed = p.lock().unwrap();
                        if managed.ipc_mode == IpcMode::Pipe {
                            Err(PluginError::shell_error(format!(
                                "Process {} is in pipe mode, use 'send' instead of 'send_json'",
                                pid
                            )))
                        } else {
                            Err(PluginError::shell_error(format!(
                                "Channel not ready for process {}",
                                pid
                            )))
                        }
                    }
                    None => Err(PluginError::shell_error(format!(
                        "Process {} not found",
                        pid
                    ))),
                }
            }
        }
    }
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

                // Try to wait for process to terminate (non-blocking with timeout)
                // Use try_wait to avoid blocking if process doesn't exit immediately
                for _ in 0..10 {
                    match managed.child.try_wait() {
                        Ok(Some(_)) => break, // Process exited
                        Ok(None) => {
                            // Still running, wait a bit
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        Err(_) => break, // Error, stop waiting
                    }
                }

                // Remove from managed (even if process hasn't fully exited)
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
    ///
    /// Uses ipckit's graceful shutdown mechanism to coordinate with background threads.
    fn kill_all(&self) -> PluginResult<Value> {
        // Signal shutdown to all background threads using ipckit
        self.shutdown_state.shutdown();
        tracing::info!(
            "[Rust:ProcessPlugin] Shutdown signaled, waiting for operations to complete..."
        );

        // Wait for pending operations to complete (with timeout)
        // This uses ipckit's wait_for_drain mechanism
        let drain_result = self
            .shutdown_state
            .wait_for_drain(Some(std::time::Duration::from_secs(2)));
        if drain_result.is_err() {
            tracing::warn!(
                "[Rust:ProcessPlugin] Drain timeout, some operations may not have completed"
            );
        }

        // Clear all IPC channels
        {
            let mut channels = self.channels.write().unwrap();
            channels.clear();
            tracing::debug!("[Rust:ProcessPlugin] Cleared all IPC channels");
        }

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

/// Options for sending JSON data to a process (channel mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendJsonOptions {
    /// Process ID
    pub pid: u32,
    /// JSON data to send
    pub data: Value,
}

impl PluginHandler for ProcessPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value> {
        // Check if we're shutting down - reject new operations
        if self.is_shutting_down()
            && command != "kill"
            && command != "kill_all"
            && command != "list"
        {
            return Err(PluginError::shell_error(
                "ProcessPlugin is shutting down, new operations not accepted",
            ));
        }

        match command {
            "spawn_ipc" => {
                let opts: SpawnIpcOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.spawn_ipc(opts, scope)
            }
            "spawn_ipc_channel" => {
                let opts: SpawnIpcOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.spawn_ipc_channel(opts, scope)
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
            "send_json" => {
                let opts: SendJsonOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;
                self.send_json_to_process(opts.pid, &opts.data)
            }
            "list" => self.list_processes(),
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "spawn_ipc",
            "spawn_ipc_channel",
            "kill",
            "kill_all",
            "send",
            "send_json",
            "list",
        ]
    }
}
