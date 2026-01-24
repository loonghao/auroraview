//! Tests for DCC configuration

use auroraview_dcc::{DccConfig, DccType};

#[test]
fn test_dcc_config_defaults() {
    let config = DccConfig::default();
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 400);
    assert_eq!(config.height, 600);
    assert!(config.url.is_none());
    assert!(config.html.is_none());
    assert!(config.parent_hwnd.is_none());
}

#[test]
fn test_dcc_config_builder() {
    let config = DccConfig::new()
        .title("Maya Tool")
        .size(500, 700)
        .url("https://example.com")
        .parent_hwnd(0x12345)
        .dcc_type(DccType::Maya)
        .devtools(true);

    assert_eq!(config.title, "Maya Tool");
    assert_eq!(config.width, 500);
    assert_eq!(config.height, 700);
    assert_eq!(config.url, Some("https://example.com".to_string()));
    assert_eq!(config.parent_hwnd, Some(0x12345));
    assert_eq!(config.dcc_type, DccType::Maya);
    assert!(config.devtools);
}

#[test]
fn test_dcc_type_names() {
    assert_eq!(DccType::Maya.name(), "Maya");
    assert_eq!(DccType::Houdini.name(), "Houdini");
    assert_eq!(DccType::Nuke.name(), "Nuke");
    assert_eq!(DccType::Blender.name(), "Blender");
    assert_eq!(DccType::Max3ds.name(), "3ds Max");
    assert_eq!(DccType::Unreal.name(), "Unreal Engine");
    assert_eq!(DccType::Unknown.name(), "Unknown");
}

#[test]
fn test_dcc_type_default() {
    let dcc = DccType::default();
    assert_eq!(dcc, DccType::Unknown);
}

#[test]
fn test_dcc_config_panel_name() {
    let config = DccConfig::new().panel_name("MyToolPanel");
    assert_eq!(config.panel_name, Some("MyToolPanel".to_string()));
}

#[test]
fn test_dcc_config_debug_port() {
    let config = DccConfig::new().debug_port(9222);
    assert_eq!(config.debug_port, 9222);
}
