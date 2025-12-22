//! Error types for auroraview-pack

use std::path::PathBuf;
use thiserror::Error;

/// Result type for pack operations
pub type PackResult<T> = Result<T, PackError>;

/// Errors that can occur during packing
#[derive(Error, Debug)]
pub enum PackError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Frontend path not found
    #[error("Frontend path not found: {0}")]
    FrontendNotFound(PathBuf),

    /// Invalid manifest file
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Overlay format error
    #[error("Invalid overlay format: {0}")]
    InvalidOverlay(String),

    /// Asset not found
    #[error("Asset not found: {0}")]
    AssetNotFound(PathBuf),

    /// Bundle error
    #[error("Bundle error: {0}")]
    Bundle(String),

    /// Icon processing error
    #[error("Icon error: {0}")]
    Icon(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Build error (PyOxidizer, etc.)
    #[error("Build error: {0}")]
    Build(String),

    /// Download error
    #[error("Download error: {0}")]
    Download(String),

    /// Resource editing error (icon, subsystem, etc.)
    #[error("Resource edit error: {0}")]
    ResourceEdit(String),
}
