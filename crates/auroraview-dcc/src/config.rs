//! DCC configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// DCC application type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DccType {
    Maya,
    Houdini,
    Nuke,
    Blender,
    Max3ds,
    Unreal,
    #[default]
    Unknown,
}

impl DccType {
    /// Detect DCC type from environment
    pub fn detect() -> Self {
        // Check for DCC-specific environment variables
        if std::env::var("MAYA_LOCATION").is_ok() {
            return Self::Maya;
        }
        if std::env::var("HFS").is_ok() {
            return Self::Houdini;
        }
        if std::env::var("NUKE_PATH").is_ok() {
            return Self::Nuke;
        }
        if std::env::var("BLENDER_SYSTEM_SCRIPTS").is_ok() {
            return Self::Blender;
        }
        Self::Unknown
    }

    /// Get DCC name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Maya => "Maya",
            Self::Houdini => "Houdini",
            Self::Nuke => "Nuke",
            Self::Blender => "Blender",
            Self::Max3ds => "3ds Max",
            Self::Unreal => "Unreal Engine",
            Self::Unknown => "Unknown",
        }
    }
}

/// DCC WebView configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DccConfig {
    /// Window/panel title
    pub title: String,

    /// Initial width
    pub width: u32,

    /// Initial height
    pub height: u32,

    /// Initial URL
    pub url: Option<String>,

    /// Initial HTML content
    pub html: Option<String>,

    /// Parent window handle (HWND on Windows)
    pub parent_hwnd: Option<isize>,

    /// DCC type (auto-detected if not set)
    pub dcc_type: DccType,

    /// DCC version string
    pub dcc_version: Option<String>,

    /// Panel name for dock registration
    pub panel_name: Option<String>,

    /// Enable DevTools
    pub devtools: bool,

    /// User data directory
    pub data_dir: Option<PathBuf>,

    /// CDP debugging port
    pub debug_port: u16,

    /// Background color (RGBA)
    pub background_color: Option<(u8, u8, u8, u8)>,
}

impl Default for DccConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView".to_string(),
            width: 400,
            height: 600,
            url: None,
            html: None,
            parent_hwnd: None,
            dcc_type: DccType::detect(),
            dcc_version: None,
            panel_name: None,
            devtools: cfg!(debug_assertions),
            data_dir: None,
            debug_port: 0,
            background_color: None,
        }
    }
}

impl DccConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn parent_hwnd(mut self, hwnd: isize) -> Self {
        self.parent_hwnd = Some(hwnd);
        self
    }

    pub fn dcc_type(mut self, dcc: DccType) -> Self {
        self.dcc_type = dcc;
        self
    }

    pub fn panel_name(mut self, name: impl Into<String>) -> Self {
        self.panel_name = Some(name.into());
        self
    }

    pub fn devtools(mut self, enable: bool) -> Self {
        self.devtools = enable;
        self
    }

    pub fn debug_port(mut self, port: u16) -> Self {
        self.debug_port = port;
        self
    }
}
