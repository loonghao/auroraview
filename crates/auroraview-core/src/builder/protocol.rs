//! Shared protocol configuration for WebView
//!
//! This module provides protocol registration helpers that can be used
//! in both standalone and DCC embedded modes.

use std::path::PathBuf;

/// Protocol configuration for WebView
#[derive(Debug, Clone, Default)]
pub struct ProtocolConfig {
    /// Asset root directory for auroraview:// protocol
    pub asset_root: Option<PathBuf>,
    /// Whether to enable file:// protocol
    pub allow_file_protocol: bool,
    /// Whether to use HTTPS scheme for custom protocols (Windows only)
    pub use_https_scheme: bool,
}

impl ProtocolConfig {
    /// Create a new protocol configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the asset root directory
    pub fn with_asset_root(mut self, path: PathBuf) -> Self {
        self.asset_root = Some(path);
        self
    }

    /// Enable file:// protocol
    pub fn with_file_protocol(mut self, enabled: bool) -> Self {
        self.allow_file_protocol = enabled;
        self
    }

    /// Use HTTPS scheme for custom protocols
    pub fn with_https_scheme(mut self, enabled: bool) -> Self {
        self.use_https_scheme = enabled;
        self
    }

    /// Check if auroraview:// protocol should be registered
    pub fn has_auroraview_protocol(&self) -> bool {
        self.asset_root.is_some()
    }
}
