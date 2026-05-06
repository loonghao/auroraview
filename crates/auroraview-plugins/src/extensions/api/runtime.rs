use serde_json::Value;

use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle runtime API calls
    pub fn handle_runtime_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "getURL" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let clean_path = path.trim_start_matches('/');
                let url = format!(
                    "https://auroraview.localhost/extension/{}/{}",
                    extension_id, clean_path
                );
                Ok(serde_json::json!(url))
            }
            "getManifest" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    if let Some(manifest) = &ext.manifest {
                        return Ok(manifest.clone());
                    }
                    // Return basic manifest info
                    return Ok(serde_json::json!({
                        "manifest_version": 3,
                        "name": ext.name,
                        "version": ext.version,
                        "description": ext.description,
                        "permissions": ext.permissions
                    }));
                }
                Ok(serde_json::json!({}))
            }
            "getPlatformInfo" => {
                let platform = if cfg!(target_os = "windows") {
                    "win"
                } else if cfg!(target_os = "macos") {
                    "mac"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else {
                    "unknown"
                };

                let arch = if cfg!(target_arch = "x86_64") {
                    "x86-64"
                } else if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else if cfg!(target_arch = "x86") {
                    "x86-32"
                } else {
                    "unknown"
                };

                Ok(serde_json::json!({
                    "os": platform,
                    "arch": arch,
                    "nacl_arch": arch
                }))
            }
            "sendMessage" => {
                let message = params.get("message").cloned().unwrap_or(Value::Null);
                let cbs = self.callbacks.read();
                if let Some(ref msg_cb) = cbs.on_runtime_message {
                    let response = msg_cb(extension_id, message);
                    Ok(response.unwrap_or(Value::Null))
                } else {
                    tracing::debug!(
                        "runtime.sendMessage: no message callback for {}",
                        extension_id
                    );
                    Ok(message)
                }
            }
            "connect" => {
                let port_id = params.get("portId").and_then(|v| v.as_str());
                let name = params.get("name").and_then(|v| v.as_str());
                // Port connections require persistent bidirectional channels
                tracing::debug!(
                    "runtime.connect: port={:?}, name={:?} for {}",
                    port_id,
                    name,
                    extension_id
                );
                Ok(serde_json::json!({
                    "portId": port_id,
                    "name": name
                }))
            }
            "portPostMessage" | "portDisconnect" => {
                // Port messaging requires persistent channels (not yet available)
                tracing::debug!("runtime.{}: port messaging stub", method);
                Ok(serde_json::json!({}))
            }
            "openOptionsPage" => {
                let state = self.state.read();
                if let Some(ext) = state.extensions.get(extension_id) {
                    if let Some(options_page) = &ext.options_page {
                        let cbs = self.callbacks.read();
                        if let Some(ref options_cb) = cbs.on_open_options_page {
                            options_cb(extension_id, options_page);
                        } else {
                            tracing::info!(
                                "runtime.openOptionsPage: no callback, page: {}",
                                options_page
                            );
                        }
                    }
                }
                Ok(serde_json::json!({}))
            }
            "setUninstallURL" => {
                // Store uninstall URL (not implemented)
                Ok(serde_json::json!({}))
            }
            "reload" => {
                let cbs = self.callbacks.read();
                if let Some(ref reload_cb) = cbs.on_reload_extension {
                    reload_cb(extension_id);
                } else {
                    tracing::info!("runtime.reload: no reload callback for {}", extension_id);
                }
                Ok(serde_json::json!({}))
            }
            "requestUpdateCheck" => Ok(serde_json::json!({
                "status": "no_update"
            })),
            "getContexts" => {
                // Return current contexts
                Ok(serde_json::json!([{
                    "contextType": "SIDE_PANEL",
                    "documentId": "main",
                    "documentOrigin": format!("https://auroraview.localhost/extension/{}", extension_id),
                    "documentUrl": format!("https://auroraview.localhost/extension/{}/sidepanel.html", extension_id)
                }]))
            }
            "sendMessageResponse" => {
                // Handle response to a message
                Ok(serde_json::json!({}))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "runtime.{}",
                method
            ))),
        }
    }
}
