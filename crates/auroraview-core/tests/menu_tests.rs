//! Menu module tests

use auroraview_core::menu::{Accelerator, Menu, MenuAction, MenuBar, MenuItem, MenuItemType};
use rstest::rstest;

// ============================================================================
// MenuAction tests
// ============================================================================

#[test]
fn menu_action_creation() {
    let action = MenuAction::new("file.new", "New File");
    assert_eq!(action.action_id, "file.new");
    assert_eq!(action.label, "New File");
    assert!(action.checked.is_none());
}

#[test]
fn menu_action_with_checked_true() {
    let action = MenuAction::new("view.sidebar", "Show Sidebar").with_checked(true);
    assert_eq!(action.checked, Some(true));
}

#[test]
fn menu_action_with_checked_false() {
    let action = MenuAction::new("view.toolbar", "Show Toolbar").with_checked(false);
    assert_eq!(action.checked, Some(false));
}

#[test]
fn menu_action_clone() {
    let action = MenuAction::new("edit.copy", "Copy").with_checked(false);
    let cloned = action.clone();
    assert_eq!(cloned.action_id, action.action_id);
    assert_eq!(cloned.label, action.label);
    assert_eq!(cloned.checked, action.checked);
}

#[test]
fn menu_action_debug() {
    let action = MenuAction::new("file.save", "Save");
    let debug = format!("{:?}", action);
    assert!(debug.contains("MenuAction") || debug.contains("file.save"));
}

#[test]
fn menu_action_empty_label() {
    let action = MenuAction::new("file.divider", "");
    assert_eq!(action.label, "");
}

#[test]
fn menu_action_unicode_label() {
    let action = MenuAction::new("edit.paste", "粘贴 (Paste)");
    assert_eq!(action.label, "粘贴 (Paste)");
}

// ============================================================================
// Accelerator tests
// ============================================================================

#[test]
fn accelerator_new() {
    let acc = Accelerator::new("Ctrl+N");
    assert_eq!(acc.key, "Ctrl+N");
}

#[test]
fn accelerator_parse_valid() {
    let acc = Accelerator::parse("Ctrl+S");
    assert!(acc.is_some());
    assert_eq!(acc.unwrap().key, "Ctrl+S");
}

#[test]
fn accelerator_parse_empty() {
    let acc = Accelerator::parse("");
    assert!(acc.is_none());
}

#[rstest]
#[case("Ctrl+N")]
#[case("Alt+F4")]
#[case("Ctrl+Shift+S")]
#[case("F5")]
#[case("Escape")]
fn accelerator_parse_various(#[case] key: &str) {
    let acc = Accelerator::parse(key);
    assert!(acc.is_some());
    assert_eq!(acc.unwrap().key, key);
}

#[test]
fn accelerator_clone() {
    let acc = Accelerator::new("Ctrl+Z");
    let cloned = acc.clone();
    assert_eq!(cloned.key, acc.key);
}

#[test]
fn accelerator_debug() {
    let acc = Accelerator::new("Ctrl+C");
    let debug = format!("{:?}", acc);
    assert!(debug.contains("Ctrl+C") || debug.contains("Accelerator"));
}

// ============================================================================
// MenuItem tests
// ============================================================================

#[test]
fn menu_item_action() {
    let item = MenuItem::action("New", "file.new", Some("Ctrl+N"));
    assert_eq!(item.label, "New");
    assert_eq!(item.action_id, Some("file.new".to_string()));
    assert!(matches!(item.item_type, MenuItemType::Action));
    assert!(item.accelerator.is_some());
    assert!(item.enabled);
    assert!(!item.checked);
}

#[test]
fn menu_item_action_no_accelerator() {
    let item = MenuItem::action("Undo", "edit.undo", None);
    assert!(item.accelerator.is_none());
}

#[test]
fn menu_item_checkbox_checked() {
    let item = MenuItem::checkbox("Show Sidebar", "view.sidebar", true, None);
    assert!(item.checked);
    assert!(matches!(item.item_type, MenuItemType::Checkbox));
}

#[test]
fn menu_item_checkbox_unchecked() {
    let item = MenuItem::checkbox("Show Toolbar", "view.toolbar", false, None);
    assert!(!item.checked);
}

#[test]
fn menu_item_separator() {
    let item = MenuItem::separator();
    assert!(matches!(item.item_type, MenuItemType::Separator));
    assert!(item.action_id.is_none());
    assert!(item.label.is_empty());
}

#[test]
fn menu_item_submenu() {
    let children = vec![
        MenuItem::action("New File", "file.new", Some("Ctrl+N")),
        MenuItem::action("Open", "file.open", Some("Ctrl+O")),
    ];
    let item = MenuItem::submenu("Recent", children);
    assert!(matches!(item.item_type, MenuItemType::Submenu));
    assert_eq!(item.children.len(), 2);
}

#[test]
fn menu_item_enabled_false() {
    let item = MenuItem::action("Paste", "edit.paste", None).enabled(false);
    assert!(!item.enabled);
}

#[test]
fn menu_item_checked_builder() {
    let item = MenuItem::checkbox("Dark Mode", "view.dark", false, None).checked(true);
    assert!(item.checked);
}

#[test]
fn menu_item_clone() {
    let item = MenuItem::action("Copy", "edit.copy", Some("Ctrl+C"));
    let cloned = item.clone();
    assert_eq!(cloned.label, item.label);
    assert_eq!(cloned.action_id, item.action_id);
}

#[test]
fn menu_item_debug() {
    let item = MenuItem::action("Test", "test.action", None);
    let debug = format!("{:?}", item);
    assert!(debug.contains("MenuItem") || debug.contains("Test"));
}

// ============================================================================
// Menu tests
// ============================================================================

#[test]
fn menu_creation() {
    let menu = Menu::new("File")
        .add_item(MenuItem::action("New", "file.new", Some("Ctrl+N")))
        .add_separator()
        .add_item(MenuItem::action("Exit", "file.exit", None));
    assert_eq!(menu.label, "File");
    assert_eq!(menu.items.len(), 3);
}

#[test]
fn menu_empty() {
    let menu = Menu::new("Help");
    assert_eq!(menu.label, "Help");
    assert!(menu.items.is_empty());
}

#[test]
fn menu_single_item() {
    let menu = Menu::new("Edit").add_item(MenuItem::action("Undo", "edit.undo", Some("Ctrl+Z")));
    assert_eq!(menu.items.len(), 1);
}

#[test]
fn menu_many_items() {
    let mut menu = Menu::new("Tools");
    for i in 0..10 {
        menu = menu.add_item(MenuItem::action(
            format!("Tool {}", i),
            format!("tools.t{}", i),
            None,
        ));

    }
    assert_eq!(menu.items.len(), 10);
}

#[test]
fn menu_with_separators() {
    let menu = Menu::new("Edit")
        .add_item(MenuItem::action("Undo", "edit.undo", Some("Ctrl+Z")))
        .add_item(MenuItem::action("Redo", "edit.redo", Some("Ctrl+Y")))
        .add_separator()
        .add_item(MenuItem::action("Cut", "edit.cut", Some("Ctrl+X")))
        .add_item(MenuItem::action("Copy", "edit.copy", Some("Ctrl+C")))
        .add_item(MenuItem::action("Paste", "edit.paste", Some("Ctrl+V")));
    assert_eq!(menu.items.len(), 6);
}

// ============================================================================
// MenuBar tests
// ============================================================================

#[test]
fn menu_bar_file_edit() {
    let bar = MenuBar::new().with_file_menu().with_edit_menu();
    assert_eq!(bar.menus.len(), 2);
}

#[test]
fn menu_bar_empty() {
    let bar = MenuBar::new();
    assert!(bar.menus.is_empty());
}

#[test]
fn menu_bar_add_menu() {
    let bar = MenuBar::new().add_menu(Menu::new("Custom"));
    assert_eq!(bar.menus.len(), 1);
    assert_eq!(bar.menus[0].label, "Custom");
}

#[test]
fn menu_bar_multiple_custom_menus() {
    let bar = MenuBar::new()
        .add_menu(Menu::new("File"))
        .add_menu(Menu::new("Edit"))
        .add_menu(Menu::new("View"))
        .add_menu(Menu::new("Help"));
    assert_eq!(bar.menus.len(), 4);
}

#[test]
fn menu_item_type_debug() {
    let variants = [
        MenuItemType::Action,
        MenuItemType::Checkbox,
        MenuItemType::Radio,
        MenuItemType::Separator,
        MenuItemType::Submenu,
    ];
    for v in &variants {
        let debug = format!("{:?}", v);
        assert!(!debug.is_empty());
    }
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn menu_action_id_formats() {
    let ids = ["file.new", "edit.copy", "dcc.maya.run_mel", "tool.apply_modifier"];
    for id in ids {
        let action = MenuAction::new(id, "Label");
        assert_eq!(action.action_id, id);
    }
}

#[test]
fn menu_item_action_sends_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MenuItem>();
    assert_send_sync::<MenuBar>();
    assert_send_sync::<Menu>();
}

#[rstest]
#[case("Ctrl+Z", "edit.undo")]
#[case("Ctrl+Y", "edit.redo")]
#[case("Ctrl+Shift+S", "file.save_as")]
#[case("F5", "view.refresh")]
fn menu_item_with_various_accelerators(#[case] key: &str, #[case] action_id: &str) {
    let item = MenuItem::action("Label", action_id, Some(key));
    assert_eq!(item.action_id.as_deref(), Some(action_id));
    assert!(item.accelerator.is_some());
    assert_eq!(item.accelerator.unwrap().key, key);
}

#[test]
fn menu_item_separator_label_empty() {
    let item = MenuItem::separator();
    assert!(item.label.is_empty());
    assert!(item.action_id.is_none());
    assert!(item.accelerator.is_none());
}

#[test]
fn menu_item_checkbox_toggle() {
    let item_on = MenuItem::checkbox("Feature", "feat.on", true, None);
    let item_off = MenuItem::checkbox("Feature", "feat.on", false, None);
    assert!(item_on.checked);
    assert!(!item_off.checked);
}

#[test]
fn menu_item_submenu_recursive() {
    let inner = Menu::new("Inner").add_item(MenuItem::action("Nested", "n.action", None));
    let outer_item = MenuItem::submenu("Outer", inner.items.clone());
    assert!(matches!(outer_item.item_type, MenuItemType::Submenu));
    assert!(!outer_item.children.is_empty());
}

#[test]
fn menu_bar_with_file_menu_has_items() {
    let bar = MenuBar::new().with_file_menu();
    assert_eq!(bar.menus.len(), 1);
    assert!(!bar.menus[0].items.is_empty());
}

#[test]
fn menu_bar_with_edit_menu_has_items() {
    let bar = MenuBar::new().with_edit_menu();
    assert_eq!(bar.menus.len(), 1);
    assert!(!bar.menus[0].items.is_empty());
}

#[test]
fn menu_with_only_separators() {
    let menu = Menu::new("Sep")
        .add_separator()
        .add_separator();
    assert_eq!(menu.items.len(), 2);
    for item in &menu.items {
        assert!(matches!(item.item_type, MenuItemType::Separator));
    }
}

#[test]
fn accelerator_parse_whitespace() {
    // Whitespace-only string: parse behavior depends on implementation.
    // We only verify it does not panic.
    let _ = Accelerator::parse("   ");
}

#[test]
fn menu_action_checked_none_by_default() {
    let action = MenuAction::new("test.id", "Test");
    assert!(action.checked.is_none());
}

#[test]
fn menu_item_action_enabled_by_default() {
    let item = MenuItem::action("A", "a.a", None);
    assert!(item.enabled);
    assert!(!item.checked);
}

#[test]
fn menu_item_clone_independence() {
    let item = MenuItem::action("Original", "orig.id", Some("Ctrl+A"));
    let mut cloned = item.clone();
    // Flip enabled on clone; original should be unaffected
    cloned = cloned.enabled(false);
    assert!(item.enabled);
    assert!(!cloned.enabled);
}
