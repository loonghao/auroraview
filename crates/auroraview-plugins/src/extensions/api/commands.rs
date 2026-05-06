use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle commands API calls
    pub fn handle_commands_api(
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
}
