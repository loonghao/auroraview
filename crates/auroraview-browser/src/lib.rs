//! AuroraView Browser - Multi-tab browser component with Edge-style UI
//!
//! This crate provides a complete multi-tab browser implementation for AuroraView,
//! featuring an Edge/Chrome-style modern UI with support for:
//!
//! - Tab management (create, close, switch, reorder)
//! - Navigation controls (back, forward, reload, home)
//! - Bookmarks management
//! - Browsing history
//! - Extension system
//! - Theme customization (Light/Dark/System)
//! - DevTools integration (F12)
//! - CDP (Chrome DevTools Protocol) remote debugging
//!
//! # Architecture
//!
//! Based on Microsoft WebView2Browser architecture:
//! - Single UI thread for all WebView operations
//! - Dual environment pattern (UI vs content WebViews)
//! - Shared content environment for cookie/cache sharing
//!
//! # Independent Feature Crates
//!
//! The browser includes built-in implementations but can optionally use
//! independent feature crates via feature flags:
//!
//! - `modular-tabs`: Use `auroraview-tabs` crate
//! - `modular-extensions`: Use `auroraview-extensions` crate
//! - `modular-bookmarks`: Use `auroraview-bookmarks` crate
//! - `modular-history`: Use `auroraview-history` crate
//! - `modular-devtools`: Use `auroraview-devtools` crate
//! - `modular`: Enable all modular features
//!
//! Optional features:
//! - `downloads`: Download management via `auroraview-downloads`
//! - `settings`: Settings/preferences via `auroraview-settings`
//! - `notifications`: Notification system via `auroraview-notifications`
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_browser::{Browser, BrowserConfig, Theme};
//!
//! let config = BrowserConfig::builder()
//!     .title("My Browser")
//!     .size(1280, 900)
//!     .home_url("https://google.com")
//!     .theme(Theme::Dark)
//!     .bookmarks_bar(true)
//!     .dev_tools(true)
//!     .remote_debugging_port(9222)  // Enable CDP
//!     .build();
//!
//! let mut browser = Browser::new(config);
//! browser.run();
//! ```

pub mod browser;
pub mod config;
pub mod devtools;
pub mod error;
pub mod extensions;
pub mod navigation;
pub mod tab;
pub mod ui;


// Re-export from independent crates when modular features are enabled
#[cfg(feature = "modular-tabs")]
pub use auroraview_tabs as tabs_crate;

#[cfg(feature = "modular-extensions")]
pub use auroraview_extensions as extensions_crate;

#[cfg(feature = "modular-bookmarks")]
pub use auroraview_bookmarks as bookmarks_crate;

#[cfg(feature = "modular-history")]
pub use auroraview_history as history_crate;

#[cfg(feature = "modular-devtools")]
pub use auroraview_devtools as devtools_crate;

// Optional feature re-exports
#[cfg(feature = "downloads")]
pub use auroraview_downloads as downloads;

#[cfg(feature = "settings")]
pub use auroraview_settings as settings;

#[cfg(feature = "notifications")]
pub use auroraview_notifications as notifications;

// Re-export main types
pub use browser::Browser;
pub use config::{BrowserConfig, BrowserFeatures};
pub use error::{BrowserError, Result};
pub use ui::{CustomTheme, Theme};

// Re-export types from browser-specific modules (built-in implementations)
pub use devtools::{
    cdp, ConsoleMessage, ConsoleMessageType, DevToolsConfig, DevToolsManager, DevToolsState,
    DockSide,
};
pub use extensions::{ChromeExtensionBridge, ChromeExtensionInfo, Extension, ExtensionManifest, ExtensionRegistry};
pub use navigation::{Bookmark, BookmarkId, BookmarkManager, HistoryEntry, HistoryManager};
pub use tab::{Tab, TabId, TabManager, TabState};

/// Browser event types
pub mod events {
    pub use crate::browser::BrowserEvent;
    pub use crate::tab::TabEvent;
}
