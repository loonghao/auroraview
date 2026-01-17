//! Browser configuration

use crate::devtools::{DevToolsConfig, DockSide};
use crate::ui::Theme;
use serde::{Deserialize, Serialize};

/// Browser features toggle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFeatures {
    /// Show bookmarks bar
    pub bookmarks_bar: bool,
    /// Enable history tracking
    pub history: bool,
    /// Enable extensions support
    pub extensions: bool,
    /// Enable downloads manager
    pub downloads: bool,
    /// Enable developer tools
    pub dev_tools: bool,
    /// Enable context menu
    pub context_menu: bool,
    /// Enable CDP (Chrome DevTools Protocol) remote debugging
    pub cdp_enabled: bool,
}

impl Default for BrowserFeatures {
    fn default() -> Self {
        Self {
            bookmarks_bar: false,
            history: true,
            extensions: true,
            downloads: true,
            dev_tools: false,
            context_menu: true,
            cdp_enabled: false,
        }
    }
}

/// Browser configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Home page URL
    pub home_url: String,
    /// Theme selection
    pub theme: Theme,
    /// Feature toggles
    pub features: BrowserFeatures,
    /// Debug mode
    pub debug: bool,
    /// Initial URLs to open as tabs
    pub initial_urls: Vec<String>,
    /// User data directory for persistence
    pub user_data_dir: Option<String>,
    /// DevTools configuration
    pub devtools: DevToolsConfig,
    /// CDP remote debugging port (0 = disabled, typically 9222)
    pub remote_debugging_port: u16,
    /// Frameless window (no native title bar)
    pub frameless: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView Browser".to_string(),
            width: 1280,
            height: 900,
            home_url: "https://www.google.com".to_string(),
            theme: Theme::System,
            features: BrowserFeatures::default(),
            debug: false,
            initial_urls: vec![],
            user_data_dir: None,
            devtools: DevToolsConfig::default(),
            remote_debugging_port: 0,
            frameless: true,
        }
    }
}

impl BrowserConfig {
    /// Create a new builder
    pub fn builder() -> BrowserConfigBuilder {
        BrowserConfigBuilder::new()
    }
}

/// Builder for BrowserConfig
#[derive(Default)]
pub struct BrowserConfigBuilder {
    config: BrowserConfig,
}

impl BrowserConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: BrowserConfig::default(),
        }
    }

    /// Set window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    /// Set window size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Set home URL
    pub fn home_url(mut self, url: impl Into<String>) -> Self {
        self.config.home_url = url.into();
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.config.theme = theme;
        self
    }

    /// Enable/disable bookmarks bar
    pub fn bookmarks_bar(mut self, enabled: bool) -> Self {
        self.config.features.bookmarks_bar = enabled;
        self
    }

    /// Enable/disable history
    pub fn history(mut self, enabled: bool) -> Self {
        self.config.features.history = enabled;
        self
    }

    /// Enable/disable extensions
    pub fn extensions(mut self, enabled: bool) -> Self {
        self.config.features.extensions = enabled;
        self
    }

    /// Enable/disable downloads
    pub fn downloads(mut self, enabled: bool) -> Self {
        self.config.features.downloads = enabled;
        self
    }

    /// Enable/disable dev tools
    pub fn dev_tools(mut self, enabled: bool) -> Self {
        self.config.features.dev_tools = enabled;
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    /// Set remote debugging port for CDP (Chrome DevTools Protocol)
    /// Use 0 to disable, typical value is 9222
    pub fn remote_debugging_port(mut self, port: u16) -> Self {
        self.config.remote_debugging_port = port;
        if port > 0 {
            self.config.features.cdp_enabled = true;
        }
        self
    }

    /// Set DevTools dock side
    pub fn devtools_dock_side(mut self, side: DockSide) -> Self {
        self.config.devtools.dock_side = side;
        self
    }

    /// Auto-open DevTools on launch
    pub fn devtools_auto_open(mut self, auto_open: bool) -> Self {
        self.config.devtools.auto_open = auto_open;
        self
    }

    /// Set initial URLs
    pub fn initial_urls(mut self, urls: Vec<String>) -> Self {
        self.config.initial_urls = urls;
        self
    }

    /// Set user data directory
    pub fn user_data_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.user_data_dir = Some(dir.into());
        self
    }

    /// Enable/disable frameless window (no native title bar)
    pub fn frameless(mut self, frameless: bool) -> Self {
        self.config.frameless = frameless;
        self
    }

    /// Build the config
    pub fn build(self) -> BrowserConfig {
        self.config
    }
}
