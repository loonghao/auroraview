//! Backend configuration module
//!
//! This module contains backend-related configuration structs:
//! - `BackendType`: enum for backend type (None/Python/Go/Rust/Node)
//! - `BackendConfig`: main backend configuration
//! - `BackendPythonConfig`, `BackendGoConfig`, `BackendRustConfig`, `BackendNodeConfig`
//! - `BackendProcessConfig`: common process settings
//! - `HealthCheckConfig`: health check settings

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::common::{default_module_search_paths, default_optimize, default_python_version, BundleStrategy};
use crate::config::PythonBundleConfig;

// ============================================================================
// Backend Type Enum
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

// ============================================================================
// Backend Config
// ============================================================================

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

// ============================================================================
// Python Backend Config
// ============================================================================

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

    /// Only allow dependency installation through `vx uv pip` (no fallback)
    #[serde(default)]
    pub pip_via_vx_only: bool,

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

fn default_strategy() -> String {
    "standalone".to_string()
}

impl Default for BackendPythonConfig {
    fn default() -> Self {
        Self {
            version: default_python_version(),
            entry_point: None,
            packages: Vec::new(),
            requirements: None,
            pip_via_vx_only: false,
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
    pub fn to_bundle_config(&self, base_dir: &Path, normalize: fn(&PathBuf) -> PathBuf) -> PythonBundleConfig {
        let resolve_path = |p: &PathBuf| -> PathBuf {
            let joined = if p.is_absolute() {
                p.clone()
            } else {
                base_dir.join(p)
            };
            normalize(&joined)
        };

        PythonBundleConfig {
            entry_point: self
                .entry_point
                .clone()
                .unwrap_or_else(|| "main:run".to_string()),
            include_paths: self.include_paths.iter().map(resolve_path).collect(),
            packages: self.packages.clone(),
            requirements: self.requirements.as_ref().map(resolve_path),
            pip_via_vx_only: self.pip_via_vx_only,
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

// ============================================================================
// Go Backend Config
// ============================================================================

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

// ============================================================================
// Rust Backend Config
// ============================================================================

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

// ============================================================================
// Node.js Backend Config
// ============================================================================

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

// ============================================================================
// Common Process Config
// ============================================================================

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

// ============================================================================
// Health Check Config
// ============================================================================

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
