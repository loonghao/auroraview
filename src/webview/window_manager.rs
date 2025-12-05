//! Window Manager for multi-window support
//!
//! Provides Tauri-like window management with:
//! - Window registry with labels
//! - Window creation and destruction
//! - Cross-window communication
//! - Window queries

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::config::WebViewConfig;
use crate::ipc::{IpcHandler, MessageQueue};

/// Window information stored in the registry
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Unique label for this window
    pub label: String,
    /// Window title
    pub title: String,
    /// Whether this is the main window
    pub is_main: bool,
    /// Current URL
    pub url: Option<String>,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether window is visible
    pub visible: bool,
    /// Whether window is focused
    pub focused: bool,
    /// Parent window label (if child window)
    pub parent_label: Option<String>,
}

/// Window handle for internal use
#[derive(Clone)]
pub struct WindowHandle {
    /// Window info
    pub info: WindowInfo,
    /// IPC handler for this window
    pub ipc_handler: Arc<IpcHandler>,
    /// Message queue for this window
    pub message_queue: Arc<MessageQueue>,
}

/// Global window manager
pub struct WindowManager {
    /// Registry of all windows by label
    windows: RwLock<HashMap<String, WindowHandle>>,
    /// Main window label
    main_window_label: RwLock<Option<String>>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            main_window_label: RwLock::new(None),
        }
    }

    /// Get the global window manager instance
    pub fn global() -> &'static WindowManager {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<WindowManager> = Lazy::new(WindowManager::new);
        &INSTANCE
    }

    /// Register a window with the manager
    pub fn register_window(
        &self,
        label: impl Into<String>,
        config: &WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
        is_main: bool,
    ) -> String {
        let label = label.into();
        let info = WindowInfo {
            label: label.clone(),
            title: config.title.clone(),
            is_main,
            url: config.url.clone(),
            width: config.width,
            height: config.height,
            visible: false,
            focused: false,
            parent_label: None,
        };

        let handle = WindowHandle {
            info,
            ipc_handler,
            message_queue,
        };

        let mut windows = self.windows.write();
        windows.insert(label.clone(), handle);

        if is_main {
            *self.main_window_label.write() = Some(label.clone());
        }

        tracing::info!("[WindowManager] Registered window: {}", label);
        label
    }

    /// Unregister a window
    pub fn unregister_window(&self, label: &str) -> bool {
        let mut windows = self.windows.write();
        let removed = windows.remove(label).is_some();

        if removed {
            tracing::info!("[WindowManager] Unregistered window: {}", label);

            // Clear main window if it was the main
            let mut main_label = self.main_window_label.write();
            if main_label.as_deref() == Some(label) {
                *main_label = None;
            }
        }

        removed
    }

    /// Get window info by label
    pub fn get_window(&self, label: &str) -> Option<WindowInfo> {
        let windows = self.windows.read();
        windows.get(label).map(|h| h.info.clone())
    }

    /// Get all windows
    pub fn get_all_windows(&self) -> Vec<WindowInfo> {
        let windows = self.windows.read();
        windows.values().map(|h| h.info.clone()).collect()
    }

    /// Get all window labels
    pub fn get_all_labels(&self) -> Vec<String> {
        let windows = self.windows.read();
        windows.keys().cloned().collect()
    }

    /// Get main window label
    pub fn get_main_window_label(&self) -> Option<String> {
        self.main_window_label.read().clone()
    }

    /// Get main window
    pub fn get_main_window(&self) -> Option<WindowInfo> {
        let label = self.get_main_window_label()?;
        self.get_window(&label)
    }

    /// Update window visibility
    pub fn set_window_visible(&self, label: &str, visible: bool) {
        let mut windows = self.windows.write();
        if let Some(handle) = windows.get_mut(label) {
            handle.info.visible = visible;
        }
    }

    /// Update window focus
    pub fn set_window_focused(&self, label: &str, focused: bool) {
        let mut windows = self.windows.write();
        if let Some(handle) = windows.get_mut(label) {
            handle.info.focused = focused;
        }
    }

    /// Update window URL
    pub fn set_window_url(&self, label: &str, url: Option<String>) {
        let mut windows = self.windows.write();
        if let Some(handle) = windows.get_mut(label) {
            handle.info.url = url;
        }
    }

    /// Emit event to a specific window
    pub fn emit_to(&self, label: &str, event: &str, data: serde_json::Value) -> bool {
        let windows = self.windows.read();
        if let Some(handle) = windows.get(label) {
            let message = auroraview_core::ipc::IpcMessage {
                event: event.to_string(),
                data,
                id: None,
            };
            if let Err(e) = handle.ipc_handler.handle_message(message) {
                tracing::error!("[WindowManager] Failed to emit to {}: {}", label, e);
                return false;
            }
            true
        } else {
            tracing::warn!("[WindowManager] Window not found: {}", label);
            false
        }
    }

    /// Emit event to all windows
    pub fn emit_all(&self, event: &str, data: serde_json::Value) {
        let windows = self.windows.read();
        for (label, handle) in windows.iter() {
            let message = auroraview_core::ipc::IpcMessage {
                event: event.to_string(),
                data: data.clone(),
                id: None,
            };
            if let Err(e) = handle.ipc_handler.handle_message(message) {
                tracing::error!("[WindowManager] Failed to emit to {}: {}", label, e);
            }
        }
    }

    /// Emit event to all windows except one
    pub fn emit_others(&self, except_label: &str, event: &str, data: serde_json::Value) {
        let windows = self.windows.read();
        for (label, handle) in windows.iter() {
            if label == except_label {
                continue;
            }
            let message = auroraview_core::ipc::IpcMessage {
                event: event.to_string(),
                data: data.clone(),
                id: None,
            };
            if let Err(e) = handle.ipc_handler.handle_message(message) {
                tracing::error!("[WindowManager] Failed to emit to {}: {}", label, e);
            }
        }
    }

    /// Get window count
    pub fn window_count(&self) -> usize {
        self.windows.read().len()
    }

    /// Check if a window exists
    pub fn has_window(&self, label: &str) -> bool {
        self.windows.read().contains_key(label)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_register_unregister() {
        let manager = WindowManager::new();
        let config = WebViewConfig::default();
        let ipc = Arc::new(IpcHandler::new());
        let mq = Arc::new(MessageQueue::new());

        // Register main window
        let label = manager.register_window("main", &config, ipc.clone(), mq.clone(), true);
        assert_eq!(label, "main");
        assert!(manager.has_window("main"));
        assert_eq!(manager.window_count(), 1);
        assert_eq!(manager.get_main_window_label(), Some("main".to_string()));

        // Register child window
        manager.register_window("child", &config, ipc, mq, false);
        assert!(manager.has_window("child"));
        assert_eq!(manager.window_count(), 2);

        // Unregister
        assert!(manager.unregister_window("child"));
        assert!(!manager.has_window("child"));
        assert_eq!(manager.window_count(), 1);
    }

    #[test]
    fn test_get_all_windows() {
        let manager = WindowManager::new();
        let config = WebViewConfig::default();
        let ipc = Arc::new(IpcHandler::new());
        let mq = Arc::new(MessageQueue::new());

        manager.register_window("win1", &config, ipc.clone(), mq.clone(), true);
        manager.register_window("win2", &config, ipc, mq, false);

        let labels = manager.get_all_labels();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&"win1".to_string()));
        assert!(labels.contains(&"win2".to_string()));
    }
}
