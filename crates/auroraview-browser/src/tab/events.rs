//! Tab events

use super::TabId;
use serde::{Deserialize, Serialize};

/// Events related to tab management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabEvent {
    // === Tab Management Events (from UI) ===
    /// Create a new tab with optional URL
    NewTab {
        url: Option<String>,
    },
    /// Close a specific tab
    CloseTab {
        tab_id: TabId,
    },
    /// Activate/switch to a tab
    ActivateTab {
        tab_id: TabId,
    },
    /// Duplicate a tab
    DuplicateTab {
        tab_id: TabId,
    },
    /// Pin/unpin a tab
    PinTab {
        tab_id: TabId,
        pinned: bool,
    },
    /// Mute/unmute a tab
    MuteTab {
        tab_id: TabId,
        muted: bool,
    },
    /// Reorder tabs (drag and drop)
    ReorderTab {
        tab_id: TabId,
        new_index: usize,
    },

    // === Navigation Events (from UI) ===
    /// Navigate the active tab to a URL
    Navigate {
        url: String,
    },
    /// Go back in active tab history
    GoBack,
    /// Go forward in active tab history
    GoForward,
    /// Reload the active tab
    Reload,
    /// Stop loading the active tab
    Stop,
    /// Navigate to home page
    Home,

    // === Tab State Update Events (from Tab WebViews) ===
    /// Tab title changed
    TitleChanged {
        tab_id: TabId,
        title: String,
    },
    /// Tab URL changed
    UrlChanged {
        tab_id: TabId,
        url: String,
    },
    /// Tab loading state changed
    LoadingChanged {
        tab_id: TabId,
        is_loading: bool,
    },
    /// Tab history state changed
    HistoryChanged {
        tab_id: TabId,
        can_go_back: bool,
        can_go_forward: bool,
    },
    /// Tab favicon changed
    FaviconChanged {
        tab_id: TabId,
        favicon_url: String,
    },
    /// Tab audible state changed
    AudibleChanged {
        tab_id: TabId,
        audible: bool,
    },

    // === Window Events ===
    /// Close the browser window
    Close,
    /// Minimize the window
    Minimize,
    /// Toggle maximize/restore the window
    ToggleMaximize,
    /// Resize the content area
    ResizeContent {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },

    // === DevTools Events ===
    /// Toggle DevTools for a tab
    ToggleDevTools {
        tab_id: Option<TabId>,
    },
    /// Open DevTools for a tab
    OpenDevTools {
        tab_id: Option<TabId>,
    },
    /// Close DevTools for a tab
    CloseDevTools {
        tab_id: Option<TabId>,
    },
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
}
