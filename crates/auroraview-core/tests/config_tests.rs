//! Configuration tests

use auroraview_core::config::CoreConfig;

#[test]
fn test_default_config() {
    let config = CoreConfig::default();
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.dev_tools);
    assert!(!config.allow_new_window);
}

#[test]
fn test_config_serialization() {
    let config = CoreConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, config.title);
    assert_eq!(parsed.width, config.width);
}

#[cfg(target_os = "windows")]
#[test]
fn test_undecorated_shadow_default() {
    let config = CoreConfig::default();
    assert!(
        config.undecorated_shadow,
        "undecorated_shadow should default to true"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn test_undecorated_shadow_disabled() {
    let config = CoreConfig {
        undecorated_shadow: false,
        ..Default::default()
    };
    assert!(!config.undecorated_shadow);
}
