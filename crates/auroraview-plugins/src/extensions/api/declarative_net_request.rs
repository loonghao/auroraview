use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle declarativeNetRequest API calls
    pub fn handle_declarative_net_request_api(
        &self,
        _extension_id: &str,
        method: &str,
        _params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "updateDynamicRules" | "updateSessionRules" | "updateEnabledRulesets" => {
                Ok(serde_json::json!({}))
            }
            "getDynamicRules" | "getSessionRules" => Ok(serde_json::json!([])),
            "getEnabledRulesets" => Ok(serde_json::json!([])),
            "getAvailableStaticRuleCount" => Ok(serde_json::json!(30000)),
            "isRegexSupported" => Ok(serde_json::json!({ "isSupported": true })),
            _ => Err(PluginError::command_not_found(&format!(
                "declarativeNetRequest.{}",
                method
            ))),
        }
    }
}
