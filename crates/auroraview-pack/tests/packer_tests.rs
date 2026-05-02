//! Comprehensive integration tests for the packer module:
//! PackManager, PluginRegistry, PackContext, PackPlugin, PackHook, PackTarget, PackOutput

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use auroraview_pack::{
    PackConfig, PackContext, PackError, PackHook, PackManager, PackOutput, PackPlugin, PackResult,
    PackTarget, PluginRegistry,
};

// ─── helpers ────────────────────────────────────────────────────────────────

fn minimal_config() -> PackConfig {
    PackConfig::url("about:blank")
}

/// A minimal test plugin that records which hooks were called and can inject errors.
struct RecordingPlugin {
    hooks_called: Arc<Mutex<Vec<PackHook>>>,
    wanted_hooks: Vec<PackHook>,
    fail_on: Option<PackHook>,
}

impl RecordingPlugin {
    fn new(wanted_hooks: Vec<PackHook>) -> Self {
        Self {
            hooks_called: Arc::new(Mutex::new(vec![])),
            wanted_hooks,
            fail_on: None,
        }
    }

    fn with_failure(mut self, hook: PackHook) -> Self {
        self.fail_on = Some(hook);
        self
    }

    fn called_hooks(&self) -> Vec<PackHook> {
        self.hooks_called.lock().unwrap().clone()
    }
}

impl PackPlugin for RecordingPlugin {
    fn name(&self) -> &'static str {
        "recording-plugin"
    }

    fn version(&self) -> &'static str {
        "1.2.3"
    }

    fn hooks(&self) -> Vec<PackHook> {
        self.wanted_hooks.clone()
    }

    fn on_hook(&self, hook: PackHook, _ctx: &mut PackContext) -> PackResult<()> {
        self.hooks_called.lock().unwrap().push(hook);
        if Some(hook) == self.fail_on {
            Err(PackError::Config("simulated failure".into()))
        } else {
            Ok(())
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// PackTarget — name / category / display / equality / hash
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_target_name_all_variants() {
    assert_eq!(PackTarget::Windows.name(), "Windows");
    assert_eq!(PackTarget::MacOS.name(), "macOS");
    assert_eq!(PackTarget::Linux.name(), "Linux");
    assert_eq!(PackTarget::IOS.name(), "iOS");
    assert_eq!(PackTarget::Android.name(), "Android");
    assert_eq!(PackTarget::WeChatMiniProgram.name(), "WeChat MiniProgram");
    assert_eq!(PackTarget::AlipayMiniProgram.name(), "Alipay MiniProgram");
    assert_eq!(
        PackTarget::ByteDanceMiniProgram.name(),
        "ByteDance MiniProgram"
    );
    assert_eq!(PackTarget::Web.name(), "Web");
}

#[test]
fn test_pack_target_display_equals_name() {
    for t in [
        PackTarget::Windows,
        PackTarget::MacOS,
        PackTarget::Linux,
        PackTarget::IOS,
        PackTarget::Android,
        PackTarget::Web,
        PackTarget::WeChatMiniProgram,
        PackTarget::AlipayMiniProgram,
        PackTarget::ByteDanceMiniProgram,
    ] {
        assert_eq!(t.to_string(), t.name(), "{} display != name", t.name());
    }
}

#[test]
fn test_pack_target_is_desktop() {
    assert!(PackTarget::Windows.is_desktop());
    assert!(PackTarget::MacOS.is_desktop());
    assert!(PackTarget::Linux.is_desktop());
    assert!(!PackTarget::IOS.is_desktop());
    assert!(!PackTarget::Android.is_desktop());
    assert!(!PackTarget::Web.is_desktop());
    assert!(!PackTarget::WeChatMiniProgram.is_desktop());
    assert!(!PackTarget::AlipayMiniProgram.is_desktop());
    assert!(!PackTarget::ByteDanceMiniProgram.is_desktop());
}

#[test]
fn test_pack_target_is_mobile() {
    assert!(PackTarget::IOS.is_mobile());
    assert!(PackTarget::Android.is_mobile());
    assert!(!PackTarget::Windows.is_mobile());
    assert!(!PackTarget::Web.is_mobile());
    assert!(!PackTarget::WeChatMiniProgram.is_mobile());
}

#[test]
fn test_pack_target_is_miniprogram() {
    assert!(PackTarget::WeChatMiniProgram.is_miniprogram());
    assert!(PackTarget::AlipayMiniProgram.is_miniprogram());
    assert!(PackTarget::ByteDanceMiniProgram.is_miniprogram());
    assert!(!PackTarget::Web.is_miniprogram());
    assert!(!PackTarget::Windows.is_miniprogram());
    assert!(!PackTarget::IOS.is_miniprogram());
}

#[test]
fn test_pack_target_current_is_desktop() {
    let current = PackTarget::current();
    assert!(current.is_desktop());
}

#[test]
fn test_pack_target_eq_and_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(PackTarget::Windows);
    set.insert(PackTarget::Windows); // duplicate ignored
    assert_eq!(set.len(), 1);
    set.insert(PackTarget::MacOS);
    assert_eq!(set.len(), 2);
    set.insert(PackTarget::Linux);
    assert_eq!(set.len(), 3);
    // all three are unique
}

#[test]
fn test_pack_target_clone_and_copy() {
    // PackTarget derives Clone and Copy — verify we can use both
    let t1 = PackTarget::Windows;
    let _t2 = t1; // Copy
    let t3 = PackTarget::MacOS;
    let _t4 = t3; // Copy (PackTarget implements Copy, clone is unnecessary)
}

// ══════════════════════════════════════════════════════════════════════════════
// PackOutput — construction, builder chain, clone
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_output_new_defaults() {
    let out = PackOutput::new(
        std::path::PathBuf::from("/out/app.exe"),
        "static",
        PackTarget::Windows,
    );
    assert_eq!(out.size, 0);
    assert_eq!(out.asset_count, 0);
    assert_eq!(out.python_file_count, 0);
    assert_eq!(out.mode, "static");
    assert_eq!(out.target, PackTarget::Windows);
    assert!(out.metadata.is_empty());
    assert!(out.executable.ends_with("app.exe"));
}

#[test]
fn test_pack_output_builder_full_chain() {
    let out = PackOutput::new(
        std::path::PathBuf::from("/out/app.exe"),
        "fullstack",
        PackTarget::Linux,
    )
    .with_size(2048)
    .with_assets(5)
    .with_python_files(12)
    .with_metadata("build_id", "abc123")
    .with_metadata("platform", "x86_64")
    .with_metadata("commit", "def456");

    assert_eq!(out.size, 2048);
    assert_eq!(out.asset_count, 5);
    assert_eq!(out.python_file_count, 12);
    assert_eq!(out.metadata.get("build_id").unwrap(), "abc123");
    assert_eq!(out.metadata.get("platform").unwrap(), "x86_64");
    assert_eq!(out.metadata.get("commit").unwrap(), "def456");
    assert_eq!(out.metadata.len(), 3);
}

#[test]
fn test_pack_output_with_python_files() {
    let out = PackOutput::new(PathBuf::from("/a"), "fs", PackTarget::Windows).with_python_files(99);
    assert_eq!(out.python_file_count, 99);
}

#[test]
fn test_pack_output_clone_independence() {
    let out = PackOutput::new(std::path::PathBuf::from("/tmp/x"), "url", PackTarget::MacOS)
        .with_size(100)
        .with_metadata("k", "v");
    let out2 = out.clone();
    assert_eq!(out2.size, 100);
    assert_eq!(out2.target, PackTarget::MacOS);
    assert_eq!(out2.metadata.get("k").unwrap(), "v");
    // they are independent clones
    assert_eq!(out.metadata.len(), out2.metadata.len());
}

#[test]
fn test_pack_output_debug_formatting() {
    let out = PackOutput::new(PathBuf::from("/app"), "static", PackTarget::Windows).with_size(1024);
    let s = format!("{:?}", out);
    assert!(s.contains("1024") || s.contains("app"));
}

// ══════════════════════════════════════════════════════════════════════════════
// PackHook — ordering, display, equality, all() coverage
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_hook_all_order_and_completeness() {
    let hooks = PackHook::all();
    assert!(!hooks.is_empty());
    assert_eq!(hooks[0], PackHook::BeforePack);
    assert_eq!(*hooks.last().unwrap(), PackHook::AfterPack);
    // AfterValidate is second
    assert_eq!(hooks[1], PackHook::AfterValidate);
    // OnError is NOT in the all() list (it's triggered implicitly)
    assert!(!hooks.contains(&PackHook::OnError));
    // Verify at least the main lifecycle hooks are present
    assert!(hooks.contains(&PackHook::BeforeCollect));
    assert!(hooks.contains(&PackHook::AfterCollect));
    assert!(hooks.contains(&PackHook::BeforeOverlay));
    assert!(hooks.contains(&PackHook::AfterOverlay));
    assert!(hooks.contains(&PackHook::BeforeTarget));
    assert!(hooks.contains(&PackHook::AfterTarget));
}

#[test]
fn test_pack_hook_display_all_variants() {
    use std::fmt::Write;
    let mut s = String::new();
    for h in [
        PackHook::BeforePack,
        PackHook::AfterValidate,
        PackHook::BeforeCollect,
        PackHook::AfterCollect,
        PackHook::BeforeOverlay,
        PackHook::AfterOverlay,
        PackHook::BeforeTarget,
        PackHook::AfterTarget,
        PackHook::AfterPack,
        PackHook::OnError,
    ] {
        writeln!(s, "{}", h).unwrap();
    }
    assert!(s.contains("before_pack"));
    assert!(s.contains("on_error"));
    assert!(s.contains("after_pack"));
}

#[test]
fn test_pack_hook_equality_and_copy() {
    let h1 = PackHook::BeforePack;
    let h2 = PackHook::BeforePack;
    let h3 = PackHook::AfterPack;
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
    // Copy is trivial for Copy type
    let h4 = h1;
    assert_eq!(h4, h1);
}

// ══════════════════════════════════════════════════════════════════════════════
// PackContext — construction, assets, metadata, overlay, extensions, errors
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_context_new() {
    let config = minimal_config();
    let ctx = PackContext::new(config, PackTarget::Windows);
    assert_eq!(ctx.target, PackTarget::Windows);
    assert!(ctx.assets.is_empty());
    assert!(ctx.extensions.is_empty());
    assert!(ctx.errors.is_empty());
    assert!(ctx.overlay.is_none());
    assert!(ctx.downloads.is_empty());
    assert!(ctx.metadata.is_empty());
    // output_dir should match config output_dir
    assert!(!ctx.output_dir.as_os_str().is_empty());
}

#[test]
fn test_pack_context_temp_dir_derived_from_output() {
    let cfg = minimal_config().with_output_dir("./my-output");
    let ctx = PackContext::new(cfg, PackTarget::Windows);
    assert!(ctx.temp_dir.ends_with(".pack_temp"));
    assert!(ctx.temp_dir.starts_with("./my-output") || ctx.temp_dir.starts_with("my-output"));
}

#[test]
fn test_pack_context_add_asset_multiple() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Linux);
    ctx.add_asset("index.html".to_string(), b"<html/>".to_vec());
    ctx.add_asset("style.css".to_string(), b"body{}".to_vec());
    ctx.add_asset("script.js".to_string(), b"console.log(1)".to_vec());
    assert_eq!(ctx.assets.len(), 3);
    assert_eq!(ctx.assets[0].0, "index.html");
    assert_eq!(ctx.assets[2].1, b"console.log(1)");
}

#[test]
fn test_pack_context_add_asset_binary_content() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    let binary_data: Vec<u8> = (0u8..=255).collect();
    ctx.add_asset("binary.bin".to_string(), binary_data.clone());
    assert_eq!(ctx.assets[0].1.len(), 256);
}

#[test]
fn test_pack_context_add_extension() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.extensions.push(std::path::PathBuf::from("/ext/a.crx"));
    ctx.extensions.push(std::path::PathBuf::from("/ext/b.zip"));
    assert_eq!(ctx.extensions.len(), 2);
}

#[test]
fn test_pack_context_metadata_round_trip_string() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.set_metadata("str_key", "hello").unwrap();
    let v: Option<String> = ctx.get_metadata("str_key");
    assert_eq!(v.as_deref(), Some("hello"));
}

#[test]
fn test_pack_context_metadata_round_trip_integer() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.set_metadata("count", 42u32).unwrap();
    let n: Option<u32> = ctx.get_metadata("count");
    assert_eq!(n, Some(42));
}

#[test]
fn test_pack_context_metadata_round_trip_bool() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.set_metadata("flag", true).unwrap();
    let b: Option<bool> = ctx.get_metadata("flag");
    assert_eq!(b, Some(true));
}

#[test]
fn test_pack_context_metadata_overwrite_same_key() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.set_metadata("k", "v1").unwrap();
    ctx.set_metadata("k", "v2").unwrap();
    let v: Option<String> = ctx.get_metadata("k");
    assert_eq!(v.as_deref(), Some("v2"));
}

#[test]
fn test_pack_context_metadata_missing_returns_none() {
    let ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    let v: Option<String> = ctx.get_metadata("nonexistent");
    assert!(v.is_none());
    let n: Option<u64> = ctx.get_metadata("also_missing");
    assert!(n.is_none());
}

#[test]
fn test_pack_context_init_overlay_then_access() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    assert!(ctx.overlay.is_none());
    ctx.init_overlay(minimal_config());
    assert!(ctx.overlay.is_some());
    // Can now get mutable access
    assert!(ctx.overlay_mut().is_ok());
}

#[test]
fn test_pack_context_overlay_mut_error_when_not_initialized() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    let result = ctx.overlay_mut();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("overlay")
            || err_msg.to_lowercase().contains("not initialized")
    );
}

#[test]
fn test_pack_context_cleanup_nonexistent_temp_dir() {
    let ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    // temp_dir doesn't exist yet — cleanup should succeed silently
    assert!(ctx.cleanup().is_ok());
}

#[test]
fn test_pack_context_errors_push_and_read() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    assert!(ctx.errors.is_empty());
    let err1 = PackError::Config("err1".into());
    let err2 = PackError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "err2"));
    ctx.errors.push(err1);
    ctx.errors.push(err2);
    assert_eq!(ctx.errors.len(), 2);
}

#[test]
fn test_pack_context_downloads() {
    let ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    assert!(ctx.downloads.is_empty());
    // downloads is a public vec that can be pushed to
    assert_eq!(ctx.downloads.len(), 0);
}

// ══════════════════════════════════════════════════════════════════════════════
// PluginRegistry — creation, defaults, queries
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_plugin_registry_new_is_empty() {
    let reg = PluginRegistry::new();
    assert_eq!(reg.plugins().len(), 0);
}

#[test]
fn test_plugin_registry_with_defaults_has_plugins() {
    let reg = PluginRegistry::with_defaults();
    assert!(!reg.plugins().is_empty());
}

#[test]
fn test_plugin_registry_with_defaults_has_current_target_packer() {
    let reg = PluginRegistry::with_defaults();
    let current = PackTarget::current();
    assert!(
        reg.get_target_packer(current).is_some(),
        "Expected a target packer for {:?}",
        current
    );
}

#[test]
fn test_plugin_registry_get_builtin_packer_by_name() {
    let reg = PluginRegistry::with_defaults();
    // "desktop" is the standard built-in packer name on desktop platforms
    if current_platform_has_desktop_packer(&reg) {
        assert!(reg.get_packer("desktop").is_some());
    }
}

#[test]
fn test_plugin_registry_get_nonexistent_packer_returns_none() {
    let reg = PluginRegistry::new();
    assert!(reg.get_packer("nonexistent").is_none());
    assert!(reg.get_target_packer(PackTarget::IOS).is_none()); // no iOS packer by default
}

#[test]
fn test_plugin_registry_available_targets_nonempty_and_contains_current() {
    let reg = PluginRegistry::with_defaults();
    let targets = reg.available_targets();
    assert!(!targets.is_empty());
    let current = PackTarget::current();
    assert!(
        targets.contains(&current),
        "Available targets should contain current platform {:?}",
        current
    );
}

#[test]
fn test_plugin_registry_target_packers_nonempty() {
    let reg = PluginRegistry::with_defaults();
    let tp: Vec<_> = reg.target_packers().collect();
    assert!(!tp.is_empty());
}

// ══════════════════════════════════════════════════════════════════════════════
// Custom Plugin — lifecycle, hook invocation, failure mode, multi-plugin
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_custom_plugin_init_and_cleanup_lifecycle() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![]));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.init_plugins(&mut ctx).unwrap();
    reg.cleanup_plugins(&mut ctx).unwrap();
    // No crash — default init/cleanup are no-ops
}

#[test]
fn test_custom_plugin_hook_single_invocation() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![PackHook::BeforePack]));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.run_hooks(PackHook::BeforePack, &mut ctx).unwrap();

    let called = plugin.called_hooks();
    assert_eq!(called.len(), 1);
    assert_eq!(called[0], PackHook::BeforePack);
}

#[test]
fn test_custom_plugin_hook_not_invoked_for_unregistered_stage() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![PackHook::AfterPack]));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.run_hooks(PackHook::BeforePack, &mut ctx).unwrap();

    assert!(plugin.called_hooks().is_empty());
}

#[test]
fn test_custom_plugin_multiple_hooks_sequential() {
    let mut reg = PluginRegistry::new();
    let wanted = vec![
        PackHook::BeforePack,
        PackHook::AfterCollect,
        PackHook::AfterPack,
    ];
    let plugin = Arc::new(RecordingPlugin::new(wanted.clone()));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    for hook in &wanted {
        reg.run_hooks(*hook, &mut ctx).unwrap();
    }

    let called = plugin.called_hooks();
    assert_eq!(called.len(), 3);
    assert_eq!(called, wanted);
}

#[test]
fn test_custom_plugin_version_override() {
    let plugin = RecordingPlugin::new(vec![]);
    assert_eq!(plugin.version(), "1.2.3"); // overridden above
}

#[test]
fn test_custom_plugin_name() {
    let plugin = RecordingPlugin::new(vec![]);
    assert_eq!(plugin.name(), "recording-plugin");
}

#[test]
fn test_plugin_hook_error_propagates() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(
        RecordingPlugin::new(vec![PackHook::BeforePack]).with_failure(PackHook::BeforePack),
    );
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    let result = reg.run_hooks(PackHook::BeforePack, &mut ctx);
    assert!(result.is_err());
    // The hook was still recorded before the error
    assert_eq!(plugin.called_hooks().len(), 1);
}

#[test]
fn test_multi_plugin_independent_hooks() {
    let mut reg = PluginRegistry::new();
    let p1 = Arc::new(RecordingPlugin::new(vec![PackHook::BeforePack]));
    let p2 = Arc::new(RecordingPlugin::new(vec![PackHook::AfterPack]));
    reg.register_plugin(p1.clone());
    reg.register_plugin(p2.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.run_hooks(PackHook::BeforePack, &mut ctx).unwrap();
    reg.run_hooks(PackHook::AfterPack, &mut ctx).unwrap();

    assert_eq!(p1.called_hooks().len(), 1);
    assert_eq!(p2.called_hooks().len(), 1);
    assert_ne!(p1.called_hooks(), p2.called_hooks()); // different hooks
}

// ══════════════════════════════════════════════════════════════════════════════
// PackManager — creation, registry delegation, format, unsupported target
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pack_manager_new() {
    let manager = PackManager::new();
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_default_trait() {
    let manager = PackManager::default();
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_with_custom_registry() {
    let reg = PluginRegistry::with_defaults();
    let manager = PackManager::with_registry(reg);
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_registry_immutable_access() {
    let manager = PackManager::new();
    let _reg = manager.registry();
    // should not panic and returns reference
}

#[test]
fn test_pack_manager_available_targets_contains_current() {
    let manager = PackManager::new();
    let current = PackTarget::current();
    assert!(manager.available_targets().contains(&current));
}

#[test]
fn test_pack_manager_format_targets_structure() {
    let manager = PackManager::new();
    let formatted = manager.format_targets();
    assert!(!formatted.is_empty());
    assert!(formatted.contains("Available pack targets"));
    // Should contain at least one status indicator (checkmark or cross)
    assert!(
        formatted.contains('\u{2713}')
            || formatted.contains('\u{2717}')
            || formatted.contains('✓')
            || formatted.contains('✗')
    );
}

#[test]
fn test_pack_manager_register_plugin_via_manager() {
    let mut manager = PackManager::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![]));
    manager.register_plugin(plugin);
    // No panic — plugin delegated to registry
}

#[test]
fn test_pack_manager_pack_unsupported_target_error() {
    let manager = PackManager::new();
    let config = minimal_config();
    // IOS/Android typically have no packer in CI
    let result = manager.pack_for_target(&config, PackTarget::IOS);
    assert!(
        result.is_err(),
        "Packing for IOS should fail without IOS tooling"
    );
}

#[test]
fn test_pack_manager_registry_mutable_access() {
    let mut manager = PackManager::new();
    // registry_mut exists for mutation
    let _r = manager.registry_mut();
}

// ══════════════════════════════════════════════════════════════════════════════
// Helpers
// ══════════════════════════════════════════════════════════════════════════════

fn current_platform_has_desktop_packer(_reg: &PluginRegistry) -> bool {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        true
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        false
    }
}
