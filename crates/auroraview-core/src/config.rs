//! WebView configuration structures (pure Rust, no PyO3)
//!
//! Core configuration types that can be shared across all crates.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Embedding mode on Windows.
/// All embedded modes (Child, Owner) create frameless windows for seamless Qt integration.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EmbedMode {
    /// No parent/owner specified (standalone top-level window)
    #[default]
    None,
    /// Create as real child window (WS_CHILD, frameless). Requires same-thread parenting.
    Child,
    /// Create as owned top-level window (GWLP_HWNDPARENT, frameless). Safe across threads.
    Owner,
}

/// Dummy enum for non-Windows (compile-time placeholder)
#[cfg(not(target_os = "windows"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EmbedMode {
    #[default]
    None,
}

/// Core WebView configuration (without protocol callbacks)
///
/// This is a pure Rust struct that can be shared across crates.
/// For Python bindings, use the full `WebViewConfig` from the main crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Window title
    pub title: String,

    /// Window width in pixels
    pub width: u32,

    /// Window height in pixels
    pub height: u32,

    /// URL to load (optional)
    pub url: Option<String>,

    /// HTML content to load (optional)
    pub html: Option<String>,

    /// Enable developer tools
    pub dev_tools: bool,

    /// Enable context menu
    pub context_menu: bool,

    /// Window resizable
    pub resizable: bool,

    /// Window decorations (title bar, borders)
    pub decorations: bool,

    /// Always on top
    pub always_on_top: bool,

    /// Transparent window
    pub transparent: bool,

    /// Background color in hex format (e.g., "#1e1e1e")
    pub background_color: Option<String>,

    /// Parent window handle (HWND on Windows)
    pub parent_hwnd: Option<u64>,

    /// Embedding mode (Windows)
    pub embed_mode: EmbedMode,

    /// Enable IPC message batching
    pub ipc_batching: bool,

    /// Maximum number of messages per batch
    pub ipc_batch_size: usize,

    /// Maximum batch age in milliseconds
    pub ipc_batch_interval_ms: u64,

    /// Asset root directory for custom protocol
    pub asset_root: Option<PathBuf>,

    /// Allow opening new windows
    pub allow_new_window: bool,

    /// Enable file:// protocol support
    pub allow_file_protocol: bool,

    /// Show shadow for undecorated (frameless) windows (Windows only).
    ///
    /// When `decorations` is false, Windows can still show a subtle shadow
    /// around the window. Set this to `false` to disable the shadow completely,
    /// which is required for truly transparent frameless windows.
    ///
    /// Default: true (show shadow for undecorated windows)
    ///
    /// # When to disable
    /// - Transparent overlay windows (e.g., floating logo buttons)
    /// - Custom-shaped windows
    /// - Windows that should blend seamlessly with the desktop
    #[cfg(target_os = "windows")]
    pub undecorated_shadow: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            dev_tools: true,
            context_menu: true,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            background_color: None,
            parent_hwnd: None,
            embed_mode: EmbedMode::None,
            ipc_batching: true,
            ipc_batch_size: 10,
            ipc_batch_interval_ms: 10,
            asset_root: None,
            allow_new_window: false,
            allow_file_protocol: false,
            #[cfg(target_os = "windows")]
            undecorated_shadow: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CoreConfig::default();
        assert_eq!(config.title, "AuroraView");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.dev_tools);
        assert!(!config.allow_new_window);
    }

    #[test]
    fn test_config_serialization() {
        let config = CoreConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: CoreConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, config.title);
        assert_eq!(parsed.width, config.width);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_undecorated_shadow_default() {
        let config = CoreConfig::default();
        assert!(
            config.undecorated_shadow,
            "undecorated_shadow should default to true"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_undecorated_shadow_disabled() {
        let config = CoreConfig {
            undecorated_shadow: false,
            ..Default::default()
        };
        assert!(!config.undecorated_shadow);
    }
}
