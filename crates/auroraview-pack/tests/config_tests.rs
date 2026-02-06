//! Tests for auroraview-pack config module

use auroraview_pack::{
    BundleStrategy, LicenseConfig, PackConfig, PackMode, PythonBundleConfig, TargetPlatform,
    WindowConfig, WindowStartPosition,
};
use std::path::PathBuf;

#[test]
fn test_url_mode() {
    let config = PackConfig::url("https://example.com");
    assert_eq!(config.mode.name(), "url");
    assert!(!config.mode.embeds_assets());
    assert_eq!(config.output_name, "example");
}

#[test]
fn test_frontend_mode() {
    let config = PackConfig::frontend("./dist");
    assert_eq!(config.mode.name(), "frontend");
    assert!(config.mode.embeds_assets());
    assert_eq!(config.output_name, "dist");
}

#[test]
fn test_builder_pattern() {
    let config = PackConfig::url("example.com")
        .with_output("my-app")
        .with_title("My App")
        .with_size(1920, 1080)
        .with_debug(true);

    assert_eq!(config.output_name, "my-app");
    assert_eq!(config.window.title, "My App");
    assert_eq!(config.window.width, 1920);
    assert_eq!(config.window.height, 1080);
    assert!(config.debug);
}

#[test]
fn test_bundle_strategy_default() {
    let strategy = BundleStrategy::default();
    assert_eq!(strategy, BundleStrategy::Standalone);
}

#[test]
fn test_bundle_strategy_serialization() {
    let strategies = [
        (BundleStrategy::Standalone, "standalone"),
        (BundleStrategy::PyOxidizer, "py_oxidizer"),
        (BundleStrategy::Embedded, "embedded"),
        (BundleStrategy::Portable, "portable"),
        (BundleStrategy::System, "system"),
    ];

    for (strategy, expected_name) in strategies {
        let json = serde_json::to_string(&strategy).unwrap();
        assert!(
            json.contains(expected_name),
            "Strategy {:?} should serialize to contain '{}'",
            strategy,
            expected_name
        );
    }
}

#[test]
fn test_bundle_strategy_deserialization() {
    let test_cases = [
        ("\"standalone\"", BundleStrategy::Standalone),
        ("\"embedded\"", BundleStrategy::Embedded),
        ("\"portable\"", BundleStrategy::Portable),
        ("\"system\"", BundleStrategy::System),
    ];

    for (json, expected) in test_cases {
        let parsed: BundleStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_python_bundle_config_default() {
    let config = PythonBundleConfig::default();
    assert!(config.entry_point.is_empty());
    assert!(config.include_paths.is_empty());
    assert!(config.packages.is_empty());
    assert!(config.requirements.is_none());
    assert_eq!(config.strategy, BundleStrategy::Standalone);
    assert_eq!(config.version, "3.11");
    assert_eq!(config.optimize, 1);
    assert!(!config.include_pip);
    assert!(!config.include_setuptools);
    assert_eq!(
        config.module_search_paths,
        vec!["$EXTRACT_DIR".to_string(), "$SITE_PACKAGES".to_string()]
    );
    assert!(config.filesystem_importer);
}

#[test]
fn test_fullstack_mode() {
    let config = PackConfig::fullstack("./dist", "main:run");
    assert_eq!(config.mode.name(), "fullstack");
    assert!(config.mode.embeds_assets());
    assert!(config.mode.has_python());

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.entry_point, "main:run");
        assert_eq!(python.strategy, BundleStrategy::Standalone);
    } else {
        panic!("Expected FullStack mode");
    }
}

#[test]
fn test_fullstack_with_config() {
    let python_config = PythonBundleConfig {
        entry_point: "app:main".to_string(),
        packages: vec!["pyyaml".to_string(), "requests".to_string()],
        version: "3.12".to_string(),
        strategy: BundleStrategy::Embedded,
        ..Default::default()
    };

    let config = PackConfig::fullstack_with_config("./dist", python_config);

    if let PackMode::FullStack { python, .. } = &config.mode {
        assert_eq!(python.entry_point, "app:main");
        assert_eq!(python.packages.len(), 2);
        assert_eq!(python.version, "3.12");
        assert_eq!(python.strategy, BundleStrategy::Embedded);
    } else {
        panic!("Expected FullStack mode");
    }
}

#[test]
fn test_pack_mode_properties() {
    let url_mode = PackMode::Url {
        url: "https://example.com".to_string(),
    };
    assert!(!url_mode.embeds_assets());
    assert!(!url_mode.has_python());

    let frontend_mode = PackMode::Frontend {
        path: PathBuf::from("./dist"),
    };
    assert!(frontend_mode.embeds_assets());
    assert!(!frontend_mode.has_python());

    let fullstack_mode = PackMode::FullStack {
        frontend_path: PathBuf::from("./dist"),
        python: Box::new(PythonBundleConfig::default()),
    };
    assert!(fullstack_mode.embeds_assets());
    assert!(fullstack_mode.has_python());
}

#[test]
fn test_license_config_time_limited() {
    let license = LicenseConfig::time_limited("2025-12-31");
    assert!(license.enabled);
    assert_eq!(license.expires_at, Some("2025-12-31".to_string()));
    assert!(!license.require_token);
}

#[test]
fn test_license_config_token_required() {
    let license = LicenseConfig::token_required();
    assert!(license.enabled);
    assert!(license.require_token);
    assert!(license.expires_at.is_none());
}

#[test]
fn test_license_config_full() {
    let license = LicenseConfig::full("2025-06-30");
    assert!(license.enabled);
    assert!(license.require_token);
    assert_eq!(license.expires_at, Some("2025-06-30".to_string()));
}

#[test]
fn test_window_config_default() {
    let config = WindowConfig::default();
    assert_eq!(config.title, "AuroraView App");
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 720);
    assert!(config.resizable);
    assert!(!config.frameless);
    assert!(!config.transparent);
    assert!(!config.always_on_top);
}

#[test]
fn test_window_start_position() {
    let center = WindowStartPosition::Center;
    let position = WindowStartPosition::Position { x: 100, y: 200 };

    // Test serialization
    let center_json = serde_json::to_string(&center).unwrap();
    let position_json = serde_json::to_string(&position).unwrap();

    assert!(center_json.contains("center"));
    assert!(position_json.contains("100"));
    assert!(position_json.contains("200"));
}

#[test]
fn test_pack_config_with_env() {
    let config = PackConfig::url("example.com")
        .with_env_var("APP_MODE", "production")
        .with_env_var("LOG_LEVEL", "info");

    assert_eq!(config.env.get("APP_MODE"), Some(&"production".to_string()));
    assert_eq!(config.env.get("LOG_LEVEL"), Some(&"info".to_string()));
}

#[test]
fn test_pack_config_with_license() {
    let config = PackConfig::url("example.com")
        .with_expiration("2025-12-31")
        .with_token_required();

    let license = config.license.unwrap();
    assert!(license.enabled);
    assert!(license.require_token);
    assert_eq!(license.expires_at, Some("2025-12-31".to_string()));
}

#[test]
fn test_target_platform() {
    assert_eq!(TargetPlatform::default(), TargetPlatform::Current);

    let platforms = [
        TargetPlatform::Current,
        TargetPlatform::Windows,
        TargetPlatform::MacOS,
        TargetPlatform::Linux,
    ];

    for platform in platforms {
        let json = serde_json::to_string(&platform).unwrap();
        let parsed: TargetPlatform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, platform);
    }
}

#[test]
fn test_collect_pattern() {
    use auroraview_pack::CollectPattern;

    let pattern = CollectPattern {
        source: "../examples/*.py".to_string(),
        dest: Some("examples".to_string()),
        preserve_structure: true,
        description: None,
    };

    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("../examples/*.py"));
    assert!(json.contains("examples"));
}
