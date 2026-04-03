//! Comprehensive tests for auroraview-pack config module
//!
//! Covers: PackConfig, PackMode, PythonBundleConfig, WindowConfig,
//! ExtensionsRuntimeConfig, IsolationConfig, DebugConfig, RuntimeConfig,
//! ProcessConfig, ProtectionConfig, CollectPattern, LicenseConfig,
//! BundleStrategy, TargetPlatform, WindowStartPosition, and manifest mapping.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use auroraview_pack::{
    CollectPattern, DebugConfig, ExtensionRemoteManifestEntry, ExtensionsManifestConfig,
    IsolationConfig, LicenseConfig, PackConfig, PackMode, ProcessConfig, ProtectionConfig,
    PythonBundleConfig, RuntimeConfig, TargetPlatform, WindowConfig, WindowStartPosition,
    BundleStrategy,
};

// ============================================================================
// PackMode — constructors & accessors
// ============================================================================

#[test]
fn test_url_mode() {
    let config = PackConfig::url("https://example.com");
    assert_eq!(config.mode.name(), "url");
    assert!(!config.mode.embeds_assets());
    assert_eq!(config.output_name, "example");
}

#[test]
fn test_url_mode_without_scheme() {
    let config = PackConfig::url("example.com");
    assert_eq!(config.output_name, "example");
    assert!(matches!(config.mode, PackMode::Url { url } if url == "example.com"));
}

#[test]
fn test_frontend_mode() {
    let config = PackConfig::frontend("./dist");
    assert_eq!(config.mode.name(), "frontend");
    assert!(config.mode.embeds_assets());
    assert_eq!(config.output_name, "dist");
}

#[test]
fn test_frontend_mode_nested_dir() {
    let config = PackConfig::frontend("./src/frontend/build");
    assert_eq!(config.output_name, "build");
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
    assert!(url_mode.url().is_some());
    assert!(url_mode.frontend_path().is_none());
    assert!(url_mode.python_config().is_none());

    let frontend_mode = PackMode::Frontend {
        path: PathBuf::from("./dist"),
    };
    assert!(frontend_mode.embeds_assets());
    assert!(!frontend_mode.has_python());
    assert!(frontend_mode.url().is_none());
    assert!(frontend_mode.frontend_path().is_some());

    let fullstack_mode = PackMode::FullStack {
        frontend_path: PathBuf::from("./dist"),
        python: Box::new(PythonBundleConfig::default()),
    };
    assert!(fullstack_mode.embeds_assets());
    assert!(fullstack_mode.has_python());
    assert!(fullstack_mode.python_config().is_some());
    assert!(fullstack_mode.frontend_path().is_some());
}

#[test]
fn test_pack_mode_frontend_path_accessor() {
    let mode = PackMode::Frontend {
        path: PathBuf::from("/opt/app/ui"),
    };
    let fp = mode.frontend_path().unwrap();
    assert_eq!(fp, Path::new("/opt/app/ui"));

    let fs_mode = PackMode::FullStack {
        frontend_path: PathBuf::from("/opt/app/ui"),
        python: Box::new(PythonBundleConfig::default()),
    };
    let fs_fp = fs_mode.frontend_path().unwrap();
    assert_eq!(fs_fp, Path::new("/opt/app/ui"));

    assert!(PackMode::Url { url: "x".into() }.frontend_path().is_none());
}

#[test]
fn test_pack_mode_url_accessor() {
    let url_mode = PackMode::Url { url: "http://a.b".into() };
    let u = url_mode.url().unwrap();
    assert_eq!(u, "http://a.b");
    assert!(PackMode::Frontend { path: PathBuf::from("/") }.url().is_none());
}

// ============================================================================
// PackConfig — builder chain
// ============================================================================

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
fn test_with_output_dir() {
    let config = PackConfig::url("x").with_output_dir("./out/bin");
    assert_eq!(config.output_dir, PathBuf::from("./out/bin"));
}

#[test]
fn test_with_frameless() {
    let c = PackConfig::url("x").with_frameless(true);
    assert!(c.window.frameless);
}

#[test]
fn test_with_always_on_top() {
    let c = PackConfig::url("x").with_always_on_top(true);
    assert!(c.window.always_on_top);
}

#[test]
fn test_with_resizable() {
    let c = PackConfig::url("x").with_resizable(false);
    assert!(!c.window.resizable);
}

#[test]
fn test_with_user_agent() {
    let c = PackConfig::url("x").with_user_agent("MyBot/1.0");
    assert_eq!(c.user_agent.as_deref(), Some("MyBot/1.0"));
}

#[test]
fn test_with_icon() {
    let c = PackConfig::url("x").with_icon("./assets/icon.png");
    assert_eq!(c.icon_path, Some(PathBuf::from("./assets/icon.png")));
}

#[test]
fn test_with_env_hashmap() {
    let mut env = HashMap::new();
    env.insert("KEY".into(), "VAL".into());
    let c = PackConfig::url("x").with_env(env);
    assert_eq!(c.env.get("KEY"), Some(&"VAL".to_string()));
}

#[test]
fn test_with_env_var_multiple() {
    let c = PackConfig::url("x")
        .with_env_var("A", "1")
        .with_env_var("B", "2");
    assert_eq!(c.env.len(), 2);
}

#[test]
fn test_with_license_object() {
    let lic = LicenseConfig::time_limited("2030-01-01");
    let c = PackConfig::url("x").with_license(lic.clone());
    assert!(c.license.as_ref().unwrap().enabled);
    assert_eq!(c.license.as_ref().unwrap().expires_at, Some("2030-01-01".into()));
}

#[test]
fn test_with_remote_debugging_port() {
    let c = PackConfig::url("x").with_remote_debugging_port(9222);
    assert_eq!(c.remote_debugging_port, Some(9222));
}

#[test]
fn test_with_expiration_enables_license() {
    let c = PackConfig::url("x").with_expiration("2026-01-01");
    let lic = c.license.unwrap();
    assert!(lic.enabled);
    assert_eq!(lic.expires_at, Some("2026-01-01".into()));
}

#[test]
fn test_with_token_required_merges_existing_license() {
    // first set expiration, then require token → should merge
    let c = PackConfig::url("x")
        .with_expiration("2026-06-01")
        .with_token_required();
    let lic = c.license.unwrap();
    assert!(lic.enabled);
    assert!(lic.require_token);
    assert_eq!(lic.expires_at, Some("2026-06-01".into()));
}

#[test]
fn test_debug_config() {
    let c = PackConfig::url("x").with_debug(true).with_remote_debugging_port(9229);
    let dc = c.debug_config();
    assert!(dc.enabled);
    assert!(dc.devtools);
    assert_eq!(dc.remote_debugging_port, Some(9229));
}

#[test]
fn test_debug_config_defaults_when_not_debug() {
    let c = PackConfig::url("x");
    let dc = c.debug_config();
    assert!(!dc.enabled);
}

// ============================================================================
// PackConfig defaults
// ============================================================================

#[test]
fn test_pack_config_default_compression_level() {
    let c = PackConfig::url("x");
    assert_eq!(c.compression_level, 19);
}

#[test]
fn test_pack_config_default_extensions() {
    let c = PackConfig::url("x");
    // ExtensionsRuntimeConfig: enabled uses #[serde(default="default_true")]
    // but Default trait gives false for bool. The actual runtime config
    // starts with serde defaults, so we check what we get from constructor.
    // Since PackConfig::url() doesn't go through serde, enabled is false.
    assert!(!c.extensions.bundle);
    assert!(c.extensions.local.is_empty());
    assert!(c.extensions.remote.is_empty());
}

#[test]
fn test_pack_config_no_csp_by_default() {
    let c = PackConfig::url("x");
    assert!(c.content_security_policy.is_none());
}

#[test]
fn test_pack_config_allow_new_window_default() {
    let c = PackConfig::url("x");
    assert!(!c.allow_new_window);
}

// ============================================================================
// PythonBundleConfig — constructor / builder / fields
// ============================================================================

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
fn test_python_bundle_config_new() {
    let cfg = PythonBundleConfig::new("app.main:run");
    assert_eq!(cfg.entry_point, "app.main:run");
    // other fields should be defaults
    assert_eq!(cfg.version, "3.11");
    assert_eq!(cfg.optimize, 1);
}

#[test]
fn test_python_bundle_config_builder_chain() {
    let cfg = PythonBundleConfig::new("main:app")
        .with_version("3.10")
        .with_include_paths(vec![PathBuf::from("src")])
        .with_strategy(BundleStrategy::Embedded)
        .with_isolation(IsolationConfig::none());

    assert_eq!(cfg.version, "3.10");
    assert_eq!(cfg.include_paths.len(), 1);
    assert_eq!(cfg.strategy, BundleStrategy::Embedded);
    assert!(!cfg.isolation.path);
}

#[test]
fn test_python_bundle_config_non_default_fields() {
    let cfg = PythonBundleConfig {
        pip_via_vx_only: true,
        optimize: 2,
        show_console: true,
        exclude: vec!["tests".into(), "__pycache__".into()],
        external_bin: vec![PathBuf::from("bin/tool.exe")],
        resources: vec![PathBuf::from("assets")],
        include_pip: true,
        include_setuptools: true,
        distribution_flavor: Some("standalone_dynamic".into()),
        pyoxidizer_path: Some(PathBuf::from("pyoxidizer.exe")),
        ..Default::default()
    };

    assert!(cfg.pip_via_vx_only);
    assert_eq!(cfg.optimize, 2);
    assert!(cfg.show_console);
    assert_eq!(cfg.exclude.len(), 2);
    assert_eq!(cfg.external_bin.len(), 1);
    assert!(cfg.include_pip);
    assert!(cfg.include_setuptools);
    assert!(cfg.distribution_flavor.as_deref() == Some("standalone_dynamic"));
}

// ============================================================================
// BundleStrategy
// ============================================================================

#[test]
fn test_bundle_strategy_default() {
    assert_eq!(BundleStrategy::default(), BundleStrategy::Standalone);
}

#[test]
fn test_bundle_strategy_parse() {
    use auroraview_pack::BundleStrategy::*;

    assert_eq!(BundleStrategy::parse("standalone"), Standalone);
    assert_eq!(BundleStrategy::parse("PyOxidizer"), PyOxidizer);   // case-insensitive
    assert_eq!(BundleStrategy::parse("EMBEDDED"), Embedded);
    assert_eq!(BundleStrategy::parse("portable"), Portable);
    assert_eq!(BundleStrategy::parse("system"), System);
    assert_eq!(BundleStrategy::parse("unknown_value"), Standalone); // fallback
}

#[test]
fn test_bundle_strategy_as_str() {
    assert_eq!(BundleStrategy::Standalone.as_str(), "standalone");
    assert_eq!(BundleStrategy::PyOxidizer.as_str(), "pyoxidizer");
    assert_eq!(BundleStrategy::Embedded.as_str(), "embedded");
    assert_eq!(BundleStrategy::Portable.as_str(), "portable");
    assert_eq!(BundleStrategy::System.as_str(), "system");
}

#[test]
fn test_bundle_strategy_bundles_runtime() {
    assert!(BundleStrategy::Standalone.bundles_runtime());
    assert!(BundleStrategy::PyOxidizer.bundles_runtime());
    assert!(BundleStrategy::Portable.bundles_runtime());
    assert!(!BundleStrategy::Embedded.bundles_runtime());
    assert!(!BundleStrategy::System.bundles_runtime());
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
    let cases = [
        ("\"standalone\"", BundleStrategy::Standalone),
        ("\"embedded\"", BundleStrategy::Embedded),
        ("\"portable\"", BundleStrategy::Portable),
        ("\"system\"", BundleStrategy::System),
    ];
    for (json, expected) in cases {
        let parsed: BundleStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_bundle_strategy_partial_ord_and_equality() {
    assert_eq!(BundleStrategy::Standalone, BundleStrategy::Standalone);
    assert_ne!(BundleStrategy::Standalone, BundleStrategy::System);
}

// ============================================================================
// WindowConfig
// ============================================================================

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
    assert!(!config.fullscreen);
    assert!(!config.maximized);
    assert!(config.visible);
    assert!(config.min_width.is_none());
    assert!(config.min_height.is_none());
    assert!(config.max_width.is_none());
    assert!(config.max_height.is_none());
    assert!(matches!(config.start_position, WindowStartPosition::Center));
}

#[test]
fn test_window_config_new() {
    let w = WindowConfig::new("TestApp");
    assert_eq!(w.title, "TestApp");
    // rest should be default
    assert_eq!(w.width, 1280);
}

#[test]
fn test_window_config_with_size() {
    let w = WindowConfig::default().with_size(800, 600);
    assert_eq!(w.width, 800);
    assert_eq!(w.height, 600);
}

#[test]
fn test_window_config_with_min_size() {
    let w = WindowConfig::default().with_min_size(400, 300);
    assert_eq!(w.min_width, Some(400));
    assert_eq!(w.min_height, Some(300));
}

#[test]
fn test_window_config_with_frameless() {
    assert!(WindowConfig::default().with_frameless(true).frameless);
}

#[test]
fn test_window_config_with_always_on_top() {
    assert!(WindowConfig::default().with_always_on_top(true).always_on_top);
}

#[test]
fn test_window_config_serde_roundtrip() {
    let w = WindowConfig {
        title: "SerdeWin".into(),
        width: 1024,
        height: 768,
        frameless: true,
        transparent: true,
        fullscreen: true,
        maximized: true,
        visible: false,
        min_width: Some(320),
        max_height: Some(2048),
        ..Default::default()
    };
    let json = serde_json::to_string(&w).unwrap();
    let restored: WindowConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.title, "SerdeWin");
    assert_eq!(restored.width, 1024);
    assert!(restored.frameless);
    assert!(restored.transparent);
}

// ============================================================================
// WindowStartPosition
// ============================================================================

#[test]
fn test_window_start_position_center() {
    let c = WindowStartPosition::Center;
    assert!(c.is_center());
    assert_eq!(c.coordinates(), (0, 0));
}

#[test]
fn test_window_start_position_custom() {
    let p = WindowStartPosition::Position { x: -10, y: 200 };
    assert!(!p.is_center());
    assert_eq!(p.coordinates(), (-10, 200));
}

#[test]
fn test_window_start_position_serialization() {
    let center = WindowStartPosition::Center;
    let position = WindowStartPosition::Position { x: 100, y: 200 };

    let center_json = serde_json::to_string(&center).unwrap();
    let position_json = serde_json::to_string(&position).unwrap();

    assert!(center_json.contains("center"));
    assert!(position_json.contains("100"));
    assert!(position_json.contains("200"));
}

#[test]
fn test_window_start_position_deserialization() {
    let center: WindowStartPosition = serde_json::from_str(r#"{"type":"center"}"#).unwrap();
    assert!(center.is_center());

    let pos: WindowStartPosition =
        serde_json::from_str(r#"{"type":"position","x":42,"y":-7}"#).unwrap();
    assert_eq!(pos.coordinates(), (42, -7));
}

// ============================================================================
// LicenseConfig
// ============================================================================

#[test]
fn test_license_config_time_limited() {
    let license = LicenseConfig::time_limited("2025-12-31");
    assert!(license.enabled);
    assert_eq!(license.expires_at, Some("2025-12-31".to_string()));
    assert!(!license.require_token);
    assert!(license.is_active());
}

#[test]
fn test_license_config_token_required() {
    let license = LicenseConfig::token_required();
    assert!(license.enabled);
    assert!(license.require_token);
    assert!(license.expires_at.is_none());
    assert!(license.is_active());
}

#[test]
fn test_license_config_full() {
    let license = LicenseConfig::full("2025-06-30");
    assert!(license.enabled);
    assert!(license.require_token);
    assert_eq!(license.expires_at, Some("2025-06-30".to_string()));
}

#[test]
fn test_license_config_default_inactive() {
    let l = LicenseConfig::default();
    assert!(!l.enabled);
    assert!(!l.is_active());
    assert!(l.allowed_machines.is_empty());
    assert_eq!(l.grace_period_days, 0);
}

#[test]
fn test_license_config_extended_fields() {
    let l = LicenseConfig {
        embedded_token: Some("tok-secret".into()),
        validation_url: Some("https://auth.example.com/validate".into()),
        allowed_machines: vec!["MACHINE-A".into(), "MACHINE-B".into()],
        grace_period_days: 7,
        expiration_message: Some("Your trial has expired.".into()),
        ..Default::default()
    };
    assert_eq!(l.embedded_token.as_deref(), Some("tok-secret"));
    assert_eq!(l.allowed_machines.len(), 2);
    assert_eq!(l.grace_period_days, 7);
}

#[test]
fn test_license_config_is_active_conditions() {
    // no enabled, no anything → inactive
    assert!(!LicenseConfig::default().is_active());

    // only enabled but no expires/token → still inactive per implementation
    let l = LicenseConfig { enabled: true, ..Default::default() };
    assert!(!l.is_active()); // requires expires_at OR require_token

    // has expires_at → active
    assert!(LicenseConfig::time_limited("2099-01-01").is_active());

    // has token required → active
    assert!(LicenseConfig::token_required().is_active());
}

#[test]
fn test_license_config_serde_roundtrip() {
    let l = LicenseConfig::full("2030-12-31");
    let json = serde_json::to_string(&l).unwrap();
    let restored: LicenseConfig = serde_json::from_str(&json).unwrap();
    assert!(restored.enabled);
    assert!(restored.require_token);
    assert_eq!(restored.expires_at, l.expires_at);
}

// ============================================================================
// ExtensionsManifestConfig & ExtensionRemoteManifestEntry
// ============================================================================

#[test]
fn test_extensions_manifest_config_default() {
    let e = ExtensionsManifestConfig::default();
    assert!(!e.enabled); // manifest config: no default_true
    assert!(!e.bundle);
    assert!(e.local.is_empty());
    assert!(e.remote.is_empty());
}

#[test]
fn test_extensions_manifest_config_serde_roundtrip() {
    let e = ExtensionsManifestConfig {
        enabled: false,
        bundle: true,
        local: vec![PathBuf::from("exts/local")],
        remote: vec![
            ExtensionRemoteManifestEntry {
                id: "ext-1".into(),
                url: "https://example.com/ext.zip".into(),
                checksum: Some("sha256:abc...".into()),
                strip_components: 1,
            },
        ],
    };
    let json = serde_json::to_string(&e).unwrap();
    let restored: ExtensionsManifestConfig = serde_json::from_str(&json).unwrap();
    assert!(!restored.enabled);
    assert!(restored.bundle);
    assert_eq!(restored.remote.len(), 1);
    assert_eq!(restored.remote[0].strip_components, 1);
}

#[test]
fn test_extension_remote_manifest_entry_default() {
    let s = ExtensionRemoteManifestEntry::default();
    assert!(s.id.is_empty());
    assert!(s.url.is_empty());
    assert!(s.checksum.is_none());
    assert_eq!(s.strip_components, 0);
}

// ============================================================================
// IsolationConfig
// ============================================================================

#[test]
fn test_isolation_config_default() {
    let i = IsolationConfig::default();
    assert!(i.pythonpath);
    assert!(i.path);
    // extra_path and clear_env are empty by default
    assert!(i.extra_path.is_empty());
    assert!(i.clear_env.is_empty());
}

#[test]
fn test_isolation_config_full() {
    let f = IsolationConfig::full();
    assert!(f.pythonpath);
    assert!(f.path);
}

#[test]
fn test_isolation_config_none() {
    let n = IsolationConfig::none();
    assert!(!n.pythonpath);
    assert!(!n.path);
}

#[test]
fn test_isolation_config_pythonpath_only() {
    let p = IsolationConfig::pythonpath_only();
    assert!(p.pythonpath);
    assert!(!p.path);
}

#[test]
fn test_isolation_config_system_path_has_entries() {
    let p = IsolationConfig::default_system_path();
    assert!(!p.is_empty());
}

#[test]
fn test_isolation_config_inherit_env_has_entries() {
    let e = IsolationConfig::default_inherit_env();
    assert!(!e.is_empty());
}

#[test]
fn test_isolation_config_custom_extra_fields() {
    let i = IsolationConfig {
        extra_path: vec!["/custom/bin".into()],
        extra_pythonpath: vec!["/custom/lib".into()],
        clear_env: vec!["BAD_VAR".into()],
        ..IsolationConfig::none()
    };
    assert_eq!(i.extra_path.len(), 1);
    assert_eq!(i.clear_env[0], "BAD_VAR");
}

// ============================================================================
// DebugConfig
// ============================================================================

#[test]
fn test_debug_config_default() {
    let d = DebugConfig::default();
    assert!(!d.enabled);
    assert!(!d.devtools);
    assert!(!d.verbose);
    assert!(d.remote_debugging_port.is_none());
}

#[test]
fn test_debug_config_enabled_factory() {
    let d = DebugConfig::enabled();
    assert!(d.enabled);
    assert!(d.devtools);
    assert!(!d.verbose);
}

#[test]
fn test_debug_config_production_factory() {
    let d = DebugConfig::production();
    assert!(!d.enabled);
    assert!(!d.devtools);
}

#[test]
fn test_debug_config_with_remote_debugging() {
    let d = DebugConfig::default().with_remote_debugging(8080);
    assert_eq!(d.remote_debugging_port, Some(8080));
}

// ============================================================================
// RuntimeConfig
// ============================================================================

#[test]
fn test_runtime_config_default() {
    let r = RuntimeConfig::default();
    assert!(r.env.is_empty());
    assert!(r.env_files.is_empty());
    assert!(r.working_dir.is_none());
}

#[test]
fn test_runtime_config_with_env() {
    let mut env = HashMap::new();
    env.insert("K".into(), "V".into());
    let r = RuntimeConfig::with_env(env);
    assert_eq!(r.env.get("K"), Some(&"V".to_string()));
}

#[test]
fn test_runtime_config_add_env() {
    let mut r = RuntimeConfig::default();
    r.add_env("X", "1");
    r.add_env("Y", "2");
    assert_eq!(r.env.len(), 2);
}

// ============================================================================
// ProcessConfig
// ============================================================================

#[test]
fn test_process_config_default() {
    let p = ProcessConfig::default();
    assert!(!p.console);
    assert!(p.filesystem_importer);
    assert!(!p.module_search_paths.is_empty());
}

// ============================================================================
// ProtectionConfig (from protection module)
// ============================================================================

#[test]
fn test_protection_config_default() {
    let p = ProtectionConfig::default();
    assert!(!p.enabled); // default is false in protection module
    assert_eq!(p.optimization, 2); // default_optimization is 2
    assert!(!p.keep_temp);
    assert!(p.exclude.is_empty());
}

#[test]
fn test_protection_config_custom() {
    let p = ProtectionConfig {
        enabled: true,
        optimization: 3,
        keep_temp: true,
        exclude: vec!["*_test.py".into()],
        ..Default::default()
    };
    assert!(p.enabled);
    assert_eq!(p.optimization, 3);
    assert!(p.keep_temp);
    assert_eq!(p.exclude.len(), 1);
}

// ============================================================================
// CollectPattern
// ============================================================================

#[test]
fn test_collect_pattern_default() {
    let p = CollectPattern::new("src/**/*.py");
    assert_eq!(p.source, "src/**/*.py");
    assert!(p.dest.is_none());
    assert!(p.preserve_structure);
    assert!(p.description.is_none());
}

#[test]
fn test_collect_pattern_with_dest() {
    let p = CollectPattern::new("assets/*").with_dest("static/");
    assert_eq!(p.dest.as_deref(), Some("static/"));
}

#[test]
fn test_collect_pattern_all_fields() {
    let p = CollectPattern {
        source: "../lib/**/*.so".into(),
        dest: Some("lib/".into()),
        preserve_structure: false,
        description: Some("Shared libraries".into()),
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("../lib/**/*.so"));
    assert!(json.contains("lib/"));
    assert!(json.contains("Shared libraries"));
}

// ============================================================================
// TargetPlatform
// ============================================================================

#[test]
fn test_target_platform_default() {
    assert_eq!(TargetPlatform::default(), TargetPlatform::Current);
}

#[test]
fn test_target_platform_current_is_host() {
    let current = TargetPlatform::current();
    #[cfg(target_os = "windows")]
    assert_eq!(current, TargetPlatform::Windows);
    #[cfg(target_os = "macos")]
    assert_eq!(current, TargetPlatform::MacOS);
    #[cfg(target_os = "linux")]
    assert_eq!(current, TargetPlatform::Linux);
}

#[test]
fn test_target_platform_exe_extension() {
    assert_eq!(TargetPlatform::Windows.exe_extension(), ".exe");
    assert_eq!(TargetPlatform::MacOS.exe_extension(), "");
    assert_eq!(TargetPlatform::Linux.exe_extension(), "");
}

#[test]
fn test_target_platform_serde_roundtrip() {
    for p in [TargetPlatform::Current, TargetPlatform::Windows, TargetPlatform::MacOS, TargetPlatform::Linux] {
        let json = serde_json::to_string(&p).unwrap();
        let back: TargetPlatform = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }
}

// ============================================================================
// Env helpers
// ============================================================================

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

// ============================================================================
// from_manifest — inject JS/CSS
// ============================================================================

#[test]
fn test_from_manifest_inject_js_code() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "inject-test"
title = "Inject Test"

[frontend]
url = "https://example.com"

[inject]
js_code = "window.__custom = true;"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert_eq!(config.inject_js.as_deref(), Some("window.__custom = true;"));
    assert!(config.inject_css.is_none());
}

#[test]
fn test_from_manifest_inject_css_code() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "inject-css-test"
title = "Inject CSS Test"

[frontend]
url = "https://example.com"

[inject]
css_code = "body { margin: 0; }"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert!(config.inject_js.is_none());
    assert_eq!(config.inject_css.as_deref(), Some("body { margin: 0; }"));
}

#[test]
fn test_from_manifest_no_inject() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "no-inject-test"
title = "No Inject Test"

[frontend]
url = "https://example.com"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert!(config.inject_js.is_none());
    assert!(config.inject_css.is_none());
}

#[test]
fn test_from_manifest_extensions_mapping() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "ext-test"
title = "Ext Test"

[frontend]
url = "https://example.com"

[extensions]
enabled = true
bundle = true
local = ["./extensions"]
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert!(config.extensions.enabled);
    assert!(config.extensions.bundle);
    assert_eq!(config.extensions.local.len(), 1);
}

#[test]
fn test_from_manifest_csp_mapping() {
    use auroraview_pack::Manifest;

    let toml = r#"
[package]
name = "csp-test"
title = "CSP Test"

[frontend]
url = "https://example.com"

[security]
content_security_policy = "default-src 'self'"
"#;
    let manifest = Manifest::parse(toml).unwrap();
    let config = PackConfig::from_manifest(&manifest, Path::new(".")).unwrap();
    assert_eq!(
        config.content_security_policy.as_deref(),
        Some("default-src 'self'")
    );
}
