//! Desktop platform packer (Windows, macOS, Linux)
//!
//! Handles packing for desktop platforms using the new builder architecture.

use super::traits::{PackContext, PackOutput, PackResult, PackTarget, Packer, TargetPacker};
use crate::{PackConfig, PackError, PackMode};

/// Desktop packer for URL, Frontend, and FullStack modes
pub struct DesktopPacker;

impl DesktopPacker {
    /// Create a new desktop packer
    pub fn new() -> Self {
        Self
    }
}

impl Default for DesktopPacker {
    fn default() -> Self {
        Self::new()
    }
}

impl Packer for DesktopPacker {
    fn name(&self) -> &'static str {
        "desktop"
    }

    fn can_pack(&self, config: &PackConfig) -> bool {
        matches!(
            config.mode,
            PackMode::Url { .. } | PackMode::Frontend { .. } | PackMode::FullStack { .. }
        )
    }

    fn validate(&self, config: &PackConfig) -> PackResult<()> {
        match &config.mode {
            PackMode::Frontend { path }
            | PackMode::FullStack {
                frontend_path: path,
                ..
            } => {
                if !path.exists() {
                    return Err(PackError::Config(format!(
                        "Frontend path does not exist: {}",
                        path.display()
                    )));
                }
                let index = path.join("index.html");
                if !index.exists() {
                    return Err(PackError::Config(format!(
                        "No index.html found in frontend path: {}",
                        path.display()
                    )));
                }
            }
            PackMode::Url { url } => {
                if url.is_empty() {
                    return Err(PackError::Config("URL cannot be empty".to_string()));
                }
            }
        }
        Ok(())
    }

    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput> {
        use crate::builder::common::{AppConfig, BuildConfig, FrontendConfig, TargetConfig};
        use crate::builder::{BuildContext, Builder, WinBuilder};

        // Convert PackContext to BuildContext for new builder
        let frontend = match &context.config.mode {
            PackMode::Url { url } => Some(FrontendConfig::Url { url: url.clone() }),
            PackMode::Frontend { path } => Some(FrontendConfig::Path { path: path.clone() }),
            PackMode::FullStack { frontend_path, .. } => Some(FrontendConfig::Path {
                path: frontend_path.clone(),
            }),
        };

        let build_config = BuildConfig {
            version: 2,
            app: AppConfig {
                name: context.config.output_name.clone(),
                version: "1.0.0".into(),
                description: None,
                author: None,
                copyright: None,
                icon: context.config.icon_path.clone(),
                identifier: None,
            },
            target: TargetConfig {
                platform: "windows".into(),
                format: Some("exe".into()),
                output_dir: context.config.output_dir.clone(),
                output_name: Some(context.config.output_name.clone()),
            },
            window: Default::default(),
            frontend,
            backend: None, // Backend config is passed via PackConfig.mode
            extensions: Default::default(),
            platform: Default::default(),
            debug: Default::default(),
        };

        let mut build_ctx = BuildContext::new(build_config, context.config.output_dir.clone());

        // Copy collected assets from PackContext to BuildContext
        // This ensures hooks.collect assets are included in the final bundle
        for (path, content) in &context.assets {
            build_ctx.add_asset(path.clone(), content.clone());
        }

        // Store original PackConfig in context for overlay builder to use
        build_ctx.set_metadata("pack_config", &context.config)?;

        let builder = WinBuilder::new();
        let output = builder.build(&mut build_ctx)?;

        let python_files = if matches!(context.config.mode, PackMode::FullStack { .. }) {
            // Count would be determined during actual Python bundling
            1 // Placeholder - indicates FullStack mode
        } else {
            0
        };

        Ok(PackOutput::new(output.path, &output.format, context.target)
            .with_size(output.size)
            .with_assets(output.asset_count)
            .with_python_files(python_files))
    }
}

/// Alias for backward compatibility
pub type StaticPacker = DesktopPacker;

/// Windows desktop target packer
#[cfg(target_os = "windows")]
pub struct WindowsTargetPacker;

#[cfg(target_os = "windows")]
impl WindowsTargetPacker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsTargetPacker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl TargetPacker for WindowsTargetPacker {
    fn target(&self) -> PackTarget {
        PackTarget::Windows
    }

    fn is_available(&self) -> bool {
        true
    }

    fn required_tools(&self) -> Vec<&'static str> {
        vec![]
    }

    fn check_tools(&self) -> PackResult<()> {
        Ok(())
    }

    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput> {
        let static_packer = StaticPacker::new();
        static_packer.pack(context)
    }
}

/// macOS desktop target packer
#[cfg(target_os = "macos")]
pub struct MacOSTargetPacker;

#[cfg(target_os = "macos")]
impl MacOSTargetPacker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
impl Default for MacOSTargetPacker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
impl TargetPacker for MacOSTargetPacker {
    fn target(&self) -> PackTarget {
        PackTarget::MacOS
    }

    fn is_available(&self) -> bool {
        true
    }

    fn required_tools(&self) -> Vec<&'static str> {
        vec!["codesign"]
    }

    fn check_tools(&self) -> PackResult<()> {
        std::process::Command::new("which")
            .arg("codesign")
            .status()
            .map_err(|_| PackError::Config("codesign not found".to_string()))?;
        Ok(())
    }

    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput> {
        let static_packer = StaticPacker::new();
        static_packer.pack(context)
    }
}

/// Linux desktop target packer
#[cfg(target_os = "linux")]
pub struct LinuxTargetPacker;

#[cfg(target_os = "linux")]
impl LinuxTargetPacker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "linux")]
impl Default for LinuxTargetPacker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl TargetPacker for LinuxTargetPacker {
    fn target(&self) -> PackTarget {
        PackTarget::Linux
    }

    fn is_available(&self) -> bool {
        true
    }

    fn required_tools(&self) -> Vec<&'static str> {
        vec![]
    }

    fn check_tools(&self) -> PackResult<()> {
        Ok(())
    }

    fn pack(&self, context: &mut PackContext) -> PackResult<PackOutput> {
        let static_packer = StaticPacker::new();
        static_packer.pack(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_packer_can_pack() {
        let packer = DesktopPacker::new();

        let url_config = PackConfig::url("https://example.com");
        assert!(packer.can_pack(&url_config));

        let frontend_config = PackConfig::frontend(std::path::PathBuf::from("./dist"));
        assert!(packer.can_pack(&frontend_config));

        let fullstack_config =
            PackConfig::fullstack(std::path::PathBuf::from("./dist"), "main:run");
        assert!(packer.can_pack(&fullstack_config));
    }
}
