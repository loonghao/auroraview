//! AuroraView Instance Registry
//!
//! File-based registry for AuroraView instance metadata, enabling CDP discovery
//! by MCP servers and other tools.
//!
//! Storage locations:
//! - Windows: %LOCALAPPDATA%/AuroraView/instances/
//! - macOS: ~/Library/Application Support/AuroraView/instances/
//! - Linux: ~/.local/share/auroraview/instances/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

use super::{Result, ServiceDiscoveryError};

/// Instance information for CDP discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Unique window identifier
    pub window_id: String,
    /// Window title
    pub title: String,
    /// CDP debugging port
    pub cdp_port: u16,

    /// Application name
    #[serde(default = "default_app_name")]
    pub app_name: String,
    /// Application version
    #[serde(default)]
    pub app_version: String,

    /// DCC type (maya, blender, houdini, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcc_type: Option<String>,
    /// DCC version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcc_version: Option<String>,

    /// Panel name (for DCC integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panel_name: Option<String>,
    /// Dock area
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock_area: Option<String>,

    /// Process ID
    pub pid: u32,
    /// Start timestamp (Unix epoch seconds)
    pub start_time: u64,

    /// Current URL
    #[serde(default)]
    pub url: String,
    /// HTML page title
    #[serde(default)]
    pub html_title: String,
    /// Loading state
    #[serde(default)]
    pub is_loading: bool,

    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_app_name() -> String {
    "AuroraView".to_string()
}

impl InstanceInfo {
    /// Create new instance info
    pub fn new(window_id: String, title: String, cdp_port: u16) -> Self {
        Self {
            window_id,
            title,
            cdp_port,
            app_name: default_app_name(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            dcc_type: None,
            dcc_version: None,
            panel_name: None,
            dock_area: None,
            pid: std::process::id(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            url: String::new(),
            html_title: String::new(),
            is_loading: false,
            metadata: HashMap::new(),
        }
    }

    /// Get WebSocket URL for CDP connection
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/devtools/page/1", self.cdp_port)
    }

    /// Get DevTools URL
    pub fn devtools_url(&self) -> String {
        format!(
            "devtools://devtools/bundled/inspector.html?ws=127.0.0.1:{}/devtools/page/1",
            self.cdp_port
        )
    }

    /// Builder: set DCC info
    pub fn with_dcc(mut self, dcc_type: &str, dcc_version: Option<&str>) -> Self {
        self.dcc_type = Some(dcc_type.to_string());
        self.dcc_version = dcc_version.map(|s| s.to_string());
        self
    }

    /// Builder: set panel info
    pub fn with_panel(mut self, panel_name: &str, dock_area: Option<&str>) -> Self {
        self.panel_name = Some(panel_name.to_string());
        self.dock_area = dock_area.map(|s| s.to_string());
        self
    }

    /// Builder: add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// File-based instance registry
pub struct InstanceRegistry {
    instances_dir: PathBuf,
    registered_ids: Mutex<Vec<String>>,
}

impl InstanceRegistry {
    /// Create new registry
    pub fn new() -> Result<Self> {
        let instances_dir = get_instances_dir()?;
        Ok(Self {
            instances_dir,
            registered_ids: Mutex::new(Vec::new()),
        })
    }

    /// Get the instances directory path
    pub fn instances_dir(&self) -> &PathBuf {
        &self.instances_dir
    }

    /// Register an instance
    pub fn register(&self, info: &InstanceInfo) -> Result<()> {
        self.write_file(info)?;

        let mut ids = self.registered_ids.lock().unwrap();
        if !ids.contains(&info.window_id) {
            ids.push(info.window_id.clone());
        }

        info!(
            "Instance registered: {} (title={}, cdp_port={})",
            info.window_id, info.title, info.cdp_port
        );
        Ok(())
    }

    /// Unregister an instance
    pub fn unregister(&self, window_id: &str) -> Result<()> {
        self.delete_file(window_id)?;

        let mut ids = self.registered_ids.lock().unwrap();
        ids.retain(|id| id != window_id);

        info!("Instance unregistered: {}", window_id);
        Ok(())
    }

    /// Update instance info
    pub fn update(&self, window_id: &str, updater: impl FnOnce(&mut InstanceInfo)) -> Result<bool> {
        if let Some(mut info) = self.get(window_id)? {
            updater(&mut info);
            self.write_file(&info)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get instance by window ID
    pub fn get(&self, window_id: &str) -> Result<Option<InstanceInfo>> {
        let file_path = self.get_file_path(window_id);
        if !file_path.exists() {
            return Ok(None);
        }
        self.read_file(&file_path)
    }

    /// Get all registered instances (cleans up stale files)
    pub fn get_all(&self) -> Result<Vec<InstanceInfo>> {
        let mut instances = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.instances_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(Some(info)) = self.read_file(&path) {
                        instances.push(info);
                    }
                }
            }
        }

        Ok(instances)
    }

    /// Get instance by CDP port
    pub fn get_by_cdp_port(&self, port: u16) -> Result<Option<InstanceInfo>> {
        for info in self.get_all()? {
            if info.cdp_port == port {
                return Ok(Some(info));
            }
        }
        Ok(None)
    }

    /// Cleanup all instances registered by this process
    pub fn cleanup(&self) {
        let ids: Vec<String> = {
            let ids = self.registered_ids.lock().unwrap();
            ids.clone()
        };

        for window_id in ids {
            if let Err(e) = self.delete_file(&window_id) {
                debug!("Failed to cleanup instance {}: {}", window_id, e);
            }
        }
    }

    fn get_file_path(&self, window_id: &str) -> PathBuf {
        let safe_id: String = window_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.instances_dir.join(format!("{}.json", safe_id))
    }

    fn write_file(&self, info: &InstanceInfo) -> Result<()> {
        let file_path = self.get_file_path(&info.window_id);
        let content = serde_json::to_string_pretty(info)
            .map_err(|e| ServiceDiscoveryError::IoError(std::io::Error::other(e)))?;
        fs::write(&file_path, content)?;
        debug!("Instance file written: {:?}", file_path);
        Ok(())
    }

    fn delete_file(&self, window_id: &str) -> Result<()> {
        let file_path = self.get_file_path(window_id);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
            debug!("Instance file deleted: {:?}", file_path);
        }
        Ok(())
    }

    fn read_file(&self, file_path: &PathBuf) -> Result<Option<InstanceInfo>> {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                debug!("Failed to read instance file {:?}: {}", file_path, e);
                return Ok(None);
            }
        };

        let info: InstanceInfo = match serde_json::from_str(&content) {
            Ok(i) => i,
            Err(e) => {
                debug!("Failed to parse instance file {:?}: {}", file_path, e);
                return Ok(None);
            }
        };

        // Check if process is still alive
        if !is_process_alive(info.pid) {
            debug!(
                "Removing stale instance file (PID {} dead): {:?}",
                info.pid, file_path
            );
            let _ = fs::remove_file(file_path);
            return Ok(None);
        }

        Ok(Some(info))
    }
}

impl Default for InstanceRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create instance registry")
    }
}

impl Drop for InstanceRegistry {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Get the instances directory
fn get_instances_dir() -> Result<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join("AppData")
                    .join("Local")
            })
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library")
            .join("Application Support")
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".local").join("share"))
    };

    let instances_dir = base.join("AuroraView").join("instances");
    fs::create_dir_all(&instances_dir)?;
    Ok(instances_dir)
}

/// Check if a process is still running
#[cfg(target_os = "windows")]
fn is_process_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(h) = handle {
            let _ = CloseHandle(h);
            true
        } else {
            false
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;

    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Global registry instance
static GLOBAL_REGISTRY: std::sync::OnceLock<InstanceRegistry> = std::sync::OnceLock::new();

/// Get the global instance registry
pub fn get_registry() -> &'static InstanceRegistry {
    GLOBAL_REGISTRY.get_or_init(|| InstanceRegistry::new().expect("Failed to create registry"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_info_creation() {
        let info = InstanceInfo::new("test-1".to_string(), "Test Window".to_string(), 9222);
        assert_eq!(info.window_id, "test-1");
        assert_eq!(info.title, "Test Window");
        assert_eq!(info.cdp_port, 9222);
        assert_eq!(info.app_name, "AuroraView");
    }

    #[test]
    fn test_instance_info_builder() {
        let info = InstanceInfo::new("test-2".to_string(), "Test".to_string(), 9223)
            .with_dcc("maya", Some("2025"))
            .with_panel("MyPanel", Some("left"))
            .with_metadata("key", "value");

        assert_eq!(info.dcc_type, Some("maya".to_string()));
        assert_eq!(info.dcc_version, Some("2025".to_string()));
        assert_eq!(info.panel_name, Some("MyPanel".to_string()));
        assert_eq!(info.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_ws_url() {
        let info = InstanceInfo::new("test".to_string(), "Test".to_string(), 9222);
        assert_eq!(info.ws_url(), "ws://127.0.0.1:9222/devtools/page/1");
    }
}
