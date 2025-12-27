//! DevTools Window Management
//!
//! This module provides functionality for creating independent DevTools windows
//! for extension views, similar to Chrome's "Inspect views" feature.
//!
//! ## Features
//!
//! - Create standalone DevTools windows connected via CDP
//! - Support for multiple simultaneous DevTools windows
//! - Integration with extension view manager
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    DevTools Manager                              │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
//! │  │  DevTools #1    │  │  DevTools #2    │  │  DevTools #3    │  │
//! │  │  (SW Debug)     │  │  (Popup Debug)  │  │  (Panel Debug)  │  │
//! │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
//! │           │                    │                    │           │
//! │           │    CDP WebSocket   │                    │           │
//! │           ▼                    ▼                    ▼           │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
//! │  │  Extension SW   │  │  Extension      │  │  Extension      │  │
//! │  │  Port: 9222     │  │  Popup: 9223    │  │  Panel: 9224    │  │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// DevTools window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsWindowInfo {
    /// Unique window ID
    pub window_id: String,
    /// Target view ID being debugged
    pub target_view_id: String,
    /// CDP port of the target
    pub target_port: u16,
    /// Window title
    pub title: String,
    /// Whether the window is open
    pub is_open: bool,
    /// Window handle (platform-specific)
    #[cfg(target_os = "windows")]
    #[serde(skip)]
    pub hwnd: Option<u64>,
}

/// Configuration for creating a DevTools window
#[derive(Debug, Clone)]
pub struct DevToolsWindowConfig {
    /// Target view ID to debug
    pub target_view_id: String,
    /// CDP port of the target
    pub target_port: u16,
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Initial X position (None for center)
    pub x: Option<i32>,
    /// Initial Y position (None for center)
    pub y: Option<i32>,
}

impl Default for DevToolsWindowConfig {
    fn default() -> Self {
        Self {
            target_view_id: String::new(),
            target_port: 9222,
            title: "DevTools".to_string(),
            width: 1200,
            height: 800,
            x: None,
            y: None,
        }
    }
}

/// Callback type for creating a DevTools WebView window
pub type CreateDevToolsWindowCallback =
    Arc<dyn Fn(DevToolsWindowConfig) -> Result<u64, String> + Send + Sync>;

/// Callback type for closing a DevTools window
pub type CloseDevToolsWindowCallback = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// DevTools Window Manager
///
/// Manages independent DevTools windows for debugging extension views.
pub struct DevToolsManager {
    /// All DevTools windows by window_id
    windows: RwLock<HashMap<String, DevToolsWindowInfo>>,
    /// Windows by target view_id
    windows_by_target: RwLock<HashMap<String, String>>,
    /// Callback for creating DevTools windows
    create_callback: RwLock<Option<CreateDevToolsWindowCallback>>,
    /// Callback for closing DevTools windows
    close_callback: RwLock<Option<CloseDevToolsWindowCallback>>,
    /// Counter for generating unique window IDs
    window_counter: std::sync::atomic::AtomicU32,
}

impl DevToolsManager {
    /// Create a new DevTools manager
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            windows_by_target: RwLock::new(HashMap::new()),
            create_callback: RwLock::new(None),
            close_callback: RwLock::new(None),
            window_counter: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Get the global DevTools manager instance
    pub fn global() -> &'static DevToolsManager {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<DevToolsManager> = Lazy::new(DevToolsManager::new);
        &INSTANCE
    }

    /// Set the callback for creating DevTools windows
    pub fn set_create_callback(&self, callback: CreateDevToolsWindowCallback) {
        let mut cb = self.create_callback.write();
        *cb = Some(callback);
    }

    /// Set the callback for closing DevTools windows
    pub fn set_close_callback(&self, callback: CloseDevToolsWindowCallback) {
        let mut cb = self.close_callback.write();
        *cb = Some(callback);
    }

    /// Generate a unique window ID
    fn generate_window_id(&self) -> String {
        let counter = self
            .window_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("devtools-{}", counter)
    }

    /// Open DevTools for a target view
    ///
    /// If DevTools is already open for this target, brings it to front.
    /// Otherwise, creates a new DevTools window.
    pub fn open_devtools(
        &self,
        config: DevToolsWindowConfig,
    ) -> Result<DevToolsWindowInfo, String> {
        // Check if already open for this target
        {
            let by_target = self.windows_by_target.read();
            if let Some(window_id) = by_target.get(&config.target_view_id) {
                let windows = self.windows.read();
                if let Some(info) = windows.get(window_id) {
                    tracing::info!(
                        "DevTools already open for {}, bringing to front",
                        config.target_view_id
                    );
                    // TODO: Bring window to front
                    return Ok(info.clone());
                }
            }
        }

        let window_id = self.generate_window_id();

        // Create DevTools window via callback
        #[allow(unused_variables)]
        let hwnd = {
            let callback = self.create_callback.read();
            if let Some(ref cb) = *callback {
                match cb(config.clone()) {
                    Ok(h) => Some(h),
                    Err(e) => {
                        tracing::error!(
                            "Failed to create DevTools window for {}: {}",
                            config.target_view_id,
                            e
                        );
                        return Err(e);
                    }
                }
            } else {
                tracing::warn!("No create_callback set for DevTools manager");
                None
            }
        };

        let info = DevToolsWindowInfo {
            window_id: window_id.clone(),
            target_view_id: config.target_view_id.clone(),
            target_port: config.target_port,
            title: config.title,
            is_open: true,
            #[cfg(target_os = "windows")]
            hwnd,
        };

        // Store window info
        {
            let mut windows = self.windows.write();
            windows.insert(window_id.clone(), info.clone());
        }

        {
            let mut by_target = self.windows_by_target.write();
            by_target.insert(config.target_view_id.clone(), window_id.clone());
        }

        tracing::info!(
            "Opened DevTools window {} for target {} (port: {})",
            window_id,
            config.target_view_id,
            config.target_port
        );

        Ok(info)
    }

    /// Close DevTools for a target view
    pub fn close_devtools(&self, target_view_id: &str) -> Result<(), String> {
        let window_id = {
            let by_target = self.windows_by_target.read();
            by_target
                .get(target_view_id)
                .cloned()
                .ok_or_else(|| format!("No DevTools open for: {}", target_view_id))?
        };

        self.close_window(&window_id)
    }

    /// Close a DevTools window by ID
    pub fn close_window(&self, window_id: &str) -> Result<(), String> {
        let info = {
            let mut windows = self.windows.write();
            windows
                .remove(window_id)
                .ok_or_else(|| format!("DevTools window not found: {}", window_id))?
        };

        {
            let mut by_target = self.windows_by_target.write();
            by_target.remove(&info.target_view_id);
        }

        // Close window via callback
        {
            let callback = self.close_callback.read();
            if let Some(ref cb) = *callback {
                if let Err(e) = cb(window_id) {
                    tracing::warn!("Failed to close DevTools window {}: {}", window_id, e);
                }
            }
        }

        tracing::info!(
            "Closed DevTools window {} for target {}",
            window_id,
            info.target_view_id
        );

        Ok(())
    }

    /// Get DevTools window info by window ID
    pub fn get_window(&self, window_id: &str) -> Option<DevToolsWindowInfo> {
        let windows = self.windows.read();
        windows.get(window_id).cloned()
    }

    /// Get DevTools window for a target view
    pub fn get_window_for_target(&self, target_view_id: &str) -> Option<DevToolsWindowInfo> {
        let by_target = self.windows_by_target.read();
        let window_id = by_target.get(target_view_id)?;
        self.get_window(window_id)
    }

    /// Check if DevTools is open for a target
    pub fn is_devtools_open(&self, target_view_id: &str) -> bool {
        let by_target = self.windows_by_target.read();
        by_target.contains_key(target_view_id)
    }

    /// Get all open DevTools windows
    pub fn get_all_windows(&self) -> Vec<DevToolsWindowInfo> {
        let windows = self.windows.read();
        windows.values().cloned().collect()
    }

    /// Close all DevTools windows
    pub fn close_all(&self) {
        let window_ids: Vec<String> = {
            let windows = self.windows.read();
            windows.keys().cloned().collect()
        };

        for window_id in window_ids {
            if let Err(e) = self.close_window(&window_id) {
                tracing::warn!("Failed to close DevTools window {}: {}", window_id, e);
            }
        }
    }

    /// Generate DevTools frontend URL for a target
    pub fn get_devtools_url(port: u16) -> String {
        // Use Chrome DevTools frontend
        format!(
            "devtools://devtools/bundled/inspector.html?ws=127.0.0.1:{}/devtools/page/1",
            port
        )
    }

    /// Generate WebSocket URL for CDP connection
    pub fn get_ws_url(port: u16) -> String {
        format!("ws://127.0.0.1:{}/devtools/page/1", port)
    }
}

impl Default for DevToolsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a DevTools window using the desktop WebView
#[cfg(all(target_os = "windows", feature = "python-bindings"))]
pub fn create_devtools_webview(config: DevToolsWindowConfig) -> Result<u64, String> {
    use crate::ipc::{IpcHandler, MessageQueue};
    use crate::webview::config::{NewWindowMode, WebViewConfig};
    use std::sync::Arc;

    let devtools_url = DevToolsManager::get_devtools_url(config.target_port);

    let webview_config = WebViewConfig {
        title: config.title,
        width: config.width,
        height: config.height,
        url: Some(devtools_url),
        dev_tools: true, // DevTools for DevTools (meta!)
        decorations: true,
        resizable: true,
        transparent: false,
        new_window_mode: NewWindowMode::SystemBrowser,
        ..Default::default()
    };

    let ipc_handler = Arc::new(IpcHandler::new());
    let message_queue = Arc::new(MessageQueue::new());

    // Create the WebView
    match crate::webview::desktop::create_desktop(webview_config, ipc_handler, message_queue) {
        Ok(webview_inner) => {
            // Get HWND
            #[cfg(target_os = "windows")]
            {
                if let Some(hwnd) = webview_inner.cached_hwnd {
                    Ok(hwnd)
                } else {
                    Err("Failed to get HWND from DevTools WebView".to_string())
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(0)
            }
        }
        Err(e) => Err(format!("Failed to create DevTools WebView: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_url_generation() {
        let url = DevToolsManager::get_devtools_url(9222);
        assert!(url.contains("9222"));
        assert!(url.contains("devtools"));
    }

    #[test]
    fn test_ws_url_generation() {
        let url = DevToolsManager::get_ws_url(9223);
        assert_eq!(url, "ws://127.0.0.1:9223/devtools/page/1");
    }

    #[test]
    fn test_window_id_generation() {
        let manager = DevToolsManager::new();
        let id1 = manager.generate_window_id();
        let id2 = manager.generate_window_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("devtools-"));
    }

    #[test]
    fn test_open_close_devtools() {
        let manager = DevToolsManager::new();

        let config = DevToolsWindowConfig {
            target_view_id: "test-view".to_string(),
            target_port: 9222,
            title: "Test DevTools".to_string(),
            ..Default::default()
        };

        // Open DevTools (without callback, should succeed but with no hwnd)
        let info = manager.open_devtools(config).unwrap();
        assert_eq!(info.target_view_id, "test-view");
        assert!(info.is_open);

        // Check it's tracked
        assert!(manager.is_devtools_open("test-view"));

        // Close DevTools
        manager.close_devtools("test-view").unwrap();
        assert!(!manager.is_devtools_open("test-view"));
    }

    #[test]
    fn test_duplicate_open() {
        let manager = DevToolsManager::new();

        let config = DevToolsWindowConfig {
            target_view_id: "test-view".to_string(),
            target_port: 9222,
            title: "Test DevTools".to_string(),
            ..Default::default()
        };

        let info1 = manager.open_devtools(config.clone()).unwrap();
        let info2 = manager.open_devtools(config).unwrap();

        // Should return the same window
        assert_eq!(info1.window_id, info2.window_id);
    }
}
