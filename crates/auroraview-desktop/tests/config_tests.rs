//! Tests for desktop configuration

use std::path::PathBuf;

use auroraview_desktop::{DesktopConfig, TrayConfig, TrayMenuItem};

#[test]
fn test_desktop_config_defaults() {
    let config = DesktopConfig::default();
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert!(config.resizable);
    assert!(config.decorations);
    assert!(!config.transparent);
    assert!(config.visible);
}

#[test]
fn test_desktop_config_builder() {
    let config = DesktopConfig::new()
        .title("Test App")
        .size(1024, 768)
        .url("https://example.com")
        .resizable(false)
        .devtools(true);

    assert_eq!(config.title, "Test App");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert_eq!(config.url, Some("https://example.com".to_string()));
    assert!(!config.resizable);
    assert!(config.devtools);
}

#[test]
fn test_tray_config_default() {
    let tray = TrayConfig::default();

    assert!(tray.icon.is_none());
    assert!(tray.tooltip.is_none());
    assert_eq!(tray.menu.len(), 3); // show, separator, quit
}

#[test]
fn test_tray_config_custom() {
    let tray = TrayConfig {
        icon: None,
        tooltip: Some("My App".to_string()),
        menu: vec![
            TrayMenuItem::Item {
                id: "action".to_string(),
                label: "Action".to_string(),
                enabled: true,
            },
            TrayMenuItem::Separator,
            TrayMenuItem::Item {
                id: "quit".to_string(),
                label: "Quit".to_string(),
                enabled: true,
            },
        ],
    };

    assert_eq!(tray.tooltip, Some("My App".to_string()));
    assert_eq!(tray.menu.len(), 3);

    match &tray.menu[0] {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "action");
            assert_eq!(label, "Action");
            assert!(enabled);
        }
        _ => panic!("Expected Item"),
    }

    assert!(matches!(tray.menu[1], TrayMenuItem::Separator));
}

#[test]
fn test_desktop_config_with_tray() {
    let config = DesktopConfig::new()
        .title("Tray App")
        .tray(TrayConfig::default());

    assert!(config.tray.is_some());
    let tray = config.tray.unwrap();
    assert_eq!(tray.menu.len(), 3);
}

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn test_desktop_config_new_same_as_default() {
    let a = DesktopConfig::new();
    let b = DesktopConfig::default();
    assert_eq!(a.title, b.title);
    assert_eq!(a.width, b.width);
    assert_eq!(a.height, b.height);
}

#[test]
fn test_desktop_config_html_builder() {
    let config = DesktopConfig::new().html("<html><body>Test</body></html>");
    assert_eq!(
        config.html,
        Some("<html><body>Test</body></html>".to_string())
    );
    assert!(config.url.is_none());
}

#[test]
fn test_desktop_config_decorations_false() {
    let config = DesktopConfig::new().decorations(false);
    assert!(!config.decorations);
}

#[test]
fn test_desktop_config_always_on_top() {
    let config = DesktopConfig::new().always_on_top(true);
    assert!(config.always_on_top);
}

#[test]
fn test_desktop_config_transparent() {
    let config = DesktopConfig::new().transparent(true);
    assert!(config.transparent);
}

#[test]
fn test_desktop_config_data_dir() {
    let config = DesktopConfig::new().data_dir("/tmp/my-app-data");
    assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/my-app-data")));
}

#[test]
fn test_desktop_config_icon() {
    let config = DesktopConfig::new().icon("assets/icon.ico");
    assert_eq!(config.icon, Some(PathBuf::from("assets/icon.ico")));
}

#[test]
fn test_desktop_config_debug_port() {
    let config = DesktopConfig::new().debug_port(9222);
    assert_eq!(config.debug_port, 9222);
}

#[test]
fn test_desktop_config_size_custom() {
    let config = DesktopConfig::new().size(1920, 1080);
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

#[test]
fn test_desktop_config_clone() {
    let original = DesktopConfig::new()
        .title("Clone Test")
        .size(800, 600)
        .devtools(true);

    let cloned = original.clone();
    assert_eq!(cloned.title, "Clone Test");
    assert_eq!(cloned.width, 800);
    assert_eq!(cloned.height, 600);
    assert!(cloned.devtools);
}

#[test]
fn test_desktop_config_serde_roundtrip() {
    let config = DesktopConfig::new()
        .title("Serde Test")
        .url("https://example.com")
        .size(800, 600)
        .resizable(false)
        .devtools(true);

    let json = serde_json::to_string(&config).unwrap();
    let restored: DesktopConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.title, "Serde Test");
    assert_eq!(restored.url, Some("https://example.com".to_string()));
    assert_eq!(restored.width, 800);
    assert_eq!(restored.height, 600);
    assert!(!restored.resizable);
    assert!(restored.devtools);
}

#[test]
fn test_desktop_config_default_no_url_no_html() {
    let config = DesktopConfig::default();
    assert!(config.url.is_none());
    assert!(config.html.is_none());
}

#[test]
fn test_desktop_config_default_flags() {
    let config = DesktopConfig::default();
    assert!(!config.always_on_top);
    assert!(!config.maximized);
    assert!(!config.minimized);
    assert!(!config.fullscreen);
    assert!(config.context_menu);
    assert!(config.hotkeys);
    assert_eq!(config.debug_port, 0);
}

#[test]
fn test_tray_config_serde_roundtrip() {
    let tray = TrayConfig {
        icon: Some(PathBuf::from("icon.png")),
        tooltip: Some("App Tooltip".to_string()),
        menu: vec![TrayMenuItem::Separator],
    };

    let json = serde_json::to_string(&tray).unwrap();
    let restored: TrayConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.tooltip, Some("App Tooltip".to_string()));
    assert_eq!(restored.menu.len(), 1);
    assert!(matches!(restored.menu[0], TrayMenuItem::Separator));
}

#[test]
fn test_tray_menu_item_disabled() {
    let item = TrayMenuItem::Item {
        id: "action".to_string(),
        label: "Disabled Action".to_string(),
        enabled: false,
    };

    match item {
        TrayMenuItem::Item { enabled, .. } => assert!(!enabled),
        _ => panic!("Expected Item"),
    }
}

#[test]
fn test_tray_menu_item_clone() {
    let item = TrayMenuItem::Item {
        id: "copy".to_string(),
        label: "Copy".to_string(),
        enabled: true,
    };
    let cloned = item.clone();
    match cloned {
        TrayMenuItem::Item { id, label, .. } => {
            assert_eq!(id, "copy");
            assert_eq!(label, "Copy");
        }
        _ => panic!("Expected Item"),
    }
}

#[test]
fn test_desktop_config_builder_chaining_all_methods() {
    let config = DesktopConfig::new()
        .title("Full Config")
        .size(1280, 720)
        .url("https://maya-tool.example.com")
        .resizable(true)
        .decorations(true)
        .always_on_top(false)
        .transparent(false)
        .devtools(false)
        .data_dir("/data")
        .icon("icon.ico")
        .tray(TrayConfig::default())
        .debug_port(0);

    assert_eq!(config.title, "Full Config");
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 720);
    assert!(config.url.is_some());
    assert!(config.tray.is_some());
    assert_eq!(config.debug_port, 0);
}

// ─── Additional coverage ──────────────────────────────────────────────────────

#[test]
fn test_desktop_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DesktopConfig>();
}

#[test]
fn test_tray_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TrayConfig>();
}

#[test]
fn test_desktop_config_debug_non_empty() {
    let config = DesktopConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_desktop_config_size_zero() {
    let config = DesktopConfig::new().size(0, 0);
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
}

#[test]
fn test_desktop_config_size_large() {
    let config = DesktopConfig::new().size(7680, 4320); // 8K resolution
    assert_eq!(config.width, 7680);
    assert_eq!(config.height, 4320);
}

#[test]
fn test_desktop_config_resizable_false() {
    let config = DesktopConfig::new().resizable(false);
    assert!(!config.resizable);
}

#[test]
fn test_desktop_config_decorations_true() {
    let config = DesktopConfig::new().decorations(true);
    assert!(config.decorations);
}

#[test]
fn test_desktop_config_always_on_top_true() {
    let config = DesktopConfig::new().always_on_top(true);
    assert!(config.always_on_top);
}

#[test]
fn test_desktop_config_debug_port_nonzero() {
    let config = DesktopConfig::new().debug_port(9222);
    assert_eq!(config.debug_port, 9222);
}

#[test]
fn test_tray_config_no_icon() {
    let tray = TrayConfig::default();
    assert!(tray.icon.is_none());
}

#[test]
fn test_tray_config_no_tooltip() {
    let tray = TrayConfig::default();
    assert!(tray.tooltip.is_none());
}

#[test]
fn test_tray_config_empty_menu() {
    // TrayConfig::default() has 3 default menu items (Show, Separator, Quit)
    let tray = TrayConfig::default();
    assert!(!tray.menu.is_empty());
    assert_eq!(tray.menu.len(), 3);
}

#[test]
fn test_tray_menu_item_separator_is_separator() {
    let item = TrayMenuItem::Separator;
    assert!(matches!(item, TrayMenuItem::Separator));
}
