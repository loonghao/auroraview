//! chrome.sidePanel API handler
//!
//! Implements the Side Panel API for AuroraView.
//! In AuroraView, the side panel is rendered in a separate WebView
//! that can be shown/hidden by the host application.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Side panel options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SidePanelOptions {
    /// Tab ID (optional, for tab-specific options)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i32>,
    /// Path to the side panel HTML
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Whether the panel is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Side panel behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PanelBehavior {
    /// Whether clicking the action opens the panel
    #[serde(default)]
    pub open_panel_on_action_click: bool,
}

/// Side panel state for an extension
#[derive(Debug, Clone, Default)]
pub struct SidePanelState {
    /// Current options
    pub options: SidePanelOptions,
    /// Current behavior
    pub behavior: PanelBehavior,
    /// Whether the panel is currently open
    pub is_open: bool,
}

/// Side panel manager
pub struct SidePanelManager {
    /// State per extension
    states: RwLock<std::collections::HashMap<ExtensionId, SidePanelState>>,
    /// Callback for when panel should open
    #[allow(clippy::type_complexity)]
    on_open: RwLock<Option<Box<dyn Fn(&str) + Send + Sync>>>,
    /// Callback for when panel should close
    #[allow(clippy::type_complexity)]
    on_close: RwLock<Option<Box<dyn Fn(&str) + Send + Sync>>>,
}

impl SidePanelManager {
    /// Create a new side panel manager
    pub fn new() -> Self {
        Self {
            states: RwLock::new(std::collections::HashMap::new()),
            on_open: RwLock::new(None),
            on_close: RwLock::new(None),
        }
    }

    /// Set the open callback
    pub fn set_on_open<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut on_open = self.on_open.write();
        *on_open = Some(Box::new(callback));
    }

    /// Set the close callback
    pub fn set_on_close<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut on_close = self.on_close.write();
        *on_close = Some(Box::new(callback));
    }

    /// Open the side panel for an extension
    pub fn open(&self, extension_id: &str) {
        {
            let mut states = self.states.write();
            let state = states.entry(extension_id.to_string()).or_default();
            state.is_open = true;
        }

        let on_open = self.on_open.read();
        if let Some(callback) = on_open.as_ref() {
            callback(extension_id);
        }
    }

    /// Close the side panel for an extension
    pub fn close(&self, extension_id: &str) {
        {
            let mut states = self.states.write();
            if let Some(state) = states.get_mut(extension_id) {
                state.is_open = false;
            }
        }

        let on_close = self.on_close.read();
        if let Some(callback) = on_close.as_ref() {
            callback(extension_id);
        }
    }

    /// Check if panel is open
    pub fn is_open(&self, extension_id: &str) -> bool {
        let states = self.states.read();
        states.get(extension_id).map(|s| s.is_open).unwrap_or(false)
    }

    /// Get options for an extension
    pub fn get_options(&self, extension_id: &str) -> SidePanelOptions {
        let states = self.states.read();
        states
            .get(extension_id)
            .map(|s| s.options.clone())
            .unwrap_or_default()
    }

    /// Set options for an extension
    pub fn set_options(&self, extension_id: &str, options: SidePanelOptions) {
        let mut states = self.states.write();
        let state = states.entry(extension_id.to_string()).or_default();
        state.options = options;
    }

    /// Get behavior for an extension
    pub fn get_behavior(&self, extension_id: &str) -> PanelBehavior {
        let states = self.states.read();
        states
            .get(extension_id)
            .map(|s| s.behavior.clone())
            .unwrap_or_default()
    }

    /// Set behavior for an extension
    pub fn set_behavior(&self, extension_id: &str, behavior: PanelBehavior) {
        let mut states = self.states.write();
        let state = states.entry(extension_id.to_string()).or_default();
        state.behavior = behavior;
    }
}

impl Default for SidePanelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Side Panel API handler
pub struct SidePanelApiHandler {
    manager: Arc<SidePanelManager>,
}

impl SidePanelApiHandler {
    /// Create a new side panel API handler
    pub fn new(manager: Arc<SidePanelManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for SidePanelApiHandler {
    fn namespace(&self) -> &str {
        "sidePanel"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "open" => {
                self.manager.open(extension_id);
                Ok(serde_json::json!({}))
            }
            "close" => {
                self.manager.close(extension_id);
                Ok(serde_json::json!({}))
            }
            "setOptions" => {
                let options: SidePanelOptions = serde_json::from_value(params)
                    .map_err(|e| ExtensionError::InvalidArgument(e.to_string()))?;
                self.manager.set_options(extension_id, options);
                Ok(serde_json::json!({}))
            }
            "getOptions" => {
                let options = self.manager.get_options(extension_id);
                Ok(serde_json::to_value(options)?)
            }
            "setPanelBehavior" => {
                let behavior: PanelBehavior = serde_json::from_value(params)
                    .map_err(|e| ExtensionError::InvalidArgument(e.to_string()))?;
                self.manager.set_behavior(extension_id, behavior);
                Ok(serde_json::json!({}))
            }
            "getPanelBehavior" => {
                let behavior = self.manager.get_behavior(extension_id);
                Ok(serde_json::to_value(behavior)?)
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "sidePanel.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec![
            "open",
            "close",
            "setOptions",
            "getOptions",
            "setPanelBehavior",
            "getPanelBehavior",
        ]
    }
}
