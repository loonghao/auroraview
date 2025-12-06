//! Menu module - Native menu bar support
//!
//! Provides cross-platform menu bar functionality with keyboard shortcuts.
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_core::menu::{Menu, MenuItem, MenuAction};
//!
//! let file_menu = Menu::new("File")
//!     .add_item(MenuItem::action("New", "new", Some("Ctrl+N")))
//!     .add_item(MenuItem::action("Open", "open", Some("Ctrl+O")))
//!     .add_separator()
//!     .add_item(MenuItem::action("Exit", "exit", Some("Alt+F4")));
//!
//! let menu_bar = MenuBar::new()
//!     .add_menu(file_menu);
//! ```

mod menu_bar;
mod menu_item;

pub use menu_bar::{Menu, MenuBar};
pub use menu_item::{Accelerator, MenuItem, MenuItemType};

/// Menu action event data
#[derive(Debug, Clone)]
pub struct MenuAction {
    /// Action identifier (e.g., "file.new", "edit.copy")
    pub action_id: String,
    /// Menu item label
    pub label: String,
    /// Whether item is checked (for checkbox items)
    pub checked: Option<bool>,
}

impl MenuAction {
    /// Create a new menu action
    pub fn new(action_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            checked: None,
        }
    }

    /// Create a checkbox action with state
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_action_creation() {
        let action = MenuAction::new("file.new", "New File");
        assert_eq!(action.action_id, "file.new");
        assert_eq!(action.label, "New File");
        assert!(action.checked.is_none());
    }

    #[test]
    fn test_menu_action_with_checked() {
        let action = MenuAction::new("view.sidebar", "Show Sidebar").with_checked(true);
        assert_eq!(action.checked, Some(true));
    }
}
