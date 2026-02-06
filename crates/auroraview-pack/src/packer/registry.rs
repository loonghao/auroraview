//! Plugin registry for extensible pack system
//!
//! This module provides a registry for pack plugins that can modify
//! pack behavior at various lifecycle points.

use super::collect_plugin::CollectPlugin;
use super::traits::{
    PackContext, PackHook, PackPlugin, PackResult, PackTarget, Packer, TargetPacker,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin registry for managing pack plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Vec<Arc<dyn PackPlugin>>,
    /// Registered packers by mode name
    packers: HashMap<String, Arc<dyn Packer>>,
    /// Registered target packers
    target_packers: HashMap<PackTarget, Arc<dyn TargetPacker>>,
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            packers: HashMap::new(),
            target_packers: HashMap::new(),
        }
    }

    /// Create a registry with default packers registered
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register default packers
        registry.register_packer(Arc::new(super::desktop::StaticPacker::new()));

        // Register default target packers
        #[cfg(target_os = "windows")]
        registry.register_target_packer(Arc::new(super::desktop::WindowsTargetPacker::new()));

        #[cfg(target_os = "macos")]
        registry.register_target_packer(Arc::new(super::desktop::MacOSTargetPacker::new()));

        #[cfg(target_os = "linux")]
        registry.register_target_packer(Arc::new(super::desktop::LinuxTargetPacker::new()));

        // Register mobile packers (available but may not work without tools)
        registry.register_target_packer(Arc::new(super::mobile::IOSTargetPacker::new()));
        registry.register_target_packer(Arc::new(super::mobile::AndroidTargetPacker::new()));

        // Register mini-program packers
        registry
            .register_target_packer(Arc::new(super::miniprogram::WeChatMiniProgramPacker::new()));
        registry
            .register_target_packer(Arc::new(super::miniprogram::AlipayMiniProgramPacker::new()));
        registry.register_target_packer(Arc::new(
            super::miniprogram::ByteDanceMiniProgramPacker::new(),
        ));

        // Register built-in plugins
        // TODO: Fix ExtensionBundlerPlugin to work with PackConfig structure
        // registry.register_plugin(Arc::new(ExtensionBundlerPlugin::new()));
        // TODO: Implement LicenseBundlerPlugin when PackContext has base_dir field
        // registry.register_plugin(Arc::new(LicenseBundlerPlugin::new()));
        registry.register_plugin(Arc::new(CollectPlugin::new(std::path::PathBuf::from("."))));

        registry
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn PackPlugin>) {
        tracing::debug!("Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Register a packer
    pub fn register_packer(&mut self, packer: Arc<dyn Packer>) {
        let name = packer.name().to_string();
        tracing::debug!("Registering packer: {}", name);
        self.packers.insert(name, packer);
    }

    /// Register a target packer
    pub fn register_target_packer(&mut self, packer: Arc<dyn TargetPacker>) {
        let target = packer.target();
        tracing::debug!("Registering target packer: {}", target);
        self.target_packers.insert(target, packer);
    }

    /// Get a packer by name
    pub fn get_packer(&self, name: &str) -> Option<Arc<dyn Packer>> {
        self.packers.get(name).cloned()
    }

    /// Get a target packer
    pub fn get_target_packer(&self, target: PackTarget) -> Option<Arc<dyn TargetPacker>> {
        self.target_packers.get(&target).cloned()
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Arc<dyn PackPlugin>] {
        &self.plugins
    }

    /// Get all registered packers
    pub fn packers(&self) -> impl Iterator<Item = &Arc<dyn Packer>> {
        self.packers.values()
    }

    /// Get all registered target packers
    pub fn target_packers(&self) -> impl Iterator<Item = &Arc<dyn TargetPacker>> {
        self.target_packers.values()
    }

    /// Get available targets (those that have tools installed)
    pub fn available_targets(&self) -> Vec<PackTarget> {
        self.target_packers
            .iter()
            .filter(|(_, packer)| packer.is_available())
            .map(|(target, _)| *target)
            .collect()
    }

    /// Initialize all plugins
    pub fn init_plugins(&self, context: &mut PackContext) -> PackResult<()> {
        for plugin in &self.plugins {
            plugin.init(context)?;
        }
        Ok(())
    }

    /// Run hooks for a specific stage
    pub fn run_hooks(&self, hook: PackHook, context: &mut PackContext) -> PackResult<()> {
        for plugin in &self.plugins {
            if plugin.hooks().contains(&hook) {
                plugin.on_hook(hook, context)?;
            }
        }
        Ok(())
    }

    /// Cleanup all plugins
    pub fn cleanup_plugins(&self, context: &mut PackContext) -> PackResult<()> {
        for plugin in &self.plugins {
            if let Err(e) = plugin.cleanup(context) {
                tracing::warn!("Plugin {} cleanup failed: {}", plugin.name(), e);
            }
        }
        Ok(())
    }
}

// TODO: Implement ExtensionBundlerPlugin when PackConfig has extensions field
// TODO: Implement LicenseBundlerPlugin when PackContext has base_dir field
