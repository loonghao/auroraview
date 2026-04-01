//! Extension registry

use super::extension::{Extension, ExtensionManifest};
use crate::browser::BrowserEvent;
use dashmap::DashMap;

/// Extension registry - manages loaded extensions
pub struct ExtensionRegistry {
    extensions: DashMap<String, Box<dyn Extension>>,
    enabled: bool,
}

impl ExtensionRegistry {
    /// Create a new extension registry
    pub fn new(enabled: bool) -> Self {
        Self {
            extensions: DashMap::new(),
            enabled,
        }
    }

    /// Register an extension
    pub fn register(&self, mut extension: Box<dyn Extension>) -> crate::Result<()> {
        if !self.enabled {
            return Err(crate::BrowserError::Extension(
                "Extensions are disabled".to_string(),
            ));
        }

        let id = extension.id().to_string();
        let name = extension.name().to_string();
        tracing::info!(
            "[ExtensionRegistry] Registering extension: {} ({})",
            name,
            id
        );

        // Call on_load before inserting
        extension.on_load();

        if self.extensions.contains_key(&id) {
            return Err(crate::BrowserError::Extension(format!(
                "Extension already registered: {}",
                id
            )));
        }
        self.extensions.insert(id, extension);

        Ok(())
    }

    /// Unregister an extension
    pub fn unregister(&self, id: &str) -> crate::Result<()> {
        let (_, mut extension) = self
            .extensions
            .remove(id)
            .ok_or_else(|| crate::BrowserError::ExtensionNotFound(id.to_string()))?;

        tracing::info!("[ExtensionRegistry] Unregistering extension: {}", id);
        extension.on_unload();
        Ok(())
    }

    /// Get extension by ID
    pub fn get(&self, id: &str) -> Option<ExtensionManifest> {
        self.extensions
            .get(id)
            .map(|ext| manifest_from_extension(ext.value().as_ref()))
    }

    /// List all extensions
    pub fn list(&self) -> Vec<ExtensionManifest> {
        self.extensions
            .iter()
            .map(|entry| manifest_from_extension(entry.value().as_ref()))
            .collect()
    }

    /// Get enabled extensions
    pub fn enabled(&self) -> Vec<ExtensionManifest> {
        self.extensions
            .iter()
            .filter(|entry| entry.value().enabled())
            .map(|entry| manifest_from_extension(entry.value().as_ref()))
            .collect()
    }

    /// Enable/disable an extension
    pub fn set_enabled(&self, id: &str, enabled: bool) -> crate::Result<()> {
        let mut entry = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| crate::BrowserError::ExtensionNotFound(id.to_string()))?;
        entry.value_mut().set_enabled(enabled);
        Ok(())
    }

    /// Dispatch event to all extensions
    pub fn dispatch_event(&self, event: &BrowserEvent) {
        if !self.enabled {
            return;
        }

        for mut entry in self.extensions.iter_mut() {
            if entry.value().enabled() {
                entry.value_mut().on_event(event);
            }
        }
    }

    /// Handle toolbar click for an extension
    pub fn on_toolbar_click(&self, id: &str) -> crate::Result<()> {
        let mut entry = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| crate::BrowserError::ExtensionNotFound(id.to_string()))?;
        entry.value_mut().on_toolbar_click();
        Ok(())
    }

    /// Get popup HTML for an extension
    pub fn popup_html(&self, id: &str) -> crate::Result<Option<String>> {
        let entry = self
            .extensions
            .get(id)
            .ok_or_else(|| crate::BrowserError::ExtensionNotFound(id.to_string()))?;
        Ok(entry.value().popup_html().map(String::from))
    }

    /// Get content scripts for a URL
    pub fn content_scripts_for_url(&self, url: &str) -> Vec<String> {
        self.extensions
            .iter()
            .filter(|entry| entry.value().enabled())
            .filter(|entry| {
                entry
                    .value()
                    .content_script_matches()
                    .iter()
                    .any(|pattern| url_matches_pattern(url, pattern))
            })
            .filter_map(|entry| entry.value().content_script().map(String::from))
            .collect()
    }

    /// Get extension count
    pub fn count(&self) -> usize {
        self.extensions.len()
    }

    /// Check if extensions are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Create ExtensionManifest from an Extension trait object
fn manifest_from_extension(ext: &dyn Extension) -> ExtensionManifest {
    ExtensionManifest {
        id: ext.id().to_string(),
        name: ext.name().to_string(),
        version: ext.version().to_string(),
        description: ext.description().to_string(),
        icon: ext.icon().map(String::from),
        enabled: ext.enabled(),
        content_script_matches: ext
            .content_script_matches()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        permissions: ext.permissions().iter().map(|s| s.to_string()).collect(),
    }
}

/// Check if a URL matches a pattern
/// Supports wildcards: *://*/* matches any URL
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    if pattern == "*://*/*" || pattern == "<all_urls>" {
        return true;
    }

    // Simple pattern matching
    let pattern_parts: Vec<&str> = pattern.split('*').collect();
    if pattern_parts.len() == 1 {
        return url == pattern;
    }

    let mut remaining = url;
    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part must match from start
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if i == pattern_parts.len() - 1 {
            // Last part must match at end
            if !remaining.ends_with(part) {
                return false;
            }
        } else {
            // Middle parts can match anywhere
            if let Some(pos) = remaining.find(part) {
                remaining = &remaining[pos + part.len()..];
            } else {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_matches_pattern() {
        assert!(url_matches_pattern("https://example.com/page", "*://*/*"));
        assert!(url_matches_pattern("http://test.org/", "*://*/*"));
        assert!(url_matches_pattern(
            "https://example.com/",
            "https://example.com/*"
        ));
        assert!(url_matches_pattern(
            "https://example.com/page",
            "https://example.com/*"
        ));
        assert!(!url_matches_pattern(
            "https://other.com/",
            "https://example.com/*"
        ));
    }
}
