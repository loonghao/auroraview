//! Configuration for Python code protection

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Protection method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionMethod {
    /// Compile `.py` -> native extension (`.pyd`/`.so`) via py2pyd
    Py2Pyd,
    /// Compile `.py` -> Python bytecode, encrypt, and load via runtime bootstrap
    Bytecode,
}

fn default_method() -> ProtectionMethod {
    // Default to bytecode encryption to avoid requiring a C/C++ toolchain.
    ProtectionMethod::Bytecode
}

/// Encryption configuration for hybrid encryption (ECC + AES-256-GCM)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable bytecode encryption
    #[serde(default)]
    pub enabled: bool,

    /// Algorithm for asymmetric encryption
    /// - "x25519" (default): X25519 ECDH + ChaCha20-Poly1305 (fast, modern)
    /// - "p256": NIST P-256 ECDH + AES-256-GCM (FIPS compliant)
    #[serde(default = "default_algorithm")]
    pub algorithm: String,

    /// Pre-generated public key (hex encoded)
    /// If not provided, a new key pair will be generated at pack time
    #[serde(default)]
    pub public_key: Option<String>,

    /// Pre-generated private key (hex encoded)
    /// WARNING: Only use for testing! In production, let the packer generate keys
    #[serde(default)]
    pub private_key: Option<String>,

    /// Output path for generated key pair (optional)
    /// If set, the generated key pair will be saved to this file
    #[serde(default)]
    pub key_output_path: Option<String>,
}

fn default_algorithm() -> String {
    "x25519".to_string()
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: default_algorithm(),
            public_key: None,
            private_key: None,
            key_output_path: None,
        }
    }
}

impl EncryptionConfig {
    /// Create a new encryption config with encryption enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Use X25519 algorithm (default, fast)
    pub fn with_x25519(mut self) -> Self {
        self.algorithm = "x25519".to_string();
        self
    }

    /// Use P-256 algorithm (FIPS compliant)
    pub fn with_p256(mut self) -> Self {
        self.algorithm = "p256".to_string();
        self
    }

    /// Set pre-generated key pair
    pub fn with_keys(mut self, public_key: String, private_key: String) -> Self {
        self.public_key = Some(public_key);
        self.private_key = Some(private_key);
        self
    }
}

/// Configuration for Python code protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectConfig {
    /// Protection method
    #[serde(default = "default_method")]
    pub method: ProtectionMethod,

    /// Python executable path (default: auto-detect via uv)
    #[serde(default)]
    pub python_path: Option<String>,

    /// Python version to use (e.g., "3.11")
    #[serde(default)]
    pub python_version: Option<String>,

    /// Optimization level
    /// - For `py2pyd`: C compiler optimization (0..=3)
    /// - For `bytecode`: Python `compile(..., optimize=...)` (0..=2)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp: bool,

    /// Additional Python packages to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Files/patterns to exclude from protection
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    /// Target DCC application (e.g., "maya", "houdini")
    #[serde(default)]
    pub target_dcc: Option<String>,

    /// Encryption configuration for hybrid encryption
    #[serde(default)]
    pub encryption: EncryptionConfig,

    /// Bytecode encryption key (in-memory only)
    #[serde(skip)]
    pub bytecode_key: Option<[u8; 32]>,

    /// Enable anti-debugging checks
    #[serde(default)]
    pub anti_debug: bool,

    /// Enable integrity verification
    #[serde(default)]
    pub integrity_check: bool,

    /// License expiration date (ISO 8601 format)
    #[serde(default)]
    pub expires_at: Option<String>,

    /// Machine IDs allowed to run the code
    #[serde(default)]
    pub bind_machines: Vec<String>,
}

fn default_optimization() -> u8 {
    2
}

fn default_exclude() -> Vec<String> {
    vec![
        "**/__pycache__/**".to_string(),
        "**/*.pyc".to_string(),
        "**/test_*.py".to_string(),
        "**/*_test.py".to_string(),
        "**/tests/**".to_string(),
        "**/setup.py".to_string(),
    ]
}

impl Default for ProtectConfig {
    fn default() -> Self {
        Self {
            method: default_method(),
            python_path: None,
            python_version: None,
            optimization: default_optimization(),
            keep_temp: false,
            packages: vec![],
            exclude: default_exclude(),
            target_dcc: None,
            encryption: EncryptionConfig::default(),
            bytecode_key: None,
            anti_debug: false,
            integrity_check: false,
            expires_at: None,
            bind_machines: vec![],
        }
    }
}

impl ProtectConfig {
    /// Create new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set Python path
    pub fn python_path(mut self, path: impl Into<String>) -> Self {
        self.python_path = Some(path.into());
        self
    }

    /// Set Python version
    pub fn python_version(mut self, version: impl Into<String>) -> Self {
        self.python_version = Some(version.into());
        self
    }

    /// Set optimization level
    pub fn optimization(mut self, level: u8) -> Self {
        self.optimization = level.min(3);
        self
    }

    /// Keep temporary files
    pub fn keep_temp(mut self, keep: bool) -> Self {
        self.keep_temp = keep;
        self
    }

    /// Add exclude pattern
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    /// Set target DCC
    pub fn target_dcc(mut self, dcc: impl Into<String>) -> Self {
        self.target_dcc = Some(dcc.into());
        self
    }

    /// Convert to py2pyd CompileConfig
    pub fn to_py2pyd_config(&self) -> py2pyd::CompileConfig {
        py2pyd::CompileConfig {
            python_path: self.python_path.as_ref().map(PathBuf::from),
            python_version: self.python_version.clone(),
            optimize_level: self.optimization,
            keep_temp_files: self.keep_temp,
            target_dcc: self.target_dcc.clone(),
            packages: self.packages.clone(),
        }
    }
}
