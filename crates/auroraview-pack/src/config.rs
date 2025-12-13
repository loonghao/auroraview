//! Pack configuration types

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
}

fn default_python_version() -> String {
    "3.11".to_string()
}

fn default_optimize() -> u8 {
    1
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
    Position {
        x: i32,
        y: i32,
    },
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
    /// Environment variables to inject at runtime
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// License configuration for authorization
    #[serde(default)]
    pub license: Option<LicenseConfig>,
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
            env: HashMap::new(),
            license: None,
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
            env: HashMap::new(),
            license: None,
        }
    }

    /// Create a fullstack mode configuration (frontend + Python backend)
    pub fn fullstack(
        frontend_path: impl Into<PathBuf>,
        entry_point: impl Into<String>,
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
            env: HashMap::new(),
            license: None,
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
            env: HashMap::new(),
            license: None,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_mode() {
        let config = PackConfig::url("https://example.com");
        assert_eq!(config.mode.name(), "url");
        assert!(!config.mode.embeds_assets());
        assert_eq!(config.output_name, "example");
    }

    #[test]
    fn test_frontend_mode() {
        let config = PackConfig::frontend("./dist");
        assert_eq!(config.mode.name(), "frontend");
        assert!(config.mode.embeds_assets());
        assert_eq!(config.output_name, "dist");
    }

    #[test]
    fn test_builder_pattern() {
        let config = PackConfig::url("example.com")
            .with_output("my-app")
            .with_title("My App")
            .with_size(1920, 1080)
            .with_debug(true);

        assert_eq!(config.output_name, "my-app");
        assert_eq!(config.window.title, "My App");
        assert_eq!(config.window.width, 1920);
        assert_eq!(config.window.height, 1080);
        assert!(config.debug);
    }

    #[test]
    fn test_bundle_strategy_default() {
        let strategy = BundleStrategy::default();
        assert_eq!(strategy, BundleStrategy::Standalone);
    }

    #[test]
    fn test_bundle_strategy_serialization() {
        let strategies = [
            (BundleStrategy::Standalone, "standalone"),
            (BundleStrategy::PyOxidizer, "py_oxidizer"),
            (BundleStrategy::Embedded, "embedded"),
            (BundleStrategy::Portable, "portable"),
            (BundleStrategy::System, "system"),
        ];

        for (strategy, expected_name) in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            assert!(
                json.contains(expected_name),
                "Strategy {:?} should serialize to contain '{}'",
                strategy,
                expected_name
            );
        }
    }

    #[test]
    fn test_bundle_strategy_deserialization() {
        let test_cases = [
            ("\"standalone\"", BundleStrategy::Standalone),
            ("\"embedded\"", BundleStrategy::Embedded),
            ("\"portable\"", BundleStrategy::Portable),
            ("\"system\"", BundleStrategy::System),
        ];

        for (json, expected) in test_cases {
            let parsed: BundleStrategy = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn test_python_bundle_config_default() {
        let config = PythonBundleConfig::default();
        assert!(config.entry_point.is_empty());
        assert!(config.include_paths.is_empty());
        assert!(config.packages.is_empty());
        assert!(config.requirements.is_none());
        assert_eq!(config.strategy, BundleStrategy::Standalone);
        assert_eq!(config.version, "3.11");
        assert_eq!(config.optimize, 1);
        assert!(!config.include_pip);
        assert!(!config.include_setuptools);
    }

    #[test]
    fn test_fullstack_mode() {
        let config = PackConfig::fullstack("./dist", "main:run");
        assert_eq!(config.mode.name(), "fullstack");
        assert!(config.mode.embeds_assets());
        assert!(config.mode.has_python());

        if let PackMode::FullStack { python, .. } = &config.mode {
            assert_eq!(python.entry_point, "main:run");
            assert_eq!(python.strategy, BundleStrategy::Standalone);
        } else {
            panic!("Expected FullStack mode");
        }
    }

    #[test]
    fn test_fullstack_with_config() {
        let python_config = PythonBundleConfig {
            entry_point: "app:main".to_string(),
            packages: vec!["pyyaml".to_string(), "requests".to_string()],
            version: "3.12".to_string(),
            strategy: BundleStrategy::Embedded,
            ..Default::default()
        };

        let config = PackConfig::fullstack_with_config("./dist", python_config);

        if let PackMode::FullStack { python, .. } = &config.mode {
            assert_eq!(python.entry_point, "app:main");
            assert_eq!(python.packages.len(), 2);
            assert_eq!(python.version, "3.12");
            assert_eq!(python.strategy, BundleStrategy::Embedded);
        } else {
            panic!("Expected FullStack mode");
        }
    }

    #[test]
    fn test_pack_mode_properties() {
        let url_mode = PackMode::Url {
            url: "https://example.com".to_string(),
        };
        assert!(!url_mode.embeds_assets());
        assert!(!url_mode.has_python());

        let frontend_mode = PackMode::Frontend {
            path: PathBuf::from("./dist"),
        };
        assert!(frontend_mode.embeds_assets());
        assert!(!frontend_mode.has_python());

        let fullstack_mode = PackMode::FullStack {
            frontend_path: PathBuf::from("./dist"),
            python: Box::new(PythonBundleConfig::default()),
        };
        assert!(fullstack_mode.embeds_assets());
        assert!(fullstack_mode.has_python());
    }

    #[test]
    fn test_license_config_time_limited() {
        let license = LicenseConfig::time_limited("2025-12-31");
        assert!(license.enabled);
        assert_eq!(license.expires_at, Some("2025-12-31".to_string()));
        assert!(!license.require_token);
    }

    #[test]
    fn test_license_config_token_required() {
        let license = LicenseConfig::token_required();
        assert!(license.enabled);
        assert!(license.require_token);
        assert!(license.expires_at.is_none());
    }

    #[test]
    fn test_license_config_full() {
        let license = LicenseConfig::full("2025-06-30");
        assert!(license.enabled);
        assert!(license.require_token);
        assert_eq!(license.expires_at, Some("2025-06-30".to_string()));
    }

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "AuroraView App");
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.resizable);
        assert!(!config.frameless);
        assert!(!config.transparent);
        assert!(!config.always_on_top);
    }

    #[test]
    fn test_window_start_position() {
        let center = WindowStartPosition::Center;
        let position = WindowStartPosition::Position { x: 100, y: 200 };

        // Test serialization
        let center_json = serde_json::to_string(&center).unwrap();
        let position_json = serde_json::to_string(&position).unwrap();

        assert!(center_json.contains("center"));
        assert!(position_json.contains("100"));
        assert!(position_json.contains("200"));
    }

    #[test]
    fn test_pack_config_with_env() {
        let config = PackConfig::url("example.com")
            .with_env_var("APP_MODE", "production")
            .with_env_var("LOG_LEVEL", "info");

        assert_eq!(config.env.get("APP_MODE"), Some(&"production".to_string()));
        assert_eq!(config.env.get("LOG_LEVEL"), Some(&"info".to_string()));
    }

    #[test]
    fn test_pack_config_with_license() {
        let config = PackConfig::url("example.com")
            .with_expiration("2025-12-31")
            .with_token_required();

        let license = config.license.unwrap();
        assert!(license.enabled);
        assert!(license.require_token);
        assert_eq!(license.expires_at, Some("2025-12-31".to_string()));
    }

    #[test]
    fn test_target_platform() {
        assert_eq!(TargetPlatform::default(), TargetPlatform::Current);

        let platforms = [
            TargetPlatform::Current,
            TargetPlatform::Windows,
            TargetPlatform::MacOS,
            TargetPlatform::Linux,
        ];

        for platform in platforms {
            let json = serde_json::to_string(&platform).unwrap();
            let parsed: TargetPlatform = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, platform);
        }
    }

    #[test]
    fn test_collect_pattern() {
        let pattern = CollectPattern {
            source: "../examples/*.py".to_string(),
            dest: Some("examples".to_string()),
            preserve_structure: true,
        };

        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("../examples/*.py"));
        assert!(json.contains("examples"));
    }
}
