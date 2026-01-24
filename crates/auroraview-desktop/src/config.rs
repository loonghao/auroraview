//! Desktop configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Desktop window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Window title
    pub title: String,

    /// Window width
    pub width: u32,

    /// Window height
    pub height: u32,

    /// Initial URL to load
    pub url: Option<String>,

    /// Initial HTML content
    pub html: Option<String>,

    /// Window is resizable
    pub resizable: bool,

    /// Window has decorations (title bar, borders)
    pub decorations: bool,

    /// Window is always on top
    pub always_on_top: bool,

    /// Window is transparent
    pub transparent: bool,

    /// Window starts maximized
    pub maximized: bool,

    /// Window starts minimized
    pub minimized: bool,

    /// Window starts visible
    pub visible: bool,

    /// Window starts fullscreen
    pub fullscreen: bool,

    /// Enable DevTools
    pub devtools: bool,

    /// User data directory for WebView
    pub data_dir: Option<PathBuf>,

    /// Window icon path
    pub icon: Option<PathBuf>,

    /// Enable system tray
    pub tray: Option<TrayConfig>,

    /// Custom user agent
    pub user_agent: Option<String>,

    /// Proxy configuration
    pub proxy: Option<String>,

    /// Enable context menu
    pub context_menu: bool,

    /// Enable hotkeys
    pub hotkeys: bool,

    /// CDP debugging port (0 = auto)
    pub debug_port: u16,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView".to_string(),
            width: 1024,
            height: 768,
            url: None,
            html: None,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            maximized: false,
            minimized: false,
            visible: true,
            fullscreen: false,
            devtools: cfg!(debug_assertions),
            data_dir: None,
            icon: None,
            tray: None,
            user_agent: None,
            proxy: None,
            context_menu: true,
            hotkeys: true,
            debug_port: 0,
        }
    }
}

impl DesktopConfig {
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

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    pub fn devtools(mut self, devtools: bool) -> Self {
        self.devtools = devtools;
        self
    }

    pub fn data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    pub fn icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn tray(mut self, config: TrayConfig) -> Self {
        self.tray = Some(config);
        self
    }

    pub fn debug_port(mut self, port: u16) -> Self {
        self.debug_port = port;
        self
    }
}

/// System tray configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayConfig {
    /// Tray icon path
    pub icon: Option<PathBuf>,

    /// Tray tooltip
    pub tooltip: Option<String>,

    /// Menu items
    pub menu: Vec<TrayMenuItem>,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            icon: None,
            tooltip: None,
            menu: vec![
                TrayMenuItem::Item {
                    id: "show".to_string(),
                    label: "Show".to_string(),
                    enabled: true,
                },
                TrayMenuItem::Separator,
                TrayMenuItem::Item {
                    id: "quit".to_string(),
                    label: "Quit".to_string(),
                    enabled: true,
                },
            ],
        }
    }
}

/// Tray menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrayMenuItem {
    Item {
        id: String,
        label: String,
        enabled: bool,
    },
    Separator,
}
