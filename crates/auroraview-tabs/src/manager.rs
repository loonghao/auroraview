//! Tab manager implementation

use crate::{Result, TabError, TabEvent, TabGroup, TabGroupId, TabId, TabState};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Tab manager - manages tab states without WebView dependency
///
/// This is a WebView-agnostic tab manager that handles:
/// - Tab state management
/// - Tab ordering
/// - Tab groups
/// - Active tab tracking
///
/// The actual WebView instances are managed by the Browser or application layer.
pub struct TabManager {
    /// All tab states indexed by ID
    tabs: RwLock<HashMap<TabId, TabState>>,
    /// Currently active tab ID
    active_tab_id: RwLock<Option<TabId>>,
    /// Order of tabs for UI display
    tab_order: RwLock<Vec<TabId>>,
    /// Tab groups
    groups: RwLock<HashMap<TabGroupId, TabGroup>>,
    /// Counter for generating unique tab IDs
    tab_counter: AtomicU32,
    /// Event handlers
    event_handlers: RwLock<Vec<Box<dyn Fn(&TabEvent) + Send + Sync>>>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            tabs: RwLock::new(HashMap::new()),
            active_tab_id: RwLock::new(None),
            tab_order: RwLock::new(Vec::new()),
            groups: RwLock::new(HashMap::new()),
            tab_counter: AtomicU32::new(0),
            event_handlers: RwLock::new(Vec::new()),
        }
    }

    /// Generate a unique tab ID
    fn next_tab_id(&self) -> TabId {
        let id = self.tab_counter.fetch_add(1, Ordering::SeqCst);
        format!("tab_{}", id)
    }

    /// Emit an event to all handlers
    fn emit(&self, event: &TabEvent) {
        let handlers = self.event_handlers.read();
        for handler in handlers.iter() {
            handler(event);
        }
    }

    // ========== Tab Operations ==========

    /// Create a new tab
    pub fn create(&self, url: impl Into<String>) -> TabId {
        let tab_id = self.next_tab_id();
        let state = TabState::new(tab_id.clone(), url);

        // Insert tab
        {
            let mut tabs = self.tabs.write();
            tabs.insert(tab_id.clone(), state);
        }

        // Add to order
        {
            let mut order = self.tab_order.write();
            order.push(tab_id.clone());
        }

        // If first tab, make active
        {
            let mut active = self.active_tab_id.write();
            if active.is_none() {
                *active = Some(tab_id.clone());
            }
        }

        self.emit(&TabEvent::Created {
            tab_id: tab_id.clone(),
        });

        tab_id
    }

    /// Create a tab with custom state
    pub fn create_with_state(&self, state: TabState) -> TabId {
        let tab_id = state.id.clone();

        // Insert tab
        {
            let mut tabs = self.tabs.write();
            tabs.insert(tab_id.clone(), state);
        }

        // Add to order
        {
            let mut order = self.tab_order.write();
            order.push(tab_id.clone());
        }

        // If first tab, make active
        {
            let mut active = self.active_tab_id.write();
            if active.is_none() {
                *active = Some(tab_id.clone());
            }
        }

        self.emit(&TabEvent::Created {
            tab_id: tab_id.clone(),
        });

        tab_id
    }

    /// Close a tab
    pub fn close(&self, tab_id: &TabId) -> Result<()> {
        // Check if tab exists
        {
            let tabs = self.tabs.read();
            if !tabs.contains_key(tab_id) {
                return Err(TabError::NotFound(tab_id.clone()));
            }
        }

        // Remove from groups
        {
            let mut groups = self.groups.write();
            for group in groups.values_mut() {
                group.remove_tab(tab_id);
            }
        }

        // Remove from order
        {
            let mut order = self.tab_order.write();
            order.retain(|id| id != tab_id);
        }

        // Remove tab
        {
            let mut tabs = self.tabs.write();
            tabs.remove(tab_id);
        }

        // Update active tab
        {
            let mut active = self.active_tab_id.write();
            if active.as_ref() == Some(tab_id) {
                let order = self.tab_order.read();
                *active = order.first().cloned();
            }
        }

        self.emit(&TabEvent::Closed {
            tab_id: tab_id.clone(),
        });

        Ok(())
    }

    /// Activate a tab
    pub fn activate(&self, tab_id: &TabId) -> Result<()> {
        let tabs = self.tabs.read();
        if !tabs.contains_key(tab_id) {
            return Err(TabError::NotFound(tab_id.clone()));
        }
        drop(tabs);

        let old_active = {
            let mut active = self.active_tab_id.write();
            let old = active.clone();
            *active = Some(tab_id.clone());
            old
        };

        if let Some(old_id) = old_active {
            if old_id != *tab_id {
                self.emit(&TabEvent::Deactivated { tab_id: old_id });
            }
        }

        self.emit(&TabEvent::Activated {
            tab_id: tab_id.clone(),
        });

        Ok(())
    }

    /// Get a tab state
    pub fn get(&self, tab_id: &TabId) -> Option<TabState> {
        let tabs = self.tabs.read();
        tabs.get(tab_id).cloned()
    }

    /// Get mutable access to tab state
    pub fn update<F, R>(&self, tab_id: &TabId, f: F) -> Option<R>
    where
        F: FnOnce(&mut TabState) -> R,
    {
        let mut tabs = self.tabs.write();
        tabs.get_mut(tab_id).map(f)
    }

    /// Get all tabs in order
    pub fn all(&self) -> Vec<TabState> {
        let tabs = self.tabs.read();
        let order = self.tab_order.read();
        order
            .iter()
            .filter_map(|id| tabs.get(id).cloned())
            .collect()
    }

    /// Get tab count
    pub fn count(&self) -> usize {
        self.tabs.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.tabs.read().is_empty()
    }

    /// Get active tab ID
    pub fn active_id(&self) -> Option<TabId> {
        self.active_tab_id.read().clone()
    }

    /// Get active tab state
    pub fn active(&self) -> Option<TabState> {
        let active_id = self.active_tab_id.read().clone()?;
        let tabs = self.tabs.read();
        tabs.get(&active_id).cloned()
    }

    /// Get tab order
    pub fn order(&self) -> Vec<TabId> {
        self.tab_order.read().clone()
    }

    // ========== Tab State Updates ==========

    /// Update tab title
    pub fn update_title(&self, tab_id: &TabId, title: impl Into<String>) {
        let title = title.into();
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_title(&title);
        }
        drop(tabs);

        self.emit(&TabEvent::TitleChanged {
            tab_id: tab_id.clone(),
            title,
        });
    }

    /// Update tab URL
    pub fn update_url(&self, tab_id: &TabId, url: impl Into<String>) {
        let url = url.into();
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_url(&url);
        }
        drop(tabs);

        self.emit(&TabEvent::UrlChanged {
            tab_id: tab_id.clone(),
            url,
        });
    }

    /// Update tab loading state
    pub fn update_loading(&self, tab_id: &TabId, is_loading: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_loading(is_loading);
        }
        drop(tabs);

        self.emit(&TabEvent::LoadingChanged {
            tab_id: tab_id.clone(),
            is_loading,
        });
    }

    /// Update tab history state
    pub fn update_history(&self, tab_id: &TabId, can_go_back: bool, can_go_forward: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_history_state(can_go_back, can_go_forward);
        }
        drop(tabs);

        self.emit(&TabEvent::HistoryChanged {
            tab_id: tab_id.clone(),
            can_go_back,
            can_go_forward,
        });
    }

    /// Update tab favicon
    pub fn update_favicon(&self, tab_id: &TabId, favicon_url: impl Into<String>) {
        let favicon_url = favicon_url.into();
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_favicon(Some(favicon_url.clone()));
        }
        drop(tabs);

        self.emit(&TabEvent::FaviconChanged {
            tab_id: tab_id.clone(),
            favicon_url,
        });
    }

    // ========== Tab Actions ==========

    /// Pin/unpin a tab
    pub fn set_pinned(&self, tab_id: &TabId, pinned: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_pinned(pinned);
        }
    }

    /// Mute/unmute a tab
    pub fn set_muted(&self, tab_id: &TabId, muted: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.set_muted(muted);
        }
    }

    /// Reorder a tab
    pub fn reorder(&self, tab_id: &TabId, new_index: usize) {
        let mut order = self.tab_order.write();
        if let Some(old_index) = order.iter().position(|id| id == tab_id) {
            let id = order.remove(old_index);
            let new_index = new_index.min(order.len());
            order.insert(new_index, id);
        }
    }

    /// Duplicate a tab (returns new tab ID)
    pub fn duplicate(&self, tab_id: &TabId) -> Result<TabId> {
        let url = {
            let tabs = self.tabs.read();
            tabs.get(tab_id)
                .map(|t| t.url.clone())
                .ok_or_else(|| TabError::NotFound(tab_id.clone()))?
        };

        Ok(self.create(url))
    }

    // ========== Tab Groups ==========

    /// Create a tab group
    pub fn create_group(&self, name: impl Into<String>) -> TabGroupId {
        let group = TabGroup::new(name);
        let group_id = group.id.clone();

        let mut groups = self.groups.write();
        groups.insert(group_id.clone(), group);

        group_id
    }

    /// Create a group with tabs
    pub fn create_group_with_tabs(
        &self,
        name: impl Into<String>,
        tab_ids: Vec<TabId>,
    ) -> TabGroupId {
        let group_id = self.create_group(name);

        for tab_id in tab_ids {
            let _ = self.add_to_group(&tab_id, &group_id);
        }

        group_id
    }

    /// Add a tab to a group
    pub fn add_to_group(&self, tab_id: &TabId, group_id: &TabGroupId) -> Result<()> {
        // Update tab state
        {
            let mut tabs = self.tabs.write();
            if let Some(tab) = tabs.get_mut(tab_id) {
                tab.set_group(Some(group_id.clone()));
            } else {
                return Err(TabError::NotFound(tab_id.clone()));
            }
        }

        // Update group
        {
            let mut groups = self.groups.write();
            if let Some(group) = groups.get_mut(group_id) {
                group.add_tab(tab_id.clone());
            } else {
                return Err(TabError::GroupNotFound(group_id.clone()));
            }
        }

        self.emit(&TabEvent::AddedToGroup {
            tab_id: tab_id.clone(),
            group_id: group_id.clone(),
        });

        Ok(())
    }

    /// Remove a tab from its group
    pub fn remove_from_group(&self, tab_id: &TabId) -> Result<()> {
        let group_id = {
            let mut tabs = self.tabs.write();
            if let Some(tab) = tabs.get_mut(tab_id) {
                let gid = tab.group_id.take();
                gid
            } else {
                return Err(TabError::NotFound(tab_id.clone()));
            }
        };

        if let Some(group_id) = group_id {
            let mut groups = self.groups.write();
            if let Some(group) = groups.get_mut(&group_id) {
                group.remove_tab(tab_id);
            }

            self.emit(&TabEvent::RemovedFromGroup {
                tab_id: tab_id.clone(),
                group_id,
            });
        }

        Ok(())
    }

    /// Get a group
    pub fn get_group(&self, group_id: &TabGroupId) -> Option<TabGroup> {
        let groups = self.groups.read();
        groups.get(group_id).cloned()
    }

    /// Get all groups
    pub fn all_groups(&self) -> Vec<TabGroup> {
        let groups = self.groups.read();
        groups.values().cloned().collect()
    }

    /// Delete a group (tabs are ungrouped)
    pub fn delete_group(&self, group_id: &TabGroupId) -> Result<()> {
        let tab_ids = {
            let groups = self.groups.read();
            groups
                .get(group_id)
                .map(|g| g.tab_ids.clone())
                .ok_or_else(|| TabError::GroupNotFound(group_id.clone()))?
        };

        // Ungroup all tabs
        {
            let mut tabs = self.tabs.write();
            for tab_id in &tab_ids {
                if let Some(tab) = tabs.get_mut(tab_id) {
                    tab.set_group(None);
                }
            }
        }

        // Remove group
        {
            let mut groups = self.groups.write();
            groups.remove(group_id);
        }

        self.emit(&TabEvent::GroupDeleted {
            group_id: group_id.clone(),
        });

        Ok(())
    }

    /// Collapse/expand a group
    pub fn set_group_collapsed(&self, group_id: &TabGroupId, collapsed: bool) -> Result<()> {
        let mut groups = self.groups.write();
        if let Some(group) = groups.get_mut(group_id) {
            group.set_collapsed(collapsed);
            drop(groups);

            self.emit(&TabEvent::GroupCollapsed {
                group_id: group_id.clone(),
                collapsed,
            });

            Ok(())
        } else {
            Err(TabError::GroupNotFound(group_id.clone()))
        }
    }

    // ========== Event Handling ==========

    /// Register an event handler
    pub fn on_event<F>(&self, handler: F)
    where
        F: Fn(&TabEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.write();
        handlers.push(Box::new(handler));
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

// TabManager is Send + Sync because all fields are protected
unsafe impl Send for TabManager {}
unsafe impl Sync for TabManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tab() {
        let manager = TabManager::new();

        let id = manager.create("https://github.com");
        let tab = manager.get(&id).unwrap();

        assert_eq!(tab.url, "https://github.com");
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_active_tab() {
        let manager = TabManager::new();

        let id1 = manager.create("https://github.com");
        let id2 = manager.create("https://gitlab.com");

        // First tab should be active
        assert_eq!(manager.active_id(), Some(id1.clone()));

        // Activate second tab
        manager.activate(&id2).unwrap();
        assert_eq!(manager.active_id(), Some(id2));
    }

    #[test]
    fn test_close_tab() {
        let manager = TabManager::new();

        let id = manager.create("https://github.com");
        assert_eq!(manager.count(), 1);

        manager.close(&id).unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_tab_order() {
        let manager = TabManager::new();

        let id1 = manager.create("https://a.com");
        let id2 = manager.create("https://b.com");
        let id3 = manager.create("https://c.com");

        assert_eq!(manager.order(), vec![id1.clone(), id2.clone(), id3.clone()]);

        manager.reorder(&id3, 0);
        assert_eq!(manager.order(), vec![id3, id1, id2]);
    }

    #[test]
    fn test_tab_groups() {
        let manager = TabManager::new();

        let tab1 = manager.create("https://github.com");
        let tab2 = manager.create("https://gitlab.com");

        let group_id = manager.create_group_with_tabs("Development", vec![tab1.clone(), tab2.clone()]);

        let group = manager.get_group(&group_id).unwrap();
        assert_eq!(group.len(), 2);

        let tab = manager.get(&tab1).unwrap();
        assert_eq!(tab.group_id, Some(group_id.clone()));

        manager.remove_from_group(&tab1).unwrap();
        let tab = manager.get(&tab1).unwrap();
        assert!(tab.group_id.is_none());
    }

    #[test]
    fn test_state_updates() {
        let manager = TabManager::new();

        let id = manager.create("https://example.com");

        manager.update_title(&id, "Example Site");
        manager.update_loading(&id, false);

        let tab = manager.get(&id).unwrap();
        assert_eq!(tab.title, "Example Site");
        assert!(!tab.is_loading);
    }
}
