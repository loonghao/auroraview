//! Integration tests for the packer module:
//! PackManager, PluginRegistry, PackContext, PackPlugin, PackHook, PackTarget, PackOutput, HookStage

use auroraview_pack::{
    PackConfig, PackContext, PackHook, PackManager, PackOutput, PackPlugin, PackResult, PackTarget,
    PluginRegistry,
};
use std::sync::{Arc, Mutex};

// ─── helpers ────────────────────────────────────────────────────────────────

fn minimal_config() -> PackConfig {
    PackConfig::url("about:blank")
}

// ─── PackTarget ──────────────────────────────────────────────────────────────

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
    ] {
        assert_eq!(t.to_string(), t.name());
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
}

#[test]
fn test_pack_target_is_mobile() {
    assert!(PackTarget::IOS.is_mobile());
    assert!(PackTarget::Android.is_mobile());
    assert!(!PackTarget::Windows.is_mobile());
    assert!(!PackTarget::Web.is_mobile());
}

#[test]
fn test_pack_target_is_miniprogram() {
    assert!(PackTarget::WeChatMiniProgram.is_miniprogram());
    assert!(PackTarget::AlipayMiniProgram.is_miniprogram());
    assert!(PackTarget::ByteDanceMiniProgram.is_miniprogram());
    assert!(!PackTarget::Web.is_miniprogram());
    assert!(!PackTarget::Windows.is_miniprogram());
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
    set.insert(PackTarget::Windows);
    assert_eq!(set.len(), 1);
    set.insert(PackTarget::MacOS);
    assert_eq!(set.len(), 2);
}

// ─── PackOutput ───────────────────────────────────────────────────────────────

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
}

#[test]
fn test_pack_output_builder_chain() {
    let out = PackOutput::new(
        std::path::PathBuf::from("/out/app.exe"),
        "fullstack",
        PackTarget::Linux,
    )
    .with_size(2048)
    .with_assets(5)
    .with_python_files(12)
    .with_metadata("build_id", "abc123")
    .with_metadata("platform", "x86_64");

    assert_eq!(out.size, 2048);
    assert_eq!(out.asset_count, 5);
    assert_eq!(out.python_file_count, 12);
    assert_eq!(out.metadata.get("build_id").unwrap(), "abc123");
    assert_eq!(out.metadata.get("platform").unwrap(), "x86_64");
}

#[test]
fn test_pack_output_clone() {
    let out = PackOutput::new(
        std::path::PathBuf::from("/tmp/x"),
        "url",
        PackTarget::MacOS,
    )
    .with_size(100);
    let out2 = out.clone();
    assert_eq!(out2.size, 100);
    assert_eq!(out2.target, PackTarget::MacOS);
}

// ─── PackHook ─────────────────────────────────────────────────────────────────

#[test]
fn test_pack_hook_all_order() {
    let hooks = PackHook::all();
    assert!(!hooks.is_empty());
    assert_eq!(hooks[0], PackHook::BeforePack);
    assert_eq!(*hooks.last().unwrap(), PackHook::AfterPack);
}

#[test]
fn test_pack_hook_display() {
    assert_eq!(PackHook::BeforePack.to_string(), "before_pack");
    assert_eq!(PackHook::AfterPack.to_string(), "after_pack");
    assert_eq!(PackHook::BeforeCollect.to_string(), "before_collect");
    assert_eq!(PackHook::AfterCollect.to_string(), "after_collect");
    assert_eq!(PackHook::BeforeOverlay.to_string(), "before_overlay");
    assert_eq!(PackHook::AfterOverlay.to_string(), "after_overlay");
    assert_eq!(PackHook::BeforeTarget.to_string(), "before_target");
    assert_eq!(PackHook::AfterTarget.to_string(), "after_target");
    assert_eq!(PackHook::OnError.to_string(), "on_error");
}

#[test]
fn test_pack_hook_eq_and_clone() {
    let h1 = PackHook::BeforePack;
    let h2 = h1;
    assert_eq!(h1, h2);
}

// ─── PackContext ──────────────────────────────────────────────────────────────

#[test]
fn test_pack_context_new() {
    let config = minimal_config();
    let ctx = PackContext::new(config, PackTarget::Windows);
    assert_eq!(ctx.target, PackTarget::Windows);
    assert!(ctx.assets.is_empty());
    assert!(ctx.extensions.is_empty());
    assert!(ctx.errors.is_empty());
    assert!(ctx.overlay.is_none());
}

#[test]
fn test_pack_context_add_asset() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Linux);
    ctx.add_asset("index.html".to_string(), b"<html/>".to_vec());
    assert_eq!(ctx.assets.len(), 1);
    assert_eq!(ctx.assets[0].0, "index.html");
    assert_eq!(ctx.assets[0].1, b"<html/>");
}

#[test]
fn test_pack_context_metadata_round_trip() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);

    ctx.set_metadata("key1", "hello").unwrap();
    ctx.set_metadata("count", 42u32).unwrap();

    let v: Option<String> = ctx.get_metadata("key1");
    assert_eq!(v.unwrap(), "hello");

    let n: Option<u32> = ctx.get_metadata("count");
    assert_eq!(n.unwrap(), 42);
}

#[test]
fn test_pack_context_metadata_missing_returns_none() {
    let ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    let v: Option<String> = ctx.get_metadata("nonexistent");
    assert!(v.is_none());
}

#[test]
fn test_pack_context_init_overlay() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    assert!(ctx.overlay.is_none());
    ctx.init_overlay(minimal_config());
    assert!(ctx.overlay.is_some());
}

#[test]
fn test_pack_context_overlay_mut_error_when_not_initialized() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    assert!(ctx.overlay_mut().is_err());
}

#[test]
fn test_pack_context_overlay_mut_ok_when_initialized() {
    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    ctx.init_overlay(minimal_config());
    assert!(ctx.overlay_mut().is_ok());
}

#[test]
fn test_pack_context_cleanup_nonexistent_temp_dir() {
    let ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    // temp_dir doesn't exist — cleanup should succeed silently
    assert!(ctx.cleanup().is_ok());
}

// ─── PluginRegistry ───────────────────────────────────────────────────────────

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
fn test_plugin_registry_with_defaults_has_current_target() {
    let reg = PluginRegistry::with_defaults();
    let current = PackTarget::current();
    assert!(reg.get_target_packer(current).is_some());
}

#[test]
fn test_plugin_registry_register_and_get_packer() {
    let reg = PluginRegistry::with_defaults();
    // "desktop" packer is registered by default
    assert!(reg.get_packer("desktop").is_some());
}

#[test]
fn test_plugin_registry_get_nonexistent_packer_returns_none() {
    let reg = PluginRegistry::new();
    assert!(reg.get_packer("nonexistent").is_none());
}

#[test]
fn test_plugin_registry_available_targets_nonempty() {
    let reg = PluginRegistry::with_defaults();
    let targets = reg.available_targets();
    assert!(!targets.is_empty());
}

// ─── PackPlugin (custom plugin) ──────────────────────────────────────────────

/// A minimal test plugin that records which hooks were called.
struct RecordingPlugin {
    hooks_called: Arc<Mutex<Vec<PackHook>>>,
    wanted_hooks: Vec<PackHook>,
}

impl RecordingPlugin {
    fn new(wanted_hooks: Vec<PackHook>) -> Self {
        Self {
            hooks_called: Arc::new(Mutex::new(vec![])),
            wanted_hooks,
        }
    }

    fn called_hooks(&self) -> Vec<PackHook> {
        self.hooks_called.lock().unwrap().clone()
    }
}

impl PackPlugin for RecordingPlugin {
    fn name(&self) -> &'static str {
        "recording-plugin"
    }

    fn hooks(&self) -> Vec<PackHook> {
        self.wanted_hooks.clone()
    }

    fn on_hook(&self, hook: PackHook, _ctx: &mut PackContext) -> PackResult<()> {
        self.hooks_called.lock().unwrap().push(hook);
        Ok(())
    }
}

#[test]
fn test_custom_plugin_init_and_cleanup() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![]));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.init_plugins(&mut ctx).unwrap();
    reg.cleanup_plugins(&mut ctx).unwrap();
    // No crash — init/cleanup default impls are no-ops
}

#[test]
fn test_custom_plugin_hook_invoked() {
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
fn test_custom_plugin_hook_not_invoked_for_other_stages() {
    let mut reg = PluginRegistry::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![PackHook::AfterPack]));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    reg.run_hooks(PackHook::BeforePack, &mut ctx).unwrap();

    assert!(plugin.called_hooks().is_empty());
}

#[test]
fn test_custom_plugin_multiple_hooks() {
    let mut reg = PluginRegistry::new();
    let wanted = vec![PackHook::BeforePack, PackHook::AfterCollect, PackHook::AfterPack];
    let plugin = Arc::new(RecordingPlugin::new(wanted.clone()));
    reg.register_plugin(plugin.clone());

    let mut ctx = PackContext::new(minimal_config(), PackTarget::Windows);
    for hook in &wanted {
        reg.run_hooks(*hook, &mut ctx).unwrap();
    }

    let called = plugin.called_hooks();
    assert_eq!(called.len(), 3);
}

#[test]
fn test_custom_plugin_version_default() {
    let plugin = RecordingPlugin::new(vec![]);
    assert_eq!(plugin.version(), "0.1.0");
}

// ─── PackManager ─────────────────────────────────────────────────────────────

#[test]
fn test_pack_manager_new() {
    let manager = PackManager::new();
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_default() {
    let manager = PackManager::default();
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_with_registry() {
    let reg = PluginRegistry::with_defaults();
    let manager = PackManager::with_registry(reg);
    assert!(!manager.available_targets().is_empty());
}

#[test]
fn test_pack_manager_registry_access() {
    let manager = PackManager::new();
    let _reg = manager.registry();
}

#[test]
fn test_pack_manager_available_targets_contains_current() {
    let manager = PackManager::new();
    let current = PackTarget::current();
    assert!(manager.available_targets().contains(&current));
}

#[test]
fn test_pack_manager_format_targets_nonempty() {
    let manager = PackManager::new();
    let formatted = manager.format_targets();
    assert!(!formatted.is_empty());
    assert!(formatted.contains("Available pack targets"));
}

#[test]
fn test_pack_manager_register_plugin() {
    let mut manager = PackManager::new();
    let plugin = Arc::new(RecordingPlugin::new(vec![]));
    manager.register_plugin(plugin);
    // No panic — plugin was registered
}

#[test]
fn test_pack_manager_pack_unsupported_target_returns_error() {
    let manager = PackManager::new();
    let config = minimal_config();
    // IOS requires tooling not available in CI — should return Err
    let result = manager.pack_for_target(&config, PackTarget::IOS);
    assert!(result.is_err());
}
