//! Bundle configuration module
//!
//! This module contains bundle-related configuration structs:
//! - `BundleConfig`: main bundle settings (icon, identifier, platform configs)
//! - `ProcessManifestConfig`: Python process settings
//! - `IsolationManifestConfig`: environment isolation settings
//! - `PyOxidizerManifestConfig`: PyOxidizer-specific settings
//! - `ProtectionManifestConfig`: code protection settings

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::common::{
    default_module_search_paths, IsolationConfig, LinuxPlatformConfig,
    MacOSPlatformConfig, PyOxidizerConfig, WindowsPlatformConfig,
};
use crate::error::PackResult;
use crate::protection::{EncryptionConfigPack, ProtectionConfig, ProtectionMethodConfig};

// ============================================================================
// Bundle Config
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
// Python Process Config
// ============================================================================

/// Python process manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

fn default_true() -> bool {
    true
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

impl From<ProcessManifestConfig> for crate::common::ProcessConfig {
    fn from(manifest: ProcessManifestConfig) -> Self {
        Self {
            console: manifest.console,
            module_search_paths: manifest.module_search_paths,
            filesystem_importer: manifest.filesystem_importer,
        }
    }
}

// ============================================================================
// Isolation Config
// ============================================================================

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

// ============================================================================
// PyOxidizer Config
// ============================================================================

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

// ============================================================================
// Protection Config
// ============================================================================

/// Code protection manifest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionManifestConfig {
    /// Enable code protection (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Protection method: "bytecode" (fast) or "py2pyd" (slow)
    #[serde(default)]
    pub method: ProtectionMethodConfig,

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
    pub encryption: EncryptionConfigPack,
}

fn default_optimization() -> u8 {
    2
}

impl Default for ProtectionManifestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: ProtectionMethodConfig::Bytecode,
            optimization: default_optimization(),
            keep_temp: false,
            exclude: Vec::new(),
            encryption: EncryptionConfigPack::default(),
        }
    }
}

impl ProtectionManifestConfig {
    /// Convert to ProtectionConfig
    pub fn to_protection_config(&self) -> ProtectionConfig {
        ProtectionConfig {
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
