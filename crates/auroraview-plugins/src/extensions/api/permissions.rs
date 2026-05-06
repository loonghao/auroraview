use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle permissions API calls
    pub fn handle_permissions_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "contains" => {
                let requested: Vec<String> = params
                    .get("permissions")
                    .and_then(|p| p.get("permissions"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
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
}
