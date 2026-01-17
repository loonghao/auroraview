//! DevTools configuration

use serde::{Deserialize, Serialize};

/// DevTools configuration
#[derive(Debug, Clone)]
pub struct DevToolsConfig {
    /// Enable DevTools access (F12)
    pub enabled: bool,
    /// Remote debugging port for CDP (0 = disabled)
    pub remote_debugging_port: u16,
    /// Auto-open DevTools on launch
    pub auto_open: bool,
    /// DevTools dock position
    pub dock_side: DockSide,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            remote_debugging_port: 0,
            auto_open: false,
            dock_side: DockSide::Right,
        }
    }
}

impl DevToolsConfig {
    /// Create a new config with DevTools enabled
    pub fn enabled() -> Self {
        Self::default()
    }

    /// Create a new config with DevTools disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set remote debugging port
    pub fn with_remote_debugging_port(mut self, port: u16) -> Self {
        self.remote_debugging_port = port;
        self
    }

    /// Set auto-open
    pub fn with_auto_open(mut self, auto_open: bool) -> Self {
        self.auto_open = auto_open;
        self
    }

    /// Set dock side
    pub fn with_dock_side(mut self, dock_side: DockSide) -> Self {
        self.dock_side = dock_side;
        self
    }

    /// Check if remote debugging is enabled
    pub fn is_remote_debugging_enabled(&self) -> bool {
        self.remote_debugging_port > 0
    }
}

/// DevTools dock position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockSide {
    /// Dock to the right side
    #[default]
    Right,
    /// Dock to the bottom
    Bottom,
    /// Dock to the left side
    Left,
    /// Undock into a separate window
    Undocked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DevToolsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.remote_debugging_port, 0);
        assert!(!config.auto_open);
        assert_eq!(config.dock_side, DockSide::Right);
    }

    #[test]
    fn test_config_builder() {
        let config = DevToolsConfig::enabled()
            .with_remote_debugging_port(9222)
            .with_auto_open(true)
            .with_dock_side(DockSide::Bottom);

        assert!(config.enabled);
        assert_eq!(config.remote_debugging_port, 9222);
        assert!(config.auto_open);
        assert_eq!(config.dock_side, DockSide::Bottom);
    }
}
