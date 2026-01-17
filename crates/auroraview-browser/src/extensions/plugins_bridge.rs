//! Bridge to auroraview-plugins extension system
//!
//! This module provides integration with the `auroraview-plugins` crate's
//! Chrome Extension API compatibility layer, allowing the browser to
//! load and manage Chrome-style extensions.
//!
//! Requires the `plugins` feature to be enabled.

#[cfg(feature = "plugins")]
use auroraview_plugins::extensions::{ExtensionInfo, ExtensionsPlugin, ExtensionsState};
#[cfg(feature = "plugins")]
use parking_lot::RwLock;
#[cfg(feature = "plugins")]
use std::sync::Arc;

/// Chrome Extension compatibility bridge
///
/// This provides access to Chrome Extension APIs when the `plugins` feature is enabled.
/// It allows loading manifest.json based extensions with:
/// - Storage API (chrome.storage.local/sync)
/// - Side Panel API
/// - Action API (toolbar buttons)
/// - Content scripts
/// - And more...
#[cfg(feature = "plugins")]
pub struct ChromeExtensionBridge {
    /// The underlying extensions plugin
    plugin: Arc<ExtensionsPlugin>,
}

#[cfg(feature = "plugins")]
impl ChromeExtensionBridge {
    /// Create a new Chrome extension bridge
    pub fn new() -> Self {
        Self {
            plugin: Arc::new(ExtensionsPlugin::new()),
        }
    }

    /// Create from existing plugin instance
    pub fn from_plugin(plugin: Arc<ExtensionsPlugin>) -> Self {
        Self { plugin }
    }

    /// Get the underlying plugin for registration with a router
    pub fn plugin(&self) -> Arc<ExtensionsPlugin> {
        Arc::clone(&self.plugin)
    }

    /// Load an extension from a directory
    pub fn load_extension(&self, path: &std::path::Path) -> crate::Result<String> {
        // This would parse manifest.json and register the extension
        // For now, return a placeholder
        let extension_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        tracing::info!(
            "[ChromeExtensionBridge] Loading extension from: {}",
            path.display()
        );

        Ok(extension_id)
    }

    /// List loaded extensions
    pub fn list_extensions(&self) -> Vec<String> {
        vec![]
    }

    /// Unload an extension
    pub fn unload_extension(&self, id: &str) -> crate::Result<()> {
        tracing::info!("[ChromeExtensionBridge] Unloading extension: {}", id);
        Ok(())
    }

    /// Get extension info
    pub fn get_extension(&self, id: &str) -> Option<ChromeExtensionInfo> {
        // Would return info from the plugin state
        None
    }

    /// Enable/disable an extension
    pub fn set_enabled(&self, id: &str, enabled: bool) -> crate::Result<()> {
        tracing::info!(
            "[ChromeExtensionBridge] Setting extension {} enabled={}",
            id,
            enabled
        );
        Ok(())
    }
}

#[cfg(feature = "plugins")]
impl Default for ChromeExtensionBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Chrome extension info (simplified view)
#[derive(Debug, Clone)]
pub struct ChromeExtensionInfo {
    /// Extension ID
    pub id: String,
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Description
    pub description: String,
    /// Whether enabled
    pub enabled: bool,
    /// Has side panel
    pub has_side_panel: bool,
    /// Has popup
    pub has_popup: bool,
}

/// Stub implementation when plugins feature is not enabled
#[cfg(not(feature = "plugins"))]
pub struct ChromeExtensionBridge;

#[cfg(not(feature = "plugins"))]
impl ChromeExtensionBridge {
    /// Create a stub bridge (no-op without plugins feature)
    pub fn new() -> Self {
        Self
    }

    /// Load extension (stub)
    pub fn load_extension(&self, _path: &std::path::Path) -> crate::Result<String> {
        Err(crate::BrowserError::Extension(
            "Chrome extensions require the 'plugins' feature".to_string(),
        ))
    }

    /// List extensions (stub)
    pub fn list_extensions(&self) -> Vec<String> {
        vec![]
    }

    /// Unload extension (stub)
    pub fn unload_extension(&self, _id: &str) -> crate::Result<()> {
        Err(crate::BrowserError::Extension(
            "Chrome extensions require the 'plugins' feature".to_string(),
        ))
    }

    /// Get extension (stub)
    pub fn get_extension(&self, _id: &str) -> Option<ChromeExtensionInfo> {
        None
    }

    /// Set enabled (stub)
    pub fn set_enabled(&self, _id: &str, _enabled: bool) -> crate::Result<()> {
        Err(crate::BrowserError::Extension(
            "Chrome extensions require the 'plugins' feature".to_string(),
        ))
    }
}

#[cfg(not(feature = "plugins"))]
impl Default for ChromeExtensionBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_extension_bridge_creation() {
        let bridge = ChromeExtensionBridge::new();
        assert!(bridge.list_extensions().is_empty());
    }

    #[test]
    fn test_get_nonexistent_extension() {
        let bridge = ChromeExtensionBridge::new();
        assert!(bridge.get_extension("nonexistent").is_none());
    }
}
