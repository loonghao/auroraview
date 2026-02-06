//! Modular packer system for AuroraView
//!
//! This module provides an extensible packer architecture supporting:
//! - Multiple pack modes (URL, Frontend, FullStack)
//! - Multiple target platforms (Desktop, iOS, Android, MiniProgram)
//! - Plugin system for custom pack behaviors
//! - Hook system for build lifecycle customization
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      PackManager                            │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │   Plugins   │  │   Packers   │  │   Targets   │         │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!           ┌──────────────────┼──────────────────┐
//!           ▼                  ▼                  ▼
//!    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//!    │   Desktop   │    │   Mobile    │    │ MiniProgram │
//!    │ Win/Mac/Lin │    │  iOS/And    │    │ WX/Ali/Byte │
//!    └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_pack::packer::{PackManager, PackTarget};
//!
//! let manager = PackManager::new();
//!
//! // Pack for current platform
//! let output = manager.pack(&config)?;
//!
//! // Pack for specific target
//! let output = manager.pack_for_target(&config, PackTarget::IOS)?;
//!
//! // List available targets
//! for target in manager.available_targets() {
//!     println!("Available: {}", target);
//! }
//! ```

mod desktop;
mod extensions;
mod hooks;
mod miniprogram;
mod mobile;
mod registry;
mod traits;

mod collect_plugin;

pub use registry::PluginRegistry;
pub use traits::{
    PackContext, PackHook, PackOutput, PackPlugin, PackResult, PackTarget, Packer, TargetPacker,
};

#[cfg(target_os = "linux")]
pub use desktop::LinuxTargetPacker;
#[cfg(target_os = "macos")]
pub use desktop::MacOSTargetPacker;

use crate::{PackConfig, PackError};
use std::sync::Arc;

/// High-level pack manager coordinating plugins, packers, and targets
pub struct PackManager {
    registry: PluginRegistry,
}

impl PackManager {
    /// Create a new pack manager with default configuration
    pub fn new() -> Self {
        let registry = PluginRegistry::with_defaults();

        // Built-in plugins are now registered in PluginRegistry::with_defaults()

        Self { registry }
    }

    /// Create a pack manager with custom registry
    pub fn with_registry(registry: PluginRegistry) -> Self {
        Self { registry }
    }

    /// Get the plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get mutable plugin registry
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn PackPlugin>) {
        self.registry.register_plugin(plugin);
    }

    /// Get available target platforms
    pub fn available_targets(&self) -> Vec<PackTarget> {
        self.registry.available_targets()
    }

    /// Pack for the current platform
    pub fn pack(&self, config: &PackConfig) -> PackResult<PackOutput> {
        let target = PackTarget::current();
        self.pack_for_target(config, target)
    }

    /// Pack for a specific target platform
    pub fn pack_for_target(
        &self,
        config: &PackConfig,
        target: PackTarget,
    ) -> PackResult<PackOutput> {
        // Create pack context
        let mut context = PackContext::new(config.clone(), target);

        // Get target packer
        let target_packer = self.registry.get_target_packer(target).ok_or_else(|| {
            PackError::Config(format!("No packer registered for target: {}", target))
        })?;

        // Check if target is available
        if !target_packer.is_available() {
            return Err(PackError::Config(format!(
                "Target {} is not available on this system. Required tools: {:?}",
                target,
                target_packer.required_tools()
            )));
        }

        // Check required tools
        target_packer.check_tools()?;

        // Initialize plugins
        self.registry.init_plugins(&mut context)?;

        // Run pack with hooks
        let result = self.run_pack_with_hooks(&mut context, target_packer);

        // Cleanup
        self.registry.cleanup_plugins(&mut context)?;
        let _ = context.cleanup();

        result
    }

    /// Run pack operation with lifecycle hooks
    fn run_pack_with_hooks(
        &self,
        context: &mut PackContext,
        target_packer: Arc<dyn TargetPacker>,
    ) -> PackResult<PackOutput> {
        // Before pack
        self.registry.run_hooks(PackHook::BeforePack, context)?;

        // Before collect
        self.registry.run_hooks(PackHook::BeforeCollect, context)?;

        // After collect
        self.registry.run_hooks(PackHook::AfterCollect, context)?;

        // Before overlay
        self.registry.run_hooks(PackHook::BeforeOverlay, context)?;

        // Before target
        self.registry.run_hooks(PackHook::BeforeTarget, context)?;

        // Execute target-specific pack
        let output = match target_packer.pack(context) {
            Ok(output) => output,
            Err(e) => {
                context.errors.push(e.clone());
                self.registry.run_hooks(PackHook::OnError, context)?;
                return Err(e);
            }
        };

        // After target
        self.registry.run_hooks(PackHook::AfterTarget, context)?;

        // After overlay
        self.registry.run_hooks(PackHook::AfterOverlay, context)?;

        // After pack
        self.registry.run_hooks(PackHook::AfterPack, context)?;

        Ok(output)
    }

    /// Print available targets and their status
    pub fn print_targets(&self) {
        println!("Available pack targets:");
        println!();

        for packer in self.registry.target_packers() {
            let target = packer.target();
            let available = packer.is_available();
            let status = if available { "✓" } else { "✗" };
            let tools = packer.required_tools().join(", ");

            println!(
                "  {} {:20} {} (tools: {})",
                status,
                target.name(),
                if available {
                    "available"
                } else {
                    "not available"
                },
                if tools.is_empty() { "none" } else { &tools }
            );
        }
    }
}

impl Default for PackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_manager_creation() {
        let manager = PackManager::new();
        assert!(!manager.available_targets().is_empty());
    }

    #[test]
    fn test_pack_manager_current_target() {
        let targets = PackManager::new().available_targets();
        let current = PackTarget::current();
        assert!(targets.contains(&current));
    }

    #[test]
    fn test_pack_target_display() {
        assert_eq!(PackTarget::Windows.to_string(), "Windows");
        assert_eq!(PackTarget::IOS.to_string(), "iOS");
        assert_eq!(
            PackTarget::WeChatMiniProgram.to_string(),
            "WeChat MiniProgram"
        );
    }
}
