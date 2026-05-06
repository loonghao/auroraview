use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle action API calls
    pub fn handle_action_api(
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
                let state = self.state.read();
                let popup_path = state
                    .actions
                    .get(extension_id)
                    .and_then(|a| a.popup.clone());
                drop(state);
                let cbs = self.callbacks.read();
                if let Some(ref popup_cb) = cbs.on_open_popup {
                    popup_cb(extension_id, popup_path.as_deref());
                } else {
                    tracing::debug!("action.openPopup: no popup callback for {}", extension_id);
                }
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
}
