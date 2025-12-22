//! Pack configuration types

use crate::protection::ProtectionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Pack mode determines how the application loads content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackMode {
    /// Load content from a URL
    Url {
        /// The URL to load (will be normalized to include https:// if missing)
        url: String,
    },
    /// Load content from embedded frontend assets
    Frontend {
        /// Path to the frontend directory or HTML file
        #[serde(skip)]
        path: PathBuf,
    },
    /// FullStack mode: Frontend + Python backend
    FullStack {
        /// Path to the frontend directory
        #[serde(skip)]
        frontend_path: PathBuf,
        /// Python configuration (boxed to reduce enum size)
        python: Box<PythonBundleConfig>,
    },
}

/// Environment isolation configuration
///
/// Controls how the packed application isolates its environment from the host system.
/// Inspired by rez's environment isolation design.
///
/// ## Design Philosophy
///
/// By default, packed applications run in an isolated environment:
/// - PYTHONPATH: Only includes bundled module paths, not inherited from host
/// - PATH: Only includes system essential paths + bundled binaries
///
/// This ensures reproducible execution regardless of the host environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Whether to isolate PYTHONPATH (default: true)
    ///
    /// When true:
    /// - PYTHONPATH is set to only include bundled paths ($EXTRACT_DIR, $SITE_PACKAGES)
    /// - Host PYTHONPATH is NOT inherited
    ///
    /// When false:
    /// - Host PYTHONPATH is inherited and bundled paths are prepended
    #[serde(default = "default_true")]
    pub isolate_pythonpath: bool,

    /// Whether to isolate PATH (default: true)
    ///
    /// When true:
    /// - PATH includes only system essential paths + bundled binaries
    /// - Host PATH is NOT inherited (except system essentials)
    ///
    /// When false:
    /// - Host PATH is inherited
    #[serde(default = "default_true")]
    pub isolate_path: bool,

    /// Additional paths to include in PATH (always included regardless of isolation)
    ///
    /// These paths are added to PATH in addition to system essentials.
    /// Useful for including bundled external binaries.
    ///
    /// Special variables:
    /// - `$EXTRACT_DIR` - The directory where files are extracted
    /// - `$RESOURCES_DIR` - The resources directory
    /// - `$PYTHON_HOME` - The Python runtime directory
    #[serde(default)]
    pub extra_path: Vec<String>,

    /// Additional paths to include in PYTHONPATH (always included regardless of isolation)
    ///
    /// These paths are added in addition to the default module_search_paths.
    #[serde(default)]
    pub extra_pythonpath: Vec<String>,

    /// System essential PATH entries to always include when PATH is isolated
    ///
    /// Default (Windows): ["C:\\Windows\\System32", "C:\\Windows"]
    /// Default (Unix): ["/usr/bin", "/bin", "/usr/local/bin"]
    ///
    /// Set to empty to use only bundled paths (fully isolated).
    #[serde(default = "default_system_path")]
    pub system_path: Vec<String>,

    /// Environment variables to explicitly inherit from host
    ///
    /// Even when isolation is enabled, these variables are inherited.
    /// Useful for: HOME, USER, TEMP, DISPLAY, etc.
    #[serde(default = "default_inherit_env")]
    pub inherit_env: Vec<String>,

    /// Environment variables to explicitly clear/remove
    ///
    /// These variables are removed from the child process environment.
    #[serde(default)]
    pub clear_env: Vec<String>,
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
            // Display/GPU related
            "DISPLAY".to_string(),
            "WAYLAND_DISPLAY".to_string(),
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

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            isolate_pythonpath: true,
            isolate_path: true,
            extra_path: Vec::new(),
            extra_pythonpath: Vec::new(),
            system_path: default_system_path(),
            inherit_env: default_inherit_env(),
            clear_env: Vec::new(),
        }
    }
}

impl IsolationConfig {
    /// Create a fully isolated configuration (no host environment inherited)
    pub fn full() -> Self {
        Self::default()
    }

    /// Create a non-isolated configuration (inherits host environment)
    pub fn none() -> Self {
        Self {
            isolate_pythonpath: false,
            isolate_path: false,
            ..Default::default()
        }
    }

    /// Create a configuration that only isolates PYTHONPATH
    pub fn pythonpath_only() -> Self {
        Self {
            isolate_pythonpath: true,
            isolate_path: false,
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

/// Python bundle configuration for FullStack mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBundleConfig {
    /// Entry point (e.g., "myapp.main:run" or "main.py")
    pub entry_point: String,
    /// Python source paths to include
    #[serde(default)]
    pub include_paths: Vec<PathBuf>,
    /// Pip packages to install
    #[serde(default)]
    pub packages: Vec<String>,
    /// Path to requirements.txt
    #[serde(default)]
    pub requirements: Option<PathBuf>,
    /// Bundle strategy
    #[serde(default)]
    pub strategy: BundleStrategy,
    /// Python version (e.g., "3.11")
    #[serde(default = "default_python_version")]
    pub version: String,
    /// Bytecode optimization level (0, 1, or 2)
    #[serde(default = "default_optimize")]
    pub optimize: u8,
    /// Exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,
    /// External binaries to bundle (paths to executables)
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,
    /// Additional resource files/directories
    #[serde(default)]
    pub resources: Vec<PathBuf>,
    /// Include pip in the bundle (for PyOxidizer)
    #[serde(default)]
    pub include_pip: bool,
    /// Include setuptools in the bundle (for PyOxidizer)
    #[serde(default)]
    pub include_setuptools: bool,
    /// PyOxidizer distribution flavor
    #[serde(default)]
    pub distribution_flavor: Option<String>,
    /// Custom PyOxidizer executable path
    #[serde(default)]
    pub pyoxidizer_path: Option<PathBuf>,
    /// Module search paths (relative to extract directory).
    /// These paths are added to PYTHONPATH at runtime.
    ///
    /// Special variables:
    /// - `$EXTRACT_DIR` - The directory where Python files are extracted
    /// - `$RESOURCES_DIR` - The resources directory
    /// - `$SITE_PACKAGES` - The site-packages directory
    ///
    /// Default: `["$EXTRACT_DIR", "$SITE_PACKAGES"]`
    #[serde(default = "default_module_search_paths")]
    pub module_search_paths: Vec<String>,
    /// Whether to use filesystem importer (allows dynamic imports).
    /// When false, only embedded modules can be imported.
    #[serde(default = "default_true")]
    pub filesystem_importer: bool,
    /// Show console window for Python process (Windows only).
    /// When false (default), Python runs without a visible console window.
    /// Set to true for debugging purposes.
    #[serde(default)]
    pub show_console: bool,
    /// Environment isolation configuration
    ///
    /// Controls how the packed application isolates its environment from the host.
    /// Default: full isolation (PYTHONPATH and PATH are isolated)
    #[serde(default)]
    pub isolation: IsolationConfig,
    /// Code protection configuration (py2pyd compilation)
    ///
    /// When enabled, `.py` files can be compiled to native extensions (`.pyd`/`.so`).
    /// Default: disabled
    #[serde(default = "default_protection_config")]
    pub protection: ProtectionConfig,
}

fn default_protection_config() -> ProtectionConfig {
    // Disabled by default: enabling protection requires build tools to compile extensions.
    ProtectionConfig::default()
}

fn default_python_version() -> String {
    "3.10".to_string()
}

fn default_optimize() -> u8 {
    1
}

fn default_module_search_paths() -> Vec<String> {
    vec!["$EXTRACT_DIR".to_string(), "$SITE_PACKAGES".to_string()]
}

impl Default for PythonBundleConfig {
    fn default() -> Self {
        Self {
            entry_point: String::new(),
            include_paths: Vec::new(),
            packages: Vec::new(),
            requirements: None,
            strategy: BundleStrategy::default(),
            version: default_python_version(),
            optimize: default_optimize(),
            exclude: Vec::new(),
            external_bin: Vec::new(),
            resources: Vec::new(),
            include_pip: false,
            include_setuptools: false,
            distribution_flavor: None,
            pyoxidizer_path: None,
            module_search_paths: default_module_search_paths(),
            filesystem_importer: true,
            show_console: false,
            isolation: IsolationConfig::default(),
            protection: default_protection_config(),
        }
    }
}

/// Bundle strategy for Python runtime
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BundleStrategy {
    /// Standalone mode: Bundle python-build-standalone runtime with the executable
    /// Downloads a pre-built Python distribution and embeds it in the overlay
    /// No external dependencies required at runtime (default)
    #[default]
    Standalone,
    /// PyOxidizer mode: Use PyOxidizer to create a single-file executable
    /// Requires PyOxidizer to be installed
    PyOxidizer,
    /// Embed Python code as overlay data (requires system Python to run)
    /// Smallest executable size, but requires Python installed on target
    Embedded,
    /// Portable directory with Python runtime extracted alongside executable
    /// Creates a directory structure with Python runtime and app files
    Portable,
    /// Use system Python (smallest output, requires Python installed)
    /// Only embeds Python source code, uses system Python to run
    System,
}

impl PackMode {
    /// Get the mode name
    pub fn name(&self) -> &'static str {
        match self {
            PackMode::Url { .. } => "url",
            PackMode::Frontend { .. } => "frontend",
            PackMode::FullStack { .. } => "fullstack",
        }
    }

    /// Check if this mode embeds assets
    pub fn embeds_assets(&self) -> bool {
        matches!(self, PackMode::Frontend { .. } | PackMode::FullStack { .. })
    }

    /// Check if this mode includes Python backend
    pub fn has_python(&self) -> bool {
        matches!(self, PackMode::FullStack { .. })
    }
}

/// Window start position
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowStartPosition {
    /// Center on screen
    #[default]
    Center,
    /// Specific position
    Position { x: i32, y: i32 },
}

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

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Minimum width
    #[serde(default)]
    pub min_width: Option<u32>,
    /// Minimum height
    #[serde(default)]
    pub min_height: Option<u32>,
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
}

fn default_true() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView App".to_string(),
            width: 1280,
            height: 720,
            min_width: None,
            min_height: None,
            start_position: WindowStartPosition::Center,
            resizable: true,
            frameless: false,
            transparent: false,
            always_on_top: false,
        }
    }
}

/// License/authorization configuration
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
}

/// Hook configuration for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    /// Commands to run before collecting files
    #[serde(default)]
    pub before_collect: Vec<String>,
    /// Additional file patterns to collect (glob patterns)
    #[serde(default)]
    pub collect_files: Vec<CollectPattern>,
    /// Commands to run after packing
    #[serde(default)]
    pub after_pack: Vec<String>,
}

/// Pattern for collecting additional files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectPattern {
    /// Source path or glob pattern
    pub source: String,
    /// Destination path in the bundle (relative to assets root)
    #[serde(default)]
    pub dest: Option<String>,
    /// Whether to preserve directory structure
    #[serde(default = "default_true")]
    pub preserve_structure: bool,
}

/// Complete pack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackConfig {
    /// Pack mode (URL or Frontend)
    pub mode: PackMode,
    /// Output executable name (without extension)
    pub output_name: String,
    /// Output directory
    #[serde(skip)]
    pub output_dir: PathBuf,
    /// Window configuration
    pub window: WindowConfig,
    /// Target platform
    #[serde(default)]
    pub target_platform: TargetPlatform,
    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,
    /// Allow opening new windows
    #[serde(default)]
    pub allow_new_window: bool,
    /// Custom user agent
    #[serde(default)]
    pub user_agent: Option<String>,
    /// JavaScript to inject
    #[serde(default)]
    pub inject_js: Option<String>,
    /// CSS to inject
    #[serde(default)]
    pub inject_css: Option<String>,
    /// Icon path (for resource injection)
    #[serde(skip)]
    pub icon_path: Option<PathBuf>,
    /// Window icon PNG data (embedded at pack time, used for window title bar icon)
    /// This is separate from icon_path which is for Windows .exe resource icon (.ico)
    #[serde(default)]
    #[serde(with = "serde_bytes_base64")]
    pub window_icon: Option<Vec<u8>>,
    /// Environment variables to inject at runtime
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// License configuration for authorization
    #[serde(default)]
    pub license: Option<LicenseConfig>,
    /// Hooks configuration for collecting additional files
    #[serde(default)]
    pub hooks: Option<HooksConfig>,
    /// Remote debugging port for CDP (Chrome DevTools Protocol) connections
    /// Default: None (disabled)
    /// When set, enables remote debugging on the specified port
    /// Playwright/Puppeteer can connect via: `browser.connect_over_cdp(f"http://localhost:{port}")`
    #[serde(default)]
    pub remote_debugging_port: Option<u16>,

    /// Windows-specific resource configuration
    #[serde(skip)]
    pub windows_resource: WindowsResourceConfig,
}

/// Serde helper module for serializing Option<Vec<u8>> as base64
mod serde_bytes_base64 {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(bytes) => serializer.serialize_some(&STANDARD.encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => STANDARD
                .decode(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

/// Windows executable resource configuration
#[derive(Debug, Clone, Default)]
pub struct WindowsResourceConfig {
    /// Path to the .ico icon file
    pub icon: Option<PathBuf>,

    /// Whether to show console window (default: false)
    /// When false, the executable runs as a Windows GUI application (no console)
    /// When true, the executable runs as a console application (shows black window)
    pub console: bool,

    /// File version (e.g., "1.0.0.0")
    pub file_version: Option<String>,

    /// Product version (e.g., "1.0.0")
    pub product_version: Option<String>,

    /// File description
    pub file_description: Option<String>,

    /// Product name
    pub product_name: Option<String>,

    /// Company name
    pub company_name: Option<String>,

    /// Copyright string
    pub copyright: Option<String>,
}

impl PackConfig {
    /// Create a URL mode configuration
    pub fn url(url: impl Into<String>) -> Self {
        let url = url.into();
        let output_name = url
            .replace("https://", "")
            .replace("http://", "")
            .replace("www.", "")
            .split('.')
            .next()
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::Url { url },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsResourceConfig::default(),
        }
    }

    /// Create a frontend mode configuration
    pub fn frontend(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let output_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::Frontend { path },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsResourceConfig::default(),
        }
    }

    /// Create a fullstack mode configuration (frontend + Python backend)
    pub fn fullstack(frontend_path: impl Into<PathBuf>, entry_point: impl Into<String>) -> Self {
        let frontend_path = frontend_path.into();
        let output_name = frontend_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::FullStack {
                frontend_path,
                python: Box::new(PythonBundleConfig {
                    entry_point: entry_point.into(),
                    ..Default::default()
                }),
            },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsResourceConfig::default(),
        }
    }

    /// Create a fullstack mode configuration with full Python config
    pub fn fullstack_with_config(
        frontend_path: impl Into<PathBuf>,
        python: PythonBundleConfig,
    ) -> Self {
        let frontend_path = frontend_path.into();
        let output_name = frontend_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::FullStack {
                frontend_path,
                python: Box::new(python),
            },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsResourceConfig::default(),
        }
    }

    /// Set the output name
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.output_name = name.into();
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set the window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Set the window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window.width = width;
        self.window.height = height;
        self
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set frameless mode
    pub fn with_frameless(mut self, frameless: bool) -> Self {
        self.window.frameless = frameless;
        self
    }

    /// Set always on top
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.window.always_on_top = always_on_top;
        self
    }

    /// Set resizable
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set icon path
    pub fn with_icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set license configuration
    pub fn with_license(mut self, license: LicenseConfig) -> Self {
        self.license = Some(license);
        self
    }

    /// Set remote debugging port for CDP connections
    /// When set, Playwright/Puppeteer can connect via `connect_over_cdp(f"http://localhost:{port}")`
    pub fn with_remote_debugging_port(mut self, port: u16) -> Self {
        self.remote_debugging_port = Some(port);
        self
    }

    /// Set expiration date (enables license)
    pub fn with_expiration(mut self, expires_at: impl Into<String>) -> Self {
        self.license = Some(LicenseConfig::time_limited(expires_at));
        self
    }

    /// Require token for authorization
    pub fn with_token_required(mut self) -> Self {
        let mut license = self.license.unwrap_or_default();
        license.enabled = true;
        license.require_token = true;
        self.license = Some(license);
        self
    }

    /// Set hooks configuration for collecting additional files
    pub fn with_hooks(mut self, hooks: HooksConfig) -> Self {
        self.hooks = Some(hooks);
        self
    }
}
