//! Window configuration module
//!
//! This module contains window-related configuration structs:
//! - `ManifestWindowConfig`: window settings from manifest
//! - `StartPosition`: window start position (center or specific coordinates)

use serde::{Deserialize, Serialize};

use crate::common::{WindowConfig, WindowStartPosition};

// ============================================================================
// Window Configuration
// ============================================================================

/// Window configuration for manifest (supports string position like "center")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWindowConfig {
    /// Window width
    #[serde(default = "default_width")]
    pub width: u32,

    /// Window height
    #[serde(default = "default_height")]
    pub height: u32,

    /// Minimum window width
    #[serde(default)]
    pub min_width: Option<u32>,

    /// Minimum window height
    #[serde(default)]
    pub min_height: Option<u32>,

    /// Maximum window width
    #[serde(default)]
    pub max_width: Option<u32>,

    /// Maximum window height
    #[serde(default)]
    pub max_height: Option<u32>,

    /// Window is resizable
    #[serde(default = "default_true")]
    pub resizable: bool,

    /// Window has no frame/decorations
    #[serde(default)]
    pub frameless: bool,

    /// Window background is transparent
    #[serde(default)]
    pub transparent: bool,

    /// Window stays on top
    #[serde(default)]
    pub always_on_top: bool,

    /// Start position: "center" or { x, y }
    #[serde(default)]
    pub start_position: StartPosition,

    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,

    /// Maximized on start
    #[serde(default)]
    pub maximized: bool,

    /// Visible on start
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

fn default_true() -> bool {
    true
}

impl Default for ManifestWindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            resizable: true,
            frameless: false,
            transparent: false,
            always_on_top: false,
            start_position: StartPosition::default(),
            fullscreen: false,
            maximized: false,
            visible: true,
        }
    }
}

impl From<ManifestWindowConfig> for WindowConfig {
    fn from(manifest: ManifestWindowConfig) -> Self {
        Self {
            title: "AuroraView App".to_string(), // Default title, will be overwritten by get_window_config()
            width: manifest.width,
            height: manifest.height,
            min_width: manifest.min_width,
            min_height: manifest.min_height,
            max_width: manifest.max_width,
            max_height: manifest.max_height,
            start_position: manifest.start_position.into(),
            resizable: manifest.resizable,
            frameless: manifest.frameless,
            transparent: manifest.transparent,
            always_on_top: manifest.always_on_top,
            fullscreen: manifest.fullscreen,
            maximized: manifest.maximized,
            visible: manifest.visible,
            strict_csp: false,
        }
    }
}

// ============================================================================
// Start Position
// ============================================================================

/// Window start position (supports string like "center" for TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StartPosition {
    /// Specific position
    Position { x: i32, y: i32 },
    /// Named position (center, etc.)
    Named(String),
}

impl Default for StartPosition {
    fn default() -> Self {
        StartPosition::Named("center".to_string())
    }
}

impl StartPosition {
    /// Check if this is the center position
    pub fn is_center(&self) -> bool {
        matches!(self, StartPosition::Named(s) if s == "center")
    }
}

impl From<StartPosition> for WindowStartPosition {
    fn from(pos: StartPosition) -> Self {
        match pos {
            StartPosition::Named(s) if s == "center" => WindowStartPosition::Center,
            StartPosition::Named(_) => WindowStartPosition::Center,
            StartPosition::Position { x, y } => WindowStartPosition::Position { x, y },
        }
    }
}

impl From<WindowStartPosition> for StartPosition {
    fn from(pos: WindowStartPosition) -> Self {
        match pos {
            WindowStartPosition::Center => StartPosition::Named("center".to_string()),
            WindowStartPosition::Position { x, y } => StartPosition::Position { x, y },
        }
    }
}
