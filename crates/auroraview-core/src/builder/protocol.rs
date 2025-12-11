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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProtocolConfig::default();
        assert!(config.asset_root.is_none());
        assert!(!config.allow_file_protocol);
        assert!(!config.use_https_scheme);
        assert!(!config.has_auroraview_protocol());
    }

    #[test]
    fn test_with_asset_root() {
        let config = ProtocolConfig::new().with_asset_root(PathBuf::from("/assets"));
        assert!(config.asset_root.is_some());
        assert!(config.has_auroraview_protocol());
    }

    #[test]
    fn test_builder_pattern() {
        let config = ProtocolConfig::new()
            .with_asset_root(PathBuf::from("/assets"))
            .with_file_protocol(true)
            .with_https_scheme(true);

        assert!(config.asset_root.is_some());
        assert!(config.allow_file_protocol);
        assert!(config.use_https_scheme);
    }
}
