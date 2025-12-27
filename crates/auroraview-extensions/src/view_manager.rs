//! Extension View Manager - Chrome-like DevTools separation
//!
//! This module provides independent WebView instances for each extension view
//! (Service Worker, Popup, Side Panel), similar to Chrome's extension debugging model.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Extension View Manager                        │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
//! │  │  Service Worker │  │     Popup       │  │   Side Panel    │  │
//! │  │    (Hidden)     │  │   (Floating)    │  │   (Embedded)    │  │
//! │  │  Port: 9222     │  │  Port: 9223     │  │  Port: 9224     │  │
//! │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
//! │           │                    │                    │           │
//! │           └────────────────────┼────────────────────┘           │
//! │                                │                                │
//! │                    ┌───────────▼───────────┐                    │
//! │                    │   DevTools Manager    │                    │
//! │                    │   (CDP Connections)   │                    │
//! │                    └───────────────────────┘                    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - Independent WebView instances per extension view
//! - Separate CDP debugging ports for each view
//! - DevTools can be opened in independent windows
//! - View lifecycle management (create, show, hide, destroy)

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::ExtensionId;

/// Base port for CDP debugging (auto-incremented for each view)
const CDP_BASE_PORT: u16 = 9222;

/// Global port counter for unique CDP ports
static CDP_PORT_COUNTER: AtomicU16 = AtomicU16::new(CDP_BASE_PORT);

/// Extension view type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionViewType {
    /// Background service worker (hidden, no UI)
    ServiceWorker,
    /// Popup window (floating, triggered by action click)
    Popup,
    /// Side panel (embedded in main window)
    SidePanel,
    /// Options page (separate tab/window)
    Options,
    /// DevTools panel (inside browser DevTools)
    DevToolsPanel,
}

impl std::fmt::Display for ExtensionViewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionViewType::ServiceWorker => write!(f, "service_worker"),
            ExtensionViewType::Popup => write!(f, "popup"),
            ExtensionViewType::SidePanel => write!(f, "side_panel"),
            ExtensionViewType::Options => write!(f, "options"),
            ExtensionViewType::DevToolsPanel => write!(f, "devtools_panel"),
        }
    }
}

/// Extension view state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionViewState {
    /// View is not created
    NotCreated,
    /// View is being created
    Creating,
    /// View is created but hidden
    Hidden,
    /// View is visible
    Visible,
    /// View is being destroyed
    Destroying,
    /// View encountered an error
    Error,
}

/// Configuration for creating an extension view
#[derive(Debug, Clone)]
pub struct ExtensionViewConfig {
    /// Extension ID
    pub extension_id: ExtensionId,
    /// View type
    pub view_type: ExtensionViewType,
    /// HTML file path (relative to extension root)
    pub html_path: String,
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Enable DevTools
    pub dev_tools: bool,
    /// CDP debugging port (auto-assigned if None)
    pub debug_port: Option<u16>,
    /// Whether the view should be visible on creation
    pub visible: bool,
    /// Parent window handle (for embedded views)
    pub parent_hwnd: Option<u64>,
}

impl Default for ExtensionViewConfig {
    fn default() -> Self {
        Self {
            extension_id: String::new(),
            view_type: ExtensionViewType::SidePanel,
            html_path: String::new(),
            title: "Extension View".to_string(),
            width: 400,
            height: 600,
            dev_tools: true,
            debug_port: None,
            visible: true,
            parent_hwnd: None,
        }
    }
}

/// Information about an extension view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionViewInfo {
    /// Unique view ID
    pub view_id: String,
    /// Extension ID
    pub extension_id: ExtensionId,
    /// View type
    pub view_type: ExtensionViewType,
    /// Current state
    pub state: ExtensionViewState,
    /// CDP debugging port
    pub debug_port: u16,
    /// DevTools URL for this view
    pub devtools_url: String,
    /// View title
    pub title: String,
    /// Whether DevTools is currently open
    pub devtools_open: bool,
}

/// Internal view handle
struct ExtensionViewHandle {
    /// View info
    info: ExtensionViewInfo,
    /// Window handle (platform-specific)
    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    hwnd: Option<u64>,
    /// DevTools window handle
    #[cfg(target_os = "windows")]
    devtools_hwnd: Option<u64>,
}

/// Extension View Manager
///
/// Manages independent WebView instances for each extension view,
/// providing Chrome-like DevTools separation.
pub struct ExtensionViewManager {
    /// All views by view_id
    views: RwLock<HashMap<String, ExtensionViewHandle>>,
    /// Views by extension_id and view_type
    views_by_extension: RwLock<HashMap<(ExtensionId, ExtensionViewType), String>>,
    /// Callback for creating WebView (set by the host application)
    create_webview_callback: RwLock<Option<CreateWebViewCallback>>,
    /// Callback for opening DevTools
    open_devtools_callback: RwLock<Option<OpenDevToolsCallback>>,
}

/// Callback type for creating a WebView
pub type CreateWebViewCallback =
    Arc<dyn Fn(ExtensionViewConfig) -> Result<u64, String> + Send + Sync>;

/// Callback type for opening DevTools
pub type OpenDevToolsCallback = Arc<dyn Fn(&str, u16) -> Result<u64, String> + Send + Sync>;

impl ExtensionViewManager {
    /// Create a new extension view manager
    pub fn new() -> Self {
        Self {
            views: RwLock::new(HashMap::new()),
            views_by_extension: RwLock::new(HashMap::new()),
            create_webview_callback: RwLock::new(None),
            open_devtools_callback: RwLock::new(None),
        }
    }

    /// Get the global extension view manager instance
    pub fn global() -> &'static ExtensionViewManager {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<ExtensionViewManager> = Lazy::new(ExtensionViewManager::new);
        &INSTANCE
    }

    /// Set the callback for creating WebViews
    pub fn set_create_webview_callback(&self, callback: CreateWebViewCallback) {
        let mut cb = self.create_webview_callback.write();
        *cb = Some(callback);
    }

    /// Set the callback for opening DevTools
    pub fn set_open_devtools_callback(&self, callback: OpenDevToolsCallback) {
        let mut cb = self.open_devtools_callback.write();
        *cb = Some(callback);
    }

    /// Allocate a unique CDP debugging port
    fn allocate_debug_port() -> u16 {
        CDP_PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a unique view ID
    fn generate_view_id(extension_id: &str, view_type: ExtensionViewType) -> String {
        format!("{}:{}", extension_id, view_type)
    }

    /// Create an extension view
    pub fn create_view(&self, config: ExtensionViewConfig) -> Result<ExtensionViewInfo, String> {
        let view_id = Self::generate_view_id(&config.extension_id, config.view_type);

        // Check if view already exists
        {
            let views = self.views.read();
            if views.contains_key(&view_id) {
                return Err(format!("View already exists: {}", view_id));
            }
        }

        // Allocate debug port
        let debug_port = config.debug_port.unwrap_or_else(Self::allocate_debug_port);

        // Generate DevTools URL
        let devtools_url = format!(
            "devtools://devtools/bundled/inspector.html?ws=127.0.0.1:{}/devtools/page/1",
            debug_port
        );

        let info = ExtensionViewInfo {
            view_id: view_id.clone(),
            extension_id: config.extension_id.clone(),
            view_type: config.view_type,
            state: ExtensionViewState::Creating,
            debug_port,
            devtools_url,
            title: config.title.clone(),
            devtools_open: false,
        };

        // Create WebView via callback
        #[allow(unused_variables)]
        let hwnd = {
            let callback = self.create_webview_callback.read();
            if let Some(ref cb) = *callback {
                let mut view_config = config.clone();
                view_config.debug_port = Some(debug_port);
                match cb(view_config) {
                    Ok(h) => Some(h),
                    Err(e) => {
                        tracing::error!("Failed to create WebView for {}: {}", view_id, e);
                        return Err(e);
                    }
                }
            } else {
                tracing::warn!("No create_webview_callback set, view will be virtual");
                None
            }
        };

        // Store view handle
        let handle = ExtensionViewHandle {
            info: ExtensionViewInfo {
                state: if config.visible {
                    ExtensionViewState::Visible
                } else {
                    ExtensionViewState::Hidden
                },
                ..info.clone()
            },
            #[cfg(target_os = "windows")]
            hwnd,
            #[cfg(target_os = "windows")]
            devtools_hwnd: None,
        };

        {
            let mut views = self.views.write();
            views.insert(view_id.clone(), handle);
        }

        {
            let mut by_ext = self.views_by_extension.write();
            by_ext.insert(
                (config.extension_id.clone(), config.view_type),
                view_id.clone(),
            );
        }

        tracing::info!("Created extension view: {} (port: {})", view_id, debug_port);

        Ok(info)
    }

    /// Get view info by view ID
    pub fn get_view(&self, view_id: &str) -> Option<ExtensionViewInfo> {
        let views = self.views.read();
        views.get(view_id).map(|h| h.info.clone())
    }

    /// Get view by extension ID and view type
    pub fn get_view_by_type(
        &self,
        extension_id: &str,
        view_type: ExtensionViewType,
    ) -> Option<ExtensionViewInfo> {
        let by_ext = self.views_by_extension.read();
        let view_id = by_ext.get(&(extension_id.to_string(), view_type))?;
        self.get_view(view_id)
    }

    /// Get all views for an extension
    pub fn get_extension_views(&self, extension_id: &str) -> Vec<ExtensionViewInfo> {
        let views = self.views.read();
        views
            .values()
            .filter(|h| h.info.extension_id == extension_id)
            .map(|h| h.info.clone())
            .collect()
    }

    /// Get all views
    pub fn get_all_views(&self) -> Vec<ExtensionViewInfo> {
        let views = self.views.read();
        views.values().map(|h| h.info.clone()).collect()
    }

    /// Open DevTools for a view
    pub fn open_devtools(&self, view_id: &str) -> Result<(), String> {
        let debug_port = {
            let views = self.views.read();
            let handle = views
                .get(view_id)
                .ok_or_else(|| format!("View not found: {}", view_id))?;
            handle.info.debug_port
        };

        // Open DevTools via callback
        #[allow(unused_variables)]
        let devtools_hwnd = {
            let callback = self.open_devtools_callback.read();
            if let Some(ref cb) = *callback {
                match cb(view_id, debug_port) {
                    Ok(h) => Some(h),
                    Err(e) => {
                        tracing::error!("Failed to open DevTools for {}: {}", view_id, e);
                        return Err(e);
                    }
                }
            } else {
                tracing::warn!("No open_devtools_callback set");
                None
            }
        };

        // Update view state
        {
            let mut views = self.views.write();
            if let Some(handle) = views.get_mut(view_id) {
                handle.info.devtools_open = true;
                #[cfg(target_os = "windows")]
                {
                    handle.devtools_hwnd = devtools_hwnd;
                }
            }
        }

        tracing::info!(
            "Opened DevTools for view: {} (port: {})",
            view_id,
            debug_port
        );
        Ok(())
    }

    /// Close DevTools for a view
    pub fn close_devtools(&self, view_id: &str) -> Result<(), String> {
        let mut views = self.views.write();
        let handle = views
            .get_mut(view_id)
            .ok_or_else(|| format!("View not found: {}", view_id))?;

        handle.info.devtools_open = false;
        #[cfg(target_os = "windows")]
        {
            handle.devtools_hwnd = None;
        }

        tracing::info!("Closed DevTools for view: {}", view_id);
        Ok(())
    }

    /// Show a view
    pub fn show_view(&self, view_id: &str) -> Result<(), String> {
        let mut views = self.views.write();
        let handle = views
            .get_mut(view_id)
            .ok_or_else(|| format!("View not found: {}", view_id))?;

        handle.info.state = ExtensionViewState::Visible;
        tracing::debug!("Showing view: {}", view_id);
        Ok(())
    }

    /// Hide a view
    pub fn hide_view(&self, view_id: &str) -> Result<(), String> {
        let mut views = self.views.write();
        let handle = views
            .get_mut(view_id)
            .ok_or_else(|| format!("View not found: {}", view_id))?;

        handle.info.state = ExtensionViewState::Hidden;
        tracing::debug!("Hiding view: {}", view_id);
        Ok(())
    }

    /// Destroy a view
    pub fn destroy_view(&self, view_id: &str) -> Result<(), String> {
        let (extension_id, view_type) = {
            let mut views = self.views.write();
            let handle = views
                .remove(view_id)
                .ok_or_else(|| format!("View not found: {}", view_id))?;
            (handle.info.extension_id, handle.info.view_type)
        };

        {
            let mut by_ext = self.views_by_extension.write();
            by_ext.remove(&(extension_id, view_type));
        }

        tracing::info!("Destroyed view: {}", view_id);
        Ok(())
    }

    /// Destroy all views for an extension
    pub fn destroy_extension_views(&self, extension_id: &str) -> Result<(), String> {
        let view_ids: Vec<String> = {
            let views = self.views.read();
            views
                .values()
                .filter(|h| h.info.extension_id == extension_id)
                .map(|h| h.info.view_id.clone())
                .collect()
        };

        for view_id in view_ids {
            self.destroy_view(&view_id)?;
        }

        Ok(())
    }

    /// Get CDP connection info for a view
    pub fn get_cdp_info(&self, view_id: &str) -> Option<CdpConnectionInfo> {
        let views = self.views.read();
        let handle = views.get(view_id)?;

        Some(CdpConnectionInfo {
            view_id: view_id.to_string(),
            host: "127.0.0.1".to_string(),
            port: handle.info.debug_port,
            ws_url: format!("ws://127.0.0.1:{}/devtools/page/1", handle.info.debug_port),
            devtools_frontend_url: handle.info.devtools_url.clone(),
        })
    }

    /// Get all CDP connections
    pub fn get_all_cdp_connections(&self) -> Vec<CdpConnectionInfo> {
        let views = self.views.read();
        views
            .values()
            .map(|h| CdpConnectionInfo {
                view_id: h.info.view_id.clone(),
                host: "127.0.0.1".to_string(),
                port: h.info.debug_port,
                ws_url: format!("ws://127.0.0.1:{}/devtools/page/1", h.info.debug_port),
                devtools_frontend_url: h.info.devtools_url.clone(),
            })
            .collect()
    }
}

impl Default for ExtensionViewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CDP connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpConnectionInfo {
    /// View ID
    pub view_id: String,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// WebSocket URL for CDP connection
    pub ws_url: String,
    /// DevTools frontend URL
    pub devtools_frontend_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_id_generation() {
        let view_id =
            ExtensionViewManager::generate_view_id("my-ext", ExtensionViewType::SidePanel);
        assert_eq!(view_id, "my-ext:side_panel");
    }

    #[test]
    fn test_port_allocation() {
        let port1 = ExtensionViewManager::allocate_debug_port();
        let port2 = ExtensionViewManager::allocate_debug_port();
        assert!(port2 > port1);
    }

    #[test]
    fn test_create_view() {
        let manager = ExtensionViewManager::new();

        let config = ExtensionViewConfig {
            extension_id: "test-ext".to_string(),
            view_type: ExtensionViewType::SidePanel,
            html_path: "sidepanel.html".to_string(),
            title: "Test Panel".to_string(),
            ..Default::default()
        };

        let info = manager.create_view(config).unwrap();
        assert_eq!(info.extension_id, "test-ext");
        assert_eq!(info.view_type, ExtensionViewType::SidePanel);
        assert!(info.debug_port >= CDP_BASE_PORT);
    }

    #[test]
    fn test_get_view() {
        let manager = ExtensionViewManager::new();

        let config = ExtensionViewConfig {
            extension_id: "test-ext".to_string(),
            view_type: ExtensionViewType::Popup,
            html_path: "popup.html".to_string(),
            title: "Test Popup".to_string(),
            ..Default::default()
        };

        let created = manager.create_view(config).unwrap();
        let retrieved = manager.get_view(&created.view_id).unwrap();

        assert_eq!(created.view_id, retrieved.view_id);
        assert_eq!(created.debug_port, retrieved.debug_port);
    }

    #[test]
    fn test_get_view_by_type() {
        let manager = ExtensionViewManager::new();

        let config = ExtensionViewConfig {
            extension_id: "test-ext".to_string(),
            view_type: ExtensionViewType::ServiceWorker,
            html_path: "sw.js".to_string(),
            title: "Service Worker".to_string(),
            visible: false,
            ..Default::default()
        };

        manager.create_view(config).unwrap();

        let view = manager
            .get_view_by_type("test-ext", ExtensionViewType::ServiceWorker)
            .unwrap();
        assert_eq!(view.view_type, ExtensionViewType::ServiceWorker);
    }

    #[test]
    fn test_destroy_view() {
        let manager = ExtensionViewManager::new();

        let config = ExtensionViewConfig {
            extension_id: "test-ext".to_string(),
            view_type: ExtensionViewType::Options,
            html_path: "options.html".to_string(),
            title: "Options".to_string(),
            ..Default::default()
        };

        let info = manager.create_view(config).unwrap();
        assert!(manager.get_view(&info.view_id).is_some());

        manager.destroy_view(&info.view_id).unwrap();
        assert!(manager.get_view(&info.view_id).is_none());
    }

    #[test]
    fn test_cdp_info() {
        let manager = ExtensionViewManager::new();

        let config = ExtensionViewConfig {
            extension_id: "test-ext".to_string(),
            view_type: ExtensionViewType::SidePanel,
            html_path: "sidepanel.html".to_string(),
            title: "Test Panel".to_string(),
            ..Default::default()
        };

        let info = manager.create_view(config).unwrap();
        let cdp = manager.get_cdp_info(&info.view_id).unwrap();

        assert_eq!(cdp.host, "127.0.0.1");
        assert_eq!(cdp.port, info.debug_port);
        assert!(cdp.ws_url.contains(&info.debug_port.to_string()));
    }
}
