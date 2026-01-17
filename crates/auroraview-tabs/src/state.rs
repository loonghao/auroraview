//! Tab state data structures

use crate::TabGroupId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a tab
pub type TabId = String;

/// Tab state - tracks the current state of a browser tab
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
    #[serde(default)]
    pub pinned: bool,
    /// Whether this tab is muted
    #[serde(default)]
    pub muted: bool,
    /// Whether this tab is playing audio
    #[serde(default)]
    pub audible: bool,
    /// Tab group ID (if in a group)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<TabGroupId>,
    /// Position in tab bar (for ordering)
    #[serde(default)]
    pub position: u32,
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
    pub fn new(id: TabId, url: impl Into<String>) -> Self {
        let url = url.into();
        let security_state = Self::determine_security_state(&url);

        Self {
            id,
            title: "New Tab".to_string(),
            url,
            is_loading: true,
            can_go_back: false,
            can_go_forward: false,
            favicon: None,
            security_state: Some(security_state),
            pinned: false,
            muted: false,
            audible: false,
            group_id: None,
            position: 0,
        }
    }

    /// Create a tab state with custom ID and title
    pub fn with_title(id: TabId, url: impl Into<String>, title: impl Into<String>) -> Self {
        let mut state = Self::new(id, url);
        state.title = title.into();
        state
    }

    /// Determine security state from URL
    fn determine_security_state(url: &str) -> SecurityState {
        if url.starts_with("https://") {
            SecurityState::Secure
        } else if url.starts_with("http://") {
            SecurityState::Insecure
        } else {
            SecurityState::Neutral
        }
    }

    /// Generate a new unique tab ID
    pub fn generate_id() -> TabId {
        uuid::Uuid::new_v4().to_string()
    }

    /// Update title
    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if !title.is_empty() {
            self.title = title;
        }
    }

    /// Update URL
    pub fn set_url(&mut self, url: impl Into<String>) {
        let url = url.into();
        self.security_state = Some(Self::determine_security_state(&url));
        self.url = url;
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

    /// Set pinned state
    pub fn set_pinned(&mut self, pinned: bool) {
        self.pinned = pinned;
    }

    /// Set muted state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Set audible state
    pub fn set_audible(&mut self, audible: bool) {
        self.audible = audible;
    }

    /// Set group ID
    pub fn set_group(&mut self, group_id: Option<TabGroupId>) {
        self.group_id = group_id;
    }

    /// Set position
    pub fn set_position(&mut self, position: u32) {
        self.position = position;
    }

    /// Check if tab is secure
    pub fn is_secure(&self) -> bool {
        self.security_state == Some(SecurityState::Secure)
    }

    /// Get domain from URL
    pub fn domain(&self) -> Option<&str> {
        self.url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .and_then(|s| s.split('/').next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_state_creation() {
        let state = TabState::new("tab-1".to_string(), "https://github.com");

        assert_eq!(state.id, "tab-1");
        assert_eq!(state.url, "https://github.com");
        assert_eq!(state.title, "New Tab");
        assert!(state.is_loading);
        assert!(state.is_secure());
    }

    #[test]
    fn test_security_state() {
        let https = TabState::new("1".to_string(), "https://example.com");
        assert_eq!(https.security_state, Some(SecurityState::Secure));

        let http = TabState::new("2".to_string(), "http://example.com");
        assert_eq!(http.security_state, Some(SecurityState::Insecure));

        let file = TabState::new("3".to_string(), "file:///path/to/file");
        assert_eq!(file.security_state, Some(SecurityState::Neutral));
    }

    #[test]
    fn test_domain() {
        let state = TabState::new("1".to_string(), "https://github.com/rust-lang/rust");
        assert_eq!(state.domain(), Some("github.com"));
    }

    #[test]
    fn test_state_updates() {
        let mut state = TabState::new("1".to_string(), "https://example.com");

        state.set_title("Example");
        assert_eq!(state.title, "Example");

        state.set_loading(false);
        assert!(!state.is_loading);

        state.set_history_state(true, false);
        assert!(state.can_go_back);
        assert!(!state.can_go_forward);

        state.set_pinned(true);
        assert!(state.pinned);
    }
}
