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

    /// vx.ensure validation failed
    #[error("vx.ensure validation failed: {0}")]
    VxEnsureFailed(String),
}

impl Clone for PackError {
    fn clone(&self) -> Self {
        match self {
            PackError::Io(_) => PackError::Config("IO error".to_string()),
            PackError::Config(s) => PackError::Config(s.clone()),
            PackError::InvalidUrl(s) => PackError::InvalidUrl(s.clone()),
            PackError::FrontendNotFound(p) => PackError::FrontendNotFound(p.clone()),
            PackError::InvalidManifest(s) => PackError::InvalidManifest(s.clone()),
            PackError::TomlParse(_) => PackError::Config("TOML parse error".to_string()),
            PackError::Json(_) => PackError::Config("JSON error".to_string()),
            PackError::InvalidOverlay(s) => PackError::InvalidOverlay(s.clone()),
            PackError::AssetNotFound(p) => PackError::AssetNotFound(p.clone()),
            PackError::Bundle(s) => PackError::Bundle(s.clone()),
            PackError::Icon(s) => PackError::Icon(s.clone()),
            PackError::Compression(s) => PackError::Compression(s.clone()),
            PackError::Build(s) => PackError::Build(s.clone()),
            PackError::Download(s) => PackError::Download(s.clone()),
            PackError::ResourceEdit(s) => PackError::ResourceEdit(s.clone()),
            PackError::VxEnsureFailed(s) => PackError::VxEnsureFailed(s.clone()),
        }
    }
}
