//! Builder trait definitions

use super::common::{BuildContext, BuildOutput, BuildResult};

/// Core builder trait for platform-specific build logic
pub trait Builder: Send + Sync {
    /// Unique builder identifier (e.g., "win", "ios", "wechat")
    fn id(&self) -> &'static str;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Supported target identifiers
    fn targets(&self) -> &'static [&'static str];

    /// Builder capabilities
    fn capabilities(&self) -> Vec<BuilderCapability>;

    /// Check if builder is available on current system
    fn is_available(&self) -> bool;

    /// Get required external tools
    fn required_tools(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Check if all required tools are installed
    fn check_tools(&self) -> BuildResult<()> {
        Ok(())
    }

    /// Validate build context before building
    fn validate(&self, ctx: &BuildContext) -> BuildResult<()>;

    /// Execute the build
    fn build(&self, ctx: &mut BuildContext) -> BuildResult<BuildOutput>;

    /// Clean up temporary files
    fn cleanup(&self, ctx: &BuildContext) -> BuildResult<()> {
        let _ = ctx;
        Ok(())
    }
}

/// Builder capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderCapability {
    /// Can build standalone executables
    Standalone,
    /// Can build installer packages
    Installer,
    /// Can build portable/zip distributions
    Portable,
    /// Supports code signing
    CodeSign,
    /// Supports notarization (macOS)
    Notarize,
    /// Can embed Python runtime
    PythonEmbed,
    /// Can embed Node.js runtime
    NodeEmbed,
    /// Supports Chrome extensions
    Extensions,
    /// Supports DevTools
    DevTools,
    /// Can build for app stores
    AppStore,
    /// Supports hot reload
    HotReload,
}

impl BuilderCapability {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Standalone => "Standalone",
            Self::Installer => "Installer",
            Self::Portable => "Portable",
            Self::CodeSign => "Code Signing",
            Self::Notarize => "Notarization",
            Self::PythonEmbed => "Python Embed",
            Self::NodeEmbed => "Node.js Embed",
            Self::Extensions => "Extensions",
            Self::DevTools => "DevTools",
            Self::AppStore => "App Store",
            Self::HotReload => "Hot Reload",
        }
    }
}

/// Output format for a builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Windows executable (.exe)
    WindowsExe,
    /// Windows MSIX package
    WindowsMsix,
    /// macOS application bundle (.app)
    MacApp,
    /// macOS disk image (.dmg)
    MacDmg,
    /// macOS installer package (.pkg)
    MacPkg,
    /// Linux AppImage
    LinuxAppImage,
    /// Debian package (.deb)
    LinuxDeb,
    /// RPM package (.rpm)
    LinuxRpm,
    /// iOS app archive (.ipa)
    IosIpa,
    /// Android APK
    AndroidApk,
    /// Android App Bundle (.aab)
    AndroidAab,
    /// Web static files
    WebStatic,
    /// Progressive Web App
    WebPwa,
    /// MiniProgram package
    MiniProgram,
}

impl OutputFormat {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::WindowsExe => "exe",
            Self::WindowsMsix => "msix",
            Self::MacApp => "app",
            Self::MacDmg => "dmg",
            Self::MacPkg => "pkg",
            Self::LinuxAppImage => "AppImage",
            Self::LinuxDeb => "deb",
            Self::LinuxRpm => "rpm",
            Self::IosIpa => "ipa",
            Self::AndroidApk => "apk",
            Self::AndroidAab => "aab",
            Self::WebStatic => "",
            Self::WebPwa => "",
            Self::MiniProgram => "",
        }
    }
}
