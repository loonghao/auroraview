//! Feature integration for WebView
//!
//! This module provides optional feature integrations for WebView applications.
//! Each feature can be enabled independently via Cargo features.
//!
//! # Available Features
//!
//! | Feature | Crate | Description |
//! |---------|-------|-------------|
//! | `tabs` | `auroraview-tabs` | Tab management |
//! | `bookmarks` | `auroraview-bookmarks` | Bookmark storage |
//! | `history` | `auroraview-history` | Browsing history |
//! | `downloads` | `auroraview-downloads` | Download management |
//! | `settings` | `auroraview-settings` | User preferences |
//! | `notifications` | `auroraview-notifications` | Notification system |
//! | `devtools-feature` | `auroraview-devtools` | DevTools management |
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview::webview::features::Features;
//!
//! let mut features = Features::default();
//!
//! // Enable features
//! features = features.with_tabs();
//! features = features.with_bookmarks(None);
//! features = features.with_downloads(Some("/tmp/downloads"));
//!
//! // Use features
//! if let Some(tabs) = &mut features.tabs {
//!     let id = tabs.create("https://github.com");
//! }
//! ```

#[cfg(any(
    feature = "feature-bookmarks",
    feature = "feature-history",
    feature = "feature-downloads",
    feature = "feature-settings"
))]
use std::path::Path;

/// Re-export feature crates for convenience
#[cfg(feature = "feature-tabs")]
pub use auroraview_tabs;

#[cfg(feature = "feature-bookmarks")]
pub use auroraview_bookmarks;

#[cfg(feature = "feature-history")]
pub use auroraview_history;

#[cfg(feature = "feature-downloads")]
pub use auroraview_downloads;

#[cfg(feature = "feature-settings")]
pub use auroraview_settings;

#[cfg(feature = "feature-notifications")]
pub use auroraview_notifications;

#[cfg(feature = "feature-devtools")]
pub use auroraview_devtools;

/// Aggregated features for WebView applications.
///
/// This struct holds optional instances of each feature manager.
/// Features are enabled by calling the corresponding `with_*` method.
///
/// # Thread Safety
///
/// Most feature managers are NOT thread-safe by default. Use appropriate
/// synchronization (e.g., `Arc<Mutex<>>`) if sharing across threads.
#[derive(Default)]
pub struct Features {
    /// Tab management (requires `feature-tabs` feature)
    #[cfg(feature = "feature-tabs")]
    pub tabs: Option<auroraview_tabs::TabManager>,

    /// Bookmark storage (requires `feature-bookmarks` feature)
    #[cfg(feature = "feature-bookmarks")]
    pub bookmarks: Option<auroraview_bookmarks::BookmarkManager>,

    /// Browsing history (requires `feature-history` feature)
    #[cfg(feature = "feature-history")]
    pub history: Option<auroraview_history::HistoryManager>,

    /// Download management (requires `feature-downloads` feature)
    #[cfg(feature = "feature-downloads")]
    pub downloads: Option<auroraview_downloads::DownloadManager>,

    /// User preferences (requires `feature-settings` feature)
    #[cfg(feature = "feature-settings")]
    pub settings: Option<auroraview_settings::SettingsManager>,

    /// Notification system (requires `feature-notifications` feature)
    #[cfg(feature = "feature-notifications")]
    pub notifications: Option<auroraview_notifications::NotificationManager>,

    /// DevTools management (requires `feature-devtools` feature)
    #[cfg(feature = "feature-devtools")]
    pub devtools_manager: Option<auroraview_devtools::DevToolsManager>,
}

impl Features {
    /// Create a new Features instance with no features enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable tab management.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let features = Features::new().with_tabs();
    /// let tabs = features.tabs.as_ref().unwrap();
    /// let id = tabs.create("https://github.com");
    /// ```
    #[cfg(feature = "feature-tabs")]
    pub fn with_tabs(mut self) -> Self {
        self.tabs = Some(auroraview_tabs::TabManager::new());
        self
    }

    /// Enable bookmark storage.
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Optional directory for storing bookmark data.
    ///                If None, bookmarks are stored in memory only.
    #[cfg(feature = "feature-bookmarks")]
    pub fn with_bookmarks(mut self, data_dir: Option<&Path>) -> Self {
        self.bookmarks = Some(auroraview_bookmarks::BookmarkManager::new(data_dir));
        self
    }

    /// Enable browsing history.
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Optional directory for storing history data.
    #[cfg(feature = "feature-history")]
    pub fn with_history(mut self, data_dir: Option<&Path>) -> Self {
        self.history = Some(auroraview_history::HistoryManager::new(data_dir));
        self
    }

    /// Enable download management.
    ///
    /// # Arguments
    ///
    /// * `download_dir` - Optional directory for storing downloads.
    ///                    If None, uses system default downloads folder.
    #[cfg(feature = "feature-downloads")]
    pub fn with_downloads(mut self, download_dir: Option<&Path>) -> Self {
        self.downloads = Some(auroraview_downloads::DownloadManager::new(download_dir));
        self
    }

    /// Enable user preferences storage.
    ///
    /// Note: SettingsManager uses in-memory storage by default.
    /// Use `settings.load_from()` to persist to a file.
    #[cfg(feature = "feature-settings")]
    #[allow(unused_variables)]
    pub fn with_settings(mut self, data_dir: Option<&Path>) -> Self {
        // SettingsManager doesn't take a path in constructor,
        // persistence is handled separately via save_to/load_from
        self.settings = Some(auroraview_settings::SettingsManager::new());
        self
    }

    /// Enable notification system.
    #[cfg(feature = "feature-notifications")]
    pub fn with_notifications(mut self) -> Self {
        self.notifications = Some(auroraview_notifications::NotificationManager::new());
        self
    }

    /// Enable DevTools management.
    #[cfg(feature = "feature-devtools")]
    pub fn with_devtools(mut self) -> Self {
        self.devtools_manager =
            Some(auroraview_devtools::DevToolsManager::new(Default::default()));
        self
    }

    /// Check if any features are enabled.
    #[allow(unused_mut)]
    pub fn has_any(&self) -> bool {
        let mut has = false;

        #[cfg(feature = "feature-tabs")]
        {
            has = has || self.tabs.is_some();
        }

        #[cfg(feature = "feature-bookmarks")]
        {
            has = has || self.bookmarks.is_some();
        }

        #[cfg(feature = "feature-history")]
        {
            has = has || self.history.is_some();
        }

        #[cfg(feature = "feature-downloads")]
        {
            has = has || self.downloads.is_some();
        }

        #[cfg(feature = "feature-settings")]
        {
            has = has || self.settings.is_some();
        }

        #[cfg(feature = "feature-notifications")]
        {
            has = has || self.notifications.is_some();
        }

        #[cfg(feature = "feature-devtools")]
        {
            has = has || self.devtools_manager.is_some();
        }

        has
    }
}

/// Configuration for enabling features.
///
/// This can be used in Python bindings or configuration files
/// to specify which features to enable.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeaturesConfig {
    /// Enable tab management
    #[serde(default)]
    pub tabs: bool,

    /// Enable bookmarks with optional data directory
    #[serde(default)]
    pub bookmarks: Option<String>,

    /// Enable history with optional data directory
    #[serde(default)]
    pub history: Option<String>,

    /// Enable downloads with optional download directory
    #[serde(default)]
    pub downloads: Option<String>,

    /// Enable settings with optional data directory
    #[serde(default)]
    pub settings: Option<String>,

    /// Enable notifications
    #[serde(default)]
    pub notifications: bool,

    /// Enable DevTools manager
    #[serde(default)]
    pub devtools: bool,
}

impl FeaturesConfig {
    /// Create Features from this configuration.
    #[allow(unused_mut)]
    pub fn build(&self) -> Features {
        let mut features = Features::new();

        #[cfg(feature = "feature-tabs")]
        if self.tabs {
            features = features.with_tabs();
        }

        #[cfg(feature = "feature-bookmarks")]
        if let Some(ref dir) = self.bookmarks {
            let path = if dir.is_empty() {
                None
            } else {
                Some(Path::new(dir))
            };
            features = features.with_bookmarks(path);
        }

        #[cfg(feature = "feature-history")]
        if let Some(ref dir) = self.history {
            let path = if dir.is_empty() {
                None
            } else {
                Some(Path::new(dir))
            };
            features = features.with_history(path);
        }

        #[cfg(feature = "feature-downloads")]
        if let Some(ref dir) = self.downloads {
            let path = if dir.is_empty() {
                None
            } else {
                Some(Path::new(dir))
            };
            features = features.with_downloads(path);
        }

        #[cfg(feature = "feature-settings")]
        if let Some(ref dir) = self.settings {
            let path = if dir.is_empty() {
                None
            } else {
                Some(Path::new(dir))
            };
            features = features.with_settings(path);
        }

        #[cfg(feature = "feature-notifications")]
        if self.notifications {
            features = features.with_notifications();
        }

        #[cfg(feature = "feature-devtools")]
        if self.devtools {
            features = features.with_devtools();
        }

        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_features_default() {
        let features = Features::new();
        assert!(!features.has_any());
    }

    #[test]
    fn test_features_config_default() {
        let config = FeaturesConfig::default();
        assert!(!config.tabs);
        assert!(config.bookmarks.is_none());
    }
}
