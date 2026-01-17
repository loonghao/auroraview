//! Tab data structures

use serde::{Deserialize, Serialize};
use wry::WebView as WryWebView;

/// Unique identifier for a tab
pub type TabId = String;

/// Tab state - tracks the current state of a browser tab
///
/// This struct mirrors the state tracking in Microsoft's Tab.cpp/Tab.h:
/// - Title updates from `document.title` changes
/// - URL updates from navigation events
/// - Loading state from `NavigationStarting`/`NavigationCompleted`
/// - History state from `HistoryChanged` events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    /// Unique tab identifier
    pub id: TabId,
    /// Tab title (from document.title)
    pub title: String,
    /// Current URL
    pub url: String,
    /// Whether the tab is currently loading
    pub is_loading: bool,
    /// Whether back navigation is possible
    pub can_go_back: bool,
    /// Whether forward navigation is possible
    pub can_go_forward: bool,
    /// Favicon URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// Security state (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_state: Option<SecurityState>,
    /// Whether this tab is pinned
    pub pinned: bool,
    /// Whether this tab is muted
    pub muted: bool,
    /// Whether this tab is playing audio
    pub audible: bool,
}

/// Security state of the current page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityState {
    /// Secure (HTTPS with valid certificate)
    Secure,
    /// Insecure (HTTP or invalid certificate)
    Insecure,
    /// Unknown/neutral
    Neutral,
}

impl TabState {
    /// Create a new tab state
    pub fn new(id: TabId, url: String) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url,
            is_loading: true,
            can_go_back: false,
            can_go_forward: false,
            favicon: None,
            security_state: None,
            pinned: false,
            muted: false,
            audible: false,
        }
    }

    /// Update title
    pub fn set_title(&mut self, title: String) {
        if !title.is_empty() {
            self.title = title;
        }
    }

    /// Update URL
    pub fn set_url(&mut self, url: String) {
        self.url = url.clone();
        // Update security state based on URL
        self.security_state = Some(if url.starts_with("https://") {
            SecurityState::Secure
        } else if url.starts_with("http://") {
            SecurityState::Insecure
        } else {
            SecurityState::Neutral
        });
    }

    /// Update loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Update history state
    pub fn set_history_state(&mut self, can_go_back: bool, can_go_forward: bool) {
        self.can_go_back = can_go_back;
        self.can_go_forward = can_go_forward;
    }

    /// Set favicon URL
    pub fn set_favicon(&mut self, favicon: Option<String>) {
        self.favicon = favicon;
    }

    /// Toggle pinned state
    pub fn set_pinned(&mut self, pinned: bool) {
        self.pinned = pinned;
    }

    /// Toggle muted state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Set audible state
    pub fn set_audible(&mut self, audible: bool) {
        self.audible = audible;
    }
}

/// A tab with its WebView
pub struct Tab {
    /// Tab state (serializable)
    pub state: TabState,
    /// WebView instance
    pub webview: WryWebView,
}

impl Tab {
    /// Create a new tab
    pub fn new(state: TabState, webview: WryWebView) -> Self {
        Self { state, webview }
    }

    /// Get tab ID
    pub fn id(&self) -> &TabId {
        &self.state.id
    }

    /// Get tab state
    pub fn state(&self) -> &TabState {
        &self.state
    }

    /// Get mutable tab state
    pub fn state_mut(&mut self) -> &mut TabState {
        &mut self.state
    }

    /// Navigate to URL
    pub fn navigate(&mut self, url: &str) -> crate::Result<()> {
        self.state.url = url.to_string();
        self.state.is_loading = true;
        self.webview
            .load_url(url)
            .map_err(|e| crate::BrowserError::Navigation(e.to_string()))
    }

    /// Go back in history
    pub fn go_back(&self) -> crate::Result<()> {
        self.webview
            .evaluate_script("history.back()")
            .map_err(|e| crate::BrowserError::Navigation(e.to_string()))
    }

    /// Go forward in history
    pub fn go_forward(&self) -> crate::Result<()> {
        self.webview
            .evaluate_script("history.forward()")
            .map_err(|e| crate::BrowserError::Navigation(e.to_string()))
    }

    /// Reload the page
    pub fn reload(&self) -> crate::Result<()> {
        self.webview
            .evaluate_script("location.reload()")
            .map_err(|e| crate::BrowserError::Navigation(e.to_string()))
    }

    /// Stop loading
    pub fn stop(&self) -> crate::Result<()> {
        self.webview
            .evaluate_script("window.stop()")
            .map_err(|e| crate::BrowserError::Navigation(e.to_string()))
    }

    /// Set visibility
    pub fn set_visible(&self, visible: bool) -> crate::Result<()> {
        self.webview
            .set_visible(visible)
            .map_err(|e| crate::BrowserError::WebViewCreation(e.to_string()))
    }
}
