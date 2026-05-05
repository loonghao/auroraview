//! Thread-safe registry of all active `WebView` instances.
//!
//! This module provides `WebViewRegistry` for tracking all WebView instances
//! created via the MCP server.

use crate::{
    error::{McpError, Result},
    types::{WebViewConfig, WebViewId, WebViewInfo},
};
use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe registry of all active `WebView` instances.
#[derive(Debug, Clone)]
pub struct WebViewRegistry {
    views: Arc<DashMap<String, WebViewInfo>>,
    /// Optional capacity limit. `None` = unlimited.
    max_webviews: Option<usize>,
}

impl Default for WebViewRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WebViewRegistry {
    /// Create a new empty registry with no capacity limit.
    #[must_use]
    pub fn new() -> Self {
        Self {
            views: Arc::new(DashMap::new()),
            max_webviews: None,
        }
    }

    /// Create a registry with a maximum capacity.
    #[must_use]
    pub fn with_capacity(max: usize) -> Self {
        Self {
            views: Arc::new(DashMap::new()),
            max_webviews: Some(max),
        }
    }

    /// Register a new `WebView` with the given config.
    ///
    /// Panics if the capacity limit would be exceeded — use [`Self::try_register`] for
    /// error-propagating variant.
    #[must_use]
    pub fn register(&self, config: &WebViewConfig) -> WebViewId {
        self.try_register(config)
            .expect("WebView registry capacity exceeded")
    }

    /// Try to register a new `WebView`. Returns `Err(McpError::CapacityExceeded)` if
    /// the optional capacity limit has been reached.
    pub fn try_register(&self, config: &WebViewConfig) -> Result<WebViewId> {
        if let Some(max) = self.max_webviews {
            if self.views.len() >= max {
                return Err(McpError::CapacityExceeded(max));
            }
        }
        let id = WebViewId::new();
        let info = WebViewInfo {
            id: id.clone(),
            title: config
                .title
                .clone()
                .unwrap_or_else(|| "AuroraView".to_string()),
            url: config.url.clone().unwrap_or_default(),
            visible: config.visible.unwrap_or(true),
            width: config.width.unwrap_or(800),
            height: config.height.unwrap_or(600),
            hwnd: 0,
            cdp_endpoint: None,
        };
        self.views.insert(id.0.clone(), info);
        Ok(id)
    }

    /// Return the capacity limit, if set.
    #[must_use]
    pub fn capacity(&self) -> Option<usize> {
        self.max_webviews
    }

    /// Update the URL for an existing `WebView`.
    #[must_use]
    pub fn update_url(&self, id: &WebViewId, url: &str) -> bool {
        if let Some(mut entry) = self.views.get_mut(&id.0) {
            entry.url = url.to_string();
            true
        } else {
            false
        }
    }

    /// Remove a `WebView` from the registry.
    #[must_use]
    pub fn remove(&self, id: &WebViewId) -> Option<WebViewInfo> {
        self.views.remove(&id.0).map(|(_, v)| v)
    }

    /// Get info for a specific `WebView`.
    #[must_use]
    pub fn get(&self, id: &WebViewId) -> Option<WebViewInfo> {
        self.views.get(&id.0).map(|v| v.clone())
    }

    /// List all registered `WebViews`.
    #[must_use]
    pub fn list(&self) -> Vec<WebViewInfo> {
        self.views.iter().map(|e| e.value().clone()).collect()
    }

    /// Number of registered `WebViews`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.views.len()
    }

    #[must_use]
    /// Return `true` if the registry has no `WebView` instances.
    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// Update the CDP endpoint for an existing `WebView`.
    /// Returns `true` if the `WebView` was found and updated.
    #[must_use]
    pub fn update_cdp_endpoint(&self, id: &WebViewId, endpoint: String) -> bool {
        if let Some(mut entry) = self.views.get_mut(&id.0) {
            entry.cdp_endpoint = Some(endpoint);
            true
        } else {
            false
        }
    }

    /// Remove all `WebView` instances from the registry.
    pub fn clear(&self) {
        self.views.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WebViewConfig;
    use rstest::rstest;

    #[test]
    fn new_registry_is_empty() {
        let reg = WebViewRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert_eq!(reg.capacity(), None);
    }

    #[test]
    fn with_capacity_sets_limit() {
        let reg = WebViewRegistry::with_capacity(5);
        assert_eq!(reg.capacity(), Some(5));
    }

    #[test]
    fn register_adds_view() {
        let reg = WebViewRegistry::new();
        let config = WebViewConfig {
            title: Some("Test".to_string()),
            url: Some("https://example.com".to_string()),
            html: None,
            width: Some(800),
            height: Some(600),
            visible: Some(true),
            debug: Some(false),
        };
        let id = reg.register(&config);
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());
        let info = reg.get(&id).unwrap();
        assert_eq!(info.title, "Test");
        assert_eq!(info.url, "https://example.com");
    }

    #[rstest]
    #[case(Some(800), Some(600))]
    #[case(None, None)]
    fn register_uses_defaults(#[case] width: Option<u32>, #[case] height: Option<u32>) {
        let reg = WebViewRegistry::new();
        let config = WebViewConfig {
            title: None,
            url: None,
            html: None,
            width,
            height,
            visible: None,
            debug: None,
        };
        let id = reg.register(&config);
        let info = reg.get(&id).unwrap();
        assert_eq!(info.title, "AuroraView");
        assert_eq!(info.url, "");
        assert_eq!(info.width, width.unwrap_or(800));
        assert_eq!(info.height, height.unwrap_or(600));
        assert!(info.visible);
    }

    #[test]
    fn try_register_returns_err_when_full() {
        let reg = WebViewRegistry::with_capacity(1);
        let config = WebViewConfig::default();
        let _id1 = reg.try_register(&config).unwrap();
        let result = reg.try_register(&config);
        assert!(result.is_err());
    }

    #[test]
    fn update_url_modifies_existing() {
        let reg = WebViewRegistry::new();
        let config = WebViewConfig {
            title: Some("Test".to_string()),
            url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        let id = reg.register(&config);
        assert!(reg.update_url(&id, "https://new-url.com"));
        let info = reg.get(&id).unwrap();
        assert_eq!(info.url, "https://new-url.com");
    }

    #[test]
    fn update_url_returns_false_for_missing() {
        let reg = WebViewRegistry::new();
        let fake_id = crate::types::WebViewId::new();
        assert!(!reg.update_url(&fake_id, "https://example.com"));
    }

    #[test]
    fn remove_deletes_view() {
        let reg = WebViewRegistry::new();
        let config = WebViewConfig::default();
        let id = reg.register(&config);
        assert_eq!(reg.len(), 1);
        let removed = reg.remove(&id);
        assert!(removed.is_some());
        assert_eq!(reg.len(), 0);
        assert!(reg.get(&id).is_none());
    }

    #[test]
    fn list_returns_all_views() {
        let reg = WebViewRegistry::new();
        for i in 0..3 {
            let config = WebViewConfig {
                title: Some(format!("View {i}")),
                ..Default::default()
            };
            let _ = reg.register(&config);
        }
        let views = reg.list();
        assert_eq!(views.len(), 3);
    }

    #[test]
    fn clear_removes_all_views() {
        let reg = WebViewRegistry::new();
        let config = WebViewConfig::default();
        let _id1 = reg.register(&config);
        let _id2 = reg.register(&config);
        assert_eq!(reg.len(), 2);

        reg.clear();
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }
}
