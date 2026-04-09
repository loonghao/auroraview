//! Tests for TrayConfig and TrayMenuItem

use std::path::PathBuf;

use auroraview_desktop::config::{TrayConfig, TrayMenuItem};
use rstest::rstest;

// ============================================================================
// TrayConfig construction
// ============================================================================

#[rstest]
fn tray_config_default() {
    let config = TrayConfig::default();
    assert!(config.icon.is_none());
    assert!(config.tooltip.is_none());
    // Default menu has 3 items: show, separator, quit
    assert_eq!(config.menu.len(), 3);
}

#[rstest]
fn tray_config_default_menu_show_item() {
    let config = TrayConfig::default();
    match &config.menu[0] {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "show");
            assert_eq!(label, "Show");
            assert!(*enabled);
        }
        other => panic!("expected Item, got {:?}", other),
    }
}

#[rstest]
fn tray_config_default_menu_separator() {
    let config = TrayConfig::default();
    assert!(matches!(config.menu[1], TrayMenuItem::Separator));
}

#[rstest]
fn tray_config_default_menu_quit_item() {
    let config = TrayConfig::default();
    match &config.menu[2] {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "quit");
            assert_eq!(label, "Quit");
            assert!(*enabled);
        }
        other => panic!("expected Item, got {:?}", other),
    }
}

#[rstest]
fn tray_config_clone() {
    let config = TrayConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.menu.len(), config.menu.len());
    assert_eq!(cloned.icon, config.icon);
    assert_eq!(cloned.tooltip, config.tooltip);
}

#[rstest]
fn tray_config_with_tooltip() {
    let config = TrayConfig {
        icon: None,
        tooltip: Some("My App".to_string()),
        menu: vec![],
    };
    assert_eq!(config.tooltip.as_deref(), Some("My App"));
}

#[rstest]
fn tray_config_with_icon_path() {
    let config = TrayConfig {
        icon: Some(PathBuf::from("/tmp/icon.png")),
        tooltip: None,
        menu: vec![],
    };
    assert!(config.icon.is_some());
    assert_eq!(config.icon.unwrap(), PathBuf::from("/tmp/icon.png"));
}

#[rstest]
fn tray_config_empty_menu() {
    let config = TrayConfig {
        icon: None,
        tooltip: None,
        menu: vec![],
    };
    assert!(config.menu.is_empty());
}

// ============================================================================
// TrayMenuItem variants
// ============================================================================

#[rstest]
fn tray_menu_item_item_variant() {
    let item = TrayMenuItem::Item {
        id: "open".to_string(),
        label: "Open".to_string(),
        enabled: true,
    };
    match &item {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "open");
            assert_eq!(label, "Open");
            assert!(*enabled);
        }
        _ => panic!("expected Item variant"),
    }
}

#[rstest]
fn tray_menu_item_disabled() {
    let item = TrayMenuItem::Item {
        id: "grayed".to_string(),
        label: "Grayed Out".to_string(),
        enabled: false,
    };
    match &item {
        TrayMenuItem::Item { enabled, .. } => assert!(!*enabled),
        _ => panic!("expected Item variant"),
    }
}

#[rstest]
fn tray_menu_item_separator_variant() {
    let sep = TrayMenuItem::Separator;
    assert!(matches!(sep, TrayMenuItem::Separator));
}

#[rstest]
fn tray_menu_item_clone_item() {
    let item = TrayMenuItem::Item {
        id: "x".to_string(),
        label: "X".to_string(),
        enabled: true,
    };
    let cloned = item.clone();
    match cloned {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "x");
            assert_eq!(label, "X");
            assert!(enabled);
        }
        _ => panic!("expected Item"),
    }
}

#[rstest]
fn tray_menu_item_clone_separator() {
    let sep = TrayMenuItem::Separator;
    let cloned = sep.clone();
    assert!(matches!(cloned, TrayMenuItem::Separator));
}

#[rstest]
fn tray_menu_item_debug_item() {
    let item = TrayMenuItem::Item {
        id: "test".to_string(),
        label: "Test".to_string(),
        enabled: true,
    };
    let debug = format!("{:?}", item);
    assert!(debug.contains("Item"));
    assert!(debug.contains("test"));
}

#[rstest]
fn tray_menu_item_debug_separator() {
    let sep = TrayMenuItem::Separator;
    let debug = format!("{:?}", sep);
    assert!(debug.contains("Separator"));
}

// ============================================================================
// TrayConfig serialization / deserialization
// ============================================================================

#[rstest]
fn tray_config_serialize_default() {
    let config = TrayConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"menu\""));
    // no tooltip / icon in default
    assert!(!json.contains("\"tooltip\":\""));
}

#[rstest]
fn tray_config_round_trip() {
    let config = TrayConfig {
        icon: None,
        tooltip: Some("Hello".to_string()),
        menu: vec![
            TrayMenuItem::Item {
                id: "action".to_string(),
                label: "Do Action".to_string(),
                enabled: true,
            },
            TrayMenuItem::Separator,
        ],
    };
    let json = serde_json::to_string(&config).unwrap();
    let decoded: TrayConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.tooltip.as_deref(), Some("Hello"));
    assert_eq!(decoded.menu.len(), 2);
}

#[rstest]
fn tray_menu_item_serialize_item() {
    let item = TrayMenuItem::Item {
        id: "save".to_string(),
        label: "Save".to_string(),
        enabled: false,
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"Item\"") || json.contains("save"));
}

#[rstest]
fn tray_menu_item_serialize_separator() {
    let sep = TrayMenuItem::Separator;
    let json = serde_json::to_string(&sep).unwrap();
    assert!(json.contains("Separator"));
}

// ============================================================================
// TrayConfig with multiple custom items
// ============================================================================

#[rstest]
#[case(0, "file", "File", true)]
#[case(1, "edit", "Edit", true)]
#[case(2, "help", "Help", false)]
fn tray_menu_custom_items(
    #[case] idx: usize,
    #[case] id: &str,
    #[case] label: &str,
    #[case] enabled: bool,
) {
    let config = TrayConfig {
        icon: None,
        tooltip: None,
        menu: vec![
            TrayMenuItem::Item {
                id: "file".to_string(),
                label: "File".to_string(),
                enabled: true,
            },
            TrayMenuItem::Item {
                id: "edit".to_string(),
                label: "Edit".to_string(),
                enabled: true,
            },
            TrayMenuItem::Item {
                id: "help".to_string(),
                label: "Help".to_string(),
                enabled: false,
            },
        ],
    };
    match &config.menu[idx] {
        TrayMenuItem::Item {
            id: i,
            label: l,
            enabled: e,
        } => {
            assert_eq!(i, id);
            assert_eq!(l, label);
            assert_eq!(*e, enabled);
        }
        _ => panic!("expected Item"),
    }
}

#[rstest]
fn tray_config_menu_all_separators() {
    let config = TrayConfig {
        icon: None,
        tooltip: None,
        menu: vec![TrayMenuItem::Separator, TrayMenuItem::Separator],
    };
    assert_eq!(config.menu.len(), 2);
    for item in &config.menu {
        assert!(matches!(item, TrayMenuItem::Separator));
    }
}

// ============================================================================
// TrayMenuItem — unicode labels
// ============================================================================

#[rstest]
fn tray_menu_item_unicode_label() {
    let item = TrayMenuItem::Item {
        id: "export".to_string(),
        label: "导出文件 📁".to_string(),
        enabled: true,
    };
    match item {
        TrayMenuItem::Item { label, .. } => assert!(label.contains("导出文件")),
        _ => panic!("expected Item"),
    }
}

#[rstest]
fn tray_menu_item_empty_label() {
    let item = TrayMenuItem::Item {
        id: "empty".to_string(),
        label: String::new(),
        enabled: true,
    };
    match &item {
        TrayMenuItem::Item { label, .. } => assert!(label.is_empty()),
        _ => panic!("expected Item"),
    }
}

// ============================================================================
// TrayConfig — many items
// ============================================================================

#[rstest]
fn tray_config_large_menu() {
    let mut items = Vec::new();
    for i in 0..20 {
        items.push(TrayMenuItem::Item {
            id: format!("item_{}", i),
            label: format!("Item {}", i),
            enabled: i % 2 == 0,
        });
        if i % 5 == 4 {
            items.push(TrayMenuItem::Separator);
        }
    }
    let config = TrayConfig {
        icon: None,
        tooltip: None,
        menu: items,
    };
    assert!(config.menu.len() > 20);
}

// ============================================================================
// TrayConfig with unicode tooltip
// ============================================================================

#[rstest]
fn tray_config_unicode_tooltip() {
    let config = TrayConfig {
        icon: None,
        tooltip: Some("AuroraView 视图工具".to_string()),
        menu: vec![],
    };
    assert_eq!(config.tooltip.as_deref(), Some("AuroraView 视图工具"));
}

// ============================================================================
// TrayConfig serde with tooltip
// ============================================================================

#[rstest]
fn tray_config_serde_with_tooltip() {
    let config = TrayConfig {
        icon: None,
        tooltip: Some("Test Tooltip".to_string()),
        menu: vec![TrayMenuItem::Separator],
    };
    let json = serde_json::to_string(&config).unwrap();
    let decoded: TrayConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.tooltip.as_deref(), Some("Test Tooltip"));
    assert_eq!(decoded.menu.len(), 1);
}

// ============================================================================
// TrayMenuItem debug output for Item with all fields
// ============================================================================

#[rstest]
fn tray_menu_item_debug_item_all_fields() {
    let item = TrayMenuItem::Item {
        id: "unique-id-xyz".to_string(),
        label: "Unique Label".to_string(),
        enabled: false,
    };
    let debug = format!("{:?}", item);
    assert!(debug.contains("unique-id-xyz"));
    assert!(debug.contains("Unique Label"));
}

// ============================================================================
// TrayConfig menu item access by type
// ============================================================================

#[rstest]
fn count_separator_items_in_default() {
    let config = TrayConfig::default();
    let separator_count = config.menu.iter().filter(|i| matches!(i, TrayMenuItem::Separator)).count();
    assert_eq!(separator_count, 1);
}

#[rstest]
fn count_action_items_in_default() {
    let config = TrayConfig::default();
    let action_count = config.menu.iter().filter(|i| matches!(i, TrayMenuItem::Item { .. })).count();
    assert_eq!(action_count, 2);  // show + quit
}

