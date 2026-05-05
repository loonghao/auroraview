use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle tabs API calls
    pub fn handle_tabs_api(
        &self,
        _extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        // AuroraView has a single "tab" representing the main WebView
        let default_tab = serde_json::json!({
            "id": 1,
            "windowId": 1,
            "index": 0,
            "active": true,
            "highlighted": true,
            "pinned": false,
            "status": "complete",
            "incognito": false,
            "url": "",
            "title": "AuroraView"
        });

        match method {
            "query" => {
                // Return single tab matching query
                Ok(serde_json::json!([default_tab]))
            }
            "getCurrent" => Ok(default_tab),
            "get" => Ok(default_tab),
            "create" => {
                // In AuroraView, "creating a tab" might open a new window or navigate
                let url = params.get("url").and_then(|v| v.as_str());
                if let Some(url) = url {
                    let cbs = self.callbacks.read();
                    if let Some(ref navigate_cb) = cbs.on_navigate {
                        navigate_cb(url);
                    } else {
                        tracing::info!("tabs.create: no navigate callback, URL: {}", url);
                    }
                }
                Ok(default_tab)
            }
            "update" => {
                // Handle tab updates (e.g., URL change)
                if let Some(url) = params.get("url").and_then(|v| v.as_str()) {
                    tracing::info!("tabs.update requested for URL: {}", url);
                }
                Ok(default_tab)
            }
            "remove" => {
                // Cannot remove the only tab
                Ok(serde_json::json!({}))
            }
            "reload" => {
                let cbs = self.callbacks.read();
                if let Some(ref reload_cb) = cbs.on_reload_page {
                    reload_cb();
                } else {
                    tracing::info!("tabs.reload: no reload callback registered");
                }
                Ok(serde_json::json!({}))
            }
            "sendMessage" => {
                let message = params.get("message").cloned().unwrap_or(Value::Null);
                let tab_id = params.get("tabId").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                let cbs = self.callbacks.read();
                if let Some(ref send_cb) = cbs.on_send_message {
                    let response = send_cb(tab_id, message);
                    Ok(response.unwrap_or(Value::Null))
                } else {
                    tracing::debug!("tabs.sendMessage: no send_message callback");
                    Ok(message)
                }
            }
            "captureVisibleTab" => {
                // Screenshot capture requires platform-specific implementation
                tracing::debug!("tabs.captureVisibleTab: not yet supported");
                Ok(serde_json::json!(""))
            }
            "executeScript" | "insertCSS" | "removeCSS" => {
                // Delegate to scripting API
                Ok(serde_json::json!([{ "frameId": 0, "result": null }]))
            }
            "setZoom" | "getZoom" => Ok(serde_json::json!(1.0)),
            "group" | "ungroup" => Ok(serde_json::json!({})),
            _ => Err(PluginError::command_not_found(&format!("tabs.{}", method))),
        }
    }
}
