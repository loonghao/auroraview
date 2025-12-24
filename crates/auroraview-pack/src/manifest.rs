//! Manifest file parser for AuroraView Pack
//!
//! This module provides support for `auroraview.pack.toml` manifest files,
//! enabling declarative configuration of packaging options.
//!
//! ## Configuration Hierarchy
//!
//! ```toml
//! [package]                    # Package metadata & identity
//! name = "my-app"
//! version = "1.0.0"
//! title = "My App"
//! identifier = "com.example.app"
//!
//! [frontend]                   # Frontend configuration
//! path = "./dist"              # Local frontend assets
//! # url = "https://example.com" # OR remote URL (mutually exclusive)
//!
//! [backend]                    # Backend abstraction layer (optional)
//! type = "python"              # "python" | "go" | "rust" | "node" | "none"
//!
//! [backend.python]             # Python-specific config (when type = "python")
//! version = "3.11"
//! entry_point = "main:run"
//! packages = ["flask"]
//!
//! [backend.go]                 # Go-specific config (when type = "go")
//! module = "github.com/user/app"
//! entry_point = "./cmd/server"
//!
//! [backend.rust]               # Rust-specific config (when type = "rust")
//! manifest = "./backend/Cargo.toml"
//! binary = "server"
//!
//! [backend.node]               # Node.js-specific config (when type = "node")
//! version = "20"
//! entry_point = "./server/index.js"
//!
//! [backend.process]            # Common process settings (all backend types)
//! args = []
//! env = {}
//! health_check = { url = "http://localhost:8080/health", timeout = 30 }
//!
//! [window]                     # Runtime window behavior
//! width = 1280
//! height = 720
//!
//! [bundle]                     # General bundling settings
//! icon = "./assets/icon.png"
//! copyright = "Copyright 2025"
//!
//! [bundle.windows]             # Windows-specific
//! icon = "./assets/icon.ico"
//! console = false
//!
//! [bundle.macos]               # macOS-specific
//! icon = "./assets/icon.icns"
//!
//! [bundle.linux]               # Linux-specific
//! categories = ["Development"]
//!
//! [build]                      # Build hooks
//! before = ["npm run build"]
//!
//! [hooks]                      # File collection
//! [[hooks.collect]]
//! source = "./examples/*.py"
//! dest = "resources/examples"
//!
//! [runtime]                    # Runtime environment
//! [runtime.env]
//! APP_ENV = "production"
//!
//! [debug]                      # Debug settings
//! enabled = false
//!
//! [license]                    # License validation
//! enabled = false
//!
//! [inject]                     # JS/CSS injection
//! js_code = "console.log('hello');"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crate::common::{
    default_module_search_paths, default_optimize, default_python_version, BundleStrategy,
    CollectPattern, DebugConfig, HooksConfig, IsolationConfig, LicenseConfig, LinuxPlatformConfig,
    MacOSPlatformConfig, ProcessConfig, PyOxidizerConfig, RuntimeConfig, WindowConfig,
    WindowStartPosition, WindowsPlatformConfig,
};
use crate::config::PythonBundleConfig;
use crate::error::{PackError, PackResult};

// Re-export common types for convenience
pub use crate::common::InjectConfig;

/// Normalize a path by removing `.` and resolving `..` components
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {} // Skip `.`
            Component::ParentDir => {
                // Pop the last component if it's a normal component
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

// ============================================================================
// Root Manifest Structure
// ============================================================================

/// Root manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata
    pub package: PackageConfig,

    /// Frontend configuration
    #[serde(default)]
    pub frontend: Option<FrontendConfig>,

    /// Backend configuration (abstraction layer for multiple backend types)
    #[serde(default)]
    pub backend: Option<BackendConfig>,

    /// Window configuration (runtime behavior)
    #[serde(default)]
    pub window: ManifestWindowConfig,

    /// Bundle configuration (icons, identifiers, platform configs)
    #[serde(default)]
    pub bundle: BundleConfig,

    /// Build hooks and resources
    #[serde(default)]
    pub build: BuildConfig,

    /// File collection hooks
    #[serde(default)]
    pub hooks: Option<HooksManifestConfig>,

    /// Runtime environment configuration
    #[serde(default)]
    pub runtime: Option<RuntimeConfig>,

    /// Debug settings
    #[serde(default)]
    pub debug: DebugConfig,

    /// License/authorization settings
    #[serde(default)]
    pub license: Option<LicenseConfig>,

    /// JavaScript/CSS injection
    #[serde(default)]
    pub inject: Option<InjectConfig>,
}

// ============================================================================
// Package Configuration
// ============================================================================

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package name (used for executable name)
    pub name: String,

    /// Package version
    #[serde(default = "default_version")]
    pub version: String,

    /// Window title
    #[serde(default)]
    pub title: Option<String>,

    /// Application identifier (e.g., "com.example.myapp")
    #[serde(default)]
    pub identifier: Option<String>,

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

    /// Custom user agent
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Allow opening new windows
    #[serde(default)]
    pub allow_new_window: bool,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

// ============================================================================
// Frontend Configuration
// ============================================================================

/// Frontend configuration
///
/// Specifies where to load frontend content from.
/// Either `path` (local) or `url` (remote) must be specified, but not both.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    /// Path to local frontend assets (directory or HTML file)
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Remote URL to load (mutually exclusive with path)
    #[serde(default)]
    pub url: Option<String>,
}

// ============================================================================
// Backend Configuration (Multi-language abstraction)
// ============================================================================

/// Backend type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    /// No backend (frontend-only mode)
    #[default]
    None,
    /// Python backend
    Python,
    /// Go backend
    Go,
    /// Rust backend
    Rust,
    /// Node.js backend
    Node,
}

impl BackendType {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "python" => BackendType::Python,
            "go" | "golang" => BackendType::Go,
            "rust" => BackendType::Rust,
            "node" | "nodejs" | "node.js" => BackendType::Node,
            "none" | "" => BackendType::None,
            _ => BackendType::None,
        }
    }
}

/// Backend configuration (abstraction layer for multiple backend types)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendConfig {
    /// Backend type: "python" | "go" | "rust" | "node" | "none"
    #[serde(default, rename = "type")]
    pub backend_type: BackendType,

    /// Python-specific configuration
    #[serde(default)]
    pub python: Option<BackendPythonConfig>,

    /// Go-specific configuration
    #[serde(default)]
    pub go: Option<BackendGoConfig>,

    /// Rust-specific configuration
    #[serde(default)]
    pub rust: Option<BackendRustConfig>,

    /// Node.js-specific configuration
    #[serde(default)]
    pub node: Option<BackendNodeConfig>,

    /// Common process configuration (applies to all backend types)
    #[serde(default)]
    pub process: Option<BackendProcessConfig>,
}

/// Python backend configuration (under [backend.python])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendPythonConfig {
    /// Python version to embed (e.g., "3.11", "3.12")
    #[serde(default = "default_python_version")]
    pub version: String,

    /// Entry point (e.g., "myapp.main:run" or "main.py")
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

    /// Bundle strategy: "standalone", "pyoxidizer", "embedded", "portable", "system"
    #[serde(default = "default_strategy")]
    pub strategy: String,

    /// Bytecode optimization level (0, 1, or 2)
    #[serde(default = "default_optimize")]
    pub optimize: u8,

    /// Include pip in the bundle
    #[serde(default)]
    pub include_pip: bool,

    /// Include setuptools in the bundle
    #[serde(default)]
    pub include_setuptools: bool,

    /// External binaries to bundle
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,

    /// Additional resource files/directories
    #[serde(default)]
    pub resources: Vec<PathBuf>,

    /// Environment variables to set for Python
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Python process configuration
    #[serde(default)]
    pub process: ProcessManifestConfig,

    /// Environment isolation configuration
    #[serde(default)]
    pub isolation: Option<IsolationManifestConfig>,

    /// PyOxidizer-specific configuration
    #[serde(default)]
    pub pyoxidizer: Option<PyOxidizerManifestConfig>,

    /// Code protection configuration
    #[serde(default)]
    pub protection: Option<ProtectionManifestConfig>,
}

impl Default for BackendPythonConfig {
    fn default() -> Self {
        Self {
            version: default_python_version(),
            entry_point: None,
            packages: Vec::new(),
            requirements: None,
            include_paths: Vec::new(),
            exclude: Vec::new(),
            strategy: default_strategy(),
            optimize: default_optimize(),
            include_pip: false,
            include_setuptools: false,
            external_bin: Vec::new(),
            resources: Vec::new(),
            env: HashMap::new(),
            process: ProcessManifestConfig::default(),
            isolation: None,
            pyoxidizer: None,
            protection: Some(ProtectionManifestConfig::default()),
        }
    }
}

impl BackendPythonConfig {
    /// Convert to PythonBundleConfig with path resolution
    pub fn to_bundle_config(&self, base_dir: &Path) -> PythonBundleConfig {
        let resolve_path = |p: &PathBuf| -> PathBuf {
            let joined = if p.is_absolute() {
                p.clone()
            } else {
                base_dir.join(p)
            };
            normalize_path(&joined)
        };

        PythonBundleConfig {
            entry_point: self
                .entry_point
                .clone()
                .unwrap_or_else(|| "main:run".to_string()),
            include_paths: self.include_paths.iter().map(resolve_path).collect(),
            packages: self.packages.clone(),
            requirements: self.requirements.as_ref().map(resolve_path),
            strategy: BundleStrategy::parse(&self.strategy),
            version: self.version.clone(),
            optimize: self.optimize,
            exclude: self.exclude.clone(),
            external_bin: self.external_bin.iter().map(resolve_path).collect(),
            resources: self.resources.iter().map(resolve_path).collect(),
            include_pip: self.include_pip,
            include_setuptools: self.include_setuptools,
            distribution_flavor: self.pyoxidizer.as_ref().and_then(|p| p.flavor.clone()),
            pyoxidizer_path: self.pyoxidizer.as_ref().and_then(|p| p.executable.clone()),
            module_search_paths: self.process.module_search_paths.clone(),
            filesystem_importer: self.process.filesystem_importer,
            show_console: self.process.console,
            isolation: self
                .isolation
                .as_ref()
                .map(|i| i.to_isolation_config())
                .unwrap_or_default(),
            protection: self
                .protection
                .as_ref()
                .map(|p| p.to_protection_config())
                .unwrap_or_default(),
        }
    }
}

/// Go backend configuration (under [backend.go])
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendGoConfig {
    /// Go module path (e.g., "github.com/user/app")
    #[serde(default)]
    pub module: Option<String>,

    /// Entry point directory (e.g., "./cmd/server")
    #[serde(default)]
    pub entry_point: Option<String>,

    /// Build flags (e.g., ["-ldflags", "-s -w"])
    #[serde(default)]
    pub build_flags: Vec<String>,

    /// Enable CGO
    #[serde(default)]
    pub cgo_enabled: bool,

    /// Go version constraint (e.g., "1.21")
    #[serde(default)]
    pub version: Option<String>,

    /// Build tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Environment variables for build
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Rust backend configuration (under [backend.rust])
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendRustConfig {
    /// Path to Cargo.toml (default: "./Cargo.toml")
    #[serde(default)]
    pub manifest: Option<PathBuf>,

    /// Binary name to build (if workspace has multiple binaries)
    #[serde(default)]
    pub binary: Option<String>,

    /// Build profile: "release" or "debug"
    #[serde(default = "default_release_profile")]
    pub profile: String,

    /// Target triple (e.g., "x86_64-pc-windows-msvc")
    #[serde(default)]
    pub target: Option<String>,

    /// Features to enable
    #[serde(default)]
    pub features: Vec<String>,

    /// Whether to use all features
    #[serde(default)]
    pub all_features: bool,

    /// Whether to disable default features
    #[serde(default)]
    pub no_default_features: bool,
}

fn default_release_profile() -> String {
    "release".to_string()
}

/// Node.js backend configuration (under [backend.node])
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendNodeConfig {
    /// Node.js version (e.g., "20", "18")
    #[serde(default)]
    pub version: Option<String>,

    /// Entry point (e.g., "./server/index.js")
    #[serde(default)]
    pub entry_point: Option<String>,

    /// Package manager: "npm", "yarn", "pnpm"
    #[serde(default = "default_package_manager")]
    pub package_manager: String,

    /// Bundle strategy: "pkg", "nexe", "sea", "portable"
    #[serde(default = "default_node_bundle_strategy")]
    pub bundle_strategy: String,

    /// Additional npm packages to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Path to package.json
    #[serde(default)]
    pub package_json: Option<PathBuf>,
}

fn default_package_manager() -> String {
    "npm".to_string()
}

fn default_node_bundle_strategy() -> String {
    "portable".to_string()
}

/// Common backend process configuration (under [backend.process])
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendProcessConfig {
    /// Command line arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Working directory
    #[serde(default)]
    pub working_dir: Option<PathBuf>,

    /// Show console window (Windows only)
    #[serde(default)]
    pub console: bool,

    /// Health check configuration
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,

    /// Restart policy on crash
    #[serde(default)]
    pub restart_on_crash: bool,

    /// Maximum restart attempts
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
}

fn default_max_restarts() -> u32 {
    3
}

/// Health check configuration for backend process
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthCheckConfig {
    /// Health check URL (e.g., "http://localhost:8080/health")
    #[serde(default)]
    pub url: Option<String>,

    /// Timeout in seconds
    #[serde(default = "default_health_timeout")]
    pub timeout: u32,

    /// Interval between checks in seconds
    #[serde(default = "default_health_interval")]
    pub interval: u32,

    /// Number of retries before considering unhealthy
    #[serde(default = "default_health_retries")]
    pub retries: u32,
}

fn default_health_timeout() -> u32 {
    30
}

fn default_health_interval() -> u32 {
    5
}

fn default_health_retries() -> u32 {
    3
}

// ============================================================================
// Window Configuration (Manifest-specific with string position)
// ============================================================================

/// Window configuration for manifest (supports string position like "center")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWindowConfig {
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
    1280
}

fn default_height() -> u32 {
    720
}

fn default_true() -> bool {
    true
}

impl Default for ManifestWindowConfig {
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

impl From<ManifestWindowConfig> for WindowConfig {
    fn from(manifest: ManifestWindowConfig) -> Self {
        Self {
            title: "AuroraView App".to_string(), // Default title, will be overwritten by get_window_config()
            width: manifest.width,
            height: manifest.height,
            min_width: manifest.min_width,
            min_height: manifest.min_height,
            max_width: manifest.max_width,
            max_height: manifest.max_height,
            start_position: manifest.start_position.into(),
            resizable: manifest.resizable,
            frameless: manifest.frameless,
            transparent: manifest.transparent,
            always_on_top: manifest.always_on_top,
            fullscreen: manifest.fullscreen,
            maximized: manifest.maximized,
            visible: manifest.visible,
        }
    }
}

/// Window start position (supports string like "center" for TOML)
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

impl From<StartPosition> for WindowStartPosition {
    fn from(pos: StartPosition) -> Self {
        match pos {
            StartPosition::Named(s) if s == "center" => WindowStartPosition::Center,
            StartPosition::Named(_) => WindowStartPosition::Center,
            StartPosition::Position { x, y } => WindowStartPosition::Position { x, y },
        }
    }
}

impl From<WindowStartPosition> for StartPosition {
    fn from(pos: WindowStartPosition) -> Self {
        match pos {
            WindowStartPosition::Center => StartPosition::Named("center".to_string()),
            WindowStartPosition::Position { x, y } => StartPosition::Position { x, y },
        }
    }
}

// ============================================================================
// Bundle Configuration
// ============================================================================

/// Bundle configuration for packaging
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleConfig {
    /// Application icon path (PNG, JPG, or ICO format)
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

    /// Short description
    #[serde(default)]
    pub short_description: Option<String>,

    /// Long description
    #[serde(default)]
    pub long_description: Option<String>,

    /// External binaries to bundle
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,

    /// Additional resources to bundle
    #[serde(default)]
    pub resources: Vec<PathBuf>,

    /// Windows-specific configuration ([bundle.windows])
    #[serde(default)]
    pub windows: Option<WindowsPlatformConfig>,

    /// macOS-specific configuration ([bundle.macos])
    #[serde(default)]
    pub macos: Option<MacOSPlatformConfig>,

    /// Linux-specific configuration ([bundle.linux])
    #[serde(default)]
    pub linux: Option<LinuxPlatformConfig>,
}

// ============================================================================
// Python Process Configuration
// ============================================================================

/// Python process manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessManifestConfig {
    /// Show console window for Python process (Windows only)
    #[serde(default)]
    pub console: bool,

    /// Module search paths
    #[serde(default = "default_module_search_paths")]
    pub module_search_paths: Vec<String>,

    /// Whether to use filesystem importer
    #[serde(default = "default_true")]
    pub filesystem_importer: bool,
}

impl Default for ProcessManifestConfig {
    fn default() -> Self {
        Self {
            console: false,
            module_search_paths: default_module_search_paths(),
            filesystem_importer: true,
        }
    }
}

impl From<ProcessManifestConfig> for ProcessConfig {
    fn from(manifest: ProcessManifestConfig) -> Self {
        Self {
            console: manifest.console,
            module_search_paths: manifest.module_search_paths,
            filesystem_importer: manifest.filesystem_importer,
        }
    }
}

/// Environment isolation manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationManifestConfig {
    /// Isolate PYTHONPATH (default: true)
    #[serde(default = "default_true")]
    pub pythonpath: bool,

    /// Isolate PATH (default: true)
    #[serde(default = "default_true")]
    pub path: bool,

    /// Additional paths to include in PATH
    #[serde(default)]
    pub extra_path: Vec<String>,

    /// Additional paths to include in PYTHONPATH
    #[serde(default)]
    pub extra_pythonpath: Vec<String>,

    /// System essential PATH entries
    #[serde(default)]
    pub system_path: Option<Vec<String>>,

    /// Environment variables to inherit from host
    #[serde(default)]
    pub inherit_env: Option<Vec<String>>,

    /// Environment variables to clear
    #[serde(default)]
    pub clear_env: Vec<String>,
}

impl Default for IsolationManifestConfig {
    fn default() -> Self {
        Self {
            pythonpath: true,
            path: true,
            extra_path: Vec::new(),
            extra_pythonpath: Vec::new(),
            system_path: None,
            inherit_env: None,
            clear_env: Vec::new(),
        }
    }
}

impl IsolationManifestConfig {
    /// Convert to IsolationConfig
    pub fn to_isolation_config(&self) -> IsolationConfig {
        IsolationConfig {
            pythonpath: self.pythonpath,
            path: self.path,
            extra_path: self.extra_path.clone(),
            extra_pythonpath: self.extra_pythonpath.clone(),
            system_path: self
                .system_path
                .clone()
                .unwrap_or_else(IsolationConfig::default_system_path),
            inherit_env: self
                .inherit_env
                .clone()
                .unwrap_or_else(IsolationConfig::default_inherit_env),
            clear_env: self.clear_env.clone(),
        }
    }
}

impl From<IsolationConfig> for IsolationManifestConfig {
    fn from(config: IsolationConfig) -> Self {
        Self {
            pythonpath: config.pythonpath,
            path: config.path,
            extra_path: config.extra_path,
            extra_pythonpath: config.extra_pythonpath,
            system_path: Some(config.system_path),
            inherit_env: Some(config.inherit_env),
            clear_env: config.clear_env,
        }
    }
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
}

impl From<PyOxidizerManifestConfig> for PyOxidizerConfig {
    fn from(manifest: PyOxidizerManifestConfig) -> Self {
        Self {
            executable: manifest.executable,
            target: manifest.target,
            flavor: manifest.flavor,
            release: manifest.release,
            filesystem_importer: manifest.filesystem_importer,
        }
    }
}

/// Code protection manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionManifestConfig {
    /// Enable code protection (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Protection method: "bytecode" (fast) or "py2pyd" (slow)
    #[serde(default)]
    pub method: crate::protection::ProtectionMethodConfig,

    /// Optimization level (0-2 for bytecode, 0-3 for py2pyd)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp: bool,

    /// Files/patterns to exclude from protection
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Encryption settings (for bytecode method)
    #[serde(default)]
    pub encryption: crate::protection::EncryptionConfigPack,
}

fn default_optimization() -> u8 {
    2
}

fn default_strategy() -> String {
    "standalone".to_string()
}

impl Default for ProtectionManifestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: crate::protection::ProtectionMethodConfig::Bytecode,
            optimization: default_optimization(),
            keep_temp: false,
            exclude: Vec::new(),
            encryption: crate::protection::EncryptionConfigPack::default(),
        }
    }
}

impl ProtectionManifestConfig {
    /// Convert to ProtectionConfig
    pub fn to_protection_config(&self) -> crate::protection::ProtectionConfig {
        crate::protection::ProtectionConfig {
            enabled: self.enabled,
            method: self.method,
            python_path: None,
            python_version: None,
            optimization: self.optimization,
            keep_temp: self.keep_temp,
            exclude: self.exclude.clone(),
            target_dcc: None,
            packages: Vec::new(),
            encryption: self.encryption.clone(),
        }
    }
}

// ============================================================================
// Build Configuration
// ============================================================================

/// Build hooks and resource configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// Commands to run before build
    #[serde(default)]
    pub before: Vec<String>,

    /// Commands to run after build
    #[serde(default)]
    pub after: Vec<String>,

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

// ============================================================================
// Hooks Configuration (Manifest format)
// ============================================================================

/// Hooks configuration for collecting additional files (manifest format)
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

impl HooksManifestConfig {
    /// Convert to HooksConfig with path resolution
    pub fn to_hooks_config(&self, base_dir: &Path) -> HooksConfig {
        HooksConfig {
            before_collect: self.before_collect.clone(),
            collect: self
                .collect
                .iter()
                .map(|c| {
                    let source = if Path::new(&c.source).is_absolute() {
                        c.source.clone()
                    } else {
                        normalize_path(&base_dir.join(&c.source))
                            .to_string_lossy()
                            .to_string()
                    };
                    CollectPattern {
                        source,
                        dest: c.dest.clone(),
                        preserve_structure: c.preserve_structure,
                        description: c.description.clone(),
                    }
                })
                .collect(),
            after_pack: self.after_pack.clone(),
        }
    }
}

impl From<HooksConfig> for HooksManifestConfig {
    fn from(config: HooksConfig) -> Self {
        Self {
            before_collect: config.before_collect,
            collect: config.collect.into_iter().map(CollectEntry::from).collect(),
            after_pack: config.after_pack,
        }
    }
}

/// Entry for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectEntry {
    /// Source path or glob pattern
    pub source: String,

    /// Destination path in the bundle
    #[serde(default)]
    pub dest: Option<String>,

    /// Whether to preserve directory structure
    #[serde(default = "default_true")]
    pub preserve_structure: bool,

    /// Optional description for this collection
    #[serde(default)]
    pub description: Option<String>,
}

impl From<CollectPattern> for CollectEntry {
    fn from(pattern: CollectPattern) -> Self {
        Self {
            source: pattern.source,
            dest: pattern.dest,
            preserve_structure: pattern.preserve_structure,
            description: pattern.description,
        }
    }
}

impl From<CollectEntry> for CollectPattern {
    fn from(entry: CollectEntry) -> Self {
        Self {
            source: entry.source,
            dest: entry.dest,
            preserve_structure: entry.preserve_structure,
            description: entry.description,
        }
    }
}

// ============================================================================
// Manifest Implementation
// ============================================================================

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

    /// Find manifest file in directory
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
        // Get frontend configuration
        let frontend = self.frontend.as_ref();
        let (frontend_path, frontend_url) = if let Some(f) = frontend {
            (f.path.clone(), f.url.clone())
        } else {
            (None, None)
        };

        // Check that either url or frontend_path is specified
        if frontend_path.is_none() && frontend_url.is_none() {
            return Err(PackError::Config(
                "Either 'path' or 'url' must be specified in [frontend]".to_string(),
            ));
        }

        // Check mutual exclusivity
        if frontend_path.is_some() && frontend_url.is_some() {
            return Err(PackError::Config(
                "'path' and 'url' are mutually exclusive in [frontend]".to_string(),
            ));
        }

        // Validate backend configuration
        if let Some(ref backend) = self.backend {
            match backend.backend_type {
                BackendType::Python => {
                    if let Some(ref py) = backend.python {
                        // Validate version format
                        if !py.version.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            return Err(PackError::Config(format!(
                                "Invalid Python version format: {}",
                                py.version
                            )));
                        }
                        // Validate optimize level
                        if py.optimize > 2 {
                            return Err(PackError::Config(
                                "Python optimize level must be 0, 1, or 2".to_string(),
                            ));
                        }
                    }
                }
                BackendType::Go => {
                    if let Some(ref go) = backend.go {
                        if go.entry_point.is_none() && go.module.is_none() {
                            return Err(PackError::Config(
                                "Go backend requires either 'entry_point' or 'module'".to_string(),
                            ));
                        }
                    }
                }
                BackendType::Rust => {
                    // Rust config is optional, defaults work
                }
                BackendType::Node => {
                    if let Some(ref node) = backend.node {
                        if node.entry_point.is_none() && node.package_json.is_none() {
                            return Err(PackError::Config(
                                "Node backend requires either 'entry_point' or 'package_json'"
                                    .to_string(),
                            ));
                        }
                    }
                }
                BackendType::None => {
                    // No backend, nothing to validate
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

    /// Check if this is a fullstack (backend + frontend) configuration
    pub fn is_fullstack(&self) -> bool {
        if let Some(ref backend) = self.backend {
            // Check if any backend is configured (either by type or by specific config)
            let has_backend = backend.backend_type != BackendType::None
                || backend.python.is_some()
                || backend.go.is_some()
                || backend.rust.is_some()
                || backend.node.is_some();
            if has_backend {
                return self.get_frontend_path().is_some();
            }
        }
        false
    }

    /// Check if this is a URL-only configuration
    pub fn is_url_mode(&self) -> bool {
        self.get_frontend_url().is_some()
    }

    /// Check if this is a frontend-only configuration
    pub fn is_frontend_mode(&self) -> bool {
        self.get_frontend_path().is_some() && !self.is_fullstack()
    }

    /// Get the effective title
    pub fn get_title(&self) -> String {
        self.package
            .title
            .clone()
            .unwrap_or_else(|| self.package.name.clone())
    }

    /// Get the effective identifier
    pub fn get_identifier(&self) -> Option<String> {
        self.package
            .identifier
            .clone()
            .or_else(|| self.bundle.identifier.clone())
    }

    /// Get the frontend path
    pub fn get_frontend_path(&self) -> Option<PathBuf> {
        self.frontend.as_ref().and_then(|f| f.path.clone())
    }

    /// Get the frontend URL
    pub fn get_frontend_url(&self) -> Option<String> {
        self.frontend.as_ref().and_then(|f| f.url.clone())
    }

    /// Get the user agent
    pub fn get_user_agent(&self) -> Option<String> {
        self.package.user_agent.clone()
    }

    /// Get the allow_new_window setting
    pub fn get_allow_new_window(&self) -> bool {
        self.package.allow_new_window
    }

    /// Get the backend type
    pub fn get_backend_type(&self) -> BackendType {
        self.backend
            .as_ref()
            .map(|b| b.backend_type.clone())
            .unwrap_or(BackendType::None)
    }

    /// Get window configuration with title from package config
    pub fn get_window_config(&self) -> WindowConfig {
        let mut config: WindowConfig = self.window.clone().into();
        config.title = self.get_title();
        config
    }

    /// Get Windows platform configuration
    pub fn get_windows_platform_config(&self) -> WindowsPlatformConfig {
        let mut config = self.bundle.windows.clone().unwrap_or_default();
        if config.copyright.is_none() {
            config.copyright = self.bundle.copyright.clone();
        }
        config
    }

    /// Get macOS platform configuration
    pub fn get_macos_platform_config(&self) -> MacOSPlatformConfig {
        self.bundle.macos.clone().unwrap_or_default()
    }

    /// Get Linux platform configuration
    pub fn get_linux_platform_config(&self) -> LinuxPlatformConfig {
        self.bundle.linux.clone().unwrap_or_default()
    }

    /// Get Windows resource configuration (alias for get_windows_platform_config)
    pub fn get_windows_resource_config(&self) -> WindowsPlatformConfig {
        self.get_windows_platform_config()
    }

    /// Get Python bundle config from backend.python
    pub fn get_python_bundle_config(&self, base_dir: &Path) -> Option<PythonBundleConfig> {
        self.backend.as_ref().and_then(|b| {
            if b.backend_type == BackendType::Python {
                b.python.as_ref().map(|p| p.to_bundle_config(base_dir))
            } else {
                None
            }
        })
    }
}

// Type aliases for convenience
pub type WindowsBundleConfig = WindowsPlatformConfig;
pub type MacOSBundleConfig = MacOSPlatformConfig;
pub type LinuxBundleConfig = LinuxPlatformConfig;
