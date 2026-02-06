//! Window Manager for Desktop mode
//!
//! Manages multiple WebView windows in standalone applications.

use crate::config::DesktopConfig;
use crate::error::{DesktopError, Result};
use crate::ipc::IpcRouter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Unique window identifier
pub type WindowId = String;

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window ID
    pub id: WindowId,
    /// Window title
    pub title: String,
    /// Current URL
    pub url: Option<String>,
    /// Visibility state
    pub visible: bool,
    /// Window size
    pub width: u32,
    pub height: u32,
}

/// Window Manager for managing multiple WebView windows in Desktop mode
pub struct WindowManager {
    /// Active windows by ID
    windows: RwLock<HashMap<WindowId, WindowState>>,

    /// Shared IPC router
    ipc_router: Arc<IpcRouter>,

    /// Next window ID counter
    next_id: RwLock<u64>,
}

/// Internal window state
struct WindowState {
    info: WindowInfo,
    // In real implementation, would hold DesktopWindow
    // For now, just tracking state
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            ipc_router: Arc::new(IpcRouter::new()),
            next_id: RwLock::new(1),
        }
    }

    /// Create a new window manager with custom IPC router
    pub fn with_router(router: Arc<IpcRouter>) -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            ipc_router: router,
            next_id: RwLock::new(1),
        }
    }

    /// Get the shared IPC router
    pub fn router(&self) -> Arc<IpcRouter> {
        self.ipc_router.clone()
    }

    /// Generate a unique window ID
    fn generate_id(&self) -> WindowId {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("window_{}", *counter);
        *counter += 1;
        id
    }

    /// Create a new window (non-blocking, returns ID)
    pub fn create(&self, config: DesktopConfig) -> Result<WindowId> {
        let id = self.generate_id();

        info!(
            "[WindowManager] Creating window: id={}, title={}",
            id, config.title
        );

        let info = WindowInfo {
            id: id.clone(),
            title: config.title.clone(),
            url: config.url.clone(),
            visible: false,
            width: config.width,
            height: config.height,
        };

        let state = WindowState { info };

        if let Ok(mut windows) = self.windows.write() {
            windows.insert(id.clone(), state);
        }

        Ok(id)
    }

    /// Show a window
    pub fn show(&self, id: &WindowId) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if let Some(state) = windows.get_mut(id) {
                state.info.visible = true;
                debug!("[WindowManager] Window shown: {}", id);
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }

    /// Hide a window
    pub fn hide(&self, id: &WindowId) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if let Some(state) = windows.get_mut(id) {
                state.info.visible = false;
                debug!("[WindowManager] Window hidden: {}", id);
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }

    /// Close and remove a window
    pub fn close(&self, id: &WindowId) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if windows.remove(id).is_some() {
                info!("[WindowManager] Window closed: {}", id);
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }

    /// Get window info
    pub fn get(&self, id: &WindowId) -> Option<WindowInfo> {
        if let Ok(windows) = self.windows.read() {
            windows.get(id).map(|s| s.info.clone())
        } else {
            None
        }
    }

    /// Get window info (alias for get())
    pub fn get_info(&self, id: &str) -> Option<WindowInfo> {
        self.get(&id.to_string())
    }

    /// Check if a window exists
    pub fn has_window(&self, id: &str) -> bool {
        if let Ok(windows) = self.windows.read() {
            windows.contains_key(id)
        } else {
            false
        }
    }

    /// Get all window IDs
    pub fn list(&self) -> Vec<WindowId> {
        if let Ok(windows) = self.windows.read() {
            windows.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all window IDs (alias for list())
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.list()
    }

    /// Get all window info
    pub fn all(&self) -> Vec<WindowInfo> {
        if let Ok(windows) = self.windows.read() {
            windows.values().map(|s| s.info.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// Get window count
    pub fn count(&self) -> usize {
        if let Ok(windows) = self.windows.read() {
            windows.len()
        } else {
            0
        }
    }

    /// Navigate a window to URL
    pub fn navigate(&self, id: &WindowId, url: &str) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if let Some(state) = windows.get_mut(id) {
                state.info.url = Some(url.to_string());
                debug!("[WindowManager] Window {} navigating to {}", id, url);
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }

    /// Resize a window
    pub fn resize(&self, id: &WindowId, width: u32, height: u32) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if let Some(state) = windows.get_mut(id) {
                state.info.width = width;
                state.info.height = height;
                debug!(
                    "[WindowManager] Window {} resized to {}x{}",
                    id, width, height
                );
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }

    /// Set window title
    pub fn set_title(&self, id: &WindowId, title: &str) -> Result<()> {
        if let Ok(mut windows) = self.windows.write() {
            if let Some(state) = windows.get_mut(id) {
                state.info.title = title.to_string();
                debug!("[WindowManager] Window {} title set to '{}'", id, title);
                return Ok(());
            }
        }
        Err(DesktopError::WindowNotFound(id.clone()))
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
