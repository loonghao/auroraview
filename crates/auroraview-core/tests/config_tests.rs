//! Configuration tests

use auroraview_core::config::{CoreConfig, EmbedMode};

// ---------------------------------------------------------------------------
// Default values
// ---------------------------------------------------------------------------

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
fn test_default_context_menu_enabled() {
    let config = CoreConfig::default();
    assert!(config.context_menu);
}

#[test]
fn test_default_resizable() {
    let config = CoreConfig::default();
    assert!(config.resizable);
}

#[test]
fn test_default_decorations() {
    let config = CoreConfig::default();
    assert!(config.decorations);
}

#[test]
fn test_default_not_always_on_top() {
    let config = CoreConfig::default();
    assert!(!config.always_on_top);
}

#[test]
fn test_default_not_transparent() {
    let config = CoreConfig::default();
    assert!(!config.transparent);
}

#[test]
fn test_default_no_url_html() {
    let config = CoreConfig::default();
    assert!(config.url.is_none());
    assert!(config.html.is_none());
}

#[test]
fn test_default_no_parent_hwnd() {
    let config = CoreConfig::default();
    assert!(config.parent_hwnd.is_none());
}

#[test]
fn test_default_ipc_batching() {
    let config = CoreConfig::default();
    assert!(config.ipc_batching);
    assert_eq!(config.ipc_batch_size, 10);
    assert_eq!(config.ipc_batch_interval_ms, 10);
}

#[test]
fn test_default_no_asset_root() {
    let config = CoreConfig::default();
    assert!(config.asset_root.is_none());
}

#[test]
fn test_default_no_background_color() {
    let config = CoreConfig::default();
    assert!(config.background_color.is_none());
}

#[test]
fn test_default_allow_file_protocol_disabled() {
    let config = CoreConfig::default();
    assert!(!config.allow_file_protocol);
}

// ---------------------------------------------------------------------------
// Serialization / deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_config_serialization() {
    let config = CoreConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, config.title);
    assert_eq!(parsed.width, config.width);
}

#[test]
fn test_config_serialization_url() {
    let config = CoreConfig {
        url: Some("https://example.com".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.url, Some("https://example.com".to_string()));
}

#[test]
fn test_config_serialization_html() {
    let config = CoreConfig {
        html: Some("<h1>Hello</h1>".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.html, Some("<h1>Hello</h1>".to_string()));
}

#[test]
fn test_config_serialization_background_color() {
    let config = CoreConfig {
        background_color: Some("#1e1e1e".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.background_color, Some("#1e1e1e".to_string()));
}

#[test]
fn test_config_clone() {
    let config = CoreConfig {
        title: "Clone Test".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.title, config.title);
    assert_eq!(cloned.width, config.width);
    assert_eq!(cloned.height, config.height);
}

#[test]
fn test_config_debug() {
    let config = CoreConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("AuroraView"));
    assert!(debug_str.contains("800"));
}

// ---------------------------------------------------------------------------
// CSP tests
// ---------------------------------------------------------------------------

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

#[test]
fn test_config_csp_strict_policy() {
    let policy = "default-src 'none'; script-src 'self'".to_string();
    let config = CoreConfig {
        content_security_policy: Some(policy.clone()),
        ..Default::default()
    };
    assert_eq!(
        config.content_security_policy.as_deref(),
        Some(policy.as_str())
    );
}

// ---------------------------------------------------------------------------
// EmbedMode tests
// ---------------------------------------------------------------------------

#[test]
fn test_embed_mode_default_is_none() {
    let mode = EmbedMode::default();
    assert_eq!(mode, EmbedMode::None);
}

#[cfg(target_os = "windows")]
#[test]
fn test_embed_mode_child() {
    let mode = EmbedMode::Child;
    assert_eq!(mode, EmbedMode::Child);
    assert_ne!(mode, EmbedMode::Owner);
}

#[cfg(target_os = "windows")]
#[test]
fn test_embed_mode_owner() {
    let mode = EmbedMode::Owner;
    assert_eq!(mode, EmbedMode::Owner);
}

#[test]
fn test_embed_mode_serde_roundtrip() {
    let mode = EmbedMode::None;
    let json = serde_json::to_string(&mode).unwrap();
    let parsed: EmbedMode = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, mode);
}

#[test]
fn test_embed_mode_debug() {
    let mode = EmbedMode::None;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("None"));
}

// ---------------------------------------------------------------------------
// Window-specific field tests
// ---------------------------------------------------------------------------

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

#[cfg(target_os = "windows")]
#[test]
fn test_undecorated_shadow_enabled() {
    let config = CoreConfig {
        undecorated_shadow: true,
        ..Default::default()
    };
    assert!(config.undecorated_shadow);
}

#[test]
fn test_config_overlay_settings() {
    let config = CoreConfig {
        always_on_top: true,
        transparent: true,
        decorations: false,
        ..Default::default()
    };
    assert!(config.always_on_top);
    assert!(config.transparent);
    assert!(!config.decorations);
}

#[test]
fn test_config_ipc_settings() {
    let config = CoreConfig {
        ipc_batching: false,
        ipc_batch_size: 50,
        ipc_batch_interval_ms: 100,
        ..Default::default()
    };
    assert!(!config.ipc_batching);
    assert_eq!(config.ipc_batch_size, 50);
    assert_eq!(config.ipc_batch_interval_ms, 100);
}

#[test]
fn test_config_parent_hwnd() {
    let config = CoreConfig {
        parent_hwnd: Some(12345),
        ..Default::default()
    };
    assert_eq!(config.parent_hwnd, Some(12345));
}

#[test]
fn test_config_custom_size() {
    let config = CoreConfig {
        width: 1920,
        height: 1080,
        ..Default::default()
    };
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

#[test]
fn test_config_custom_title() {
    let config = CoreConfig {
        title: "My DCC Tool".to_string(),
        ..Default::default()
    };
    assert_eq!(config.title, "My DCC Tool");
}
