//! Menu and MenuBar containers

use super::MenuItem;

/// A menu (dropdown from menu bar)
#[derive(Debug, Clone)]
pub struct Menu {
    /// Menu label (e.g., "File", "Edit")
    pub label: String,
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Whether menu is enabled
    pub enabled: bool,
}

impl Menu {
    /// Create a new menu
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            enabled: true,
        }
    }

    /// Add an item to the menu
    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add a separator
    pub fn add_separator(self) -> Self {
        self.add_item(MenuItem::separator())
    }

    /// Add multiple items
    pub fn add_items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Menu bar (top-level menu container)
#[derive(Debug, Clone, Default)]
pub struct MenuBar {
    /// Top-level menus
    pub menus: Vec<Menu>,
}

impl MenuBar {
    /// Create an empty menu bar
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Add a menu to the bar
    pub fn add_menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    /// Add multiple menus
    pub fn add_menus(mut self, menus: impl IntoIterator<Item = Menu>) -> Self {
        self.menus.extend(menus);
        self
    }

    /// Create a standard File menu
    pub fn with_file_menu(self) -> Self {
        let file_menu = Menu::new("&File")
            .add_item(MenuItem::action("&New", "file.new", Some("Ctrl+N")))
            .add_item(MenuItem::action("&Open...", "file.open", Some("Ctrl+O")))
            .add_item(MenuItem::action("&Save", "file.save", Some("Ctrl+S")))
            .add_item(MenuItem::action(
                "Save &As...",
                "file.save_as",
                Some("Ctrl+Shift+S"),
            ))
            .add_separator()
            .add_item(MenuItem::action("E&xit", "file.exit", Some("Alt+F4")));
        self.add_menu(file_menu)
    }

    /// Create a standard Edit menu
    pub fn with_edit_menu(self) -> Self {
        let edit_menu = Menu::new("&Edit")
            .add_item(MenuItem::action("&Undo", "edit.undo", Some("Ctrl+Z")))
            .add_item(MenuItem::action("&Redo", "edit.redo", Some("Ctrl+Y")))
            .add_separator()
            .add_item(MenuItem::action("Cu&t", "edit.cut", Some("Ctrl+X")))
            .add_item(MenuItem::action("&Copy", "edit.copy", Some("Ctrl+C")))
            .add_item(MenuItem::action("&Paste", "edit.paste", Some("Ctrl+V")))
            .add_separator()
            .add_item(MenuItem::action(
                "Select &All",
                "edit.select_all",
                Some("Ctrl+A"),
            ));
        self.add_menu(edit_menu)
    }

    /// Create a standard View menu with common items
    pub fn with_view_menu(self) -> Self {
        let view_menu = Menu::new("&View")
            .add_item(MenuItem::checkbox(
                "Show &Toolbar",
                "view.toolbar",
                true,
                None,
            ))
            .add_item(MenuItem::checkbox(
                "Show &Sidebar",
                "view.sidebar",
                true,
                None,
            ))
            .add_separator()
            .add_item(MenuItem::action("&Zoom In", "view.zoom_in", Some("Ctrl++")))
            .add_item(MenuItem::action(
                "Zoom &Out",
                "view.zoom_out",
                Some("Ctrl+-"),
            ))
            .add_item(MenuItem::action(
                "&Reset Zoom",
                "view.zoom_reset",
                Some("Ctrl+0"),
            ));
        self.add_menu(view_menu)
    }

    /// Create a standard Help menu
    pub fn with_help_menu(self, app_name: &str) -> Self {
        let help_menu = Menu::new("&Help")
            .add_item(MenuItem::action("&Documentation", "help.docs", Some("F1")))
            .add_item(MenuItem::action("&Check for Updates", "help.updates", None))
            .add_separator()
            .add_item(MenuItem::action(
                format!("&About {}", app_name),
                "help.about",
                None,
            ));
        self.add_menu(help_menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_creation() {
        let menu = Menu::new("File")
            .add_item(MenuItem::action("New", "file.new", Some("Ctrl+N")))
            .add_separator()
            .add_item(MenuItem::action("Exit", "file.exit", None));
        assert_eq!(menu.label, "File");
        assert_eq!(menu.items.len(), 3);
    }

    #[test]
    fn test_menu_bar() {
        let bar = MenuBar::new().with_file_menu().with_edit_menu();
        assert_eq!(bar.menus.len(), 2);
    }
}
