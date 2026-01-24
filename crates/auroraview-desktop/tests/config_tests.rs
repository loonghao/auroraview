//! Tests for desktop configuration

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

    // Verify first item
    match &tray.menu[0] {
        TrayMenuItem::Item { id, label, enabled } => {
            assert_eq!(id, "action");
            assert_eq!(label, "Action");
            assert!(enabled);
        }
        _ => panic!("Expected Item"),
    }

    // Verify separator
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
