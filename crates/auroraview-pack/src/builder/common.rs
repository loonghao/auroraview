//! Common types for builders

use crate::PackError;
use std::collections::HashMap;
use std::path::PathBuf;

/// Build result type
pub type BuildResult<T> = Result<T, PackError>;

/// Build context shared across build stages
pub struct BuildContext {
    /// Build configuration (version-migrated)
    pub config: BuildConfig,
    /// Output directory
    pub output_dir: PathBuf,
    /// Temporary directory
    pub temp_dir: PathBuf,
    /// Collected assets (path -> content)
    pub assets: Vec<(String, Vec<u8>)>,
    /// Frontend bundle (if any)
    pub frontend: Option<FrontendBundle>,
    /// Python bundle (if any)
    pub python: Option<PythonBundle>,
    /// Extensions to bundle
    pub extensions: Vec<ExtensionBundle>,
    /// Build metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Build start time
    pub start_time: std::time::Instant,
}

impl BuildContext {
    /// Create new build context
    pub fn new(config: BuildConfig, output_dir: PathBuf) -> Self {
        let temp_dir = output_dir.join(".build_temp");
        Self {
            config,
            output_dir,
            temp_dir,
            assets: Vec::new(),
            frontend: None,
            python: None,
            extensions: Vec::new(),
            metadata: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Add asset
    pub fn add_asset(&mut self, path: impl Into<String>, content: Vec<u8>) {
        self.assets.push((path.into(), content));
    }

    /// Set metadata
    pub fn set_metadata<T: serde::Serialize>(&mut self, key: &str, value: T) -> BuildResult<()> {
        let json = serde_json::to_value(value)
            .map_err(|e| PackError::Config(format!("Failed to serialize metadata: {}", e)))?;
        self.metadata.insert(key.to_string(), json);
        Ok(())
    }

    /// Get metadata
    pub fn get_metadata<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Create temp directory
    pub fn ensure_temp_dir(&self) -> BuildResult<()> {
        std::fs::create_dir_all(&self.temp_dir)?;
        Ok(())
    }

    /// Clean temp directory
    pub fn cleanup(&self) -> BuildResult<()> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Build output
#[derive(Debug, Clone)]
pub struct BuildOutput {
    /// Output path (file or directory)
    pub path: PathBuf,
    /// Output size in bytes
    pub size: u64,
    /// Output format
    pub format: String,
    /// Asset count
    pub asset_count: usize,
    /// Build duration
    pub duration: std::time::Duration,
    /// Additional info
    pub info: HashMap<String, String>,
}

impl BuildOutput {
    /// Create new output
    pub fn new(path: PathBuf, format: &str) -> Self {
        Self {
            path,
            size: 0,
            format: format.to_string(),
            asset_count: 0,
            duration: std::time::Duration::ZERO,
            info: HashMap::new(),
        }
    }

    /// Set size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set asset count
    pub fn with_assets(mut self, count: usize) -> Self {
        self.asset_count = count;
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add info
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.info.insert(key.to_string(), value.to_string());
        self
    }
}

/// Versioned build configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildConfig {
    /// Config schema version (for migrations)
    #[serde(default = "default_version")]
    pub version: u32,

    /// App metadata
    pub app: AppConfig,

    /// Build target
    pub target: TargetConfig,

    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,

    /// Frontend configuration
    pub frontend: Option<FrontendConfig>,

    /// Backend configuration
    pub backend: Option<BackendConfig>,

    /// Extensions configuration
    #[serde(default)]
    pub extensions: ExtensionsConfig,

    /// Platform-specific settings
    #[serde(default)]
    pub platform: PlatformConfig,

    /// Debug settings
    #[serde(default)]
    pub debug: DebugConfig,
}

fn default_version() -> u32 {
    1
}

/// App metadata
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// App name
    pub name: String,
    /// App version
    #[serde(default = "default_app_version")]
    pub version: String,
    /// App description
    pub description: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Copyright
    pub copyright: Option<String>,
    /// App icon path
    pub icon: Option<PathBuf>,
    /// Bundle identifier (com.example.app)
    pub identifier: Option<String>,
}

fn default_app_version() -> String {
    "1.0.0".to_string()
}

/// Build target configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TargetConfig {
    /// Target platform (win, mac, linux, ios, android, web, wechat, etc.)
    pub platform: String,
    /// Output format (exe, dmg, appimage, ipa, apk, etc.)
    pub format: Option<String>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Output file name (without extension)
    pub output_name: Option<String>,
}

/// Window configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: Option<String>,
    /// Window width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Window height
    #[serde(default = "default_height")]
    pub height: u32,
    /// Minimum width
    pub min_width: Option<u32>,
    /// Minimum height
    pub min_height: Option<u32>,
    /// Maximum width
    pub max_width: Option<u32>,
    /// Maximum height
    pub max_height: Option<u32>,
    /// Resizable
    #[serde(default = "default_true")]
    pub resizable: bool,
    /// Fullscreen
    #[serde(default)]
    pub fullscreen: bool,
    /// Frameless
    #[serde(default)]
    pub frameless: bool,
    /// Transparent
    #[serde(default)]
    pub transparent: bool,
    /// Always on top
    #[serde(default)]
    pub always_on_top: bool,
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

/// Frontend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum FrontendConfig {
    /// URL mode
    Url { url: String },
    /// Local path mode
    Path { path: PathBuf },
}

/// Backend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BackendConfig {
    /// No backend
    #[serde(rename = "none")]
    None,
    /// Python backend
    #[serde(rename = "python")]
    Python(PythonConfig),
    /// Node.js backend
    #[serde(rename = "node")]
    Node(NodeConfig),
    /// Process backend (external executable)
    #[serde(rename = "process")]
    Process(ProcessConfig),
}

/// Python backend configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PythonConfig {
    /// Python version
    #[serde(default = "default_python_version")]
    pub version: String,
    /// Entry point (file path or module:function)
    pub entry: String,
    /// Requirements file or list
    pub requirements: Option<Vec<String>>,
    /// Additional Python paths
    #[serde(default)]
    pub paths: Vec<PathBuf>,
    /// Bundle strategy
    #[serde(default)]
    pub strategy: PythonStrategy,
}

fn default_python_version() -> String {
    "3.11".to_string()
}

/// Python bundle strategy
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PythonStrategy {
    /// Standalone Python runtime (default)
    #[default]
    Standalone,
    /// Use PyOxidizer
    PyOxidizer,
    /// Embedded Python
    Embedded,
    /// System Python
    System,
}

/// Node.js backend configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    /// Node.js version
    pub version: Option<String>,
    /// Entry point
    pub entry: String,
    /// Package directory
    pub package_dir: Option<PathBuf>,
}

/// Process backend configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProcessConfig {
    /// Executable path
    pub executable: PathBuf,
    /// Arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Extensions configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtensionsConfig {
    /// Enable extensions
    #[serde(default)]
    pub enabled: bool,
    /// Chrome Web Store extensions
    #[serde(default)]
    pub store: Vec<StoreExtension>,
    /// Local extensions
    #[serde(default)]
    pub local: Vec<LocalExtension>,
}

/// Chrome Web Store extension
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoreExtension {
    /// Extension ID
    pub id: String,
    /// Optional version
    pub version: Option<String>,
    /// Display name
    pub name: Option<String>,
}

/// Local extension
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalExtension {
    /// Extension path
    pub path: PathBuf,
    /// Display name
    pub name: Option<String>,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PlatformConfig {
    /// Windows settings
    pub windows: Option<WindowsPlatform>,
    /// macOS settings
    pub macos: Option<MacOSPlatform>,
    /// Linux settings
    pub linux: Option<LinuxPlatform>,
    /// iOS settings
    pub ios: Option<IOSPlatform>,
    /// Android settings
    pub android: Option<AndroidPlatform>,
}

/// Windows platform settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WindowsPlatform {
    /// Console mode
    #[serde(default)]
    pub console: bool,
    /// Code signing certificate
    pub sign_cert: Option<PathBuf>,
    /// MSIX publisher
    pub msix_publisher: Option<String>,
}

/// macOS platform settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MacOSPlatform {
    /// Bundle identifier
    pub bundle_id: Option<String>,
    /// Code signing identity
    pub sign_identity: Option<String>,
    /// Entitlements file
    pub entitlements: Option<PathBuf>,
    /// Notarize credentials
    pub notarize: Option<NotarizeConfig>,
}

/// Notarization configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NotarizeConfig {
    /// Apple ID
    pub apple_id: Option<String>,
    /// Team ID
    pub team_id: Option<String>,
}

/// Linux platform settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LinuxPlatform {
    /// Desktop category
    pub category: Option<String>,
    /// Desktop file extras
    #[serde(default)]
    pub desktop_extras: HashMap<String, String>,
}

/// iOS platform settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IOSPlatform {
    /// Bundle identifier
    pub bundle_id: Option<String>,
    /// Development team
    pub team_id: Option<String>,
    /// Provisioning profile
    pub provisioning_profile: Option<String>,
    /// Minimum iOS version
    pub min_ios_version: Option<String>,
}

/// Android platform settings
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AndroidPlatform {
    /// Package name
    pub package_name: Option<String>,
    /// Minimum SDK version
    pub min_sdk: Option<u32>,
    /// Target SDK version
    pub target_sdk: Option<u32>,
    /// Keystore path
    pub keystore: Option<PathBuf>,
    /// Key alias
    pub key_alias: Option<String>,
}

/// Debug configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DebugConfig {
    /// Enable DevTools
    #[serde(default)]
    pub devtools: bool,
    /// Enable hot reload
    #[serde(default)]
    pub hot_reload: bool,
    /// Verbose logging
    #[serde(default)]
    pub verbose: bool,
}

/// Frontend bundle (collected files)
#[derive(Debug, Clone)]
pub struct FrontendBundle {
    /// Root path
    pub root: PathBuf,
    /// Files (relative path -> content)
    pub files: Vec<(String, Vec<u8>)>,
}

/// Python bundle
#[derive(Debug, Clone)]
pub struct PythonBundle {
    /// Python version
    pub version: String,
    /// Entry point
    pub entry: String,
    /// Python files
    pub files: Vec<(String, Vec<u8>)>,
    /// Runtime archive (if standalone)
    pub runtime: Option<Vec<u8>>,
}

/// Extension bundle
#[derive(Debug, Clone)]
pub struct ExtensionBundle {
    /// Extension ID
    pub id: String,
    /// Extension name
    pub name: Option<String>,
    /// Extension files
    pub files: Vec<(String, Vec<u8>)>,
}
