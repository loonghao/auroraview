//! Tab events

use crate::{TabGroupId, TabId};
use serde::{Deserialize, Serialize};

/// Events related to tab management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabEvent {
    // === Tab Lifecycle Events ===
    /// Tab created
    Created { tab_id: TabId },
    /// Tab closed
    Closed { tab_id: TabId },
    /// Tab activated
    Activated { tab_id: TabId },
    /// Tab deactivated
    Deactivated { tab_id: TabId },

    // === Tab Management Events ===
    /// Create a new tab with optional URL
    NewTab { url: Option<String> },
    /// Close a specific tab
    CloseTab { tab_id: TabId },
    /// Activate/switch to a tab
    ActivateTab { tab_id: TabId },
    /// Duplicate a tab
    DuplicateTab { tab_id: TabId },
    /// Pin/unpin a tab
    PinTab { tab_id: TabId, pinned: bool },
    /// Mute/unmute a tab
    MuteTab { tab_id: TabId, muted: bool },
    /// Reorder tabs (drag and drop)
    ReorderTab { tab_id: TabId, new_index: usize },

    // === Tab State Update Events ===
    /// Tab title changed
    TitleChanged { tab_id: TabId, title: String },
    /// Tab URL changed
    UrlChanged { tab_id: TabId, url: String },
    /// Tab loading state changed
    LoadingChanged { tab_id: TabId, is_loading: bool },
    /// Tab history state changed
    HistoryChanged {
        tab_id: TabId,
        can_go_back: bool,
        can_go_forward: bool,
    },
    /// Tab favicon changed
    FaviconChanged { tab_id: TabId, favicon_url: String },
    /// Tab audible state changed
    AudibleChanged { tab_id: TabId, audible: bool },

    // === Tab Group Events ===
    /// Tab added to group
    AddedToGroup { tab_id: TabId, group_id: TabGroupId },
    /// Tab removed from group
    RemovedFromGroup { tab_id: TabId, group_id: TabGroupId },
    /// Group created
    GroupCreated { group_id: TabGroupId, name: String },
    /// Group deleted
    GroupDeleted { group_id: TabGroupId },
    /// Group collapsed/expanded
    GroupCollapsed { group_id: TabGroupId, collapsed: bool },

    // === Navigation Events ===
    /// Navigate to a URL
    Navigate { url: String },
    /// Go back in history
    GoBack,
    /// Go forward in history
    GoForward,
    /// Reload the page
    Reload,
    /// Stop loading
    Stop,
    /// Navigate to home page
    Home,
}

impl TabEvent {
    /// Create a new tab event
    pub fn new_tab(url: Option<String>) -> Self {
        Self::NewTab { url }
    }

    /// Create a close tab event
    pub fn close_tab(tab_id: TabId) -> Self {
        Self::CloseTab { tab_id }
    }

    /// Create an activate tab event
    pub fn activate_tab(tab_id: TabId) -> Self {
        Self::ActivateTab { tab_id }
    }

    /// Create a navigate event
    pub fn navigate(url: impl Into<String>) -> Self {
        Self::Navigate { url: url.into() }
    }

    /// Create a title changed event
    pub fn title_changed(tab_id: TabId, title: impl Into<String>) -> Self {
        Self::TitleChanged {
            tab_id,
            title: title.into(),
        }
    }

    /// Create a URL changed event
    pub fn url_changed(tab_id: TabId, url: impl Into<String>) -> Self {
        Self::UrlChanged {
            tab_id,
            url: url.into(),
        }
    }

    /// Create a loading changed event
    pub fn loading_changed(tab_id: TabId, is_loading: bool) -> Self {
        Self::LoadingChanged { tab_id, is_loading }
    }

    /// Check if this event is a tab lifecycle event
    pub fn is_lifecycle_event(&self) -> bool {
        matches!(
            self,
            Self::Created { .. }
                | Self::Closed { .. }
                | Self::Activated { .. }
                | Self::Deactivated { .. }
        )
    }

    /// Check if this event is a state update event
    pub fn is_state_update(&self) -> bool {
        matches!(
            self,
            Self::TitleChanged { .. }
                | Self::UrlChanged { .. }
                | Self::LoadingChanged { .. }
                | Self::HistoryChanged { .. }
                | Self::FaviconChanged { .. }
                | Self::AudibleChanged { .. }
        )
    }

    /// Check if this event is a group event
    pub fn is_group_event(&self) -> bool {
        matches!(
            self,
            Self::AddedToGroup { .. }
                | Self::RemovedFromGroup { .. }
                | Self::GroupCreated { .. }
                | Self::GroupDeleted { .. }
                | Self::GroupCollapsed { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let new_tab = TabEvent::new_tab(Some("https://example.com".to_string()));
        assert!(matches!(new_tab, TabEvent::NewTab { .. }));

        let navigate = TabEvent::navigate("https://github.com");
        assert!(matches!(navigate, TabEvent::Navigate { .. }));
    }

    #[test]
    fn test_event_categories() {
        let created = TabEvent::Created {
            tab_id: "1".to_string(),
        };
        assert!(created.is_lifecycle_event());

        let title = TabEvent::title_changed("1".to_string(), "Test");
        assert!(title.is_state_update());

        let group = TabEvent::GroupCreated {
            group_id: "g1".to_string(),
            name: "Work".to_string(),
        };
        assert!(group.is_group_event());
    }
}
