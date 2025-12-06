//! Menu item types and accelerator keys

/// Keyboard accelerator (shortcut)
#[derive(Debug, Clone)]
pub struct Accelerator {
    /// Key combination string (e.g., "Ctrl+N", "Alt+F4", "Ctrl+Shift+S")
    pub key: String,
}

impl Accelerator {
    /// Create a new accelerator from key string
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    /// Parse accelerator from string
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }
        Some(Self::new(s))
    }
}

/// Menu item type
#[derive(Debug, Clone)]
pub enum MenuItemType {
    /// Normal clickable item
    Action,
    /// Checkbox item (toggleable)
    Checkbox,
    /// Radio item (exclusive selection within group)
    Radio,
    /// Separator line
    Separator,
    /// Submenu container
    Submenu,
}

/// A single menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Item label (with & for mnemonic, e.g., "&File")
    pub label: String,
    /// Action identifier for event handling
    pub action_id: Option<String>,
    /// Item type
    pub item_type: MenuItemType,
    /// Keyboard shortcut
    pub accelerator: Option<Accelerator>,
    /// Whether item is enabled
    pub enabled: bool,
    /// Whether item is checked (for checkbox/radio items)
    pub checked: bool,
    /// Submenu items (for Submenu type)
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    /// Create an action menu item
    pub fn action(
        label: impl Into<String>,
        action_id: impl Into<String>,
        accelerator: Option<&str>,
    ) -> Self {
        Self {
            label: label.into(),
            action_id: Some(action_id.into()),
            item_type: MenuItemType::Action,
            accelerator: accelerator.and_then(Accelerator::parse),
            enabled: true,
            checked: false,
            children: Vec::new(),
        }
    }

    /// Create a checkbox menu item
    pub fn checkbox(
        label: impl Into<String>,
        action_id: impl Into<String>,
        checked: bool,
        accelerator: Option<&str>,
    ) -> Self {
        Self {
            label: label.into(),
            action_id: Some(action_id.into()),
            item_type: MenuItemType::Checkbox,
            accelerator: accelerator.and_then(Accelerator::parse),
            enabled: true,
            checked,
            children: Vec::new(),
        }
    }

    /// Create a separator
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            action_id: None,
            item_type: MenuItemType::Separator,
            accelerator: None,
            enabled: true,
            checked: false,
            children: Vec::new(),
        }
    }

    /// Create a submenu
    pub fn submenu(label: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            action_id: None,
            item_type: MenuItemType::Submenu,
            accelerator: None,
            enabled: true,
            checked: false,
            children,
        }
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_item() {
        let item = MenuItem::action("New", "file.new", Some("Ctrl+N"));
        assert_eq!(item.label, "New");
        assert_eq!(item.action_id, Some("file.new".to_string()));
        assert!(matches!(item.item_type, MenuItemType::Action));
        assert!(item.accelerator.is_some());
    }

    #[test]
    fn test_checkbox_item() {
        let item = MenuItem::checkbox("Show Sidebar", "view.sidebar", true, None);
        assert!(item.checked);
        assert!(matches!(item.item_type, MenuItemType::Checkbox));
    }

    #[test]
    fn test_separator() {
        let item = MenuItem::separator();
        assert!(matches!(item.item_type, MenuItemType::Separator));
    }
}
