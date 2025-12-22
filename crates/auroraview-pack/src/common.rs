//! Common configuration types shared between manifest and runtime config
//!
//! This module provides unified type definitions that are used by both:
//! - `manifest.rs` - TOML manifest file parsing
//! - `config.rs` - Runtime configuration for packing
//!
//! ## Configuration Hierarchy
//!
//! ```text
//! [package]           - PackageConfig: Package metadata & identity
//! [frontend]          - FrontendConfig: Frontend configuration (path or url)
//! [backend]           - BackendConfig: Backend abstraction layer
//! [backend.python]    - Python backend settings
//! [backend.go]        - Go backend settings
//! [backend.rust]      - Rust backend settings
//! [backend.node]      - Node.js backend settings
//! [backend.process]   - Common process settings
//! [window]            - WindowConfig: Runtime window behavior
//! [bundle]            - BundleConfig: General bundling settings
//! [bundle.windows]    - Windows-specific bundling
//! [bundle.macos]      - macOS-specific bundling
//! [bundle.linux]      - Linux-specific bundling
//! [build]             - BuildConfig: Build hooks and resources
//! [hooks]             - HooksConfig: File collection hooks
//! [runtime]           - RuntimeConfig: Runtime environment
//! [debug]             - DebugConfig: Debug settings
//! [license]           - LicenseConfig: License validation
//! [inject]            - InjectConfig: JS/CSS injection
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Default Value Functions
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

/// Default Python version
pub fn default_python_version() -> String {
    "3.11".to_string()
}

/// Default bytecode optimization level
pub fn default_optimize() -> u8 {
    1
}

/// Default module search paths for Python
pub fn default_module_search_paths() -> Vec<String> {
    vec!["$EXTRACT_DIR".to_string(), "$SITE_PACKAGES".to_string()]
}

fn default_system_path() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "C:\\Windows\\System32".to_string(),
            "C:\\Windows".to_string(),
            "C:\\Windows\\System32\\Wbem".to_string(),
        ]
    } else {
        vec![
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ]
    }
}

fn default_inherit_env() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "SYSTEMROOT".to_string(),
            "SYSTEMDRIVE".to_string(),
            "TEMP".to_string(),
            "TMP".to_string(),
            "USERPROFILE".to_string(),
            "APPDATA".to_string(),
            "LOCALAPPDATA".to_string(),
            "HOMEDRIVE".to_string(),
            "HOMEPATH".to_string(),
            "COMPUTERNAME".to_string(),
            "USERNAME".to_string(),
        ]
    } else {
        vec![
            "HOME".to_string(),
            "USER".to_string(),
            "LOGNAME".to_string(),
            "SHELL".to_string(),
            "TERM".to_string(),
            "LANG".to_string(),
            "LC_ALL".to_string(),
            "DISPLAY".to_string(),
            "WAYLAND_DISPLAY".to_string(),
            "XDG_RUNTIME_DIR".to_string(),
            "XDG_SESSION_TYPE".to_string(),
            "DBUS_SESSION_BUS_ADDRESS".to_string(),
        ]
    }
}

// ============================================================================
// Window Configuration
// ============================================================================

/// Window start position
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowStartPosition {
    /// Center on screen
    #[default]
    Center,
    /// Specific position
    Position { x: i32, y: i32 },
}

impl WindowStartPosition {
    /// Check if this is the center position
    pub fn is_center(&self) -> bool {
        matches!(self, WindowStartPosition::Center)
    }

    /// Get position coordinates, returns (0, 0) for center
    pub fn coordinates(&self) -> (i32, i32) {
        match self {
            WindowStartPosition::Center => (0, 0),
            WindowStartPosition::Position { x, y } => (*x, *y),
        }
    }
}

/// Window configuration - controls runtime window behavior
///
/// This is separate from platform-specific bundle configurations.
/// `[window]` controls how the window behaves at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title (usually from [app].title)
    #[serde(default)]
    pub title: String,

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

    /// Start position
    #[serde(default)]
    pub start_position: WindowStartPosition,

    /// Whether the window is resizable
    #[serde(default = "default_true")]
    pub resizable: bool,

    /// Whether the window is frameless (no title bar)
    #[serde(default)]
    pub frameless: bool,

    /// Whether the window is transparent
    #[serde(default)]
    pub transparent: bool,

    /// Whether the window is always on top
    #[serde(default)]
    pub always_on_top: bool,

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

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView App".to_string(),
            width: default_width(),
            height: default_height(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            start_position: WindowStartPosition::Center,
            resizable: true,
            frameless: false,
            transparent: false,
            always_on_top: false,
            fullscreen: false,
            maximized: false,
            visible: true,
        }
    }
}

impl WindowConfig {
    /// Create a new window config with title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set minimum size
    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Set frameless mode
    pub fn with_frameless(mut self, frameless: bool) -> Self {
        self.frameless = frameless;
        self
    }

    /// Set always on top
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }
}

// ============================================================================
// Platform-Specific Bundle Configuration
// ============================================================================

/// Windows platform bundle configuration
///
/// Located at `[bundle.platform.windows]` in TOML.
/// Controls Windows-specific executable resources and behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsPlatformConfig {
    /// Path to the .ico icon file (overrides [bundle].icon)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Whether to show console window (default: false for GUI apps)
    #[serde(default)]
    pub console: bool,

    /// File version (e.g., "1.0.0.0")
    #[serde(default)]
    pub file_version: Option<String>,

    /// Product version (e.g., "1.0.0")
    #[serde(default)]
    pub product_version: Option<String>,

    /// File description
    #[serde(default)]
    pub file_description: Option<String>,

    /// Product name
    #[serde(default)]
    pub product_name: Option<String>,

    /// Company name
    #[serde(default)]
    pub company_name: Option<String>,

    /// Copyright string
    #[serde(default)]
    pub copyright: Option<String>,

    /// Code signing certificate path
    #[serde(default)]
    pub certificate: Option<PathBuf>,

    /// Certificate password (or env var name)
    #[serde(default)]
    pub certificate_password: Option<String>,

    /// Timestamp server URL for code signing
    #[serde(default)]
    pub timestamp_url: Option<String>,
}

impl WindowsPlatformConfig {
    /// Check if any resource modifications are needed
    pub fn has_modifications(&self) -> bool {
        self.icon.is_some()
            || self.file_version.is_some()
            || self.product_version.is_some()
            || self.file_description.is_some()
            || self.product_name.is_some()
            || self.company_name.is_some()
            || self.copyright.is_some()
    }
}

/// macOS platform bundle configuration
///
/// Located at `[bundle.platform.macos]` in TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacOSPlatformConfig {
    /// Path to .icns icon file
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Bundle identifier (e.g., "com.example.app")
    #[serde(default)]
    pub bundle_identifier: Option<String>,

    /// Minimum macOS version
    #[serde(default)]
    pub minimum_system_version: Option<String>,

    /// Code signing identity
    #[serde(default)]
    pub signing_identity: Option<String>,

    /// Entitlements file path
    #[serde(default)]
    pub entitlements: Option<PathBuf>,

    /// Create DMG installer
    #[serde(default)]
    pub dmg: bool,

    /// Notarization configuration
    #[serde(default)]
    pub notarization: Option<NotarizationConfig>,
}

/// macOS notarization configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotarizationConfig {
    /// Apple ID
    #[serde(default)]
    pub apple_id: Option<String>,

    /// Team ID
    #[serde(default)]
    pub team_id: Option<String>,

    /// App-specific password (or env var name)
    #[serde(default)]
    pub password: Option<String>,
}

/// Linux platform bundle configuration
///
/// Located at `[bundle.platform.linux]` in TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinuxPlatformConfig {
    /// Path to icon file (PNG or SVG)
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

/// Platform-specific configurations container
///
/// Located at `[bundle.platform]` in TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Windows-specific configuration
    #[serde(default)]
    pub windows: Option<WindowsPlatformConfig>,

    /// macOS-specific configuration
    #[serde(default)]
    pub macos: Option<MacOSPlatformConfig>,

    /// Linux-specific configuration
    #[serde(default)]
    pub linux: Option<LinuxPlatformConfig>,
}

// ============================================================================
// Python Configuration
// ============================================================================

/// Bundle strategy for Python runtime
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BundleStrategy {
    /// Standalone mode: Bundle python-build-standalone runtime
    #[default]
    Standalone,
    /// PyOxidizer mode: Use PyOxidizer to create a single-file executable
    PyOxidizer,
    /// Embed Python code as overlay data (requires system Python)
    Embedded,
    /// Portable directory with Python runtime
    Portable,
    /// Use system Python (smallest output)
    System,
}

impl BundleStrategy {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "standalone" => BundleStrategy::Standalone,
            "pyoxidizer" => BundleStrategy::PyOxidizer,
            "embedded" => BundleStrategy::Embedded,
            "portable" => BundleStrategy::Portable,
            "system" => BundleStrategy::System,
            _ => BundleStrategy::Standalone,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            BundleStrategy::Standalone => "standalone",
            BundleStrategy::PyOxidizer => "pyoxidizer",
            BundleStrategy::Embedded => "embedded",
            BundleStrategy::Portable => "portable",
            BundleStrategy::System => "system",
        }
    }

    /// Check if this strategy bundles Python runtime
    pub fn bundles_runtime(&self) -> bool {
        matches!(
            self,
            BundleStrategy::Standalone | BundleStrategy::PyOxidizer | BundleStrategy::Portable
        )
    }
}

/// Python process configuration
///
/// Located at `[python.process]` in TOML.
/// Controls how the Python subprocess behaves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// Show console window for Python process (Windows only)
    #[serde(default)]
    pub console: bool,

    /// Module search paths (runtime PYTHONPATH)
    /// Special variables: $EXTRACT_DIR, $RESOURCES_DIR, $SITE_PACKAGES, $PYTHON_HOME
    #[serde(default = "default_module_search_paths")]
    pub module_search_paths: Vec<String>,

    /// Enable filesystem importer for dynamic imports
    #[serde(default = "default_true")]
    pub filesystem_importer: bool,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            console: false,
            module_search_paths: default_module_search_paths(),
            filesystem_importer: true,
        }
    }
}

/// Environment isolation configuration (rez-style)
///
/// Located at `[python.isolation]` in TOML.
/// Controls how the packed application isolates its environment from the host system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Whether to isolate PYTHONPATH (default: true)
    #[serde(default = "default_true")]
    pub pythonpath: bool,

    /// Whether to isolate PATH (default: true)
    #[serde(default = "default_true")]
    pub path: bool,

    /// Additional paths to include in PATH
    #[serde(default)]
    pub extra_path: Vec<String>,

    /// Additional paths to include in PYTHONPATH
    #[serde(default)]
    pub extra_pythonpath: Vec<String>,

    /// System essential PATH entries
    #[serde(default = "default_system_path")]
    pub system_path: Vec<String>,

    /// Environment variables to inherit from host
    #[serde(default = "default_inherit_env")]
    pub inherit_env: Vec<String>,

    /// Environment variables to clear
    #[serde(default)]
    pub clear_env: Vec<String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            pythonpath: true,
            path: true,
            extra_path: Vec::new(),
            extra_pythonpath: Vec::new(),
            system_path: default_system_path(),
            inherit_env: default_inherit_env(),
            clear_env: Vec::new(),
        }
    }
}

impl IsolationConfig {
    /// Create a fully isolated configuration
    pub fn full() -> Self {
        Self::default()
    }

    /// Create a non-isolated configuration
    pub fn none() -> Self {
        Self {
            pythonpath: false,
            path: false,
            ..Default::default()
        }
    }

    /// Create a configuration that only isolates PYTHONPATH
    pub fn pythonpath_only() -> Self {
        Self {
            pythonpath: true,
            path: false,
            ..Default::default()
        }
    }

    /// Get default system PATH entries
    pub fn default_system_path() -> Vec<String> {
        default_system_path()
    }

    /// Get default inherit environment variables
    pub fn default_inherit_env() -> Vec<String> {
        default_inherit_env()
    }
}

/// PyOxidizer-specific configuration
///
/// Located at `[python.pyoxidizer]` in TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PyOxidizerConfig {
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

/// Code protection configuration
///
/// Located at `[python.protection]` in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    /// Enable code protection (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Optimization level for C compiler (0-3)
    #[serde(default = "default_protection_optimization")]
    pub optimization: u8,

    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp: bool,

    /// Files/patterns to exclude from compilation
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_protection_optimization() -> u8 {
    2
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            optimization: default_protection_optimization(),
            keep_temp: false,
            exclude: Vec::new(),
        }
    }
}

// ============================================================================
// Hooks Configuration
// ============================================================================

/// Hook configuration for collecting additional files
///
/// Located at `[hooks]` in TOML.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    /// Commands to run before collecting files
    #[serde(default)]
    pub before_collect: Vec<String>,

    /// Additional file patterns to collect
    #[serde(default)]
    pub collect: Vec<CollectPattern>,

    /// Commands to run after packing
    #[serde(default)]
    pub after_pack: Vec<String>,
}

/// Pattern for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectPattern {
    /// Source path or glob pattern
    pub source: String,

    /// Destination path in the bundle
    #[serde(default)]
    pub dest: Option<String>,

    /// Whether to preserve directory structure
    #[serde(default = "default_true")]
    pub preserve_structure: bool,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

impl CollectPattern {
    /// Create a new collect pattern
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            dest: None,
            preserve_structure: true,
            description: None,
        }
    }

    /// Set destination path
    pub fn with_dest(mut self, dest: impl Into<String>) -> Self {
        self.dest = Some(dest.into());
        self
    }
}

// ============================================================================
// Debug Configuration
// ============================================================================

/// Debug configuration
///
/// Located at `[debug]` in TOML.
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

    /// Remote debugging port for CDP connections
    #[serde(default)]
    pub remote_debugging_port: Option<u16>,
}

impl DebugConfig {
    /// Create a debug-enabled configuration
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            devtools: true,
            verbose: false,
            remote_debugging_port: None,
        }
    }

    /// Create a production configuration (all disabled)
    pub fn production() -> Self {
        Self::default()
    }

    /// Set remote debugging port
    pub fn with_remote_debugging(mut self, port: u16) -> Self {
        self.remote_debugging_port = Some(port);
        self
    }
}

// ============================================================================
// Runtime Environment Configuration
// ============================================================================

/// Runtime environment configuration
///
/// Located at `[runtime]` in TOML.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    /// Environment variables to inject at runtime
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from files
    #[serde(default)]
    pub env_files: Vec<PathBuf>,

    /// Working directory override
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
}

impl RuntimeConfig {
    /// Create with environment variables
    pub fn with_env(env: HashMap<String, String>) -> Self {
        Self {
            env,
            ..Default::default()
        }
    }

    /// Add an environment variable
    pub fn add_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }
}

// ============================================================================
// License Configuration
// ============================================================================

/// License/authorization configuration
///
/// Located at `[license]` in TOML.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseConfig {
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

impl LicenseConfig {
    /// Create a time-limited license
    pub fn time_limited(expires_at: impl Into<String>) -> Self {
        Self {
            enabled: true,
            expires_at: Some(expires_at.into()),
            ..Default::default()
        }
    }

    /// Create a token-required license
    pub fn token_required() -> Self {
        Self {
            enabled: true,
            require_token: true,
            ..Default::default()
        }
    }

    /// Create a license with both time limit and token
    pub fn full(expires_at: impl Into<String>) -> Self {
        Self {
            enabled: true,
            expires_at: Some(expires_at.into()),
            require_token: true,
            ..Default::default()
        }
    }

    /// Check if license validation is active
    pub fn is_active(&self) -> bool {
        self.enabled && (self.expires_at.is_some() || self.require_token)
    }
}

// ============================================================================
// Inject Configuration
// ============================================================================

/// JavaScript/CSS injection configuration
///
/// Located at `[inject]` in TOML.
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

// ============================================================================
// Target Platform
// ============================================================================

/// Target platform for the packed executable
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetPlatform {
    /// Current platform
    #[default]
    Current,
    /// Windows
    Windows,
    /// macOS
    MacOS,
    /// Linux
    Linux,
}

impl TargetPlatform {
    /// Get the current platform
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            TargetPlatform::Windows
        } else if cfg!(target_os = "macos") {
            TargetPlatform::MacOS
        } else if cfg!(target_os = "linux") {
            TargetPlatform::Linux
        } else {
            TargetPlatform::Current
        }
    }

    /// Get executable extension for this platform
    pub fn exe_extension(&self) -> &'static str {
        match self {
            TargetPlatform::Windows => ".exe",
            TargetPlatform::Current if cfg!(target_os = "windows") => ".exe",
            _ => "",
        }
    }
}

// ============================================================================
// Backward Compatibility Aliases
// ============================================================================

/// Alias for WindowsPlatformConfig (backward compatibility)
pub type WindowsResourceConfig = WindowsPlatformConfig;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_start_position() {
        let center = WindowStartPosition::Center;
        assert!(center.is_center());
        assert_eq!(center.coordinates(), (0, 0));

        let pos = WindowStartPosition::Position { x: 100, y: 200 };
        assert!(!pos.is_center());
        assert_eq!(pos.coordinates(), (100, 200));
    }

    #[test]
    fn test_bundle_strategy() {
        assert_eq!(
            BundleStrategy::parse("standalone"),
            BundleStrategy::Standalone
        );
        assert_eq!(
            BundleStrategy::parse("PYOXIDIZER"),
            BundleStrategy::PyOxidizer
        );
        assert_eq!(BundleStrategy::parse("unknown"), BundleStrategy::Standalone);

        assert!(BundleStrategy::Standalone.bundles_runtime());
        assert!(!BundleStrategy::System.bundles_runtime());
    }

    #[test]
    fn test_license_config() {
        let license = LicenseConfig::time_limited("2025-12-31");
        assert!(license.enabled);
        assert!(license.is_active());
        assert_eq!(license.expires_at, Some("2025-12-31".to_string()));

        let token_license = LicenseConfig::token_required();
        assert!(token_license.require_token);
        assert!(token_license.is_active());
    }

    #[test]
    fn test_isolation_config() {
        let full = IsolationConfig::full();
        assert!(full.pythonpath);
        assert!(full.path);

        let none = IsolationConfig::none();
        assert!(!none.pythonpath);
        assert!(!none.path);
    }

    #[test]
    fn test_process_config_defaults() {
        let config = ProcessConfig::default();
        assert!(!config.console);
        assert!(config.filesystem_importer);
        assert!(!config.module_search_paths.is_empty());
    }
}
