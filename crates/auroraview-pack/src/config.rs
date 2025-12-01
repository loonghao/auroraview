//! Pack configuration types

use std::path::PathBuf;

/// Pack mode determines how the application is packaged
#[derive(Debug, Clone, PartialEq)]
pub enum PackMode {
    /// Pack a URL into standalone app (no embedded resources)
    Url { url: String },

    /// Pack frontend assets into standalone app (embedded HTML/CSS/JS)
    Frontend { path: PathBuf },

    /// Pack frontend + Python backend (requires PyOxidizer)
    FullStack {
        frontend_path: PathBuf,
        backend_entry: String, // e.g., "myapp.main:run"
    },
}

/// Configuration for pack operation
#[derive(Debug, Clone)]
pub struct PackConfig {
    /// Pack mode (URL, Frontend, or FullStack)
    pub mode: PackMode,

    /// Output executable name (without extension)
    pub output_name: String,

    /// Output directory
    pub output_dir: PathBuf,

    /// Window title
    pub window_title: String,

    /// Window width
    pub window_width: u32,

    /// Window height
    pub window_height: u32,

    /// Window icon path (optional)
    pub icon_path: Option<PathBuf>,

    /// Target platform (default: current)
    pub target_platform: TargetPlatform,

    /// Enable debug mode in packed app
    pub debug: bool,

    /// Allow opening new windows
    pub allow_new_window: bool,

    /// Keep window always on top
    pub always_on_top: bool,
}

/// Target platform for cross-compilation
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TargetPlatform {
    #[default]
    Current,
    Windows,
    MacOS,
    Linux,
}

impl PackConfig {
    /// Create a new PackConfig for URL mode
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            mode: PackMode::Url { url: url.into() },
            output_name: "app".to_string(),
            output_dir: PathBuf::from("."),
            window_title: "AuroraView App".to_string(),
            window_width: 1024,
            window_height: 768,
            icon_path: None,
            target_platform: TargetPlatform::default(),
            debug: false,
            allow_new_window: false,
            always_on_top: false,
        }
    }

    /// Create a new PackConfig for frontend mode
    pub fn frontend(path: impl Into<PathBuf>) -> Self {
        Self {
            mode: PackMode::Frontend { path: path.into() },
            output_name: "app".to_string(),
            output_dir: PathBuf::from("."),
            window_title: "AuroraView App".to_string(),
            window_width: 1024,
            window_height: 768,
            icon_path: None,
            target_platform: TargetPlatform::default(),
            debug: false,
            allow_new_window: false,
            always_on_top: false,
        }
    }

    /// Create a new PackConfig for full-stack mode
    pub fn fullstack(frontend: impl Into<PathBuf>, backend: impl Into<String>) -> Self {
        Self {
            mode: PackMode::FullStack {
                frontend_path: frontend.into(),
                backend_entry: backend.into(),
            },
            output_name: "app".to_string(),
            output_dir: PathBuf::from("."),
            window_title: "AuroraView App".to_string(),
            window_width: 1024,
            window_height: 768,
            icon_path: None,
            target_platform: TargetPlatform::default(),
            debug: false,
            allow_new_window: false,
            always_on_top: false,
        }
    }

    /// Set output name
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.output_name = name.into();
        self
    }

    /// Set window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.window_title = title.into();
        self
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }
}
