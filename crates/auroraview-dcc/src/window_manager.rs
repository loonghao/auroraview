//! Window Manager for DCC mode
//!
//! Manages multiple WebView windows within DCC applications.

use crate::config::DccConfig;
use crate::error::{DccError, Result};
use crate::ipc::IpcRouter;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
#[cfg(target_os = "windows")]
use tracing::debug;
use tracing::info;

/// Unique window identifier
pub type WindowId = String;

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window ID
    pub id: WindowId,
    /// Window title
    pub title: String,
    /// Parent HWND
    pub parent_hwnd: Option<isize>,
    /// Current URL
    pub url: Option<String>,
    /// Visibility state
    pub visible: bool,
    /// Window size
    pub width: u32,
    pub height: u32,
}

/// Window Manager for managing multiple WebView windows in DCC mode
///
/// Provides centralized management of multiple WebView instances,
/// shared IPC routing, and window lifecycle control.
pub struct WindowManager {
    /// Active windows by ID (lock-free concurrent map)
    windows: DashMap<WindowId, WindowState>,

    /// Shared IPC router
    ipc_router: Arc<IpcRouter>,

    /// Next window ID counter (atomic, no lock needed)
    next_id: AtomicU64,
}

/// Internal window state
struct WindowState {
    info: WindowInfo,
    #[cfg(target_os = "windows")]
    webview: Option<crate::webview::DccWebView>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: DashMap::new(),
            ipc_router: Arc::new(IpcRouter::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Create a new window manager with custom IPC router
    pub fn with_router(router: Arc<IpcRouter>) -> Self {
        Self {
            windows: DashMap::new(),
            ipc_router: router,
            next_id: AtomicU64::new(1),
        }
    }

    /// Get the shared IPC router
    pub fn router(&self) -> Arc<IpcRouter> {
        self.ipc_router.clone()
    }

    /// Generate a unique window ID
    fn generate_id(&self) -> WindowId {
        let counter = self.next_id.fetch_add(1, Ordering::Relaxed);
        format!("window_{}", counter)
    }

    /// Create a new window
    #[cfg(target_os = "windows")]
    pub fn create(&self, config: DccConfig) -> Result<WindowId> {
        let id = self.generate_id();

        info!(
            "[WindowManager] Creating window: id={}, title={}",
            id, config.title
        );

        // Extract fields for WindowInfo before moving config into DccWebView
        let title = config.title.clone();
        let parent_hwnd = config.parent_hwnd;
        let url = config.url.clone();
        let width = config.width;
        let height = config.height;

        let webview = crate::webview::DccWebView::new(config)?;

        let info = WindowInfo {
            id: id.clone(),
            title,
            parent_hwnd,
            url,
            visible: false,
            width,
            height,
        };

        let state = WindowState {
            info,
            webview: Some(webview),
        };

        self.windows.insert(id.clone(), state);

        Ok(id)
    }

    /// Create a new window (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn create(&self, config: DccConfig) -> Result<WindowId> {
        let id = self.generate_id();

        let info = WindowInfo {
            id: id.clone(),
            title: config.title,
            parent_hwnd: config.parent_hwnd,
            url: config.url,
            visible: false,
            width: config.width,
            height: config.height,
        };

        let state = WindowState { info };

        self.windows.insert(id.clone(), state);

        Ok(id)
    }

    /// Initialize a window (must call from UI thread)
    #[cfg(target_os = "windows")]
    pub fn init(&self, id: &WindowId) -> Result<()> {
        if let Some(state) = self.windows.get(id) {
            if let Some(ref webview) = state.webview {
                webview.init()?;
                info!("[WindowManager] Window initialized: {}", id);
                return Ok(());
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Initialize a window (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn init(&self, id: &WindowId) -> Result<()> {
        if self.windows.contains_key(id) {
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Show a window
    #[cfg(target_os = "windows")]
    pub fn show(&self, id: &WindowId) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            if let Some(ref webview) = state.webview {
                webview.show()?;
                state.info.visible = true;
                debug!("[WindowManager] Window shown: {}", id);
                return Ok(());
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Show a window (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn show(&self, id: &WindowId) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            state.info.visible = true;
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Hide a window
    #[cfg(target_os = "windows")]
    pub fn hide(&self, id: &WindowId) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            if let Some(ref webview) = state.webview {
                webview.hide()?;
                state.info.visible = false;
                debug!("[WindowManager] Window hidden: {}", id);
                return Ok(());
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Hide a window (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn hide(&self, id: &WindowId) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            state.info.visible = false;
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Close and remove a window
    pub fn close(&self, id: &WindowId) -> Result<()> {
        if self.windows.remove(id).is_some() {
            info!("[WindowManager] Window closed: {}", id);
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Get window info
    pub fn get(&self, id: &WindowId) -> Option<WindowInfo> {
        self.windows.get(id).map(|s| s.info.clone())
    }

    /// Get window info (alias for get())
    pub fn get_info(&self, id: &str) -> Option<WindowInfo> {
        self.windows.get(id).map(|s| s.info.clone())
    }

    /// Check if a window exists
    pub fn has_window(&self, id: &str) -> bool {
        self.windows.contains_key(id)
    }

    /// Get all window IDs
    pub fn list(&self) -> Vec<WindowId> {
        self.windows.iter().map(|e| e.key().clone()).collect()
    }

    /// Get all window IDs (alias for list())
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.list()
    }

    /// Get all window info
    pub fn all(&self) -> Vec<WindowInfo> {
        self.windows.iter().map(|e| e.info.clone()).collect()
    }

    /// Get window count
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Process events for all windows
    ///
    /// Call this periodically from the DCC's main thread (e.g., Qt timer)
    #[cfg(target_os = "windows")]
    pub fn process_events(&self) {
        for entry in self.windows.iter() {
            if let Some(ref webview) = entry.webview {
                if webview.process_events() {
                    debug!("[WindowManager] Window {} has pending events", entry.key());
                }
            }
        }
    }

    /// Process events (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn process_events(&self) {
        // No-op on non-Windows
    }

    /// Navigate a window to URL
    #[cfg(target_os = "windows")]
    pub fn navigate(&self, id: &WindowId, url: &str) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            if let Some(ref webview) = state.webview {
                webview.navigate(url)?;
                state.info.url = Some(url.to_string());
                return Ok(());
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Navigate (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn navigate(&self, id: &WindowId, url: &str) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            state.info.url = Some(url.to_string());
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Resize a window
    #[cfg(target_os = "windows")]
    pub fn resize(&self, id: &WindowId, width: u32, height: u32) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            if let Some(ref webview) = state.webview {
                webview.resize(width, height)?;
                state.info.width = width;
                state.info.height = height;
                return Ok(());
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Resize (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn resize(&self, id: &WindowId, width: u32, height: u32) -> Result<()> {
        if let Some(mut state) = self.windows.get_mut(id) {
            state.info.width = width;
            state.info.height = height;
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Evaluate JavaScript in a window
    #[cfg(target_os = "windows")]
    pub fn eval(&self, id: &WindowId, script: &str) -> Result<()> {
        if let Some(state) = self.windows.get(id) {
            if let Some(ref webview) = state.webview {
                return webview.eval(script);
            }
        }
        Err(DccError::WindowNotFound(id.clone()))
    }

    /// Evaluate JavaScript (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn eval(&self, id: &WindowId, _script: &str) -> Result<()> {
        if self.windows.contains_key(id) {
            return Ok(());
        }
        Err(DccError::WindowNotFound(id.clone()))
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
