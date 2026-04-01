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

#[test]
fn test_config_csp_default_none() {
    let config = CoreConfig::default();
    assert!(config.content_security_policy.is_none());
}

#[test]
fn test_config_csp_set() {
    let policy = "default-src 'self'".to_string();
    let config = CoreConfig {
        content_security_policy: Some(policy.clone()),
        ..Default::default()
    };
    assert_eq!(config.content_security_policy, Some(policy));
}

#[test]
fn test_config_csp_survives_serialization() {
    let policy = "default-src 'self'; img-src *".to_string();
    let config = CoreConfig {
        content_security_policy: Some(policy.clone()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.content_security_policy, Some(policy));
}

#[cfg(target_os = "windows")]
#[test]
fn test_undecorated_shadow_default() {
    let config = CoreConfig::default();
    assert!(
        !config.undecorated_shadow,
        "undecorated_shadow should default to false"
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
