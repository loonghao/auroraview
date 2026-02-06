//! Packer trait definitions for extensible pack system
//!
//! This module defines the core traits that enable:
//! - Multiple pack targets (Desktop, iOS, Android, MiniProgram)
//! - Plugin architecture for custom pack behaviors
//! - Hook system for build lifecycle customization

#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use crate::{PackConfig, PackError};

/// Pack result type
pub type PackResult<T> = Result<T, PackError>;

/// Result of a pack operation
#[derive(Debug, Clone)]
pub struct PackOutput {
    /// Path to the generated executable or directory
    pub executable: std::path::PathBuf,
    /// Size of the executable in bytes
    pub size: u64,
    /// Number of embedded assets
    pub asset_count: usize,
    /// Number of Python files bundled (for fullstack mode)
    pub python_file_count: usize,
    /// Pack mode used
    pub mode: String,
    /// Target platform
    pub target: PackTarget,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl PackOutput {
    /// Create a new PackOutput
    pub fn new(executable: std::path::PathBuf, mode: &str, target: PackTarget) -> Self {
        Self {
            executable,
            size: 0,
            asset_count: 0,
            python_file_count: 0,
            mode: mode.to_string(),
            target,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the output size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set asset count
    pub fn with_assets(mut self, count: usize) -> Self {
        self.asset_count = count;
        self
    }

    /// Set Python file count
    pub fn with_python_files(mut self, count: usize) -> Self {
        self.python_file_count = count;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Pack target platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackTarget {
    /// Windows desktop application
    Windows,
    /// macOS desktop application
    MacOS,
    /// Linux desktop application
    Linux,
    /// iOS application
    IOS,
    /// Android application
    Android,
    /// WeChat MiniProgram
    WeChatMiniProgram,
    /// Alipay MiniProgram
    AlipayMiniProgram,
    /// ByteDance MiniProgram
    ByteDanceMiniProgram,
    /// Web (PWA or static)
    Web,
}

impl PackTarget {
    /// Get the target for current platform
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Self::Windows;
        #[cfg(target_os = "macos")]
        return Self::MacOS;
        #[cfg(target_os = "linux")]
        return Self::Linux;
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Self::Web;
    }

    /// Check if target is desktop
    pub fn is_desktop(&self) -> bool {
        matches!(self, Self::Windows | Self::MacOS | Self::Linux)
    }

    /// Check if target is mobile
    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::IOS | Self::Android)
    }

    /// Check if target is mini program
    pub fn is_miniprogram(&self) -> bool {
        matches!(
            self,
            Self::WeChatMiniProgram | Self::AlipayMiniProgram | Self::ByteDanceMiniProgram
        )
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
            Self::Linux => "Linux",
            Self::IOS => "iOS",
            Self::Android => "Android",
            Self::WeChatMiniProgram => "WeChat MiniProgram",
            Self::AlipayMiniProgram => "Alipay MiniProgram",
            Self::ByteDanceMiniProgram => "ByteDance MiniProgram",
            Self::Web => "Web",
        }
    }
}

impl std::fmt::Display for PackTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Core packer trait for different pack modes
///
/// Implement this trait to add support for new pack modes (URL, Frontend, FullStack, etc.)
pub trait Packer: Send + Sync {
    /// Pack mode name
    fn name(&self) -> &'static str;

    /// Check if this packer can handle the given config
    fn can_pack(&self, config: &PackConfig) -> bool;

    /// Validate the configuration before packing
    fn validate(&self, config: &PackConfig) -> PackResult<()>;

    /// Execute the pack operation
    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput>;
}

/// Target-specific packer trait
///
/// Implement this trait to add support for new target platforms (iOS, Android, etc.)
pub trait TargetPacker: Send + Sync {
    /// Target platform
    fn target(&self) -> PackTarget;

    /// Check if this target is available on the current system
    fn is_available(&self) -> bool;

    /// Get required tools/dependencies for this target
    fn required_tools(&self) -> Vec<&'static str>;

    /// Check if all required tools are installed
    fn check_tools(&self) -> PackResult<()>;

    /// Execute target-specific pack
    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput>;
}

/// Pack lifecycle hook
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackHook {
    /// Before any pack operation starts
    BeforePack,
    /// After configuration validation
    AfterValidate,
    /// Before assets are collected
    BeforeCollect,
    /// After assets are collected
    AfterCollect,
    /// Before overlay is written
    BeforeOverlay,
    /// After overlay is written
    AfterOverlay,
    /// Before target-specific processing
    BeforeTarget,
    /// After target-specific processing
    AfterTarget,
    /// After pack completes successfully
    AfterPack,
    /// On pack error (for cleanup)
    OnError,
}

impl PackHook {
    /// Get all hooks in execution order
    pub fn all() -> &'static [PackHook] {
        &[
            Self::BeforePack,
            Self::AfterValidate,
            Self::BeforeCollect,
            Self::AfterCollect,
            Self::BeforeOverlay,
            Self::AfterOverlay,
            Self::BeforeTarget,
            Self::AfterTarget,
            Self::AfterPack,
        ]
    }
}

impl std::fmt::Display for PackHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::BeforePack => "before_pack",
            Self::AfterValidate => "after_validate",
            Self::BeforeCollect => "before_collect",
            Self::AfterCollect => "after_collect",
            Self::BeforeOverlay => "before_overlay",
            Self::AfterOverlay => "after_overlay",
            Self::BeforeTarget => "before_target",
            Self::AfterTarget => "after_target",
            Self::AfterPack => "after_pack",
            Self::OnError => "on_error",
        };
        write!(f, "{}", name)
    }
}

/// Hook handler function type
pub type HookHandler = Box<dyn Fn(&mut PackContext) -> PackResult<()> + Send + Sync>;

/// Pack plugin trait
///
/// Plugins can modify pack behavior at various lifecycle points
pub trait PackPlugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &'static str;

    /// Plugin version
    fn version(&self) -> &'static str {
        "0.1.0"
    }

    /// Initialize plugin with context
    fn init(&self, _context: &mut PackContext) -> PackResult<()> {
        Ok(())
    }

    /// Get hooks this plugin wants to handle
    fn hooks(&self) -> Vec<PackHook> {
        vec![]
    }

    /// Handle a hook
    fn on_hook(&self, hook: PackHook, context: &mut PackContext) -> PackResult<()>;

    /// Cleanup on completion
    fn cleanup(&self, _context: &mut PackContext) -> PackResult<()> {
        Ok(())
    }
}

/// Pack context shared across plugins and packers
pub struct PackContext {
    /// Pack configuration
    pub config: PackConfig,
    /// Target platform
    pub target: PackTarget,
    /// Output directory
    pub output_dir: std::path::PathBuf,
    /// Temporary directory for intermediate files
    pub temp_dir: std::path::PathBuf,
    /// Overlay data being built
    pub overlay: Option<crate::overlay::OverlayData>,
    /// Collected assets
    pub assets: Vec<(String, Vec<u8>)>,
    /// Extension files to bundle
    pub extensions: Vec<std::path::PathBuf>,
    /// Download entries
    pub downloads: Vec<crate::DownloadEntry>,
    /// Custom metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Errors encountered (for OnError hook)
    pub errors: Vec<PackError>,
}

impl PackContext {
    /// Create a new pack context
    pub fn new(config: PackConfig, target: PackTarget) -> Self {
        let output_dir = config.output_dir.clone();
        let temp_dir = output_dir.join(".pack_temp");

        Self {
            config,
            target,
            output_dir,
            temp_dir,
            overlay: None,
            assets: Vec::new(),
            extensions: Vec::new(),
            downloads: Vec::new(),
            metadata: std::collections::HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Add an asset to the pack
    pub fn add_asset(&mut self, path: String, content: Vec<u8>) {
        self.assets.push((path, content));
    }

    /// Set metadata value
    pub fn set_metadata<T: serde::Serialize>(&mut self, key: &str, value: T) -> PackResult<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| PackError::Config(format!("Failed to serialize metadata: {}", e)))?;
        self.metadata.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Get metadata value
    pub fn get_metadata<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Initialize overlay
    pub fn init_overlay(&mut self, config: PackConfig) {
        self.overlay = Some(crate::overlay::OverlayData::new(config));
    }

    /// Get mutable reference to overlay
    pub fn overlay_mut(&mut self) -> PackResult<&mut crate::overlay::OverlayData> {
        self.overlay
            .as_mut()
            .ok_or_else(|| PackError::Config("Overlay not initialized".to_string()))
    }

    /// Cleanup temporary files
    pub fn cleanup(&self) -> PackResult<()> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_target_current() {
        let target = PackTarget::current();
        assert!(target.is_desktop());
    }

    #[test]
    fn test_pack_target_categories() {
        assert!(PackTarget::Windows.is_desktop());
        assert!(PackTarget::MacOS.is_desktop());
        assert!(PackTarget::Linux.is_desktop());

        assert!(PackTarget::IOS.is_mobile());
        assert!(PackTarget::Android.is_mobile());

        assert!(PackTarget::WeChatMiniProgram.is_miniprogram());
        assert!(PackTarget::AlipayMiniProgram.is_miniprogram());
    }

    #[test]
    fn test_pack_output_builder() {
        let output = PackOutput::new(
            std::path::PathBuf::from("/test/app.exe"),
            "static",
            PackTarget::Windows,
        )
        .with_size(1024)
        .with_assets(10)
        .with_metadata("version", "1.0.0");

        assert_eq!(output.size, 1024);
        assert_eq!(output.asset_count, 10);
        assert_eq!(output.metadata.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_pack_hook_order() {
        let hooks = PackHook::all();
        assert_eq!(hooks[0], PackHook::BeforePack);
        assert_eq!(hooks[hooks.len() - 1], PackHook::AfterPack);
    }
}
