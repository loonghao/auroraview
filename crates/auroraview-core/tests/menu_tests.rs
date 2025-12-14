//! Menu module tests

use auroraview_core::menu::{Menu, MenuAction, MenuBar, MenuItem, MenuItemType};

// ============================================================================
// MenuAction Tests (from mod.rs)
// ============================================================================

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

// ============================================================================
// MenuItem Tests (from menu_item.rs)
// ============================================================================

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

// ============================================================================
// Menu & MenuBar Tests (from menu_bar.rs)
// ============================================================================

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
