//! AuroraView Pack - Zero-Dependency Standalone Executable Packaging
//!
//! This crate provides functionality to package AuroraView-based applications
//! into standalone executables **without requiring any build tools**.
//!
//! # Design Philosophy
//!
//! Unlike traditional packaging tools that generate source code and require
//! compilation, AuroraView Pack uses a **self-replicating approach**:
//!
//! 1. The `auroraview` CLI itself is a fully functional WebView shell
//! 2. During `pack`, it copies itself and appends configuration + assets as overlay data
//! 3. On startup, the packed exe detects the overlay and runs as a standalone app
//!
//! This means users only need the `auroraview` binary - no Rust, Cargo, or any
//! other build tools required!
//!
//! # Features
//!
//! - **URL Mode**: Wrap any website into a desktop app
//! - **Frontend Mode**: Bundle local HTML/CSS/JS into a standalone app
//! - **Manifest Support**: Declarative configuration via `auroraview.pack.toml`
//! - **Zero Dependencies**: No build tools required on user's machine
//!
//! # Quick Start
//!
//! ## Command Line Usage
//!
//! ```bash
//! # Wrap a website
//! auroraview pack --url www.example.com --output my-app
//!
//! # Bundle local frontend
//! auroraview pack --frontend ./dist --output my-app
//!
//! # Use manifest file
//! auroraview pack --config auroraview.pack.toml
//! ```
//!
//! ## Manifest File (auroraview.pack.toml)
//!
//! ```toml
//! [package]
//! name = "my-app"
//! version = "1.0.0"
//!
//! [app]
//! title = "My Application"
//! url = "https://example.com"
//! # OR
//! # frontend_path = "./dist"
//!
//! [window]
//! width = 1280
//! height = 720
//!
//! [bundle]
//! icon = "./assets/icon.png"
//! ```
//!
//! # Technical Details
//!
//! ## Overlay Format
//!
//! The packed executable contains:
//! ```text
//! [Original auroraview.exe]
//! [Overlay Data]
//!   - Magic: "AVPK" (4 bytes)
//!   - Version: u32 (4 bytes)
//!   - Config Length: u64 (8 bytes)
//!   - Assets Length: u64 (8 bytes)
//!   - Config JSON (compressed)
//!   - Assets Archive (tar.zstd)
//! [Footer]
//!   - Overlay Offset: u64 (8 bytes)
//!   - Magic: "AVPK" (4 bytes)
//! ```

mod bundle;
mod config;
mod error;
mod license;
mod manifest;
mod overlay;
mod packer;
mod pyoxidizer;
mod python_standalone;

// Re-export public API
pub use bundle::{AssetBundle, BundleBuilder};
pub use config::{
    BundleStrategy, CollectPattern, HooksConfig, LicenseConfig, PackConfig, PackMode,
    PythonBundleConfig, TargetPlatform, WindowConfig, WindowStartPosition,
};
pub use error::{PackError, PackResult};
pub use license::{get_machine_id, LicenseReason, LicenseStatus, LicenseValidator};
pub use pyoxidizer::{
    check_pyoxidizer, installation_instructions, DistributionFlavor, ExternalBinary,
    PyOxidizerBuilder, PyOxidizerConfig, ResourceFile,
};
pub use python_standalone::{
    extract_runtime, get_runtime_cache_dir, PythonRuntimeMeta, PythonStandalone,
    PythonStandaloneConfig, PythonTarget,
};
pub use manifest::{
    AppConfig, BuildConfig, BundleConfig, CollectEntry, DebugConfig, HooksManifestConfig,
    InjectConfig, LicenseManifestConfig, LinuxBundleConfig, MacOSBundleConfig, Manifest,
    PackageConfig, PythonConfig, PyOxidizerManifestConfig, RuntimeConfig, StartPosition,
    WindowConfig as ManifestWindowConfig, WindowsBundleConfig,
};
pub use overlay::{OverlayData, OverlayReader, OverlayWriter, OVERLAY_MAGIC, OVERLAY_VERSION};
pub use packer::Packer;

/// Alias for backward compatibility with CLI
pub type PackGenerator = Packer;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if the current executable has overlay data (is a packed app)
pub fn is_packed() -> bool {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    OverlayReader::has_overlay(&exe_path).unwrap_or(false)
}

/// Read overlay data from the current executable
pub fn read_overlay() -> PackResult<Option<OverlayData>> {
    let exe_path = std::env::current_exe()?;
    OverlayReader::read(&exe_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(VERSION.contains('.'), "VERSION should contain a dot");
    }

    #[test]
    fn test_is_packed() {
        // In test environment, should not be packed
        assert!(!is_packed());
    }
}
