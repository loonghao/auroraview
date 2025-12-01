//! Pack module error types

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during pack operations
#[derive(Error, Debug)]
pub enum PackError {
    /// Invalid configuration
    #[error("Invalid pack configuration: {0}")]
    InvalidConfig(String),

    /// Frontend path not found
    #[error("Frontend path not found: {0}")]
    FrontendNotFound(PathBuf),

    /// Backend entry point invalid
    #[error("Invalid backend entry point: {0}. Expected format: 'module:function'")]
    InvalidBackendEntry(String),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Template generation error
    #[error("Template generation failed: {0}")]
    TemplateError(String),

    /// Build error
    #[error("Build failed: {0}")]
    BuildError(String),

    /// Python runtime not available (PyOxidizer not installed)
    #[error("Python runtime embedding requires PyOxidizer. Backend mode is not yet supported.")]
    PythonRuntimeNotAvailable,

    /// Unsupported platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}
