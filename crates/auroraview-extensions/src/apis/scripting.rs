//! chrome.scripting API handler
//!
//! Implements script injection capabilities for extensions.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};

/// Script injection target
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectionTarget {
    /// Tab ID to inject into
    pub tab_id: i32,
    /// Frame IDs to inject into (optional)
    #[serde(default)]
    pub frame_ids: Option<Vec<i32>>,
    /// Whether to inject into all frames
    #[serde(default)]
    pub all_frames: bool,
}

/// Script injection details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInjection {
    /// Target for injection
    pub target: InjectionTarget,
    /// JavaScript files to inject
    #[serde(default)]
    pub files: Option<Vec<String>>,
    /// JavaScript function to inject
    #[serde(default)]
    pub func: Option<String>,
    /// Arguments to pass to the function
    #[serde(default)]
    pub args: Option<Vec<Value>>,
    /// World to inject into
    #[serde(default)]
    pub world: Option<String>,
    /// Whether to inject at document start
    #[serde(default)]
    pub inject_immediately: bool,
}

/// CSS injection details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssInjection {
    /// Target for injection
    pub target: InjectionTarget,
    /// CSS files to inject
    #[serde(default)]
    pub files: Option<Vec<String>>,
    /// CSS code to inject
    #[serde(default)]
    pub css: Option<String>,
    /// Origin of the CSS
    #[serde(default)]
    pub origin: Option<String>,
}

/// Registered content script
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredContentScript {
    /// Script ID
    pub id: String,
    /// URL patterns to match
    #[serde(default)]
    pub matches: Vec<String>,
    /// URL patterns to exclude
    #[serde(default)]
    pub exclude_matches: Vec<String>,
    /// JavaScript files
    #[serde(default)]
    pub js: Vec<String>,
    /// CSS files
    #[serde(default)]
    pub css: Vec<String>,
    /// Whether to run in all frames
    #[serde(default)]
    pub all_frames: bool,
    /// When to run
    #[serde(default)]
    pub run_at: Option<String>,
    /// World to run in
    #[serde(default)]
    pub world: Option<String>,
    /// Whether script persists across sessions
    #[serde(default)]
    pub persist_across_sessions: bool,
}

/// Injection result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectionResult {
    /// Frame ID where script was injected
    pub frame_id: i32,
    /// Result of the injection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error if injection failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Scripting state manager
pub struct ScriptingManager {
    /// Registered content scripts per extension
    registered_scripts: RwLock<std::collections::HashMap<String, Vec<RegisteredContentScript>>>,
    /// Callback for script execution
    #[allow(clippy::type_complexity)]
    on_execute:
        RwLock<Option<Box<dyn Fn(&str, &ScriptInjection) -> Vec<InjectionResult> + Send + Sync>>>,
    /// Callback for CSS injection
    #[allow(clippy::type_complexity)]
    on_insert_css: RwLock<Option<Box<dyn Fn(&str, &CssInjection) + Send + Sync>>>,
}

impl ScriptingManager {
    /// Create a new scripting manager
    pub fn new() -> Self {
        Self {
            registered_scripts: RwLock::new(std::collections::HashMap::new()),
            on_execute: RwLock::new(None),
            on_insert_css: RwLock::new(None),
        }
    }

    /// Set the script execution callback
    pub fn set_on_execute<F>(&self, callback: F)
    where
        F: Fn(&str, &ScriptInjection) -> Vec<InjectionResult> + Send + Sync + 'static,
    {
        let mut on_execute = self.on_execute.write();
        *on_execute = Some(Box::new(callback));
    }

    /// Set the CSS injection callback
    pub fn set_on_insert_css<F>(&self, callback: F)
    where
        F: Fn(&str, &CssInjection) + Send + Sync + 'static,
    {
        let mut on_insert_css = self.on_insert_css.write();
        *on_insert_css = Some(Box::new(callback));
    }

    /// Execute a script
    pub fn execute_script(
        &self,
        extension_id: &str,
        injection: &ScriptInjection,
    ) -> Vec<InjectionResult> {
        let on_execute = self.on_execute.read();
        if let Some(callback) = on_execute.as_ref() {
            callback(extension_id, injection)
        } else {
            // Default: return empty result
            vec![InjectionResult {
                frame_id: 0,
                result: None,
                error: Some("Script execution not configured".to_string()),
            }]
        }
    }

    /// Insert CSS
    pub fn insert_css(&self, extension_id: &str, injection: &CssInjection) {
        let on_insert_css = self.on_insert_css.read();
        if let Some(callback) = on_insert_css.as_ref() {
            callback(extension_id, injection);
        }
    }

    /// Register content scripts
    pub fn register_content_scripts(
        &self,
        extension_id: &str,
        scripts: Vec<RegisteredContentScript>,
    ) {
        let mut registered = self.registered_scripts.write();
        let ext_scripts = registered.entry(extension_id.to_string()).or_default();
        ext_scripts.extend(scripts);
    }

    /// Unregister content scripts
    pub fn unregister_content_scripts(&self, extension_id: &str, ids: Option<Vec<String>>) {
        let mut registered = self.registered_scripts.write();
        if let Some(ext_scripts) = registered.get_mut(extension_id) {
            match ids {
                Some(ids) => {
                    ext_scripts.retain(|s| !ids.contains(&s.id));
                }
                None => {
                    ext_scripts.clear();
                }
            }
        }
    }

    /// Get registered content scripts
    pub fn get_registered_content_scripts(
        &self,
        extension_id: &str,
        ids: Option<Vec<String>>,
    ) -> Vec<RegisteredContentScript> {
        let registered = self.registered_scripts.read();
        registered
            .get(extension_id)
            .map(|scripts| match ids {
                Some(ids) => scripts
                    .iter()
                    .filter(|s| ids.contains(&s.id))
                    .cloned()
                    .collect(),
                None => scripts.clone(),
            })
            .unwrap_or_default()
    }
}

impl Default for ScriptingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Scripting API handler
pub struct ScriptingApiHandler {
    manager: Arc<ScriptingManager>,
}

impl ScriptingApiHandler {
    /// Create a new scripting API handler
    pub fn new(manager: Arc<ScriptingManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for ScriptingApiHandler {
    fn namespace(&self) -> &str {
        "scripting"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "executeScript" => {
                let injection: ScriptInjection = serde_json::from_value(params)
                    .map_err(|e| ExtensionError::InvalidArgument(e.to_string()))?;

                let results = self.manager.execute_script(extension_id, &injection);
                Ok(serde_json::to_value(results)?)
            }
            "insertCSS" => {
                let injection: CssInjection = serde_json::from_value(params)
                    .map_err(|e| ExtensionError::InvalidArgument(e.to_string()))?;

                self.manager.insert_css(extension_id, &injection);
                Ok(serde_json::json!({}))
            }
            "removeCSS" => {
                // TODO: Implement CSS removal
                Ok(serde_json::json!({}))
            }
            "registerContentScripts" => {
                let scripts: Vec<RegisteredContentScript> = params
                    .get("scripts")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| {
                        ExtensionError::InvalidArgument("scripts is required".to_string())
                    })?;

                self.manager.register_content_scripts(extension_id, scripts);
                Ok(serde_json::json!({}))
            }
            "unregisterContentScripts" => {
                let ids: Option<Vec<String>> = params
                    .get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                self.manager.unregister_content_scripts(extension_id, ids);
                Ok(serde_json::json!({}))
            }
            "getRegisteredContentScripts" => {
                let ids: Option<Vec<String>> = params
                    .get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let scripts = self
                    .manager
                    .get_registered_content_scripts(extension_id, ids);
                Ok(serde_json::to_value(scripts)?)
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "scripting.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec![
            "executeScript",
            "insertCSS",
            "removeCSS",
            "registerContentScripts",
            "unregisterContentScripts",
            "getRegisteredContentScripts",
        ]
    }
}
