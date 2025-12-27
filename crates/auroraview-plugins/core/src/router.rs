//! Plugin router for dispatching commands to plugins

use crate::{PluginHandler, PluginRequest, PluginResponse, ScopeConfig};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Event callback type for plugins to emit events
pub type PluginEventCallback = Arc<dyn Fn(&str, Value) + Send + Sync>;

/// Plugin router for dispatching commands to plugins
pub struct PluginRouter {
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn PluginHandler>>,
    /// Global scope configuration
    scope: ScopeConfig,
    /// Event callback for plugins to emit events to frontend
    event_callback: Arc<RwLock<Option<PluginEventCallback>>>,
}

impl Default for PluginRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRouter {
    /// Create a new plugin router (without any plugins registered)
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            scope: ScopeConfig::new(),
            event_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom scope configuration
    pub fn with_scope(scope: ScopeConfig) -> Self {
        let mut router = Self::new();
        router.scope = scope;
        router
    }

    /// Get the shared event callback reference (for plugins that need it)
    pub fn event_callback_ref(&self) -> Arc<RwLock<Option<PluginEventCallback>>> {
        self.event_callback.clone()
    }

    /// Set the event callback for plugins to emit events
    ///
    /// This callback will be invoked when plugins (like ProcessPlugin) need
    /// to send events to the frontend (e.g., process stdout/stderr output).
    pub fn set_event_callback(&self, callback: PluginEventCallback) {
        tracing::info!("[PluginRouter] Setting event callback");
        let mut cb = self.event_callback.write().unwrap();
        *cb = Some(callback);
        tracing::info!("[PluginRouter] Event callback set successfully");
    }

    /// Clear the event callback
    pub fn clear_event_callback(&self) {
        let mut cb = self.event_callback.write().unwrap();
        *cb = None;
    }

    /// Emit an event through the callback (if set)
    pub fn emit_event(&self, event_name: &str, data: Value) {
        if let Some(callback) = self.event_callback.read().unwrap().as_ref() {
            callback(event_name, data);
        }
    }

    /// Register a plugin
    pub fn register(&mut self, name: impl Into<String>, plugin: Arc<dyn PluginHandler>) {
        self.plugins.insert(name.into(), plugin);
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn PluginHandler>> {
        self.plugins.remove(name)
    }

    /// Handle a plugin command
    pub fn handle(&self, request: PluginRequest) -> PluginResponse {
        tracing::debug!(
            "[PluginRouter] Handling request for plugin '{}', command '{}'. Enabled plugins: {:?}",
            request.plugin,
            request.command,
            self.scope.enabled_plugins
        );

        // Check if plugin is enabled
        if !self.scope.is_plugin_enabled(&request.plugin) {
            tracing::warn!(
                "[PluginRouter] Plugin '{}' is disabled. Enabled: {:?}",
                request.plugin,
                self.scope.enabled_plugins
            );
            return PluginResponse::err(
                format!("Plugin '{}' is disabled", request.plugin),
                "PLUGIN_DISABLED",
            )
            .with_id(request.id);
        }

        let plugin = match self.plugins.get(&request.plugin) {
            Some(p) => p,
            None => {
                return PluginResponse::err(
                    format!("Plugin '{}' not found", request.plugin),
                    "PLUGIN_NOT_FOUND",
                )
                .with_id(request.id);
            }
        };

        match plugin.handle(&request.command, request.args.clone(), &self.scope) {
            Ok(data) => PluginResponse::ok(data).with_id(request.id),
            Err(e) => PluginResponse::err(e.message(), e.code()).with_id(request.id),
        }
    }

    /// Check if a plugin is registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get list of registered plugin names
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Get the scope configuration
    pub fn scope(&self) -> &ScopeConfig {
        &self.scope
    }

    /// Get mutable scope configuration
    pub fn scope_mut(&mut self) -> &mut ScopeConfig {
        &mut self.scope
    }

    /// Update scope configuration
    pub fn set_scope(&mut self, scope: ScopeConfig) {
        self.scope = scope;
    }
}
