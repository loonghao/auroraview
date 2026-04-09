//! Integration tests for the builder module:
//! BuilderRegistry, BuilderCapability, OutputFormat, and all platform builders.

use std::fs;
use std::path::PathBuf;

use auroraview_pack::builder::{
    common::{
        AppConfig, BackendConfig, BuildConfig, BuildContext, DebugConfig, ExtensionsConfig,
        FrontendBundle, FrontendConfig, PlatformConfig, PythonStrategy, TargetConfig, WindowConfig,
    },
    traits::OutputFormat,
    AlipayBuilder, AndroidBuilder, Builder, BuilderCapability, BuilderRegistry, ByteDanceBuilder,
    IOSBuilder, LinuxBuilder, MacBuilder, WeChatBuilder, WebBuilder, WinBuilder,
};
use auroraview_pack::PackError;
use tempfile::tempdir;

fn minimal_build_config(platform: &str, output_dir: PathBuf) -> BuildConfig {
    BuildConfig {
        version: 1,
        app: AppConfig {
            name: "AuroraView".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            copyright: None,
            icon: None,
            identifier: None,
        },
        target: TargetConfig {
            platform: platform.to_string(),
            format: None,
            output_dir,
            output_name: Some("auroraview-test".to_string()),
        },
        window: WindowConfig::default(),
        frontend: None,
        backend: None,
        extensions: ExtensionsConfig::default(),
        platform: PlatformConfig::default(),
        debug: DebugConfig::default(),
    }
}

fn minimal_build_context(platform: &str, output_dir: PathBuf) -> BuildContext {
    let config = minimal_build_config(platform, output_dir.clone());
    BuildContext::new(config, output_dir)
}

// ─────────────────────────────────────────────────────────────
// BuilderCapability
// ─────────────────────────────────────────────────────────────

#[test]
fn builder_capability_name_all_variants() {
    let cases = [
        (BuilderCapability::Standalone, "Standalone"),
        (BuilderCapability::Installer, "Installer"),
        (BuilderCapability::Portable, "Portable"),
        (BuilderCapability::CodeSign, "Code Signing"),
        (BuilderCapability::Notarize, "Notarization"),
        (BuilderCapability::PythonEmbed, "Python Embed"),
        (BuilderCapability::NodeEmbed, "Node.js Embed"),
        (BuilderCapability::Extensions, "Extensions"),
        (BuilderCapability::DevTools, "DevTools"),
        (BuilderCapability::AppStore, "App Store"),
        (BuilderCapability::HotReload, "Hot Reload"),
    ];
    for (cap, expected) in cases {
        assert_eq!(cap.name(), expected);
    }
}

#[test]
fn builder_capability_eq() {
    assert_eq!(BuilderCapability::Standalone, BuilderCapability::Standalone);
    assert_ne!(BuilderCapability::Installer, BuilderCapability::Portable);
}

#[test]
fn builder_capability_clone() {
    let cap = BuilderCapability::PythonEmbed;
    let cap2 = cap;
    assert_eq!(cap, cap2);
}

// ─────────────────────────────────────────────────────────────
// OutputFormat
// ─────────────────────────────────────────────────────────────

#[test]
fn output_format_extension_all_variants() {
    let cases = [
        (OutputFormat::WindowsExe, "exe"),
        (OutputFormat::WindowsMsix, "msix"),
        (OutputFormat::MacApp, "app"),
        (OutputFormat::MacDmg, "dmg"),
        (OutputFormat::MacPkg, "pkg"),
        (OutputFormat::LinuxAppImage, "AppImage"),
        (OutputFormat::LinuxDeb, "deb"),
        (OutputFormat::LinuxRpm, "rpm"),
        (OutputFormat::IosIpa, "ipa"),
        (OutputFormat::AndroidApk, "apk"),
        (OutputFormat::AndroidAab, "aab"),
        (OutputFormat::WebStatic, ""),
        (OutputFormat::WebPwa, ""),
        (OutputFormat::MiniProgram, ""),
    ];
    for (fmt, expected) in cases {
        assert_eq!(fmt.extension(), expected);
    }
}

// ─────────────────────────────────────────────────────────────
// WinBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn win_builder_id() {
    let b = WinBuilder::new();
    assert_eq!(b.id(), "win");
}

#[test]
fn win_builder_name() {
    let b = WinBuilder::new();
    assert!(!b.name().is_empty());
    assert!(b.name().to_lowercase().contains("windows"));
}

#[test]
fn win_builder_targets_contain_windows() {
    let b = WinBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"windows"),
        "WinBuilder targets should contain 'windows'"
    );
}

#[test]
fn win_builder_capabilities_has_standalone() {
    let b = WinBuilder::new();
    let caps = b.capabilities();
    assert!(caps.contains(&BuilderCapability::Standalone));
}

#[test]
fn win_builder_capabilities_has_python_embed() {
    let b = WinBuilder::new();
    let caps = b.capabilities();
    assert!(caps.contains(&BuilderCapability::PythonEmbed));
}

#[test]
fn win_builder_portable_mode() {
    let b = WinBuilder::new().portable(true);
    // Portable builder should still be valid
    assert_eq!(b.id(), "win");
}

#[test]
fn win_builder_is_available_on_windows() {
    let b = WinBuilder::new();
    if cfg!(target_os = "windows") {
        assert!(
            b.is_available(),
            "WinBuilder should be available on Windows"
        );
    }
    // On other platforms it may be unavailable — just don't panic
    let _ = b.is_available();
}

// ─────────────────────────────────────────────────────────────
// MacBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn mac_builder_id() {
    let b = MacBuilder::new();
    assert_eq!(b.id(), "mac");
}

#[test]
fn mac_builder_targets_contain_macos() {
    let b = MacBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"macos") || targets.contains(&"darwin"),
        "MacBuilder targets should contain 'macos' or 'darwin'"
    );
}

#[test]
fn mac_builder_capabilities() {
    let b = MacBuilder::new();
    let caps = b.capabilities();
    assert!(caps.contains(&BuilderCapability::Standalone));
    assert!(!caps.is_empty());
}

// ─────────────────────────────────────────────────────────────
// LinuxBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn linux_builder_id() {
    let b = LinuxBuilder::new();
    assert_eq!(b.id(), "linux");
}

#[test]
fn linux_builder_targets_contain_linux() {
    let b = LinuxBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"linux"),
        "LinuxBuilder targets should contain 'linux'"
    );
}

#[test]
fn linux_builder_capabilities() {
    let b = LinuxBuilder::new();
    let caps = b.capabilities();
    assert!(!caps.is_empty());
}

// ─────────────────────────────────────────────────────────────
// IOSBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn ios_builder_id() {
    let b = IOSBuilder::new();
    assert_eq!(b.id(), "ios");
}

#[test]
fn ios_builder_targets_contain_ios() {
    let b = IOSBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"ios"),
        "IOSBuilder targets should contain 'ios'"
    );
}

#[test]
fn ios_builder_not_available_on_non_mac() {
    let b = IOSBuilder::new();
    // iOS builds require macOS; on other platforms it must report unavailable
    if !cfg!(target_os = "macos") {
        assert!(
            !b.is_available(),
            "IOSBuilder should not be available on non-macOS"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// AndroidBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn android_builder_id() {
    let b = AndroidBuilder::new();
    assert_eq!(b.id(), "android");
}

#[test]
fn android_builder_targets_contain_android() {
    let b = AndroidBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"android"),
        "AndroidBuilder targets should contain 'android'"
    );
}

// ─────────────────────────────────────────────────────────────
// WebBuilder
// ─────────────────────────────────────────────────────────────

#[test]
fn web_builder_id() {
    let b = WebBuilder::new();
    assert_eq!(b.id(), "web");
}

#[test]
fn web_builder_targets_contain_web() {
    let b = WebBuilder::new();
    let targets = b.targets();
    assert!(
        targets.contains(&"web"),
        "WebBuilder targets should contain 'web'"
    );
}

#[test]
fn web_builder_is_always_available() {
    let b = WebBuilder::new();
    // Web builder should always be available
    assert!(b.is_available());
}

// ─────────────────────────────────────────────────────────────
// BuilderRegistry
// ─────────────────────────────────────────────────────────────

#[test]
fn builder_registry_with_defaults_has_9_builders() {
    let registry = BuilderRegistry::with_defaults();
    let count = registry.all().count();
    // win, mac, linux, ios, android, web, wechat, alipay, bytedance
    assert_eq!(count, 9, "Expected 9 default builders, got {}", count);
}

#[test]
fn builder_registry_get_win() {
    let registry = BuilderRegistry::with_defaults();
    let builder = registry.get("win");
    assert!(builder.is_some());
    assert_eq!(builder.unwrap().id(), "win");
}

#[test]
fn builder_registry_get_mac() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("mac").is_some());
}

#[test]
fn builder_registry_get_linux() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("linux").is_some());
}

#[test]
fn builder_registry_get_ios() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("ios").is_some());
}

#[test]
fn builder_registry_get_android() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("android").is_some());
}

#[test]
fn builder_registry_get_web() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("web").is_some());
}

#[test]
fn builder_registry_get_wechat() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("wechat").is_some());
}

#[test]
fn builder_registry_get_alipay() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("alipay").is_some());
}

#[test]
fn builder_registry_get_bytedance() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("bytedance").is_some());
}

#[test]
fn builder_registry_get_nonexistent_returns_none() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.get("nonexistent_builder").is_none());
}

#[test]
fn builder_registry_find_for_target_windows() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("windows");
    assert!(b.is_some());
    assert_eq!(b.unwrap().id(), "win");
}

#[test]
fn builder_registry_find_for_target_win64() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("win64");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_macos() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("macos");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_linux() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("linux");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_ios() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("ios");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_android() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("android");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_web() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("web");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_wechat() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("wechat");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_douyin() {
    let registry = BuilderRegistry::with_defaults();
    let b = registry.find_for_target("douyin");
    assert!(b.is_some());
}

#[test]
fn builder_registry_find_for_target_nonexistent() {
    let registry = BuilderRegistry::with_defaults();
    assert!(registry.find_for_target("nonexistent").is_none());
}

#[test]
fn builder_registry_empty_has_no_builders() {
    let registry = BuilderRegistry::new();
    assert_eq!(registry.all().count(), 0);
}

#[test]
fn builder_registry_custom_register() {
    use std::sync::Arc;
    let mut registry = BuilderRegistry::new();
    registry.register(Arc::new(WinBuilder::new()));
    assert_eq!(registry.all().count(), 1);
    assert!(registry.get("win").is_some());
}

#[test]
fn builder_registry_default_same_as_with_defaults() {
    let registry = BuilderRegistry::default();
    assert_eq!(registry.all().count(), 9);
}

#[test]
fn builder_registry_available_subset_of_all() {
    let registry = BuilderRegistry::with_defaults();
    let all_count = registry.all().count();
    let avail_count = registry.available().len();
    assert!(avail_count <= all_count);
}

#[test]
fn builder_common_build_config_deserializes_defaults() {
    let config: BuildConfig = serde_json::from_value(serde_json::json!({
        "app": { "name": "Demo" },
        "target": { "platform": "web", "output_dir": "dist" },
        "window": {},
        "frontend": { "url": "https://example.com" }
    }))
    .unwrap();

    assert_eq!(config.version, 1);
    assert_eq!(config.app.version, "1.0.0");
    assert_eq!(config.window.width, 1280);
    assert_eq!(config.window.height, 720);
    assert!(config.window.resizable);
    assert!(matches!(
        config.frontend,
        Some(FrontendConfig::Url { ref url }) if url == "https://example.com"
    ));
}

#[test]
fn builder_common_frontend_backend_serde_supports_variants() {
    let url_frontend: FrontendConfig =
        serde_json::from_value(serde_json::json!({ "url": "https://example.com" })).unwrap();
    assert!(matches!(
        url_frontend,
        FrontendConfig::Url { ref url } if url == "https://example.com"
    ));

    let path_frontend: FrontendConfig =
        serde_json::from_value(serde_json::json!({ "path": "./dist" })).unwrap();
    assert!(matches!(
        path_frontend,
        FrontendConfig::Path { ref path } if path == &PathBuf::from("./dist")
    ));

    let backend: BackendConfig = serde_json::from_value(serde_json::json!({
        "type": "python",
        "entry": "main:run",
        "strategy": "embedded"
    }))
    .unwrap();

    match backend {
        BackendConfig::Python(config) => {
            assert_eq!(config.entry, "main:run");
            assert_eq!(config.version, "3.11");
            assert!(matches!(config.strategy, PythonStrategy::Embedded));
        }
        other => panic!("expected python backend, got {:?}", other),
    }

    let json = serde_json::to_string(&PythonStrategy::Standalone).unwrap();
    assert_eq!(json, "\"standalone\"");
}

#[test]
fn builder_common_extensions_helpers_follow_enabled_flag() {
    let disabled: ExtensionsConfig = serde_json::from_value(serde_json::json!({
        "enabled": false,
        "local": [{ "path": "./ext-one" }]
    }))
    .unwrap();
    assert_eq!(disabled.extension_count(), 1);
    assert!(!disabled.has_extensions());

    let enabled: ExtensionsConfig = serde_json::from_value(serde_json::json!({
        "enabled": true,
        "local": [{ "path": "./ext-one" }],
        "store": [{ "id": "abcdefghijklmnop" }]
    }))
    .unwrap();
    assert_eq!(enabled.extension_count(), 2);
    assert!(enabled.has_extensions());
}

#[test]
fn win_builder_validate_rejects_missing_frontend_path() {
    let temp = tempdir().unwrap();
    let mut config = minimal_build_config("win", temp.path().join("out"));
    config.frontend = Some(FrontendConfig::Path {
        path: temp.path().join("missing-dist"),
    });
    let ctx = BuildContext::new(config, temp.path().join("out"));

    let err = WinBuilder::new().validate(&ctx).unwrap_err();
    match err {
        PackError::FrontendNotFound(path) => {
            assert!(path.ends_with("missing-dist"));
        }
        other => panic!("expected FrontendNotFound, got {:?}", other),
    }
}

#[test]
fn win_builder_validate_rejects_empty_frontend_url() {
    let temp = tempdir().unwrap();
    let mut config = minimal_build_config("win", temp.path().join("out"));
    config.frontend = Some(FrontendConfig::Url { url: String::new() });
    let ctx = BuildContext::new(config, temp.path().join("out"));

    let err = WinBuilder::new().validate(&ctx).unwrap_err();
    assert!(matches!(
        err,
        PackError::InvalidUrl(message) if message.contains("URL cannot be empty")
    ));
}

#[test]
fn wechat_builder_validate_requires_app_id() {
    let temp = tempdir().unwrap();
    let ctx = minimal_build_context("wechat", temp.path().join("out"));

    let err = WeChatBuilder::new().validate(&ctx).unwrap_err();
    assert!(matches!(
        err,
        PackError::Config(message) if message.contains("App ID")
    ));
}

#[test]
fn wechat_builder_build_generates_project_files_and_webview_page() {
    let temp = tempdir().unwrap();
    let cli_path = temp.path().join("cli.bat");
    fs::write(&cli_path, "@echo off\r\n").unwrap();

    let output_dir = temp.path().join("build");
    let mut ctx = minimal_build_context("wechat", output_dir.clone());
    ctx.config.app.name = "AuroraView Demo".to_string();
    ctx.config.app.identifier = Some("wx1234567890".to_string());
    ctx.frontend = Some(FrontendBundle {
        root: temp.path().join("frontend"),
        files: vec![("index.html".to_string(), b"<html></html>".to_vec())],
    });
    ctx.add_asset("assets/icon.png", b"icon".to_vec());

    let output = WeChatBuilder::new()
        .cli_path(cli_path.clone())
        .build(&mut ctx)
        .unwrap();

    let project_dir = output_dir.join("wechat-miniprogram");
    assert_eq!(output.path, project_dir);
    assert_eq!(output.format, "wechat-miniprogram");
    assert_eq!(output.asset_count, 1);
    assert_eq!(
        output.info.get("app_id").map(String::as_str),
        Some("wx1234567890")
    );
    assert_eq!(
        output.info.get("cli_path").map(String::as_str),
        Some(cli_path.to_string_lossy().as_ref())
    );

    let project_config = fs::read_to_string(project_dir.join("project.config.json")).unwrap();
    assert!(project_config.contains("\"appId\": \"wx1234567890\""));
    assert!(project_config.contains("\"projectName\": \"AuroraView Demo\""));

    let app_json = fs::read_to_string(project_dir.join("app.json")).unwrap();
    assert!(app_json.contains("pages/index/index"));

    let index_wxml = fs::read_to_string(project_dir.join("pages/index/index.wxml")).unwrap();
    assert_eq!(index_wxml, "<web-view src=\"/index.html\"></web-view>");

    let copied_asset = fs::read(project_dir.join("assets/icon.png")).unwrap();
    assert_eq!(copied_asset, b"icon");
}

#[test]
fn wechat_builder_build_prefers_builder_app_id_and_plain_view_without_frontend() {
    let temp = tempdir().unwrap();
    let cli_path = temp.path().join("cli.bat");
    fs::write(&cli_path, "@echo off\r\n").unwrap();

    let output_dir = temp.path().join("plain-build");
    let mut ctx = minimal_build_context("wechat", output_dir.clone());
    ctx.config.app.name.clear();
    ctx.config.app.identifier = Some("wx-config-id".to_string());

    let output = WeChatBuilder::new()
        .app_id("wx-builder-id")
        .cli_path(cli_path)
        .build(&mut ctx)
        .unwrap();

    assert_eq!(
        output.info.get("app_id").map(String::as_str),
        Some("wx-builder-id")
    );

    let project_dir = output_dir.join("wechat-miniprogram");
    let project_config = fs::read_to_string(project_dir.join("project.config.json")).unwrap();
    assert!(project_config.contains("\"appId\": \"wx-builder-id\""));
    assert!(project_config.contains("\"projectName\": \"AuroraView App\""));

    let index_wxml = fs::read_to_string(project_dir.join("pages/index/index.wxml")).unwrap();
    assert_eq!(
        index_wxml,
        "<view class=\"container\"><text>AuroraView App</text></view>"
    );
}

#[test]
fn miniprogram_stub_builders_report_tools_and_unimplemented_errors() {
    let alipay = AlipayBuilder::new().app_id("ali123");
    assert_eq!(alipay.required_tools(), vec!["alipay-devtools"]);
    assert!(alipay.targets().contains(&"zhifubao"));

    let bytedance = ByteDanceBuilder::new().app_id("tt123");
    assert_eq!(bytedance.required_tools(), vec!["bytedance-devtools"]);
    assert!(bytedance.targets().contains(&"douyin"));

    let temp = tempdir().unwrap();
    let mut alipay_ctx = minimal_build_context("alipay", temp.path().join("alipay-out"));
    let mut bytedance_ctx = minimal_build_context("bytedance", temp.path().join("tt-out"));

    let alipay_err = alipay.build(&mut alipay_ctx).unwrap_err();
    let bytedance_err = bytedance.build(&mut bytedance_ctx).unwrap_err();

    assert!(matches!(
        alipay_err,
        PackError::Build(message) if message.contains("Alipay MiniProgram builder not yet implemented")
    ));
    assert!(matches!(
        bytedance_err,
        PackError::Build(message) if message.contains("ByteDance MiniProgram builder not yet implemented")
    ));
}
