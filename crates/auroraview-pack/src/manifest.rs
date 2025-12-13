//! Manifest file parser for AuroraView Pack
//!
//! This module provides support for `auroraview.pack.toml` manifest files,
//! enabling declarative configuration of packaging options.
//!
//! ## Manifest Format
//!
//! ```toml
//! [package]
//! name = "my-app"
//! version = "1.0.0"
//! description = "My awesome application"
//! authors = ["Author Name <email@example.com>"]
//!
//! [app]
//! title = "My Application"
//! url = "https://example.com"  # or frontend_path = "./dist"
//!
//! [window]
//! width = 1024
//! height = 768
//! min_width = 800
//! min_height = 600
//! resizable = true
//! frameless = false
//! transparent = false
//! always_on_top = false
//! start_position = "center"  # or { x = 100, y = 100 }
//!
//! [bundle]
//! icon = "./assets/icon.png"
//! identifier = "com.example.myapp"
//! copyright = "Copyright © 2025"
//! category = "Utility"
//!
//! [bundle.windows]
//! icon = "./assets/icon.ico"
//!
//! [bundle.macos]
//! icon = "./assets/icon.icns"
//!
//! [python]
//! enabled = true
//! version = "3.11"
//! entry_point = "myapp.main:run"
//! packages = ["auroraview", "requests"]
//! requirements = "./requirements.txt"
//! include_paths = ["./python", "./lib"]
//!
//! [build]
//! before_build = ["npm run build", "python scripts/prepare.py"]
//! after_build = ["python scripts/sign.py"]
//! resources = ["./assets", "./data"]
//! exclude = ["*.pyc", "__pycache__", ".git"]
//!
//! [inject]
//! js = "./inject.js"
//! css = "./inject.css"
//!
//! [debug]
//! enabled = false
//! devtools = false
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{PackError, PackResult};

/// Root manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Package metadata
    pub package: PackageConfig,

    /// Application configuration
    pub app: AppConfig,

    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,

    /// Bundle configuration (icons, identifiers, etc.)
    #[serde(default)]
    pub bundle: BundleConfig,

    /// Python embedding configuration
    #[serde(default)]
    pub python: Option<PythonConfig>,

    /// Build hooks and resources
    #[serde(default)]
    pub build: BuildConfig,

    /// JavaScript/CSS injection
    #[serde(default)]
    pub inject: Option<InjectConfig>,

    /// Debug settings
    #[serde(default)]
    pub debug: DebugConfig,

    /// License/authorization settings
    #[serde(default)]
    pub license: Option<LicenseManifestConfig>,

    /// Runtime environment configuration
    #[serde(default)]
    pub runtime: Option<RuntimeConfig>,

    /// Hooks for collecting additional files
    #[serde(default)]
    pub hooks: Option<HooksManifestConfig>,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package name (used for executable name)
    pub name: String,

    /// Package version
    #[serde(default = "default_version")]
    pub version: String,

    /// Package description
    #[serde(default)]
    pub description: Option<String>,

    /// Package authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// License
    #[serde(default)]
    pub license: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window title
    pub title: String,

    /// URL to load (mutually exclusive with frontend_path)
    #[serde(default)]
    pub url: Option<String>,

    /// Path to frontend assets (mutually exclusive with url)
    #[serde(default)]
    pub frontend_path: Option<PathBuf>,

    /// Backend entry point for fullstack mode (e.g., "myapp.main:run")
    #[serde(default)]
    pub backend_entry: Option<String>,

    /// Custom user agent
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Allow opening new windows
    #[serde(default)]
    pub allow_new_window: bool,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width
    #[serde(default = "default_width")]
    pub width: u32,

    /// Window height
    #[serde(default = "default_height")]
    pub height: u32,

    /// Minimum window width
    #[serde(default)]
    pub min_width: Option<u32>,

    /// Minimum window height
    #[serde(default)]
    pub min_height: Option<u32>,

    /// Maximum window width
    #[serde(default)]
    pub max_width: Option<u32>,

    /// Maximum window height
    #[serde(default)]
    pub max_height: Option<u32>,

    /// Window is resizable
    #[serde(default = "default_true")]
    pub resizable: bool,

    /// Window has no frame/decorations
    #[serde(default)]
    pub frameless: bool,

    /// Window background is transparent
    #[serde(default)]
    pub transparent: bool,

    /// Window stays on top
    #[serde(default)]
    pub always_on_top: bool,

    /// Start position: "center" or { x, y }
    #[serde(default)]
    pub start_position: StartPosition,

    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,

    /// Maximized on start
    #[serde(default)]
    pub maximized: bool,

    /// Visible on start
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_width() -> u32 {
    1024
}
fn default_height() -> u32 {
    768
}
fn default_true() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            resizable: true,
            frameless: false,
            transparent: false,
            always_on_top: false,
            start_position: StartPosition::default(),
            fullscreen: false,
            maximized: false,
            visible: true,
        }
    }
}

/// Window start position
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StartPosition {
    /// Specific position
    Position { x: i32, y: i32 },
    /// Named position (center, etc.)
    Named(String),
}

impl Default for StartPosition {
    fn default() -> Self {
        StartPosition::Named("center".to_string())
    }
}

impl StartPosition {
    /// Check if this is the center position
    pub fn is_center(&self) -> bool {
        matches!(self, StartPosition::Named(s) if s == "center")
    }
}

/// Bundle configuration for platform-specific packaging
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleConfig {
    /// Default icon path (PNG format, will be converted)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Application identifier (e.g., "com.example.myapp")
    #[serde(default)]
    pub identifier: Option<String>,

    /// Copyright string
    #[serde(default)]
    pub copyright: Option<String>,

    /// Application category
    #[serde(default)]
    pub category: Option<String>,

    /// Short description (for Windows)
    #[serde(default)]
    pub short_description: Option<String>,

    /// Long description
    #[serde(default)]
    pub long_description: Option<String>,

    /// Windows-specific bundle config
    #[serde(default)]
    pub windows: Option<WindowsBundleConfig>,

    /// macOS-specific bundle config
    #[serde(default)]
    pub macos: Option<MacOSBundleConfig>,

    /// Linux-specific bundle config
    #[serde(default)]
    pub linux: Option<LinuxBundleConfig>,

    /// External binaries to bundle
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,

    /// Additional resources to bundle
    #[serde(default)]
    pub resources: Vec<PathBuf>,
}

/// Windows-specific bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowsBundleConfig {
    /// Windows icon (.ico)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Windows installer type: "msi", "nsis", or "wix"
    #[serde(default)]
    pub installer: Option<String>,

    /// Code signing certificate
    #[serde(default)]
    pub certificate: Option<PathBuf>,

    /// Certificate password (or env var name)
    #[serde(default)]
    pub certificate_password: Option<String>,

    /// Timestamp server URL
    #[serde(default)]
    pub timestamp_url: Option<String>,

    /// File version (for Windows resources)
    #[serde(default)]
    pub file_version: Option<String>,

    /// Product version (for Windows resources)
    #[serde(default)]
    pub product_version: Option<String>,
}

/// macOS-specific bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacOSBundleConfig {
    /// macOS icon (.icns)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Bundle identifier
    #[serde(default)]
    pub bundle_identifier: Option<String>,

    /// Minimum macOS version
    #[serde(default)]
    pub minimum_system_version: Option<String>,

    /// Code signing identity
    #[serde(default)]
    pub signing_identity: Option<String>,

    /// Notarization credentials
    #[serde(default)]
    pub notarization: Option<MacOSNotarization>,

    /// Entitlements file
    #[serde(default)]
    pub entitlements: Option<PathBuf>,

    /// Create DMG installer
    #[serde(default)]
    pub dmg: bool,
}

/// macOS notarization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacOSNotarization {
    /// Apple ID
    pub apple_id: Option<String>,
    /// Team ID
    pub team_id: Option<String>,
    /// App-specific password (or env var name)
    pub password: Option<String>,
}

/// Linux-specific bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinuxBundleConfig {
    /// Linux icon (PNG or SVG)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Desktop file categories
    #[serde(default)]
    pub categories: Vec<String>,

    /// Create AppImage
    #[serde(default)]
    pub appimage: bool,

    /// Create Debian package
    #[serde(default)]
    pub deb: bool,

    /// Create RPM package
    #[serde(default)]
    pub rpm: bool,
}

/// Python embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Enable Python embedding
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Python version to embed
    #[serde(default = "default_python_version")]
    pub version: String,

    /// Entry point (e.g., "myapp.main:run")
    #[serde(default)]
    pub entry_point: Option<String>,

    /// Pip packages to include
    #[serde(default)]
    pub packages: Vec<String>,

    /// Path to requirements.txt
    #[serde(default)]
    pub requirements: Option<PathBuf>,

    /// Additional Python paths to include
    #[serde(default)]
    pub include_paths: Vec<PathBuf>,

    /// Exclude patterns for Python files
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Include pip in the bundle
    #[serde(default)]
    pub include_pip: bool,

    /// Include setuptools in the bundle
    #[serde(default)]
    pub include_setuptools: bool,

    /// Bytecode optimization level (0, 1, or 2)
    #[serde(default = "default_optimize")]
    pub optimize: u8,

    /// Environment variables to set
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Bundle strategy: "pyoxidizer", "embedded", "portable", or "system"
    #[serde(default = "default_strategy")]
    pub strategy: String,

    /// External binaries to bundle
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,

    /// Additional resource files/directories
    #[serde(default)]
    pub resources: Vec<PathBuf>,

    /// PyOxidizer-specific configuration
    #[serde(default)]
    pub pyoxidizer: Option<PyOxidizerManifestConfig>,
}

fn default_strategy() -> String {
    "pyoxidizer".to_string()
}

/// PyOxidizer-specific manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PyOxidizerManifestConfig {
    /// Path to PyOxidizer executable
    #[serde(default)]
    pub executable: Option<PathBuf>,

    /// Target triple (e.g., "x86_64-pc-windows-msvc")
    #[serde(default)]
    pub target: Option<String>,

    /// Distribution flavor: "standalone", "standalone_dynamic", or "system"
    #[serde(default)]
    pub flavor: Option<String>,

    /// Build in release mode
    #[serde(default = "default_true")]
    pub release: bool,

    /// Enable filesystem importer fallback
    #[serde(default)]
    pub filesystem_importer: bool,

    /// Additional PyOxidizer config options
    #[serde(default)]
    pub extra_config: HashMap<String, String>,
}

fn default_python_version() -> String {
    "3.10".to_string()
}

fn default_optimize() -> u8 {
    1
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            version: default_python_version(),
            entry_point: None,
            packages: Vec::new(),
            requirements: None,
            include_paths: Vec::new(),
            exclude: Vec::new(),
            include_pip: false,
            include_setuptools: false,
            optimize: default_optimize(),
            env: HashMap::new(),
            strategy: default_strategy(),
            external_bin: Vec::new(),
            resources: Vec::new(),
            pyoxidizer: None,
        }
    }
}

/// Build hooks and resource configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// Commands to run before build
    #[serde(default)]
    pub before_build: Vec<String>,

    /// Commands to run after build
    #[serde(default)]
    pub after_build: Vec<String>,

    /// Commands to run before bundling
    #[serde(default)]
    pub before_bundle: Vec<String>,

    /// Commands to run after bundling
    #[serde(default)]
    pub after_bundle: Vec<String>,

    /// Additional resources to include
    #[serde(default)]
    pub resources: Vec<PathBuf>,

    /// Patterns to exclude from resources
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Output directory
    #[serde(default)]
    pub out_dir: Option<PathBuf>,

    /// Target platforms to build for
    #[serde(default)]
    pub targets: Vec<String>,

    /// Enable release mode
    #[serde(default = "default_true")]
    pub release: bool,

    /// Features to enable
    #[serde(default)]
    pub features: Vec<String>,
}

/// JavaScript/CSS injection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InjectConfig {
    /// JavaScript file to inject
    #[serde(default)]
    pub js: Option<PathBuf>,

    /// CSS file to inject
    #[serde(default)]
    pub css: Option<PathBuf>,

    /// Inline JavaScript code
    #[serde(default)]
    pub js_code: Option<String>,

    /// Inline CSS code
    #[serde(default)]
    pub css_code: Option<String>,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebugConfig {
    /// Enable debug mode
    #[serde(default)]
    pub enabled: bool,

    /// Enable DevTools
    #[serde(default)]
    pub devtools: bool,

    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
}

/// License/authorization manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseManifestConfig {
    /// Whether license validation is enabled
    #[serde(default)]
    pub enabled: bool,

    /// License expiration date (ISO 8601 format: YYYY-MM-DD)
    #[serde(default)]
    pub expires_at: Option<String>,

    /// Whether a token is required to run
    #[serde(default)]
    pub require_token: bool,

    /// Pre-embedded token (for pre-authorized builds)
    #[serde(default)]
    pub embedded_token: Option<String>,

    /// Token validation URL (for online validation)
    #[serde(default)]
    pub validation_url: Option<String>,

    /// Allowed machine IDs (for hardware binding)
    #[serde(default)]
    pub allowed_machines: Vec<String>,

    /// Grace period in days after expiration
    #[serde(default)]
    pub grace_period_days: u32,

    /// Custom expiration message
    #[serde(default)]
    pub expiration_message: Option<String>,
}

/// Runtime environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    /// Environment variables to inject at runtime
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from files (key = var name, value = file path)
    #[serde(default)]
    pub env_files: Vec<PathBuf>,

    /// Working directory override
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
}

/// Hooks configuration for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksManifestConfig {
    /// Commands to run before collecting files
    #[serde(default)]
    pub before_collect: Vec<String>,

    /// Additional file patterns to collect
    #[serde(default)]
    pub collect: Vec<CollectEntry>,

    /// Commands to run after packing
    #[serde(default)]
    pub after_pack: Vec<String>,
}

/// Entry for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectEntry {
    /// Source path or glob pattern
    pub source: String,

    /// Destination path in the bundle (relative to assets root)
    #[serde(default)]
    pub dest: Option<String>,

    /// Whether to preserve directory structure
    #[serde(default = "default_true")]
    pub preserve_structure: bool,

    /// Optional description for this collection
    #[serde(default)]
    pub description: Option<String>,
}

impl Manifest {
    /// Load manifest from a file
    pub fn from_file(path: impl AsRef<Path>) -> PackResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            PackError::Config(format!(
                "Failed to read manifest file {}: {}",
                path.display(),
                e
            ))
        })?;
        Self::parse(&content)
    }

    /// Parse manifest from TOML string
    pub fn parse(content: &str) -> PackResult<Self> {
        toml::from_str(content)
            .map_err(|e| PackError::Config(format!("Failed to parse manifest: {}", e)))
    }

    /// Find manifest file in directory (looks for auroraview.pack.toml or pack.toml)
    pub fn find_in_dir(dir: impl AsRef<Path>) -> Option<PathBuf> {
        let dir = dir.as_ref();
        let candidates = [
            "auroraview.pack.toml",
            "pack.toml",
            "auroraview.toml",
            ".auroraview/pack.toml",
        ];

        for name in candidates {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Validate the manifest configuration
    pub fn validate(&self) -> PackResult<()> {
        // Check app configuration
        if self.app.url.is_none() && self.app.frontend_path.is_none() {
            return Err(PackError::Config(
                "Either 'url' or 'frontend_path' must be specified in [app]".to_string(),
            ));
        }

        if self.app.url.is_some() && self.app.frontend_path.is_some() {
            return Err(PackError::Config(
                "'url' and 'frontend_path' are mutually exclusive in [app]".to_string(),
            ));
        }

        // Validate Python config if enabled
        if let Some(ref python) = self.python {
            if python.enabled {
                // Validate version format
                if !python
                    .version
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.')
                {
                    return Err(PackError::Config(format!(
                        "Invalid Python version format: {}",
                        python.version
                    )));
                }

                // Validate optimize level
                if python.optimize > 2 {
                    return Err(PackError::Config(
                        "Python optimize level must be 0, 1, or 2".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get the effective icon path for the current platform
    pub fn get_icon_path(&self) -> Option<&PathBuf> {
        #[cfg(target_os = "windows")]
        {
            self.bundle
                .windows
                .as_ref()
                .and_then(|w| w.icon.as_ref())
                .or(self.bundle.icon.as_ref())
        }
        #[cfg(target_os = "macos")]
        {
            self.bundle
                .macos
                .as_ref()
                .and_then(|m| m.icon.as_ref())
                .or(self.bundle.icon.as_ref())
        }
        #[cfg(target_os = "linux")]
        {
            self.bundle
                .linux
                .as_ref()
                .and_then(|l| l.icon.as_ref())
                .or(self.bundle.icon.as_ref())
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            self.bundle.icon.as_ref()
        }
    }

    /// Check if this is a fullstack (Python + frontend) configuration
    pub fn is_fullstack(&self) -> bool {
        self.python.as_ref().map(|p| p.enabled).unwrap_or(false) && self.app.frontend_path.is_some()
    }

    /// Check if this is a URL-only configuration
    pub fn is_url_mode(&self) -> bool {
        self.app.url.is_some()
    }

    /// Check if this is a frontend-only configuration
    pub fn is_frontend_mode(&self) -> bool {
        self.app.frontend_path.is_some() && !self.is_fullstack()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[package]
name = "test-app"

[app]
title = "Test App"
url = "https://example.com"
"#;
        let manifest = Manifest::parse(toml).unwrap();
        assert_eq!(manifest.package.name, "test-app");
        assert_eq!(manifest.app.title, "Test App");
        assert_eq!(manifest.app.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[package]
name = "my-app"
version = "1.0.0"
description = "My awesome app"
authors = ["Test Author"]

[app]
title = "My Application"
frontend_path = "./dist"
backend_entry = "myapp.main:run"

[window]
width = 1280
height = 720
resizable = true
frameless = false

[bundle]
icon = "./assets/icon.png"
identifier = "com.example.myapp"

[bundle.windows]
icon = "./assets/icon.ico"

[python]
enabled = true
version = "3.11"
entry_point = "myapp.main:run"
packages = ["auroraview", "requests"]

[build]
before_build = ["npm run build"]
after_build = ["echo done"]

[debug]
enabled = true
devtools = true
"#;
        let manifest = Manifest::parse(toml).unwrap();
        assert_eq!(manifest.package.name, "my-app");
        assert_eq!(manifest.package.version, "1.0.0");
        assert!(manifest.python.is_some());
        assert!(manifest.is_fullstack());
    }

    #[test]
    fn test_validate_manifest() {
        // Missing both url and frontend_path
        let toml = r#"
[package]
name = "test"

[app]
title = "Test"
"#;
        let manifest = Manifest::parse(toml).unwrap();
        assert!(manifest.validate().is_err());

        // Both url and frontend_path specified
        let toml = r#"
[package]
name = "test"

[app]
title = "Test"
url = "https://example.com"
frontend_path = "./dist"
"#;
        let manifest = Manifest::parse(toml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_start_position_parsing() {
        // Center position
        let toml = r#"
[package]
name = "test"

[app]
title = "Test"
url = "https://example.com"

[window]
start_position = "center"
"#;
        let manifest = Manifest::parse(toml).unwrap();
        assert!(manifest.window.start_position.is_center());

        // Specific position
        let toml = r#"
[package]
name = "test"

[app]
title = "Test"
url = "https://example.com"

[window]
start_position = { x = 100, y = 200 }
"#;
        let manifest = Manifest::parse(toml).unwrap();
        if let StartPosition::Position { x, y } = manifest.window.start_position {
            assert_eq!(x, 100);
            assert_eq!(y, 200);
        } else {
            panic!("Expected Position variant");
        }
    }
}
