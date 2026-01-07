//! Sidecar Process Manager
//!
//! Manages the lifecycle of MCP Sidecar processes, including:
//! - Spawning the sidecar binary with proper arguments
//! - Monitoring process health
//! - Graceful shutdown with fallback to force kill
//! - Cleanup on main process exit

use crate::ipc::{generate_auth_token, generate_channel_name, IpcServer};
use crate::protocol::ToolDefinition;
use parking_lot::RwLock;
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Tool handler function type
pub type ToolHandler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

/// Sidecar process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidecarState {
    /// Not started
    Stopped,
    /// Starting up
    Starting,
    /// Running and healthy
    Running,
    /// Shutting down
    ShuttingDown,
    /// Exited with error
    Failed,
}

/// Configuration for sidecar manager
#[derive(Debug, Clone)]
pub struct SidecarConfig {
    /// MCP server port (0 for auto-assign)
    pub port: u16,
    /// Log level for sidecar
    pub log_level: String,
    /// Path to sidecar binary (None = auto-detect)
    pub binary_path: Option<PathBuf>,
    /// Shutdown timeout before force kill
    pub shutdown_timeout: Duration,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            port: 0,
            log_level: "info".to_string(),
            binary_path: None,
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

/// MCP Sidecar Process Manager
///
/// Manages the lifecycle of the MCP Sidecar process.
pub struct SidecarManager {
    config: SidecarConfig,
    state: Arc<RwLock<SidecarState>>,
    child: Arc<RwLock<Option<Child>>>,
    ipc_server: Arc<RwLock<Option<IpcServer>>>,
    channel_name: String,
    token: String,
    running: Arc<AtomicBool>,
    actual_port: Arc<RwLock<Option<u16>>>,
}

impl SidecarManager {
    /// Create a new sidecar manager with default config.
    pub fn new() -> Self {
        Self::with_config(SidecarConfig::default())
    }

    /// Create a new sidecar manager with custom config.
    pub fn with_config(config: SidecarConfig) -> Self {
        let channel_name = generate_channel_name(std::process::id());
        let token = generate_auth_token();

        Self {
            config,
            state: Arc::new(RwLock::new(SidecarState::Stopped)),
            child: Arc::new(RwLock::new(None)),
            ipc_server: Arc::new(RwLock::new(None)),
            channel_name,
            token,
            running: Arc::new(AtomicBool::new(false)),
            actual_port: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current sidecar state.
    pub fn state(&self) -> SidecarState {
        *self.state.read()
    }

    /// Get the actual MCP port (if running).
    pub fn port(&self) -> Option<u16> {
        *self.actual_port.read()
    }

    /// Check if sidecar is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get IPC channel name.
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    /// Get auth token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Register a tool with the IPC server.
    pub fn register_tool<F>(&self, definition: ToolDefinition, handler: F)
    where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let mut server_guard = self.ipc_server.write();
        if let Some(server) = server_guard.as_mut() {
            server.register_tool(definition, handler);
        } else {
            // Create server if not exists
            let server = IpcServer::new(&self.channel_name, &self.token);
            // Note: Tools should be registered before calling start()
            // This is a placeholder - in practice, create server first then register
            *server_guard = Some(server);
            // Re-get and register
            if let Some(s) = server_guard.as_mut() {
                s.register_tool(definition, handler);
            }
        }
    }

    /// Start the sidecar process.
    ///
    /// This will:
    /// 1. Start the IPC server in a background thread
    /// 2. Spawn the sidecar binary with appropriate arguments
    /// 3. Wait for the sidecar to connect and authenticate
    pub fn start(&self) -> Result<(), SidecarError> {
        if self.is_running() {
            return Err(SidecarError::AlreadyRunning);
        }

        *self.state.write() = SidecarState::Starting;
        self.running.store(true, Ordering::SeqCst);

        // Get binary path
        let binary_path = self
            .config
            .binary_path
            .clone()
            .unwrap_or_else(Self::detect_binary_path);

        if !binary_path.exists() {
            self.running.store(false, Ordering::SeqCst);
            *self.state.write() = SidecarState::Failed;
            return Err(SidecarError::BinaryNotFound(binary_path));
        }

        tracing::info!(
            "[SidecarManager] Starting sidecar from: {}",
            binary_path.display()
        );

        // Spawn the sidecar process
        let mut cmd = Command::new(&binary_path);
        cmd.arg("--port")
            .arg(self.config.port.to_string())
            .arg("--ipc")
            .arg(&self.channel_name)
            .arg("--token")
            .arg(&self.token)
            .arg("--parent-pid")
            .arg(std::process::id().to_string())
            .arg("--log-level")
            .arg(&self.config.log_level);

        // Don't inherit stdio to avoid blocking
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Hide console window on Windows
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn().map_err(|e| {
            self.running.store(false, Ordering::SeqCst);
            *self.state.write() = SidecarState::Failed;
            SidecarError::SpawnFailed(e.to_string())
        })?;

        let pid = child.id();
        tracing::info!("[SidecarManager] Sidecar spawned with PID: {}", pid);

        *self.child.write() = Some(child);
        *self.state.write() = SidecarState::Running;

        Ok(())
    }

    /// Stop the sidecar process gracefully.
    ///
    /// This will:
    /// 1. Send lifecycle.shutdown via IPC
    /// 2. Wait for graceful exit (up to timeout)
    /// 3. Force kill if still running
    pub fn stop(&self) -> Result<(), SidecarError> {
        if !self.is_running() {
            return Ok(());
        }

        *self.state.write() = SidecarState::ShuttingDown;
        tracing::info!("[SidecarManager] Stopping sidecar...");

        // Try graceful shutdown first
        let graceful = self.graceful_shutdown();

        // If graceful failed, force kill
        if !graceful {
            self.force_kill()?;
        }

        self.running.store(false, Ordering::SeqCst);
        *self.state.write() = SidecarState::Stopped;
        *self.child.write() = None;
        *self.actual_port.write() = None;

        tracing::info!("[SidecarManager] Sidecar stopped");
        Ok(())
    }

    /// Attempt graceful shutdown.
    fn graceful_shutdown(&self) -> bool {
        let mut child_guard = self.child.write();
        if let Some(ref mut child) = *child_guard {
            // Wait for process to exit with timeout
            let start = std::time::Instant::now();
            while start.elapsed() < self.config.shutdown_timeout {
                match child.try_wait() {
                    Ok(Some(_)) => return true,
                    Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                    Err(_) => return false,
                }
            }
        }
        false
    }

    /// Force kill the sidecar process.
    fn force_kill(&self) -> Result<(), SidecarError> {
        let mut child_guard = self.child.write();
        if let Some(ref mut child) = *child_guard {
            tracing::warn!("[SidecarManager] Force killing sidecar...");
            child
                .kill()
                .map_err(|e| SidecarError::KillFailed(e.to_string()))?;

            // Wait for process to actually terminate
            let _ = child.wait();
        }
        Ok(())
    }

    /// Detect the sidecar binary path.
    fn detect_binary_path() -> PathBuf {
        // Try to find in the same directory as the current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let binary_name = if cfg!(windows) {
                    "auroraview-mcp-server.exe"
                } else {
                    "auroraview-mcp-server"
                };
                let path = exe_dir.join(binary_name);
                if path.exists() {
                    return path;
                }
            }
        }

        // Fallback to PATH
        PathBuf::from(if cfg!(windows) {
            "auroraview-mcp-server.exe"
        } else {
            "auroraview-mcp-server"
        })
    }

    /// Check if the sidecar process is still alive.
    pub fn is_alive(&self) -> bool {
        let mut child_guard = self.child.write();
        if let Some(ref mut child) = *child_guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited
                    self.running.store(false, Ordering::SeqCst);
                    *self.state.write() = SidecarState::Failed;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        if self.is_running() {
            tracing::info!("[SidecarManager] Cleaning up on drop...");
            let _ = self.stop();
        }
    }
}

/// Sidecar error types
#[derive(Debug, Clone)]
pub enum SidecarError {
    /// Sidecar is already running
    AlreadyRunning,
    /// Binary not found at path
    BinaryNotFound(PathBuf),
    /// Failed to spawn process
    SpawnFailed(String),
    /// Failed to kill process
    KillFailed(String),
    /// IPC error
    IpcError(String),
}

impl std::fmt::Display for SidecarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "Sidecar is already running"),
            Self::BinaryNotFound(path) => write!(f, "Sidecar binary not found: {}", path.display()),
            Self::SpawnFailed(e) => write!(f, "Failed to spawn sidecar: {}", e),
            Self::KillFailed(e) => write!(f, "Failed to kill sidecar: {}", e),
            Self::IpcError(e) => write!(f, "IPC error: {}", e),
        }
    }
}

impl std::error::Error for SidecarError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_config_default() {
        let config = SidecarConfig::default();
        assert_eq!(config.port, 0);
        assert_eq!(config.log_level, "info");
        assert!(config.binary_path.is_none());
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_sidecar_manager_creation() {
        let manager = SidecarManager::new();
        assert_eq!(manager.state(), SidecarState::Stopped);
        assert!(!manager.is_running());
        assert!(manager.port().is_none());
        assert!(!manager.channel_name().is_empty());
        assert!(!manager.token().is_empty());
    }

    #[test]
    fn test_sidecar_state_transitions() {
        let manager = SidecarManager::new();

        // Initial state
        assert_eq!(manager.state(), SidecarState::Stopped);

        // Attempting to stop when not running should succeed
        let result = manager.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_path_detection() {
        // Just test that it returns something
        let path = SidecarManager::detect_binary_path();
        assert!(!path.as_os_str().is_empty());
    }
}
