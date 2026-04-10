use crate::{
    error::{McpError, Result},
    types::{WebViewConfig, WebViewId, WebViewInfo},
};
use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe registry of all active WebView instances.
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
    pub fn new() -> Self {
        Self {
            views: Arc::new(DashMap::new()),
            max_webviews: None,
        }
    }

    /// Create a registry with a maximum capacity.
    pub fn with_capacity(max: usize) -> Self {
        Self {
            views: Arc::new(DashMap::new()),
            max_webviews: Some(max),
        }
    }

    /// Register a new WebView with the given config.
    ///
    /// Panics if the capacity limit would be exceeded — use [`try_register`] for
    /// error-propagating variant.
    pub fn register(&self, config: &WebViewConfig) -> WebViewId {
        self.try_register(config)
            .expect("WebView registry capacity exceeded")
    }

    /// Try to register a new WebView. Returns `Err(McpError::CapacityExceeded)` if
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
        };
        self.views.insert(id.0.clone(), info);
        Ok(id)
    }

    /// Return the capacity limit, if set.
    pub fn capacity(&self) -> Option<usize> {
        self.max_webviews
    }

    /// Update the URL for an existing WebView.
    pub fn update_url(&self, id: &WebViewId, url: &str) -> bool {
        if let Some(mut entry) = self.views.get_mut(&id.0) {
            entry.url = url.to_string();
            true
        } else {
            false
        }
    }

    /// Remove a WebView from the registry.
    pub fn remove(&self, id: &WebViewId) -> Option<WebViewInfo> {
        self.views.remove(&id.0).map(|(_, v)| v)
    }

    /// Get info for a specific WebView.
    pub fn get(&self, id: &WebViewId) -> Option<WebViewInfo> {
        self.views.get(&id.0).map(|v| v.clone())
    }

    /// List all registered WebViews.
    pub fn list(&self) -> Vec<WebViewInfo> {
        self.views.iter().map(|e| e.value().clone()).collect()
    }

    /// Number of registered WebViews.
    pub fn len(&self) -> usize {
        self.views.len()
    }

    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}
