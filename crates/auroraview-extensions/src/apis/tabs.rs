//! chrome.tabs API handler
//!
//! In AuroraView, there's typically only one "tab" (the main WebView).
//! This API provides compatibility with extensions that use tabs API.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    /// Tab ID
    pub id: i32,
    /// Window ID
    pub window_id: i32,
    /// Tab index in window
    pub index: i32,
    /// Whether tab is active
    pub active: bool,
    /// Whether tab is highlighted
    pub highlighted: bool,
    /// Whether tab is pinned
    pub pinned: bool,
    /// Tab URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Tab title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Favicon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fav_icon_url: Option<String>,
    /// Tab status
    pub status: TabStatus,
    /// Whether tab is incognito
    pub incognito: bool,
    /// Tab width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Tab height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

/// Tab loading status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TabStatus {
    /// Tab is loading
    Loading,
    /// Tab has finished loading
    #[default]
    Complete,
}

/// Tab query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TabQuery {
    /// Filter by active state
    #[serde(default)]
    pub active: Option<bool>,
    /// Filter by current window
    #[serde(default)]
    pub current_window: Option<bool>,
    /// Filter by URL pattern
    #[serde(default)]
    pub url: Option<String>,
    /// Filter by title
    #[serde(default)]
    pub title: Option<String>,
    /// Filter by window ID
    #[serde(default)]
    pub window_id: Option<i32>,
    /// Filter by status
    #[serde(default)]
    pub status: Option<TabStatus>,
}

/// Tab state manager
pub struct TabState {
    /// Current tab info
    current_tab: RwLock<Tab>,
    /// Tab ID counter
    #[allow(dead_code)]
    next_id: RwLock<i32>,
}

impl TabState {
    /// Create a new tab state
    pub fn new() -> Self {
        Self {
            current_tab: RwLock::new(Tab {
                id: 1,
                window_id: 1,
                index: 0,
                active: true,
                highlighted: true,
                pinned: false,
                url: None,
                title: None,
                fav_icon_url: None,
                status: TabStatus::Complete,
                incognito: false,
                width: None,
                height: None,
            }),
            next_id: RwLock::new(2),
        }
    }

    /// Update current tab URL
    pub fn set_url(&self, url: &str) {
        let mut tab = self.current_tab.write();
        tab.url = Some(url.to_string());
    }

    /// Update current tab title
    pub fn set_title(&self, title: &str) {
        let mut tab = self.current_tab.write();
        tab.title = Some(title.to_string());
    }

    /// Get current tab
    pub fn get_current(&self) -> Tab {
        self.current_tab.read().clone()
    }
}

impl Default for TabState {
    fn default() -> Self {
        Self::new()
    }
}

/// Tabs API handler
pub struct TabsApiHandler {
    state: Arc<TabState>,
}

impl TabsApiHandler {
    /// Create a new tabs API handler
    pub fn new(state: Arc<TabState>) -> Self {
        Self { state }
    }
}

impl ApiHandler for TabsApiHandler {
    fn namespace(&self) -> &str {
        "tabs"
    }

    fn handle(&self, method: &str, params: Value, _extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "query" => {
                let _query: TabQuery = serde_json::from_value(params).unwrap_or_default();

                // In AuroraView, we only have one "tab"
                let tab = self.state.get_current();

                // Return array of matching tabs
                Ok(serde_json::json!([tab]))
            }
            "get" => {
                let tab_id: i32 = params
                    .get("tabId")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("tabId is required".to_string())
                    })?;

                let tab = self.state.get_current();
                if tab.id == tab_id {
                    Ok(serde_json::to_value(tab)?)
                } else {
                    Err(ExtensionError::NotFound(format!(
                        "Tab {} not found",
                        tab_id
                    )))
                }
            }
            "getCurrent" => {
                let tab = self.state.get_current();
                Ok(serde_json::to_value(tab)?)
            }
            "create" => {
                // In AuroraView, we can't really create new tabs
                // But we can navigate the current view
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(url) = url {
                    self.state.set_url(&url);
                    // TODO: Actually navigate the WebView
                }

                let tab = self.state.get_current();
                Ok(serde_json::to_value(tab)?)
            }
            "update" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(url) = url {
                    self.state.set_url(&url);
                    // TODO: Actually navigate the WebView
                }

                let tab = self.state.get_current();
                Ok(serde_json::to_value(tab)?)
            }
            "remove" => {
                // Can't remove tabs in AuroraView
                Ok(serde_json::json!({}))
            }
            "sendMessage" => {
                // TODO: Implement message passing to content scripts
                let _tab_id: i32 = params
                    .get("tabId")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("tabId is required".to_string())
                    })?;

                let _message = params.get("message").cloned().unwrap_or(Value::Null);

                // For now, just return undefined (no response)
                Ok(Value::Null)
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "tabs.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec![
            "query",
            "get",
            "getCurrent",
            "create",
            "update",
            "remove",
            "sendMessage",
        ]
    }
}
