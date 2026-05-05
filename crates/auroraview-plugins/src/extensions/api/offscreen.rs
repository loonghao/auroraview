use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle offscreen API calls
    pub fn handle_offscreen_api(
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
}
