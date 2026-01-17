//! Tab group support

use crate::TabId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a tab group
pub type TabGroupId = String;

/// Tab group - groups related tabs together
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabGroup {
    /// Unique identifier
    pub id: TabGroupId,
    /// Group name
    pub name: String,
    /// Group color (CSS color value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Whether the group is collapsed
    #[serde(default)]
    pub collapsed: bool,
    /// Tab IDs in this group (in order)
    #[serde(default)]
    pub tab_ids: Vec<TabId>,
}

impl TabGroup {
    /// Create a new tab group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            color: None,
            collapsed: false,
            tab_ids: Vec::new(),
        }
    }

    /// Create a group with a specific ID
    pub fn with_id(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color: None,
            collapsed: false,
            tab_ids: Vec::new(),
        }
    }

    /// Set color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set initial tabs
    pub fn with_tabs(mut self, tab_ids: Vec<TabId>) -> Self {
        self.tab_ids = tab_ids;
        self
    }

    /// Add a tab to the group
    pub fn add_tab(&mut self, tab_id: TabId) {
        if !self.tab_ids.contains(&tab_id) {
            self.tab_ids.push(tab_id);
        }
    }

    /// Add a tab at a specific position
    pub fn add_tab_at(&mut self, tab_id: TabId, index: usize) {
        if !self.tab_ids.contains(&tab_id) {
            let index = index.min(self.tab_ids.len());
            self.tab_ids.insert(index, tab_id);
        }
    }

    /// Remove a tab from the group
    pub fn remove_tab(&mut self, tab_id: &TabId) -> bool {
        if let Some(pos) = self.tab_ids.iter().position(|id| id == tab_id) {
            self.tab_ids.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if tab is in this group
    pub fn contains(&self, tab_id: &TabId) -> bool {
        self.tab_ids.contains(tab_id)
    }

    /// Get number of tabs in group
    pub fn len(&self) -> usize {
        self.tab_ids.len()
    }

    /// Check if group is empty
    pub fn is_empty(&self) -> bool {
        self.tab_ids.is_empty()
    }

    /// Set group name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Set group color
    pub fn set_color(&mut self, color: Option<String>) {
        self.color = color;
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Set collapsed state
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    /// Reorder a tab within the group
    pub fn reorder_tab(&mut self, tab_id: &TabId, new_index: usize) {
        if let Some(old_index) = self.tab_ids.iter().position(|id| id == tab_id) {
            let id = self.tab_ids.remove(old_index);
            let new_index = new_index.min(self.tab_ids.len());
            self.tab_ids.insert(new_index, id);
        }
    }
}

/// Predefined group colors (Chrome-style)
#[allow(dead_code)]
pub mod colors {
    /// Grey color for default groups
    pub const GREY: &str = "#5f6368";
    /// Blue color
    pub const BLUE: &str = "#1a73e8";
    /// Red color
    pub const RED: &str = "#d93025";
    /// Yellow color
    pub const YELLOW: &str = "#f9ab00";
    /// Green color
    pub const GREEN: &str = "#1e8e3e";
    /// Pink color
    pub const PINK: &str = "#d01884";
    /// Purple color
    pub const PURPLE: &str = "#9334e6";
    /// Cyan color
    pub const CYAN: &str = "#007b83";
    /// Orange color
    pub const ORANGE: &str = "#e8710a";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_creation() {
        let group = TabGroup::new("Work");

        assert_eq!(group.name, "Work");
        assert!(!group.collapsed);
        assert!(group.is_empty());
    }

    #[test]
    fn test_group_with_color() {
        let group = TabGroup::new("Development").with_color(colors::BLUE);

        assert_eq!(group.color, Some(colors::BLUE.to_string()));
    }

    #[test]
    fn test_add_remove_tabs() {
        let mut group = TabGroup::new("Test");

        group.add_tab("tab-1".to_string());
        group.add_tab("tab-2".to_string());

        assert_eq!(group.len(), 2);
        assert!(group.contains(&"tab-1".to_string()));

        group.remove_tab(&"tab-1".to_string());
        assert_eq!(group.len(), 1);
        assert!(!group.contains(&"tab-1".to_string()));
    }

    #[test]
    fn test_reorder_tab() {
        let mut group = TabGroup::new("Test")
            .with_tabs(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        group.reorder_tab(&"c".to_string(), 0);

        assert_eq!(group.tab_ids, vec!["c", "a", "b"]);
    }

    #[test]
    fn test_collapsed() {
        let mut group = TabGroup::new("Test");

        assert!(!group.collapsed);
        group.toggle_collapsed();
        assert!(group.collapsed);
        group.toggle_collapsed();
        assert!(!group.collapsed);
    }
}
