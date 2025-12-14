//! WebContext creation utilities
//!
//! This module provides shared logic for creating wry::WebContext
//! with proper data directory configuration.

use std::path::PathBuf;

#[cfg(feature = "wry-builder")]
use wry::WebContext;

/// Configuration for WebContext creation
#[derive(Debug, Clone, Default)]
pub struct WebContextConfig {
    /// Custom data directory path (highest priority)
    pub data_directory: Option<PathBuf>,
    /// Shared warmup folder path (used if data_directory is None)
    pub shared_warmup_folder: Option<PathBuf>,
}

impl WebContextConfig {
    /// Create a new WebContextConfig
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom data directory
    pub fn with_data_directory(mut self, path: PathBuf) -> Self {
        self.data_directory = Some(path);
        self
    }

    /// Set shared warmup folder
    pub fn with_shared_warmup_folder(mut self, path: PathBuf) -> Self {
        self.shared_warmup_folder = Some(path);
        self
    }
}

/// Create a WebContext with the given configuration
///
/// Priority:
/// 1. Custom data_directory if provided
/// 2. Shared warmup folder if available
/// 3. System default
///
/// # Arguments
/// * `config` - Configuration for WebContext creation
///
/// # Returns
/// A configured WebContext instance
#[cfg(feature = "wry-builder")]
pub fn create_web_context(config: &WebContextConfig) -> WebContext {
    if let Some(ref data_dir) = config.data_directory {
        tracing::info!("Using custom data directory: {:?}", data_dir);
        WebContext::new(Some(data_dir.clone()))
    } else if let Some(ref shared_dir) = config.shared_warmup_folder {
        tracing::info!("Using shared warmup data directory: {:?}", shared_dir);
        WebContext::new(Some(shared_dir.clone()))
    } else {
        tracing::debug!("Using default data directory");
        WebContext::default()
    }
}
