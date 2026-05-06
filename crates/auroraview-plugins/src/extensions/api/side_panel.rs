use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle sidePanel API calls
    pub fn handle_side_panel_api(
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
                    panel.options = Some(super::super::SidePanelOptions {
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
                    "enabled": panel
                        .and_then(|p| p.options.as_ref())
                        .and_then(|o| o.enabled)
                        .unwrap_or(true)
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
}
