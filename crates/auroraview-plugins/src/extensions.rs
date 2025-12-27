//! Extensions Plugin
//!
//! Provides Chrome Extension API compatibility for AuroraView.
//! This plugin handles API calls from extensions and routes them to the
//! appropriate handlers in auroraview-extensions crate.
//!
//! ## Commands
//!
//! - `api_call` - Route an API call to the extension system
//! - `get_side_panel` - Get side panel HTML for an extension
//! - `list_extensions` - List all loaded extensions
//! - `get_extension` - Get details about a specific extension
//! - `open_side_panel` - Open the side panel for an extension
//! - `close_side_panel` - Close the side panel for an extension
//! - `get_polyfill` - Get the Chrome API polyfill script
//! - `dispatch_event` - Dispatch an event to extension listeners
//!
//! ## Example
//!
//! ```javascript
//! // Call a Chrome API from extension context
//! const result = await auroraview.invoke("plugin:extensions|api_call", {
//!     extensionId: "my-extension",
//!     api: "storage",
//!     method: "get",
//!     params: { area: "local", keys: ["key1"] }
//! });
//!
//! // List loaded extensions
//! const extensions = await auroraview.invoke("plugin:extensions|list_extensions");
//!
//! // Open side panel
//! await auroraview.invoke("plugin:extensions|open_side_panel", {
//!     extensionId: "my-extension"
//! });
//! ```

use auroraview_plugin_core::{PluginError, PluginHandler, PluginResult, ScopeConfig};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Extensions plugin
pub struct ExtensionsPlugin {
    name: String,
    /// Extension host state (shared with the application)
    state: Arc<RwLock<ExtensionsState>>,
}

/// State for the extensions plugin
#[derive(Default)]
pub struct ExtensionsState {
    /// Loaded extensions
    pub extensions: HashMap<String, ExtensionInfo>,
    /// Storage data per extension per area
    pub storage: HashMap<String, HashMap<String, Value>>,
    /// Side panel state per extension
    pub side_panels: HashMap<String, SidePanelState>,
    /// Action state per extension
    pub actions: HashMap<String, ActionState>,
    /// Alarms per extension
    pub alarms: HashMap<String, HashMap<String, AlarmInfo>>,
    /// Notifications per extension
    pub notifications: HashMap<String, HashMap<String, NotificationInfo>>,
    /// Context menus per extension
    pub context_menus: HashMap<String, HashMap<String, MenuItemInfo>>,
    /// Registered content scripts
    pub content_scripts: HashMap<String, Vec<ContentScriptInfo>>,
    /// Message handlers (for runtime.onMessage)
    pub message_handlers: HashMap<String, Vec<String>>,
    /// Extensions directory
    pub extensions_dir: Option<PathBuf>,
    /// Storage directory
    pub storage_dir: Option<PathBuf>,
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    /// Extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension description
    pub description: String,
    /// Whether extension is enabled
    pub enabled: bool,
    /// Side panel path (if any)
    pub side_panel_path: Option<String>,
    /// Popup path (if any)
    pub popup_path: Option<String>,
    /// Options page path (if any)
    pub options_page: Option<String>,
    /// Root directory
    pub root_dir: String,
    /// Permissions
    pub permissions: Vec<String>,
    /// Host permissions
    pub host_permissions: Vec<String>,
    /// Manifest data
    pub manifest: Option<Value>,
}

/// Side panel state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidePanelState {
    /// Whether the panel is open
    pub is_open: bool,
    /// Current path
    pub path: Option<String>,
    /// Panel options
    pub options: Option<SidePanelOptions>,
}

/// Side panel options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidePanelOptions {
    pub path: Option<String>,
    pub enabled: Option<bool>,
}

/// Action (toolbar button) state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionState {
    pub title: Option<String>,
    pub badge_text: Option<String>,
    pub badge_background_color: Option<String>,
    pub badge_text_color: Option<String>,
    pub popup: Option<String>,
    pub enabled: bool,
    pub icon: Option<Value>,
}

/// Alarm information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmInfo {
    pub name: String,
    pub scheduled_time: f64,
    pub period_in_minutes: Option<f64>,
}

/// Notification information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInfo {
    pub id: String,
    pub title: String,
    pub message: String,
    pub icon_url: Option<String>,
    pub notification_type: String,
    pub created_at: i64,
}

/// Context menu item information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemInfo {
    pub id: String,
    pub title: Option<String>,
    pub item_type: String,
    pub contexts: Vec<String>,
    pub parent_id: Option<String>,
    pub enabled: bool,
    pub visible: bool,
}

/// Content script information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentScriptInfo {
    pub id: String,
    pub matches: Vec<String>,
    pub js: Vec<String>,
    pub css: Vec<String>,
    pub run_at: String,
    pub all_frames: bool,
}

/// API call request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCallRequest {
    /// Extension ID
    pub extension_id: String,
    /// API namespace (storage, tabs, etc.)
    pub api: String,
    /// Method name
    pub method: String,
    /// Parameters
    #[serde(default)]
    pub params: Value,
}

/// Extension ID request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionIdRequest {
    /// Extension ID
    pub extension_id: String,
}

/// Event dispatch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDispatchRequest {
    pub extension_id: String,
    pub api: String,
    pub event: String,
    pub args: Vec<Value>,
}

/// View type for API requests
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewTypeRequest {
    ServiceWorker,
    Popup,
    SidePanel,
    Options,
    DevToolsPanel,
}

impl From<ViewTypeRequest> for auroraview_extensions::ExtensionViewType {
    fn from(req: ViewTypeRequest) -> Self {
        match req {
            ViewTypeRequest::ServiceWorker => {
                auroraview_extensions::ExtensionViewType::ServiceWorker
            }
            ViewTypeRequest::Popup => auroraview_extensions::ExtensionViewType::Popup,
            ViewTypeRequest::SidePanel => auroraview_extensions::ExtensionViewType::SidePanel,
            ViewTypeRequest::Options => auroraview_extensions::ExtensionViewType::Options,
            ViewTypeRequest::DevToolsPanel => {
                auroraview_extensions::ExtensionViewType::DevToolsPanel
            }
        }
    }
}

/// Create view request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateViewRequest {
    pub extension_id: String,
    pub view_type: ViewTypeRequest,
    pub html_path: String,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub dev_tools: Option<bool>,
    pub debug_port: Option<u16>,
    pub visible: Option<bool>,
    pub parent_hwnd: Option<u64>,
}

/// View ID request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewIdRequest {
    pub view_id: String,
}

impl ExtensionsPlugin {
    /// Create a new extensions plugin
    pub fn new() -> Self {
        Self {
            name: "extensions".to_string(),
            state: Arc::new(RwLock::new(ExtensionsState::default())),
        }
    }

    /// Create with shared state
    pub fn with_state(state: Arc<RwLock<ExtensionsState>>) -> Self {
        Self {
            name: "extensions".to_string(),
            state,
        }
    }

    /// Get the shared state
    pub fn state(&self) -> Arc<RwLock<ExtensionsState>> {
        self.state.clone()
    }

    /// Register an extension
    pub fn register_extension(&self, info: ExtensionInfo) {
        let mut state = self.state.write();
        let id = info.id.clone();
        state.extensions.insert(id.clone(), info);
        // Initialize default states
        state
            .actions
            .entry(id.clone())
            .or_insert_with(|| ActionState {
                enabled: true,
                ..Default::default()
            });
        state.side_panels.entry(id.clone()).or_default();
        state.alarms.entry(id.clone()).or_default();
        state.notifications.entry(id.clone()).or_default();
        state.context_menus.entry(id).or_default();
    }

    /// Handle storage API calls
    fn handle_storage_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        let area = params
            .get("area")
            .and_then(|v| v.as_str())
            .unwrap_or("local");
        let storage_key = format!("{}:{}", extension_id, area);

        match method {
            "get" => {
                let state = self.state.read();
                let data = state.storage.get(&storage_key).cloned().unwrap_or_default();

                // Handle different key formats
                let keys = params.get("keys");
                let result = match keys {
                    None | Some(Value::Null) => {
                        // Return all data
                        data
                    }
                    Some(Value::String(key)) => {
                        // Single key
                        let mut result = HashMap::new();
                        if let Some(value) = data.get(key) {
                            result.insert(key.clone(), value.clone());
                        }
                        result
                    }
                    Some(Value::Array(arr)) => {
                        // Array of keys
                        let mut result = HashMap::new();
                        for key in arr {
                            if let Some(key_str) = key.as_str() {
                                if let Some(value) = data.get(key_str) {
                                    result.insert(key_str.to_string(), value.clone());
                                }
                            }
                        }
                        result
                    }
                    Some(Value::Object(obj)) => {
                        // Object with defaults
                        let mut result = HashMap::new();
                        for (key, default) in obj {
                            let value = data.get(key).cloned().unwrap_or(default.clone());
                            result.insert(key.clone(), value);
                        }
                        result
                    }
                    _ => data,
                };

                Ok(serde_json::to_value(result).unwrap())
            }
            "set" => {
                let items: HashMap<String, Value> = params
                    .get("items")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| PluginError::invalid_args("items is required"))?;

                let mut state = self.state.write();
                let data = state.storage.entry(storage_key.clone()).or_default();

                // Track changes for onChanged event
                let mut changes = HashMap::new();
                for (key, new_value) in items {
                    let old_value = data.get(&key).cloned();
                    changes.insert(
                        key.clone(),
                        serde_json::json!({
                            "oldValue": old_value,
                            "newValue": new_value
                        }),
                    );
                    data.insert(key, new_value);
                }

                // TODO: Persist to disk and emit onChanged event
                Ok(serde_json::json!({}))
            }
            "remove" => {
                let keys: Vec<String> = match params.get("keys") {
                    Some(Value::String(s)) => vec![s.clone()],
                    Some(Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => return Err(PluginError::invalid_args("keys is required")),
                };

                let mut state = self.state.write();
                if let Some(data) = state.storage.get_mut(&storage_key) {
                    for key in keys {
                        data.remove(&key);
                    }
                }

                Ok(serde_json::json!({}))
            }
            "clear" => {
                let mut state = self.state.write();
                state.storage.remove(&storage_key);
                Ok(serde_json::json!({}))
            }
            "getBytesInUse" => {
                let state = self.state.read();
                let data = state.storage.get(&storage_key).cloned().unwrap_or_default();
                let json = serde_json::to_string(&data).unwrap_or_default();
                Ok(serde_json::json!(json.len()))
            }
            "setAccessLevel" => {
                // Not implemented for now
                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "storage.{}",
                method
            ))),
        }
    }

    /// Handle tabs API calls
    fn handle_tabs_api(
        &self,
        _extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        // AuroraView has a single "tab" representing the main WebView
        let default_tab = serde_json::json!({
            "id": 1,
            "windowId": 1,
            "index": 0,
            "active": true,
            "highlighted": true,
            "pinned": false,
            "status": "complete",
            "incognito": false,
            "url": "",
            "title": "AuroraView"
        });

        match method {
            "query" => {
                // Return single tab matching query
                Ok(serde_json::json!([default_tab]))
            }
            "getCurrent" => Ok(default_tab),
            "get" => Ok(default_tab),
            "create" => {
                // In AuroraView, "creating a tab" might open a new window or navigate
                let url = params.get("url").and_then(|v| v.as_str());
                if let Some(url) = url {
                    // TODO: Implement navigation or window opening
                    tracing::info!("tabs.create requested for URL: {}", url);
                }
                Ok(default_tab)
            }
            "update" => {
                // Handle tab updates (e.g., URL change)
                if let Some(url) = params.get("url").and_then(|v| v.as_str()) {
                    tracing::info!("tabs.update requested for URL: {}", url);
                }
                Ok(default_tab)
            }
            "remove" => {
                // Cannot remove the only tab
                Ok(serde_json::json!({}))
            }
            "reload" => {
                // TODO: Implement page reload
                Ok(serde_json::json!({}))
            }
            "sendMessage" => {
                let message = params.get("message").cloned().unwrap_or(Value::Null);
                // TODO: Implement message passing to content scripts
                Ok(message)
            }
            "captureVisibleTab" => {
                // TODO: Implement screenshot
                Ok(serde_json::json!(""))
            }
            "executeScript" | "insertCSS" | "removeCSS" => {
                // Delegate to scripting API
                Ok(serde_json::json!([{ "frameId": 0, "result": null }]))
            }
            "setZoom" | "getZoom" => Ok(serde_json::json!(1.0)),
            "group" | "ungroup" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!("tabs.{}", method))),
        }
    }

    /// Handle sidePanel API calls
    fn handle_side_panel_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "open" => {
                let mut state = self.state.write();
                let panel = state
                    .side_panels
                    .entry(extension_id.to_string())
                    .or_default();
                panel.is_open = true;
                Ok(serde_json::json!({}))
            }
            "close" => {
                let mut state = self.state.write();
                if let Some(panel) = state.side_panels.get_mut(extension_id) {
                    panel.is_open = false;
                }
                Ok(serde_json::json!({}))
            }
            "setOptions" => {
                let mut state = self.state.write();
                let panel = state
                    .side_panels
                    .entry(extension_id.to_string())
                    .or_default();

                if let Some(path) = params.get("path").and_then(|v| v.as_str()) {
                    panel.path = Some(path.to_string());
                }
                if let Some(enabled) = params.get("enabled").and_then(|v| v.as_bool()) {
                    panel.options = Some(SidePanelOptions {
                        path: panel.path.clone(),
                        enabled: Some(enabled),
                    });
                }
                Ok(serde_json::json!({}))
            }
            "getOptions" => {
                let state = self.state.read();
                let panel = state.side_panels.get(extension_id);
                Ok(serde_json::json!({
                    "path": panel.and_then(|p| p.path.clone()),
                    "enabled": panel.and_then(|p| p.options.as_ref()).and_then(|o| o.enabled).unwrap_or(true)
                }))
            }
            "setPanelBehavior" => {
                // Store panel behavior settings
                Ok(serde_json::json!({}))
            }
            "getPanelBehavior" => Ok(serde_json::json!({
                "openPanelOnActionClick": true
            })),
            _ => Err(PluginError::command_not_found(&format!(
                "sidePanel.{}",
                method
            ))),
        }
    }

    /// Handle runtime API calls
    fn handle_runtime_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "getURL" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let clean_path = path.trim_start_matches('/');
                let url = format!(
                    "https://auroraview.localhost/extension/{}/{}",
                    extension_id, clean_path
                );
                Ok(serde_json::json!(url))
            }
            "getManifest" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    if let Some(manifest) = &ext.manifest {
                        return Ok(manifest.clone());
                    }
                    // Return basic manifest info
                    return Ok(serde_json::json!({
                        "manifest_version": 3,
                        "name": ext.name,
                        "version": ext.version,
                        "description": ext.description,
                        "permissions": ext.permissions
                    }));
                }
                Ok(serde_json::json!({}))
            }
            "getPlatformInfo" => {
                let platform = if cfg!(target_os = "windows") {
                    "win"
                } else if cfg!(target_os = "macos") {
                    "mac"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else {
                    "unknown"
                };

                let arch = if cfg!(target_arch = "x86_64") {
                    "x86-64"
                } else if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else if cfg!(target_arch = "x86") {
                    "x86-32"
                } else {
                    "unknown"
                };

                Ok(serde_json::json!({
                    "os": platform,
                    "arch": arch,
                    "nacl_arch": arch
                }))
            }
            "sendMessage" => {
                let message = params.get("message").cloned().unwrap_or(Value::Null);
                // TODO: Implement message routing to background/content scripts
                // For now, just acknowledge the message
                Ok(message)
            }
            "connect" => {
                let port_id = params.get("portId").and_then(|v| v.as_str());
                let name = params.get("name").and_then(|v| v.as_str());
                // TODO: Implement port connections
                Ok(serde_json::json!({
                    "portId": port_id,
                    "name": name
                }))
            }
            "portPostMessage" | "portDisconnect" => {
                // TODO: Implement port messaging
                Ok(serde_json::json!({}))
            }
            "openOptionsPage" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    if let Some(options_page) = &ext.options_page {
                        // TODO: Open options page
                        tracing::info!("Opening options page: {}", options_page);
                    }
                }
                Ok(serde_json::json!({}))
            }
            "setUninstallURL" => {
                // Store uninstall URL (not implemented)
                Ok(serde_json::json!({}))
            }
            "reload" => {
                // TODO: Implement extension reload
                Ok(serde_json::json!({}))
            }
            "requestUpdateCheck" => Ok(serde_json::json!({
                "status": "no_update"
            })),
            "getContexts" => {
                // Return current contexts
                Ok(serde_json::json!([{
                    "contextType": "SIDE_PANEL",
                    "documentId": "main",
                    "documentOrigin": format!("https://auroraview.localhost/extension/{}", extension_id),
                    "documentUrl": format!("https://auroraview.localhost/extension/{}/sidepanel.html", extension_id)
                }]))
            }
            "sendMessageResponse" => {
                // Handle response to a message
                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "runtime.{}",
                method
            ))),
        }
    }

    /// Handle action API calls
    fn handle_action_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "setTitle" => {
                let title = params.get("title").and_then(|v| v.as_str());
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.title = title.map(String::from);
                }
                Ok(serde_json::json!({}))
            }
            "getTitle" => {
                let state = self.state.read();
                let title = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.title.clone())
                    .unwrap_or_default();
                Ok(serde_json::json!(title))
            }
            "setBadgeText" => {
                let text = params.get("text").and_then(|v| v.as_str());
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.badge_text = text.map(String::from);
                }
                Ok(serde_json::json!({}))
            }
            "getBadgeText" => {
                let state = self.state.read();
                let text = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.badge_text.clone())
                    .unwrap_or_default();
                Ok(serde_json::json!(text))
            }
            "setBadgeBackgroundColor" => {
                let color = params.get("color");
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.badge_background_color = color.map(|c| c.to_string());
                }
                Ok(serde_json::json!({}))
            }
            "getBadgeBackgroundColor" => {
                let state = self.state.read();
                let color = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.badge_background_color.clone())
                    .unwrap_or_else(|| "[0, 0, 0, 255]".to_string());
                Ok(serde_json::json!(color))
            }
            "setBadgeTextColor" => {
                let color = params.get("color");
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.badge_text_color = color.map(|c| c.to_string());
                }
                Ok(serde_json::json!({}))
            }
            "getBadgeTextColor" => {
                let state = self.state.read();
                let color = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.badge_text_color.clone())
                    .unwrap_or_else(|| "[255, 255, 255, 255]".to_string());
                Ok(serde_json::json!(color))
            }
            "setPopup" => {
                let popup = params.get("popup").and_then(|v| v.as_str());
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.popup = popup.map(String::from);
                }
                Ok(serde_json::json!({}))
            }
            "getPopup" => {
                let state = self.state.read();
                let popup = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.popup.clone())
                    .unwrap_or_default();
                Ok(serde_json::json!(popup))
            }
            "setIcon" => {
                let icon = params.get("imageData").or_else(|| params.get("path"));
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.icon = icon.cloned();
                }
                Ok(serde_json::json!({}))
            }
            "enable" => {
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.enabled = true;
                }
                Ok(serde_json::json!({}))
            }
            "disable" => {
                let mut state = self.state.write();
                if let Some(action) = state.actions.get_mut(extension_id) {
                    action.enabled = false;
                }
                Ok(serde_json::json!({}))
            }
            "isEnabled" => {
                let state = self.state.read();
                let enabled = state
                    .actions
                    .get(extension_id)
                    .map(|a| a.enabled)
                    .unwrap_or(true);
                Ok(serde_json::json!(enabled))
            }
            "openPopup" => {
                // TODO: Implement popup opening
                Ok(serde_json::json!({}))
            }
            "getUserSettings" => Ok(serde_json::json!({
                "isOnToolbar": true
            })),
            _ => Err(PluginError::command_not_found(&format!(
                "action.{}",
                method
            ))),
        }
    }

    /// Handle scripting API calls
    fn handle_scripting_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "executeScript" => {
                // TODO: Implement actual script execution
                let func = params.get("func");
                let files = params.get("files");

                tracing::info!(
                    "scripting.executeScript: func={:?}, files={:?}",
                    func.is_some(),
                    files
                );

                Ok(serde_json::json!([{ "frameId": 0, "result": null }]))
            }
            "insertCSS" => {
                // TODO: Implement CSS injection
                Ok(serde_json::json!({}))
            }
            "removeCSS" => {
                // TODO: Implement CSS removal
                Ok(serde_json::json!({}))
            }
            "registerContentScripts" => {
                let scripts: Vec<ContentScriptInfo> = params
                    .get("scripts")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let mut state = self.state.write();
                let ext_scripts = state
                    .content_scripts
                    .entry(extension_id.to_string())
                    .or_default();
                ext_scripts.extend(scripts);

                Ok(serde_json::json!({}))
            }
            "unregisterContentScripts" => {
                let ids: Vec<String> = params
                    .get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let mut state = self.state.write();
                if let Some(ext_scripts) = state.content_scripts.get_mut(extension_id) {
                    if ids.is_empty() {
                        ext_scripts.clear();
                    } else {
                        ext_scripts.retain(|s| !ids.contains(&s.id));
                    }
                }

                Ok(serde_json::json!({}))
            }
            "getRegisteredContentScripts" => {
                let state = self.state.read();
                let scripts = state
                    .content_scripts
                    .get(extension_id)
                    .cloned()
                    .unwrap_or_default();
                Ok(serde_json::to_value(scripts).unwrap())
            }
            "updateContentScripts" => {
                // TODO: Implement script updates
                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "scripting.{}",
                method
            ))),
        }
    }

    /// Handle alarms API calls
    fn handle_alarms_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let alarm_info = params.get("alarmInfo").cloned().unwrap_or(params.clone());

                let delay_in_minutes = alarm_info
                    .get("delayInMinutes")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let period_in_minutes = alarm_info.get("periodInMinutes").and_then(|v| v.as_f64());
                let when = alarm_info.get("when").and_then(|v| v.as_f64());

                let scheduled_time = when.unwrap_or_else(|| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as f64;
                    now + (delay_in_minutes * 60.0 * 1000.0)
                });

                let alarm = AlarmInfo {
                    name: name.clone(),
                    scheduled_time,
                    period_in_minutes,
                };

                let mut state = self.state.write();
                let ext_alarms = state.alarms.entry(extension_id.to_string()).or_default();
                ext_alarms.insert(name, alarm);

                Ok(serde_json::json!({}))
            }
            "get" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let state = self.state.read();
                let alarm = state
                    .alarms
                    .get(extension_id)
                    .and_then(|a| a.get(name))
                    .cloned();
                Ok(serde_json::to_value(alarm).unwrap())
            }
            "getAll" => {
                let state = self.state.read();
                let alarms: Vec<AlarmInfo> = state
                    .alarms
                    .get(extension_id)
                    .map(|a| a.values().cloned().collect())
                    .unwrap_or_default();
                Ok(serde_json::to_value(alarms).unwrap())
            }
            "clear" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let mut state = self.state.write();
                let cleared = state
                    .alarms
                    .get_mut(extension_id)
                    .map(|a| a.remove(name).is_some())
                    .unwrap_or(false);
                Ok(serde_json::json!(cleared))
            }
            "clearAll" => {
                let mut state = self.state.write();
                let cleared = state
                    .alarms
                    .get_mut(extension_id)
                    .map(|a| {
                        let had_alarms = !a.is_empty();
                        a.clear();
                        had_alarms
                    })
                    .unwrap_or(false);
                Ok(serde_json::json!(cleared))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "alarms.{}",
                method
            ))),
        }
    }

    /// Handle notifications API calls
    fn handle_notifications_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| {
                        format!(
                            "notif_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        )
                    });

                let options = params.get("options").cloned().unwrap_or(params.clone());
                let title = options
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let message = options
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let icon_url = options
                    .get("iconUrl")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let notification_type = options
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("basic")
                    .to_string();

                let notification = NotificationInfo {
                    id: id.clone(),
                    title,
                    message,
                    icon_url,
                    notification_type,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64,
                };

                // TODO: Show actual system notification
                tracing::info!("Creating notification: {:?}", notification);

                let mut state = self.state.write();
                let ext_notifs = state
                    .notifications
                    .entry(extension_id.to_string())
                    .or_default();
                ext_notifs.insert(id.clone(), notification);

                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("notificationId is required"))?;

                let state = self.state.read();
                let exists = state
                    .notifications
                    .get(extension_id)
                    .map(|n| n.contains_key(id))
                    .unwrap_or(false);

                Ok(serde_json::json!(exists))
            }
            "clear" => {
                let id = params
                    .get("notificationId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("notificationId is required"))?;

                let mut state = self.state.write();
                let cleared = state
                    .notifications
                    .get_mut(extension_id)
                    .map(|n| n.remove(id).is_some())
                    .unwrap_or(false);

                Ok(serde_json::json!(cleared))
            }
            "getAll" => {
                let state = self.state.read();
                let notifs: HashMap<String, bool> = state
                    .notifications
                    .get(extension_id)
                    .map(|n| n.keys().map(|k| (k.clone(), true)).collect())
                    .unwrap_or_default();
                Ok(serde_json::to_value(notifs).unwrap())
            }
            "getPermissionLevel" => Ok(serde_json::json!("granted")),
            _ => Err(PluginError::command_not_found(&format!(
                "notifications.{}",
                method
            ))),
        }
    }

    /// Handle contextMenus API calls
    fn handle_context_menus_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| {
                        format!(
                            "menu_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        )
                    });

                let menu_item = MenuItemInfo {
                    id: id.clone(),
                    title: params
                        .get("title")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    item_type: params
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("normal")
                        .to_string(),
                    contexts: params
                        .get("contexts")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_else(|| vec!["page".to_string()]),
                    parent_id: params
                        .get("parentId")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    enabled: params
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    visible: params
                        .get("visible")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                };

                let mut state = self.state.write();
                let ext_menus = state
                    .context_menus
                    .entry(extension_id.to_string())
                    .or_default();
                ext_menus.insert(id.clone(), menu_item);

                Ok(serde_json::json!(id))
            }
            "update" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("id is required"))?;

                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    if let Some(menu) = ext_menus.get_mut(id) {
                        if let Some(title) = params
                            .get("updateProperties")
                            .and_then(|u| u.get("title"))
                            .and_then(|v| v.as_str())
                        {
                            menu.title = Some(title.to_string());
                        }
                    }
                }

                Ok(serde_json::json!({}))
            }
            "remove" => {
                let id = params
                    .get("menuItemId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("menuItemId is required"))?;

                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    ext_menus.remove(id);
                }

                Ok(serde_json::json!({}))
            }
            "removeAll" => {
                let mut state = self.state.write();
                if let Some(ext_menus) = state.context_menus.get_mut(extension_id) {
                    ext_menus.clear();
                }

                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "contextMenus.{}",
                method
            ))),
        }
    }

    /// Handle windows API calls
    fn handle_windows_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        // AuroraView has a single window
        let default_window = serde_json::json!({
            "id": 1,
            "focused": true,
            "top": 0,
            "left": 0,
            "width": 1920,
            "height": 1080,
            "incognito": false,
            "type": "normal",
            "state": "normal",
            "alwaysOnTop": false
        });

        match method {
            "get" | "getCurrent" | "getLastFocused" => Ok(default_window),
            "getAll" => Ok(serde_json::json!([default_window])),
            "create" => {
                // TODO: Implement window creation
                Ok(default_window)
            }
            "update" | "remove" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!(
                "windows.{}",
                method
            ))),
        }
    }

    /// Handle commands API calls
    fn handle_commands_api(
        &self,
        extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "getAll" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    if let Some(manifest) = &ext.manifest {
                        if let Some(commands) = manifest.get("commands") {
                            return Ok(commands.clone());
                        }
                    }
                }
                Ok(serde_json::json!([]))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "commands.{}",
                method
            ))),
        }
    }

    /// Handle permissions API calls
    fn handle_permissions_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "contains" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    let requested: Vec<String> = params
                        .get("permissions")
                        .and_then(|p| p.get("permissions"))
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();

                    let has_all = requested.iter().all(|p| ext.permissions.contains(p));
                    return Ok(serde_json::json!(has_all));
                }
                Ok(serde_json::json!(false))
            }
            "getAll" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    return Ok(serde_json::json!({
                        "permissions": ext.permissions,
                        "origins": ext.host_permissions
                    }));
                }
                Ok(serde_json::json!({ "permissions": [], "origins": [] }))
            }
            "request" => {
                // Auto-grant permissions in AuroraView
                Ok(serde_json::json!(true))
            }
            "remove" => {
                // Cannot remove permissions in AuroraView
                Ok(serde_json::json!(false))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "permissions.{}",
                method
            ))),
        }
    }

    /// Handle identity API calls
    fn handle_identity_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "getAuthToken" => {
                // TODO: Implement OAuth flow
                Err(PluginError::shell_error("OAuth not implemented"))
            }
            "removeCachedAuthToken" => Ok(serde_json::json!({})),
            "launchWebAuthFlow" => {
                let url = params.get("url").and_then(|v| v.as_str());
                if let Some(url) = url {
                    tracing::info!("launchWebAuthFlow: {}", url);
                    // TODO: Implement web auth flow
                }
                Err(PluginError::shell_error("Web auth flow not implemented"))
            }
            "getProfileUserInfo" => Ok(serde_json::json!({
                "email": "",
                "id": ""
            })),
            "getRedirectURL" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                Ok(serde_json::json!(format!(
                    "https://auroraview.localhost/oauth/{}/{}",
                    extension_id, path
                )))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "identity.{}",
                method
            ))),
        }
    }

    /// Handle webRequest API calls
    fn handle_web_request_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "addListener" | "removeListener" => {
                // TODO: Implement request interception
                Ok(serde_json::json!({}))
            }
            "handlerBehaviorChanged" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!(
                "webRequest.{}",
                method
            ))),
        }
    }

    /// Handle declarativeNetRequest API calls
    fn handle_declarative_net_request_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "updateDynamicRules" | "updateSessionRules" | "updateEnabledRulesets" => {
                Ok(serde_json::json!({}))
            }
            "getDynamicRules" | "getSessionRules" => Ok(serde_json::json!([])),
            "getEnabledRulesets" => Ok(serde_json::json!([])),
            "getAvailableStaticRuleCount" => Ok(serde_json::json!(30000)),
            "isRegexSupported" => Ok(serde_json::json!({ "isSupported": true })),
            _ => Err(PluginError::command_not_found(&format!(
                "declarativeNetRequest.{}",
                method
            ))),
        }
    }

    /// Handle offscreen API calls
    fn handle_offscreen_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "createDocument" => Ok(serde_json::json!({})),
            "closeDocument" => Ok(serde_json::json!({})),
            "hasDocument" => Ok(serde_json::json!(false)),
            _ => Err(PluginError::command_not_found(&format!(
                "offscreen.{}",
                method
            ))),
        }
    }

    /// Handle management API calls
    fn handle_management_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        let state = self.state.read();

        match method {
            "getAll" => {
                // Return all extensions
                let extensions: Vec<serde_json::Value> = state
                    .extensions
                    .values()
                    .map(|ext| {
                        serde_json::json!({
                            "id": ext.id,
                            "name": ext.name,
                            "version": ext.version,
                            "description": ext.description,
                            "enabled": ext.enabled,
                            "mayDisable": true,
                            "mayEnable": true,
                            "isApp": false,
                            "type": "extension",
                            "offlineEnabled": true,
                            "permissions": ext.permissions,
                            "hostPermissions": ext.host_permissions,
                            "installType": "development"
                        })
                    })
                    .collect();
                Ok(serde_json::to_value(extensions).unwrap())
            }
            "get" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                match state.extensions.get(id) {
                    Some(ext) => Ok(serde_json::json!({
                        "id": ext.id,
                        "name": ext.name,
                        "version": ext.version,
                        "description": ext.description,
                        "enabled": ext.enabled,
                        "mayDisable": true,
                        "mayEnable": true,
                        "isApp": false,
                        "type": "extension",
                        "offlineEnabled": true,
                        "permissions": ext.permissions,
                        "hostPermissions": ext.host_permissions,
                        "installType": "development"
                    })),
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        id
                    ))),
                }
            }
            "getSelf" => match state.extensions.get(extension_id) {
                Some(ext) => Ok(serde_json::json!({
                    "id": ext.id,
                    "name": ext.name,
                    "version": ext.version,
                    "description": ext.description,
                    "enabled": ext.enabled,
                    "mayDisable": true,
                    "mayEnable": true,
                    "isApp": false,
                    "type": "extension",
                    "offlineEnabled": true,
                    "permissions": ext.permissions,
                    "hostPermissions": ext.host_permissions,
                    "installType": "development"
                })),
                None => Err(PluginError::invalid_args(format!(
                    "Extension not found: {}",
                    extension_id
                ))),
            },
            "setEnabled" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;
                let enabled = params
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| PluginError::invalid_args("Missing enabled"))?;

                drop(state);
                let mut state = self.state.write();

                // Update if extension exists in our state, otherwise just return success
                // (the extension might be managed by WebView2 directly)
                if let Some(ext) = state.extensions.get_mut(id) {
                    ext.enabled = enabled;
                }

                // Always return success - the actual enabled state is managed by the frontend
                Ok(serde_json::json!(null))
            }
            "getPermissionWarningsById" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                match state.extensions.get(id) {
                    Some(ext) => {
                        // Generate permission warnings based on permissions
                        let warnings: Vec<String> = ext
                            .permissions
                            .iter()
                            .filter_map(|p| match p.as_str() {
                                "tabs" => Some("Read your browsing history".to_string()),
                                "history" => {
                                    Some("Read and change your browsing history".to_string())
                                }
                                "downloads" => Some("Manage your downloads".to_string()),
                                "bookmarks" => Some("Read and change your bookmarks".to_string()),
                                "cookies" => {
                                    Some("Read and change all your data on websites".to_string())
                                }
                                "storage" => Some("Store data in this application".to_string()),
                                _ => None,
                            })
                            .collect();
                        Ok(serde_json::to_value(warnings).unwrap())
                    }
                    // Return empty array if extension not found in our state
                    // (it might be managed by WebView2 directly)
                    None => Ok(serde_json::json!([])),
                }
            }
            "uninstall" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::invalid_args("Missing id"))?;

                drop(state);
                let mut state = self.state.write();

                if state.extensions.remove(id).is_some() {
                    Ok(serde_json::json!(null))
                } else {
                    Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        id
                    )))
                }
            }
            "uninstallSelf" => {
                drop(state);
                let mut state = self.state.write();
                state.extensions.remove(extension_id);
                Ok(serde_json::json!(null))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "management.{}",
                method
            ))),
        }
    }
}

impl Default for ExtensionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHandler for ExtensionsPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "api_call" => {
                let req: ApiCallRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Route to appropriate API handler
                match req.api.as_str() {
                    "storage" => {
                        self.handle_storage_api(&req.extension_id, &req.method, &req.params)
                    }
                    "tabs" => self.handle_tabs_api(&req.extension_id, &req.method, &req.params),
                    "sidePanel" => {
                        self.handle_side_panel_api(&req.extension_id, &req.method, &req.params)
                    }
                    "runtime" => {
                        self.handle_runtime_api(&req.extension_id, &req.method, &req.params)
                    }
                    "action" => self.handle_action_api(&req.extension_id, &req.method, &req.params),
                    "scripting" => {
                        self.handle_scripting_api(&req.extension_id, &req.method, &req.params)
                    }
                    "alarms" => self.handle_alarms_api(&req.extension_id, &req.method, &req.params),
                    "notifications" => {
                        self.handle_notifications_api(&req.extension_id, &req.method, &req.params)
                    }
                    "contextMenus" => {
                        self.handle_context_menus_api(&req.extension_id, &req.method, &req.params)
                    }
                    "windows" => {
                        self.handle_windows_api(&req.extension_id, &req.method, &req.params)
                    }
                    "commands" => {
                        self.handle_commands_api(&req.extension_id, &req.method, &req.params)
                    }
                    "permissions" => {
                        self.handle_permissions_api(&req.extension_id, &req.method, &req.params)
                    }
                    "identity" => {
                        self.handle_identity_api(&req.extension_id, &req.method, &req.params)
                    }
                    "webRequest" => {
                        self.handle_web_request_api(&req.extension_id, &req.method, &req.params)
                    }
                    "declarativeNetRequest" => self.handle_declarative_net_request_api(
                        &req.extension_id,
                        &req.method,
                        &req.params,
                    ),
                    "offscreen" => {
                        self.handle_offscreen_api(&req.extension_id, &req.method, &req.params)
                    }
                    "management" => {
                        self.handle_management_api(&req.extension_id, &req.method, &req.params)
                    }
                    _ => Err(PluginError::command_not_found(&format!(
                        "Unknown API: {}",
                        req.api
                    ))),
                }
            }
            "list_extensions" => {
                let state = self.state.read();
                let extensions: Vec<&ExtensionInfo> = state.extensions.values().collect();
                Ok(serde_json::to_value(extensions).unwrap())
            }
            "get_extension" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                match state.extensions.get(&req.extension_id) {
                    Some(ext) => Ok(serde_json::to_value(ext).unwrap()),
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        req.extension_id
                    ))),
                }
            }
            "get_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                match state.extensions.get(&req.extension_id) {
                    Some(ext) => {
                        if let Some(path) = &ext.side_panel_path {
                            let full_path = PathBuf::from(&ext.root_dir).join(path);
                            match std::fs::read_to_string(&full_path) {
                                Ok(html) => Ok(serde_json::json!({
                                    "html": html,
                                    "path": full_path.to_string_lossy()
                                })),
                                Err(e) => Err(PluginError::invalid_args(format!(
                                    "Failed to read side panel: {}",
                                    e
                                ))),
                            }
                        } else {
                            Err(PluginError::invalid_args("Extension has no side panel"))
                        }
                    }
                    None => Err(PluginError::invalid_args(format!(
                        "Extension not found: {}",
                        req.extension_id
                    ))),
                }
            }
            "open_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut state = self.state.write();
                let panel = state
                    .side_panels
                    .entry(req.extension_id.clone())
                    .or_default();
                panel.is_open = true;

                Ok(serde_json::json!({ "success": true }))
            }
            "close_side_panel" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut state = self.state.write();
                if let Some(panel) = state.side_panels.get_mut(&req.extension_id) {
                    panel.is_open = false;
                }

                Ok(serde_json::json!({ "success": true }))
            }
            "get_side_panel_state" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let state = self.state.read();
                let panel_state = state
                    .side_panels
                    .get(&req.extension_id)
                    .cloned()
                    .unwrap_or_default();

                Ok(serde_json::to_value(panel_state).unwrap())
            }
            "get_polyfill" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // Get extension info for path and manifest
                let state = self.state.read();
                let (extension_path, manifest) = state
                    .extensions
                    .get(&req.extension_id)
                    .map(|ext| (ext.root_dir.clone(), ext.manifest.clone()))
                    .unwrap_or_else(|| (String::new(), None));
                drop(state);

                // Generate the polyfill script using SDK
                let polyfill = auroraview_extensions::generate_polyfill_from_sdk(
                    &req.extension_id.clone(),
                    &extension_path,
                    manifest.as_ref(),
                    None, // messages - TODO: load from _locales if needed
                );
                let wxt_shim = auroraview_extensions::generate_wxt_shim();

                Ok(serde_json::json!({
                    "polyfill": polyfill,
                    "wxtShim": wxt_shim
                }))
            }
            "dispatch_event" => {
                let req: EventDispatchRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                // TODO: Implement event dispatching to extension
                tracing::info!(
                    "Dispatching event {}.{} to {}",
                    req.api,
                    req.event,
                    req.extension_id
                );

                Ok(serde_json::json!({ "success": true }))
            }
            // ============================================================
            // Extension View Management (Chrome DevTools-like)
            // ============================================================
            "create_view" => {
                let req: CreateViewRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                let config = auroraview_extensions::ExtensionViewConfig {
                    extension_id: req.extension_id,
                    view_type: req.view_type.into(),
                    html_path: req.html_path,
                    title: req.title.unwrap_or_else(|| "Extension View".to_string()),
                    width: req.width.unwrap_or(400),
                    height: req.height.unwrap_or(600),
                    dev_tools: req.dev_tools.unwrap_or(true),
                    debug_port: req.debug_port,
                    visible: req.visible.unwrap_or(true),
                    parent_hwnd: req.parent_hwnd,
                };

                match view_manager.create_view(config) {
                    Ok(info) => Ok(serde_json::to_value(info).unwrap()),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "get_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.get_view(&req.view_id) {
                    Some(info) => Ok(serde_json::to_value(info).unwrap()),
                    None => Err(PluginError::invalid_args(format!(
                        "View not found: {}",
                        req.view_id
                    ))),
                }
            }
            "get_extension_views" => {
                let req: ExtensionIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                let views = view_manager.get_extension_views(&req.extension_id);
                Ok(serde_json::to_value(views).unwrap())
            }
            "get_all_views" => {
                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                let views = view_manager.get_all_views();
                Ok(serde_json::to_value(views).unwrap())
            }
            "open_devtools" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.open_devtools(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "close_devtools" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.close_devtools(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "show_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.show_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "hide_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.hide_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "destroy_view" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.destroy_view(&req.view_id) {
                    Ok(()) => Ok(serde_json::json!({ "success": true })),
                    Err(e) => Err(PluginError::from_plugin("extensions", e)),
                }
            }
            "get_cdp_info" => {
                let req: ViewIdRequest = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                match view_manager.get_cdp_info(&req.view_id) {
                    Some(info) => Ok(serde_json::to_value(info).unwrap()),
                    None => Err(PluginError::invalid_args(format!(
                        "View not found: {}",
                        req.view_id
                    ))),
                }
            }
            "get_all_cdp_connections" => {
                let view_manager = auroraview_extensions::ExtensionViewManager::global();
                let connections = view_manager.get_all_cdp_connections();
                Ok(serde_json::to_value(connections).unwrap())
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "api_call",
            "list_extensions",
            "get_extension",
            "get_side_panel",
            "open_side_panel",
            "close_side_panel",
            "get_side_panel_state",
            "get_polyfill",
            "dispatch_event",
            // Extension View Management (Chrome DevTools-like)
            "create_view",
            "get_view",
            "get_extension_views",
            "get_all_views",
            "open_devtools",
            "close_devtools",
            "show_view",
            "hide_view",
            "destroy_view",
            "get_cdp_info",
            "get_all_cdp_connections",
        ]
    }
}
