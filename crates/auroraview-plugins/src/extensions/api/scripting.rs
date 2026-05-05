use serde_json::Value;

use crate::extensions::ContentScriptInfo;
use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle scripting API calls
    pub fn handle_scripting_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "executeScript" => {
                let cbs = self.callbacks.read();
                if let Some(ref exec_cb) = cbs.on_execute_script {
                    let results = exec_cb(extension_id, params);
                    let injection_results: Vec<Value> = results
                        .into_iter()
                        .map(|r| serde_json::json!({ "frameId": 0, "result": r }))
                        .collect();
                    if injection_results.is_empty() {
                        Ok(serde_json::json!([{ "frameId": 0, "result": null }]))
                    } else {
                        Ok(Value::Array(injection_results))
                    }
                } else {
                    let func = params.get("func");
                    let files = params.get("files");
                    tracing::info!(
                        "scripting.executeScript: no callback, func={:?}, files={:?}",
                        func.is_some(),
                        files
                    );
                    Ok(serde_json::json!([{ "frameId": 0, "result": null }]))
                }
            }
            "insertCSS" => {
                let cbs = self.callbacks.read();
                if let Some(ref css_cb) = cbs.on_insert_css {
                    css_cb(extension_id, params);
                } else {
                    tracing::debug!("scripting.insertCSS: no callback for {}", extension_id);
                }
                Ok(serde_json::json!({}))
            }
            "removeCSS" => {
                let cbs = self.callbacks.read();
                if let Some(ref css_cb) = cbs.on_remove_css {
                    css_cb(extension_id, params);
                } else {
                    tracing::debug!("scripting.removeCSS: no callback for {}", extension_id);
                }
                Ok(serde_json::json!({}))
            }
            "registerContentScripts" => {
                let scripts: Vec<ContentScriptInfo> = params
                    .get("scripts")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let mut state = self.state.write();
                let ext_scripts = state
                    .content_scripts
                    .entry(extension_id.to_string())
                    .or_default();

                ext_scripts.extend(scripts);

                Ok(serde_json::json!({}))
            }
            "unregisterContentScripts" => {
                let ids: Vec<String> = params
                    .get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let mut state = self.state.write();
                if let Some(ext_scripts) = state.content_scripts.get_mut(extension_id) {
                    if ids.is_empty() {
                        ext_scripts.clear();
                    } else {
                        ext_scripts.retain(|s| !ids.contains(&s.id));
                    }
                }

                Ok(serde_json::json!({}))
            }
            "getRegisteredContentScripts" => {
                let state = self.state.read();
                let scripts = state
                    .content_scripts
                    .get(extension_id)
                    .cloned()
                    .unwrap_or_default();
                serde_json::to_value(scripts).map_err(PluginError::serialization_error)
            }
            "updateContentScripts" => {
                // Update registered content scripts by merging with existing
                let updates: Vec<ContentScriptInfo> = params
                    .get("scripts")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let mut state = self.state.write();
                let ext_scripts = state
                    .content_scripts
                    .entry(extension_id.to_string())
                    .or_default();

                for update in updates {
                    if let Some(existing) = ext_scripts.iter_mut().find(|s| s.id == update.id) {
                        *existing = update;
                    }
                }

                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "scripting.{}",
                method
            ))),
        }
    }
}
