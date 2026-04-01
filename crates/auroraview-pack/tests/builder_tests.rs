//! Integration tests for the builder module:
//! BuilderRegistry, BuilderCapability, OutputFormat, and all platform builders.

use auroraview_pack::builder::{
    AndroidBuilder, Builder, BuilderCapability, BuilderRegistry, IOSBuilder, LinuxBuilder,
    MacBuilder, WebBuilder, WinBuilder,
};
use auroraview_pack::builder::traits::OutputFormat;

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
        assert!(b.is_available(), "WinBuilder should be available on Windows");
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
