//! Plugin registry for extensible pack system
//!
//! This module provides a registry for pack plugins that can modify
//! pack behavior at various lifecycle points.

use std::collections::HashMap;
use std::sync::Arc;

use super::collect_plugin::CollectPlugin;
use super::traits::{
    PackContext, PackHook, PackPlugin, PackResult, PackTarget, Packer, TargetPacker,
};

/// Plugin registry for managing pack plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Vec<Arc<dyn PackPlugin>>,
    /// Registered packers by mode name
    packers: HashMap<String, Arc<dyn Packer>>,
    /// Registered target packers
    target_packers: HashMap<PackTarget, Arc<dyn TargetPacker>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
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
        registry.register_plugin(Arc::new(ExtensionBundlerPlugin));
        registry.register_plugin(Arc::new(LicenseBundlerPlugin));
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

/// Plugin that bundles Chrome extensions into the pack overlay.
///
/// Runs during `BeforeOverlay` to collect local and pre-cached store
/// extension files into the overlay assets under `extensions/<id>/`.
struct ExtensionBundlerPlugin;

impl PackPlugin for ExtensionBundlerPlugin {
    fn name(&self) -> &'static str {
        "extension-bundler"
    }

    fn hooks(&self) -> Vec<PackHook> {
        vec![PackHook::BeforeOverlay]
    }

    fn on_hook(&self, hook: PackHook, context: &mut PackContext) -> PackResult<()> {
        if hook != PackHook::BeforeOverlay {
            return Ok(());
        }

        if context.extensions.is_empty() {
            return Ok(());
        }

        let overlay = match context.overlay.as_mut() {
            Some(o) => o,
            None => return Ok(()),
        };

        let mut bundled = 0usize;
        for ext_path in &context.extensions.clone() {
            if !ext_path.exists() {
                tracing::warn!(
                    "ExtensionBundlerPlugin: extension path not found: {}",
                    ext_path.display()
                );
                continue;
            }

            let ext_name = ext_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("ext_{}", bundled));

            // Recursively add extension directory contents
            if ext_path.is_dir() {
                if let Err(e) =
                    add_dir_to_overlay(ext_path, &format!("extensions/{}", ext_name), overlay)
                {
                    tracing::warn!(
                        "ExtensionBundlerPlugin: failed to bundle '{}': {}",
                        ext_name,
                        e
                    );
                    continue;
                }
            } else {
                // Single file (e.g. .crx)
                match std::fs::read(ext_path) {
                    Ok(content) => {
                        let key = format!("extensions/{}", ext_name);
                        overlay.add_asset(key, content);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "ExtensionBundlerPlugin: failed to read '{}': {}",
                            ext_path.display(),
                            e
                        );
                        continue;
                    }
                }
            }

            bundled += 1;
        }

        if bundled > 0 {
            tracing::info!("ExtensionBundlerPlugin: bundled {} extensions", bundled);
        }

        Ok(())
    }
}

/// Plugin that embeds a license file into the pack overlay.
///
/// Runs during `BeforeOverlay`. If a `license` section is present in the
/// pack config, the plugin looks for a LICENSE file in the output directory
/// and adds it to the overlay under `license/LICENSE`.
struct LicenseBundlerPlugin;

impl PackPlugin for LicenseBundlerPlugin {
    fn name(&self) -> &'static str {
        "license-bundler"
    }

    fn hooks(&self) -> Vec<PackHook> {
        vec![PackHook::BeforeOverlay]
    }

    fn on_hook(&self, hook: PackHook, context: &mut PackContext) -> PackResult<()> {
        if hook != PackHook::BeforeOverlay {
            return Ok(());
        }

        // Only run when license config is present
        if context.config.license.is_none() {
            return Ok(());
        }

        let overlay = match context.overlay.as_mut() {
            Some(o) => o,
            None => return Ok(()),
        };

        // Search for license files in common locations
        let search_dirs = [&context.output_dir, &context.temp_dir];
        let license_names = ["LICENSE", "LICENSE.md", "LICENSE.txt", "LICENCE"];

        for dir in &search_dirs {
            for name in &license_names {
                let license_path = dir.join(name);
                if license_path.exists() {
                    match std::fs::read(&license_path) {
                        Ok(content) => {
                            overlay.add_asset(format!("license/{}", name), content);
                            tracing::info!(
                                "LicenseBundlerPlugin: embedded {}",
                                license_path.display()
                            );
                            return Ok(());
                        }
                        Err(e) => {
                            tracing::warn!(
                                "LicenseBundlerPlugin: failed to read {}: {}",
                                license_path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        tracing::debug!("LicenseBundlerPlugin: no license file found");
        Ok(())
    }
}

/// Recursively add a directory's contents to an overlay
fn add_dir_to_overlay(
    dir: &std::path::Path,
    prefix: &str,
    overlay: &mut crate::overlay::OverlayData,
) -> PackResult<()> {
    for entry in std::fs::read_dir(dir).map_err(|e| {
        crate::PackError::Build(format!("Failed to read directory {}: {}", dir.display(), e))
    })? {
        let entry = entry.map_err(|e| crate::PackError::Build(e.to_string()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let key = format!("{}/{}", prefix, name);

        if path.is_dir() {
            add_dir_to_overlay(&path, &key, overlay)?;
        } else {
            let content = std::fs::read(&path).map_err(|e| {
                crate::PackError::Build(format!("Failed to read {}: {}", path.display(), e))
            })?;
            overlay.add_asset(key, content);
        }
    }
    Ok(())
}
