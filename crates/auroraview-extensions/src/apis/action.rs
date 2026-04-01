//! chrome.action API handler
//!
//! Implements the Action API (toolbar button) for extensions.
//! In AuroraView, actions can be triggered programmatically or through UI.

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Action state for a tab
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActionState {
    /// Badge text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_text: Option<String>,
    /// Badge background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_background_color: Option<String>,
    /// Badge text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_text_color: Option<String>,
    /// Title (tooltip)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Icon path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Popup path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popup: Option<String>,
    /// Whether the action is enabled
    pub enabled: bool,
}

/// Action manager
pub struct ActionManager {
    /// Global state per extension
    global_states: RwLock<std::collections::HashMap<ExtensionId, ActionState>>,
    /// Per-tab state per extension
    tab_states:
        RwLock<std::collections::HashMap<ExtensionId, std::collections::HashMap<i32, ActionState>>>,
    /// Callback for action clicks
    #[allow(clippy::type_complexity)]
    on_clicked: RwLock<Option<Box<dyn Fn(&str, i32) + Send + Sync>>>,
    /// Callback for opening the popup
    #[allow(clippy::type_complexity)]
    on_open_popup: RwLock<Option<Box<dyn Fn(&str, Option<&str>) + Send + Sync>>>,
}

impl ActionManager {
    /// Create a new action manager
    pub fn new() -> Self {
        Self {
            global_states: RwLock::new(std::collections::HashMap::new()),
            tab_states: RwLock::new(std::collections::HashMap::new()),
            on_clicked: RwLock::new(None),
            on_open_popup: RwLock::new(None),
        }
    }

    /// Set the click callback
    pub fn set_on_clicked<F>(&self, callback: F)
    where
        F: Fn(&str, i32) + Send + Sync + 'static,
    {
        let mut on_clicked = self.on_clicked.write();
        *on_clicked = Some(Box::new(callback));
    }

    /// Set the popup open callback
    pub fn set_on_open_popup<F>(&self, callback: F)
    where
        F: Fn(&str, Option<&str>) + Send + Sync + 'static,
    {
        let mut cb = self.on_open_popup.write();
        *cb = Some(Box::new(callback));
    }

    /// Trigger action click
    pub fn trigger_click(&self, extension_id: &str, tab_id: i32) {
        let on_clicked = self.on_clicked.read();
        if let Some(callback) = on_clicked.as_ref() {
            callback(extension_id, tab_id);
        }
    }

    /// Open the popup for an extension
    pub fn open_popup(&self, extension_id: &str) {
        let state = self.get_state(extension_id, None);
        let popup_path = state.popup.as_deref();
        let cb = self.on_open_popup.read();
        if let Some(callback) = cb.as_ref() {
            callback(extension_id, popup_path);
        } else {
            tracing::debug!(
                "No popup callback registered for extension {}",
                extension_id
            );
        }
    }

    /// Get effective state for a tab (merges global and tab-specific state)
    pub fn get_state(&self, extension_id: &str, tab_id: Option<i32>) -> ActionState {
        let global_states = self.global_states.read();
        let mut state = global_states
            .get(extension_id)
            .cloned()
            .unwrap_or(ActionState {
                enabled: true,
                ..Default::default()
            });

        // Merge tab-specific state if available
        if let Some(tab_id) = tab_id {
            let tab_states = self.tab_states.read();
            if let Some(ext_tabs) = tab_states.get(extension_id) {
                if let Some(tab_state) = ext_tabs.get(&tab_id) {
                    // Override with tab-specific values
                    if tab_state.badge_text.is_some() {
                        state.badge_text = tab_state.badge_text.clone();
                    }
                    if tab_state.badge_background_color.is_some() {
                        state.badge_background_color = tab_state.badge_background_color.clone();
                    }
                    if tab_state.badge_text_color.is_some() {
                        state.badge_text_color = tab_state.badge_text_color.clone();
                    }
                    if tab_state.title.is_some() {
                        state.title = tab_state.title.clone();
                    }
                    if tab_state.icon.is_some() {
                        state.icon = tab_state.icon.clone();
                    }
                    if tab_state.popup.is_some() {
                        state.popup = tab_state.popup.clone();
                    }
                }
            }
        }

        state
    }

    /// Update global state
    pub fn update_global_state<F>(&self, extension_id: &str, updater: F)
    where
        F: FnOnce(&mut ActionState),
    {
        let mut states = self.global_states.write();
        let state = states
            .entry(extension_id.to_string())
            .or_insert(ActionState {
                enabled: true,
                ..Default::default()
            });
        updater(state);
    }

    /// Update tab-specific state
    pub fn update_tab_state<F>(&self, extension_id: &str, tab_id: i32, updater: F)
    where
        F: FnOnce(&mut ActionState),
    {
        let mut tab_states = self.tab_states.write();
        let ext_tabs = tab_states.entry(extension_id.to_string()).or_default();
        let state = ext_tabs.entry(tab_id).or_default();
        updater(state);
    }
}

impl Default for ActionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Action API handler
pub struct ActionApiHandler {
    manager: Arc<ActionManager>,
}

impl ActionApiHandler {
    /// Create a new action API handler
    pub fn new(manager: Arc<ActionManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for ActionApiHandler {
    fn namespace(&self) -> &str {
        "action"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        let tab_id: Option<i32> = params
            .get("tabId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        match method {
            "setTitle" => {
                let title = params
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.title = title;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.title = title;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "getTitle" => {
                let state = self.manager.get_state(extension_id, tab_id);
                Ok(serde_json::json!(state.title.unwrap_or_default()))
            }
            "setBadgeText" => {
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.badge_text = text;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.badge_text = text;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "getBadgeText" => {
                let state = self.manager.get_state(extension_id, tab_id);
                Ok(serde_json::json!(state.badge_text.unwrap_or_default()))
            }
            "setBadgeBackgroundColor" => {
                let color = params.get("color").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        // Convert [r, g, b, a] to rgba string
                        let values: Vec<u8> = arr
                            .iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                            .collect();
                        if values.len() >= 3 {
                            let a = values.get(3).copied().unwrap_or(255);
                            Some(format!(
                                "rgba({}, {}, {}, {})",
                                values[0],
                                values[1],
                                values[2],
                                a as f32 / 255.0
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.badge_background_color = color;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.badge_background_color = color;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "getBadgeBackgroundColor" => {
                let state = self.manager.get_state(extension_id, tab_id);
                Ok(serde_json::json!(state
                    .badge_background_color
                    .unwrap_or_default()))
            }
            "setPopup" => {
                let popup = params
                    .get("popup")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.popup = popup;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.popup = popup;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "getPopup" => {
                let state = self.manager.get_state(extension_id, tab_id);
                Ok(serde_json::json!(state.popup.unwrap_or_default()))
            }
            "enable" => {
                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.enabled = true;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.enabled = true;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "disable" => {
                if let Some(tab_id) = tab_id {
                    self.manager
                        .update_tab_state(extension_id, tab_id, |state| {
                            state.enabled = false;
                        });
                } else {
                    self.manager.update_global_state(extension_id, |state| {
                        state.enabled = false;
                    });
                }
                Ok(serde_json::json!({}))
            }
            "isEnabled" => {
                let state = self.manager.get_state(extension_id, tab_id);
                Ok(serde_json::json!(state.enabled))
            }
            "openPopup" => {
                self.manager.open_popup(extension_id);
                Ok(serde_json::json!({}))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "action.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec![
            "setTitle",
            "getTitle",
            "setBadgeText",
            "getBadgeText",
            "setBadgeBackgroundColor",
            "getBadgeBackgroundColor",
            "setPopup",
            "getPopup",
            "enable",
            "disable",
            "isEnabled",
            "openPopup",
        ]
    }
}
