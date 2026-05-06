use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle windows API calls
    pub fn handle_windows_api(
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
                let cbs = self.callbacks.read();
                if let Some(ref create_cb) = cbs.on_create_window {
                    let result = create_cb(_params);
                    Ok(result)
                } else {
                    tracing::debug!("windows.create: no callback, returning default window");
                    Ok(default_window)
                }
            }
            "update" | "remove" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!(
                "windows.{}",
                method
            ))),
        }
    }
}
