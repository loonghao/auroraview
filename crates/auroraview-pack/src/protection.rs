//! Python code protection integration for auroraview-pack
//!
//! This module provides integration with aurora-protect for
//! protecting Python code during the packing process using py2pyd compilation.
//!
//! ## How it works:
//! 1. Scan Python files in the project
//! 2. Use py2pyd (via uv + Cython) to compile `.py` → `.pyd`/`.so`
//! 3. Replace original `.py` files with compiled extensions
//!
//! ## Requirements:
//! - py2pyd handles all dependencies automatically via uv
//! - No manual installation of Cython or C compiler required

#[cfg(feature = "code-protection")]
use aurora_protect::{ProtectConfig, Protector};

use crate::{PackError, PackResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Protection configuration for Python code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    /// Enable code protection (py2pyd compilation)
    #[serde(default)]
    pub enabled: bool,

    /// Python executable path (default: auto-detect via uv)
    #[serde(default)]
    pub python_path: Option<String>,

    /// Python version to use (e.g., "3.11")
    #[serde(default)]
    pub python_version: Option<String>,

    /// Optimization level for C compiler (0, 1, 2, 3)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp: bool,

    /// Files/patterns to exclude from compilation
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Target DCC application (e.g., "maya", "houdini")
    #[serde(default)]
    pub target_dcc: Option<String>,

    /// Additional Python packages to install
    #[serde(default)]
    pub packages: Vec<String>,
}

fn default_optimization() -> u8 {
    2
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            python_path: None,
            python_version: None,
            optimization: default_optimization(),
            keep_temp: false,
            exclude: Vec::new(),
            target_dcc: None,
            packages: Vec::new(),
        }
    }
}

impl ProtectionConfig {
    /// Create a new protection config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable protection with default settings
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create a config for maximum optimization
    pub fn maximum() -> Self {
        Self {
            enabled: true,
            optimization: 3,
            ..Default::default()
        }
    }
}

/// Result of protecting Python code
#[derive(Debug)]
pub struct ProtectionResult {
    /// Number of files compiled
    pub files_compiled: usize,
    /// Number of files skipped
    pub files_skipped: usize,
    /// Total original size in bytes
    pub original_size: u64,
    /// Total compiled size in bytes
    pub compiled_size: u64,
    /// Extension suffix (.pyd on Windows, .so on Linux/Mac)
    pub extension_suffix: String,
}

/// Protect Python code in a directory using py2pyd compilation
///
/// This function:
/// 1. Scans for Python files
/// 2. Uses py2pyd to compile each to native extension (.pyd/.so)
/// 3. Copies extensions to output directory
#[cfg(feature = "code-protection")]
pub fn protect_python_code(
    input_dir: &Path,
    output_dir: &Path,
    config: &ProtectionConfig,
) -> PackResult<ProtectionResult> {
    if !config.enabled {
        return Err(PackError::Config("Protection is not enabled".to_string()));
    }

    tracing::info!(
        "Compiling Python code to native extensions: {}",
        input_dir.display()
    );

    // Convert to aurora-protect config
    let mut protect_config = ProtectConfig::new()
        .optimization(config.optimization)
        .keep_temp(config.keep_temp);

    if let Some(ref python_path) = config.python_path {
        protect_config = protect_config.python_path(python_path);
    }

    if let Some(ref python_version) = config.python_version {
        protect_config = protect_config.python_version(python_version);
    }

    if let Some(ref target_dcc) = config.target_dcc {
        protect_config = protect_config.target_dcc(target_dcc);
    }

    // Add exclude patterns
    for pattern in &config.exclude {
        protect_config = protect_config.exclude(pattern);
    }

    // Set packages
    protect_config.packages = config.packages.clone();

    // Create protector
    let protector = Protector::new(protect_config);

    // Compile directory
    let result = protector
        .protect_directory(input_dir, output_dir)
        .map_err(|e| PackError::Bundle(format!("py2pyd compilation failed: {}", e)))?;

    tracing::info!(
        "Compiled {} files ({} skipped), {:.2} KB -> {:.2} KB",
        result.compiled.len(),
        result.skipped.len(),
        result.total_original_size as f64 / 1024.0,
        result.total_compiled_size as f64 / 1024.0
    );

    Ok(ProtectionResult {
        files_compiled: result.compiled.len(),
        files_skipped: result.skipped.len(),
        original_size: result.total_original_size,
        compiled_size: result.total_compiled_size,
        extension_suffix: Protector::extension_suffix().to_string(),
    })
}

/// Stub implementation when code-protection feature is not enabled
#[cfg(not(feature = "code-protection"))]
pub fn protect_python_code(
    _input_dir: &Path,
    _output_dir: &Path,
    _config: &ProtectionConfig,
) -> PackResult<ProtectionResult> {
    Err(PackError::Config(
        "Code protection feature is not enabled. Rebuild with --features code-protection"
            .to_string(),
    ))
}

/// Check if code protection is available
pub fn is_protection_available() -> bool {
    cfg!(feature = "code-protection")
}

/// Check if py2pyd build tools are available
pub fn check_build_tools_available() -> PackResult<()> {
    #[cfg(feature = "code-protection")]
    {
        aurora_protect::py2pyd::verify_build_tools()
            .map(|_| ())
            .map_err(|e| PackError::Config(format!("Build tools not available: {}", e)))
    }
    #[cfg(not(feature = "code-protection"))]
    {
        Err(PackError::Config(
            "Code protection feature is not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_config_default() {
        let config = ProtectionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.optimization, 2);
        assert!(config.python_path.is_none());
    }

    #[test]
    fn test_protection_config_maximum() {
        let config = ProtectionConfig::maximum();
        assert!(config.enabled);
        assert_eq!(config.optimization, 3);
    }

    #[test]
    fn test_protection_config_enabled() {
        let config = ProtectionConfig::enabled();
        assert!(config.enabled);
    }
}
