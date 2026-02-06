//! Error types for aurora-protect

use thiserror::Error;

/// Protection error types
#[derive(Debug, Error)]
pub enum ProtectError {
    /// py2pyd compilation failed
    #[error("Compilation failed: {0}")]
    Compilation(String),

    /// Python bytecode compilation failed
    #[error("Python compile failed: {0}")]
    PythonCompile(String),

    /// Encryption failed
    #[error("Encryption failed: {0}")]
    Encryption(String),

    /// Decryption failed
    #[error("Decryption failed: {0}")]
    Decryption(String),

    /// Obfuscation failed
    #[error("Obfuscation failed: {0}")]
    Obfuscation(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// py2pyd error
    #[error("py2pyd error: {0}")]
    Py2Pyd(#[from] anyhow::Error),

    /// Key generation error
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKey(String),
}

/// Result type for protection operations
pub type ProtectResult<T> = Result<T, ProtectError>;
