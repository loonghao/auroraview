use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle webRequest API calls
    pub fn handle_web_request_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "addListener" | "removeListener" => {
                // Request interception is handled at the WebView level, not in the plugin
                tracing::debug!("webRequest.{}: stub acknowledgement", method);
                Ok(serde_json::json!({}))
            }
            "handlerBehaviorChanged" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!(
                "webRequest.{}",
                method
            ))),
        }
    }
}
