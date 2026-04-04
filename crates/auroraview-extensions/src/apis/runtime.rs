//! chrome.runtime API handler
//!
//! Implements runtime messaging and lifecycle APIs.

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Message sender information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    /// Extension ID of the sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// URL of the sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Tab that sent the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<crate::apis::tabs::Tab>,
    /// Frame ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i32>,
}

/// Port for long-lived connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    /// Port name
    pub name: String,
    /// Sender information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<MessageSender>,
}

/// Install reason
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnInstalledReason {
    /// Extension was installed
    Install,
    /// Extension was updated
    Update,
    /// Browser was updated
    ChromeUpdate,
    /// Shared module was updated
    SharedModuleUpdate,
}

/// Install details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnInstalledDetails {
    /// Reason for the event
    pub reason: OnInstalledReason,
    /// Previous version (for updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_version: Option<String>,
    /// ID of the shared module (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Message handler type
pub type MessageHandler = Box<dyn Fn(&str, Value, MessageSender) -> Option<Value> + Send + Sync>;

/// Callback for opening an options page
pub type OptionsPageCallback = Box<dyn Fn(&str, &str) + Send + Sync>;

/// Callback for extension reload
pub type ReloadCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Runtime manager
pub struct RuntimeManager {
    /// Message handlers per extension
    message_handlers: RwLock<std::collections::HashMap<ExtensionId, Vec<MessageHandler>>>,
    /// Uninstall URLs per extension
    uninstall_urls: RwLock<std::collections::HashMap<ExtensionId, String>>,
    /// Options page open callback
    on_open_options_page: RwLock<Option<OptionsPageCallback>>,
    /// Extension reload callback
    on_reload: RwLock<Option<ReloadCallback>>,
}

impl RuntimeManager {
    /// Create a new runtime manager
    pub fn new() -> Self {
        Self {
            message_handlers: RwLock::new(std::collections::HashMap::new()),
            uninstall_urls: RwLock::new(std::collections::HashMap::new()),
            on_open_options_page: RwLock::new(None),
            on_reload: RwLock::new(None),
        }
    }

    /// Set the options page callback
    pub fn set_on_open_options_page<F>(&self, callback: F)
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let mut cb = self.on_open_options_page.write();
        *cb = Some(Box::new(callback));
    }

    /// Set the reload callback
    pub fn set_on_reload<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut cb = self.on_reload.write();
        *cb = Some(Box::new(callback));
    }

    /// Register a message handler for an extension
    pub fn add_message_handler(&self, extension_id: &str, handler: MessageHandler) {
        let mut handlers = self.message_handlers.write();
        let ext_handlers = handlers.entry(extension_id.to_string()).or_default();
        ext_handlers.push(handler);
    }

    /// Send a message to an extension
    pub fn send_message(
        &self,
        extension_id: &str,
        message: Value,
        sender: MessageSender,
    ) -> Option<Value> {
        let handlers = self.message_handlers.read();
        if let Some(ext_handlers) = handlers.get(extension_id) {
            for handler in ext_handlers {
                if let Some(response) = handler(extension_id, message.clone(), sender.clone()) {
                    return Some(response);
                }
            }
        }
        None
    }

    /// Broadcast a message to all extensions
    pub fn broadcast_message(&self, message: Value, sender: MessageSender) {
        let handlers = self.message_handlers.read();
        for (_, ext_handlers) in handlers.iter() {
            for handler in ext_handlers {
                handler("", message.clone(), sender.clone());
            }
        }
    }

    /// Store uninstall URL for an extension
    pub fn set_uninstall_url(&self, extension_id: &str, url: &str) {
        let mut urls = self.uninstall_urls.write();
        urls.insert(extension_id.to_string(), url.to_string());
    }

    /// Get uninstall URL for an extension
    pub fn get_uninstall_url(&self, extension_id: &str) -> Option<String> {
        let urls = self.uninstall_urls.read();
        urls.get(extension_id).cloned()
    }

    /// Open options page for an extension
    pub fn open_options_page(&self, extension_id: &str, options_page: &str) {
        let cb = self.on_open_options_page.read();
        if let Some(callback) = cb.as_ref() {
            callback(extension_id, options_page);
        } else {
            tracing::info!(
                "No options page callback registered, ignoring openOptionsPage for {}",
                extension_id
            );
        }
    }

    /// Reload an extension
    pub fn reload_extension(&self, extension_id: &str) {
        let cb = self.on_reload.read();
        if let Some(callback) = cb.as_ref() {
            callback(extension_id);
        } else {
            tracing::info!(
                "No reload callback registered, ignoring reload for {}",
                extension_id
            );
        }
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime API handler
pub struct RuntimeApiHandler {
    manager: Arc<RuntimeManager>,
}

impl RuntimeApiHandler {
    /// Create a new runtime API handler
    pub fn new(manager: Arc<RuntimeManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for RuntimeApiHandler {
    fn namespace(&self) -> &str {
        "runtime"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "sendMessage" => {
                let message = params.get("message").cloned().unwrap_or(Value::Null);
                let sender = MessageSender {
                    id: Some(extension_id.to_string()),
                    url: None,
                    tab: None,
                    frame_id: None,
                };

                let response = self.manager.send_message(extension_id, message, sender);
                Ok(response.unwrap_or(Value::Null))
            }
            "getURL" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

                let url = format!("auroraview-extension://{}/{}", extension_id, path);
                Ok(serde_json::json!(url))
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
                } else if cfg!(target_arch = "x86") {
                    "x86-32"
                } else if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else {
                    "unknown"
                };

                Ok(serde_json::json!({
                    "os": platform,
                    "arch": arch,
                    "nacl_arch": arch
                }))
            }
            "getBackgroundPage" => {
                // Not applicable in service worker model
                Ok(Value::Null)
            }
            "openOptionsPage" => {
                let options_url = format!(
                    "auroraview-extension://{}/options.html",
                    extension_id
                );
                self.manager.open_options_page(extension_id, &options_url);
                Ok(serde_json::json!({}))
            }
            "setUninstallURL" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.manager.set_uninstall_url(extension_id, url);
                Ok(serde_json::json!({}))
            }
            "reload" => {
                self.manager.reload_extension(extension_id);
                Ok(serde_json::json!({}))
            }
            "requestUpdateCheck" => {
                // Not applicable for local extensions
                Ok(serde_json::json!({
                    "status": "no_update"
                }))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "runtime.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec![
            "sendMessage",
            "getURL",
            "getPlatformInfo",
            "getBackgroundPage",
            "openOptionsPage",
            "setUninstallURL",
            "reload",
            "requestUpdateCheck",
        ]
    }
}
