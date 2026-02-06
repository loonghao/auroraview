//! Python code protection integration for auroraview-pack
//!
//! This module provides integration with aurora-protect for
//! protecting Python code during the packing process.
//!
//! ## Protection Methods:
//!
//! ### 1. Bytecode Encryption (default, fast)
//! - Compiles `.py` → `.pyc` bytecode
//! - Encrypts with ECC + AES-256-GCM
//! - Decrypts at runtime via bootstrap loader
//! - No C compiler required
//!
//! ### 2. py2pyd Compilation (slow, maximum protection)
//! - Compiles `.py` → native `.pyd`/`.so` via Cython
//! - Requires C/C++ toolchain
//! - Each file creates a new virtual environment (slow)
//!
//! ## Requirements:
//! - Bytecode: Python only (fast)
//! - py2pyd: C compiler + Cython via uv (slow)

#[cfg(feature = "code-protection")]
use auroraview_protect::{
    protect_with_bytecode, EncryptionConfig, ProtectConfig, ProtectionMethod, Protector,
};

use crate::{PackError, PackResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Protection method for Python code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionMethodConfig {
    /// Bytecode encryption (ECC + AES-256-GCM) - fast, no C compiler needed
    #[default]
    Bytecode,
    /// Native compilation via py2pyd/Cython - slow, requires C compiler
    Py2Pyd,
}

/// Encryption configuration for bytecode protection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncryptionConfigPack {
    /// Enable encryption (default: true when method is bytecode)
    #[serde(default = "default_encryption_enabled")]
    pub enabled: bool,

    /// Algorithm: "x25519" (fast) or "p256" (FIPS compliant)
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

fn default_encryption_enabled() -> bool {
    true
}

fn default_algorithm() -> String {
    "x25519".to_string()
}

/// Protection configuration for Python code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    /// Enable code protection
    #[serde(default)]
    pub enabled: bool,

    /// Protection method: "bytecode" (fast) or "py2pyd" (slow)
    #[serde(default)]
    pub method: ProtectionMethodConfig,

    /// Python executable path (default: auto-detect via uv)
    #[serde(default)]
    pub python_path: Option<String>,

    /// Python version to use (e.g., "3.11")
    #[serde(default)]
    pub python_version: Option<String>,

    /// Optimization level (0-2 for bytecode, 0-3 for py2pyd)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp: bool,

    /// Files/patterns to exclude from protection
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Target DCC application (e.g., "maya", "houdini")
    #[serde(default)]
    pub target_dcc: Option<String>,

    /// Additional Python packages to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Encryption settings (for bytecode method)
    #[serde(default)]
    pub encryption: EncryptionConfigPack,
}

fn default_optimization() -> u8 {
    2
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            method: ProtectionMethodConfig::Bytecode,
            python_path: None,
            python_version: None,
            optimization: default_optimization(),
            keep_temp: false,
            exclude: Vec::new(),
            target_dcc: None,
            packages: Vec::new(),
            encryption: EncryptionConfigPack::default(),
        }
    }
}

impl ProtectionConfig {
    /// Create a new protection config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable protection with bytecode encryption (fast)
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            method: ProtectionMethodConfig::Bytecode,
            encryption: EncryptionConfigPack {
                enabled: true,
                algorithm: "x25519".to_string(),
            },
            ..Default::default()
        }
    }

    /// Create a config for maximum protection (py2pyd, slow)
    pub fn maximum() -> Self {
        Self {
            enabled: true,
            method: ProtectionMethodConfig::Py2Pyd,
            optimization: 3,
            ..Default::default()
        }
    }
}

/// Result of protecting Python code
#[derive(Debug)]
pub struct ProtectionResult {
    /// Number of files protected
    pub files_compiled: usize,
    /// Number of files skipped
    pub files_skipped: usize,
    /// Total original size in bytes
    pub original_size: u64,
    /// Total protected size in bytes
    pub compiled_size: u64,
    /// Protection method used
    pub method: ProtectionMethodConfig,
    /// Path to bootstrap file (for bytecode method)
    pub bootstrap_path: Option<std::path::PathBuf>,
}

/// Protect Python code in a directory
///
/// Supports two methods:
/// - `bytecode`: Fast encryption (ECC + AES-256-GCM)
/// - `py2pyd`: Native compilation via Cython (slow)
#[cfg(feature = "code-protection")]
pub fn protect_python_code(
    input_dir: &Path,
    output_dir: &Path,
    config: &ProtectionConfig,
) -> PackResult<ProtectionResult> {
    if !config.enabled {
        return Err(PackError::Config("Protection is not enabled".to_string()));
    }

    match config.method {
        ProtectionMethodConfig::Bytecode => {
            protect_with_bytecode_method(input_dir, output_dir, config)
        }
        ProtectionMethodConfig::Py2Pyd => protect_with_py2pyd_method(input_dir, output_dir, config),
    }
}

/// Protect using bytecode encryption (fast)
#[cfg(feature = "code-protection")]
fn protect_with_bytecode_method(
    input_dir: &Path,
    output_dir: &Path,
    config: &ProtectionConfig,
) -> PackResult<ProtectionResult> {
    tracing::info!(
        "Encrypting Python bytecode (ECC + AES-256-GCM): {}",
        input_dir.display()
    );

    // Build aurora-protect config
    let protect_config = ProtectConfig {
        method: ProtectionMethod::Bytecode,
        python_path: config.python_path.clone(),
        python_version: config.python_version.clone(),
        optimization: config.optimization,
        keep_temp: config.keep_temp,
        packages: config.packages.clone(),
        exclude: config.exclude.clone(),
        target_dcc: config.target_dcc.clone(),
        encryption: EncryptionConfig {
            enabled: config.encryption.enabled,
            algorithm: config.encryption.algorithm.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    let result = protect_with_bytecode(input_dir, output_dir, &protect_config)
        .map_err(|e| PackError::Bundle(format!("Bytecode encryption failed: {}", e)))?;

    tracing::info!(
        "Encrypted {} files ({} skipped), {:.2} KB source -> {:.2} KB encrypted",
        result.files_compiled,
        result.files_skipped,
        result.source_size as f64 / 1024.0,
        result.encrypted_size as f64 / 1024.0
    );

    Ok(ProtectionResult {
        files_compiled: result.files_compiled,
        files_skipped: result.files_skipped,
        original_size: result.source_size,
        compiled_size: result.encrypted_size,
        method: ProtectionMethodConfig::Bytecode,
        bootstrap_path: Some(result.bootstrap_path),
    })
}

/// Protect using py2pyd compilation (slow)
#[cfg(feature = "code-protection")]
fn protect_with_py2pyd_method(
    input_dir: &Path,
    output_dir: &Path,
    config: &ProtectionConfig,
) -> PackResult<ProtectionResult> {
    tracing::info!(
        "Compiling Python to native extensions (py2pyd): {}",
        input_dir.display()
    );

    // Convert to aurora-protect config
    let mut protect_config = ProtectConfig::new()
        .optimization(config.optimization)
        .keep_temp(config.keep_temp);

    protect_config.method = ProtectionMethod::Py2Pyd;

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
        method: ProtectionMethodConfig::Py2Pyd,
        bootstrap_path: None,
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

/// Check if build tools are available for the specified method
pub fn check_build_tools_available(method: ProtectionMethodConfig) -> PackResult<()> {
    match method {
        ProtectionMethodConfig::Bytecode => {
            // Bytecode encryption only needs Python, which is handled by uv
            Ok(())
        }
        ProtectionMethodConfig::Py2Pyd => {
            #[cfg(feature = "code-protection")]
            {
                auroraview_protect::py2pyd::verify_build_tools()
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_config_default() {
        let config = ProtectionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.method, ProtectionMethodConfig::Bytecode);
        assert_eq!(config.optimization, 2);
        assert!(config.python_path.is_none());
    }

    #[test]
    fn test_protection_config_maximum() {
        let config = ProtectionConfig::maximum();
        assert!(config.enabled);
        assert_eq!(config.method, ProtectionMethodConfig::Py2Pyd);
        assert_eq!(config.optimization, 3);
    }

    #[test]
    fn test_protection_config_enabled() {
        let config = ProtectionConfig::enabled();
        assert!(config.enabled);
        assert_eq!(config.method, ProtectionMethodConfig::Bytecode);
        assert!(config.encryption.enabled);
    }

    #[test]
    fn test_bytecode_tools_available() {
        // Bytecode method should always pass (no C compiler needed)
        assert!(check_build_tools_available(ProtectionMethodConfig::Bytecode).is_ok());
    }
}
