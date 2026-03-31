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

/// Core browser window and lifecycle management.
pub mod browser;
/// Browser configuration and feature flags.
pub mod config;
/// DevTools and Chrome DevTools Protocol (CDP) integration.
pub mod devtools;
/// Browser-specific error types.
pub mod error;
/// Chrome extension system and manifest handling.
pub mod extensions;
/// Navigation controls, bookmarks, and history (built-in).
pub mod navigation;
/// Tab state, ordering, and lifecycle management.
pub mod tab;
/// Theme, toolbar, and UI customization.
pub mod ui;

/// Independent tab management crate (when `modular-tabs` feature is enabled).
#[cfg(feature = "modular-tabs")]
pub use auroraview_tabs as tabs_crate;

/// Independent extension system crate (when `modular-extensions` feature is enabled).
#[cfg(feature = "modular-extensions")]
pub use auroraview_extensions as extensions_crate;

/// Independent bookmark management crate (when `modular-bookmarks` feature is enabled).
#[cfg(feature = "modular-bookmarks")]
pub use auroraview_bookmarks as bookmarks_crate;

/// Independent history management crate (when `modular-history` feature is enabled).
#[cfg(feature = "modular-history")]
pub use auroraview_history as history_crate;

/// Independent DevTools crate (when `modular-devtools` feature is enabled).
#[cfg(feature = "modular-devtools")]
pub use auroraview_devtools as devtools_crate;

/// Download management (when `downloads` feature is enabled).
#[cfg(feature = "downloads")]
pub use auroraview_downloads as downloads;

/// Settings and preferences (when `settings` feature is enabled).
#[cfg(feature = "settings")]
pub use auroraview_settings as settings;

/// Notification system (when `notifications` feature is enabled).
#[cfg(feature = "notifications")]
pub use auroraview_notifications as notifications;

/// Core browser types re-exported for convenience.
pub use browser::Browser;
/// Browser configuration and feature flag types.
pub use config::{BrowserConfig, BrowserFeatures};
/// Error and result types for browser operations.
pub use error::{BrowserError, Result};
/// Theme and custom theme types.
pub use ui::{CustomTheme, Theme};

/// DevTools types including CDP, console messages, and dock configuration.
pub use devtools::{
    cdp, ConsoleMessage, ConsoleMessageType, DevToolsConfig, DevToolsManager, DevToolsState,
    DockSide,
};
/// Extension system types for Chrome extension compatibility.
pub use extensions::{
    ChromeExtensionBridge, ChromeExtensionInfo, Extension, ExtensionManifest, ExtensionRegistry,
};
/// Built-in navigation types: bookmarks and history entries.
pub use navigation::{Bookmark, BookmarkId, BookmarkManager, HistoryEntry, HistoryManager};
/// Tab management types: tab state, identifiers, and manager.
pub use tab::{Tab, TabId, TabManager, TabState};

/// Browser event types
pub mod events {
    pub use crate::browser::BrowserEvent;
    pub use crate::tab::TabEvent;
}
