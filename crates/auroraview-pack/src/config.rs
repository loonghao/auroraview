//! Pack configuration types
//!
//! This module provides configuration structures for packaging AuroraView applications
//! into standalone executables. It supports three modes:
//!
//! - **URL Mode**: Wrap a remote URL into a desktop app
//! - **Frontend Mode**: Bundle local HTML/CSS/JS assets
//! - **FullStack Mode**: Bundle frontend + Python backend (PyOxidizer)

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

impl PackMode {
    /// Returns true if this mode embeds local assets
    pub fn embeds_assets(&self) -> bool {
        matches!(self, PackMode::Frontend { .. } | PackMode::FullStack { .. })
    }

    /// Returns true if this mode requires PyOxidizer
    pub fn requires_pyoxidizer(&self) -> bool {
        matches!(self, PackMode::FullStack { .. })
    }

    /// Get the mode name as a string
    pub fn name(&self) -> &'static str {
        match self {
            PackMode::Url { .. } => "url",
            PackMode::Frontend { .. } => "frontend",
            PackMode::FullStack { .. } => "fullstack",
        }
    }
}

/// Window start position mode
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WindowStartPosition {
    /// Center the window on screen
    #[default]
    Center,
    /// Position at specific coordinates
    Position { x: i32, y: i32 },
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

    /// Minimum window width (optional)
    pub min_width: Option<u32>,

    /// Minimum window height (optional)
    pub min_height: Option<u32>,

    /// Window start position
    pub start_position: WindowStartPosition,

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

    /// Enable window resizing
    pub resizable: bool,

    /// Enable window frame/decorations
    pub frameless: bool,

    /// Enable transparent window
    pub transparent: bool,

    /// Custom user agent string (optional)
    pub user_agent: Option<String>,

    /// Inject custom JavaScript on page load (optional)
    pub inject_js: Option<String>,

    /// Inject custom CSS on page load (optional)
    pub inject_css: Option<String>,
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

impl TargetPlatform {
    /// Get Rust target triple for this platform
    pub fn target_triple(&self) -> Option<&'static str> {
        match self {
            TargetPlatform::Current => None,
            TargetPlatform::Windows => Some("x86_64-pc-windows-msvc"),
            TargetPlatform::MacOS => Some("x86_64-apple-darwin"),
            TargetPlatform::Linux => Some("x86_64-unknown-linux-gnu"),
        }
    }
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            mode: PackMode::Url {
                url: "https://example.com".to_string(),
            },
            output_name: "app".to_string(),
            output_dir: PathBuf::from("."),
            window_title: "AuroraView App".to_string(),
            window_width: 1024,
            window_height: 768,
            min_width: None,
            min_height: None,
            start_position: WindowStartPosition::default(),
            icon_path: None,
            target_platform: TargetPlatform::default(),
            debug: false,
            allow_new_window: false,
            always_on_top: false,
            resizable: true,
            frameless: false,
            transparent: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
        }
    }
}

impl PackConfig {
    /// Create a new PackConfig for URL mode
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            mode: PackMode::Url { url: url.into() },
            ..Default::default()
        }
    }

    /// Create a new PackConfig for frontend mode
    pub fn frontend(path: impl Into<PathBuf>) -> Self {
        Self {
            mode: PackMode::Frontend { path: path.into() },
            ..Default::default()
        }
    }

    /// Create a new PackConfig for full-stack mode
    pub fn fullstack(frontend: impl Into<PathBuf>, backend: impl Into<String>) -> Self {
        Self {
            mode: PackMode::FullStack {
                frontend_path: frontend.into(),
                backend_entry: backend.into(),
            },
            ..Default::default()
        }
    }

    /// Set output name
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.output_name = name.into();
        self
    }

    /// Set output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
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

    /// Set minimum window size
    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Set window icon
    pub fn with_icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set target platform for cross-compilation
    pub fn with_target(mut self, platform: TargetPlatform) -> Self {
        self.target_platform = platform;
        self
    }

    /// Enable always on top
    pub fn with_always_on_top(mut self, enabled: bool) -> Self {
        self.always_on_top = enabled;
        self
    }

    /// Enable frameless window
    pub fn with_frameless(mut self, enabled: bool) -> Self {
        self.frameless = enabled;
        self
    }

    /// Enable transparent window
    pub fn with_transparent(mut self, enabled: bool) -> Self {
        self.transparent = enabled;
        self
    }

    /// Set window resizable
    pub fn with_resizable(mut self, enabled: bool) -> Self {
        self.resizable = enabled;
        self
    }

    /// Set custom user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set JavaScript to inject on page load
    pub fn with_inject_js(mut self, js: impl Into<String>) -> Self {
        self.inject_js = Some(js.into());
        self
    }

    /// Set CSS to inject on page load
    pub fn with_inject_css(mut self, css: impl Into<String>) -> Self {
        self.inject_css = Some(css.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_mode_embeds_assets() {
        assert!(!PackMode::Url {
            url: "https://example.com".to_string()
        }
        .embeds_assets());
        assert!(PackMode::Frontend {
            path: PathBuf::from("dist")
        }
        .embeds_assets());
        assert!(PackMode::FullStack {
            frontend_path: PathBuf::from("dist"),
            backend_entry: "app:main".to_string()
        }
        .embeds_assets());
    }

    #[test]
    fn test_pack_mode_requires_pyoxidizer() {
        assert!(!PackMode::Url {
            url: "https://example.com".to_string()
        }
        .requires_pyoxidizer());
        assert!(!PackMode::Frontend {
            path: PathBuf::from("dist")
        }
        .requires_pyoxidizer());
        assert!(PackMode::FullStack {
            frontend_path: PathBuf::from("dist"),
            backend_entry: "app:main".to_string()
        }
        .requires_pyoxidizer());
    }

    #[test]
    fn test_pack_config_builder() {
        let config = PackConfig::url("https://example.com")
            .with_output("my-app")
            .with_title("My Application")
            .with_size(1280, 720)
            .with_debug(true)
            .with_always_on_top(true);

        assert_eq!(config.output_name, "my-app");
        assert_eq!(config.window_title, "My Application");
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert!(config.debug);
        assert!(config.always_on_top);
    }

    #[test]
    fn test_target_platform_triple() {
        assert_eq!(TargetPlatform::Current.target_triple(), None);
        assert_eq!(
            TargetPlatform::Windows.target_triple(),
            Some("x86_64-pc-windows-msvc")
        );
        assert_eq!(
            TargetPlatform::MacOS.target_triple(),
            Some("x86_64-apple-darwin")
        );
        assert_eq!(
            TargetPlatform::Linux.target_triple(),
            Some("x86_64-unknown-linux-gnu")
        );
    }
}
