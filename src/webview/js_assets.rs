//! JavaScript assets management
//!
//! This module manages all JavaScript code that is injected into the WebView.
//! JavaScript files are stored in `assets/js/` and embedded at compile time
//! using the `include_str!` macro.
//!
//! ## Architecture
//!
//! - **Core scripts**: Always included, provide fundamental functionality
//! - **Feature scripts**: Conditionally included based on WebViewConfig
//!
//! ## Usage
//!
//! ```rust
//! use crate::webview::js_assets;
//! use crate::webview::WebViewConfig;
//!
//! let config = WebViewConfig::default();
//! let init_script = js_assets::build_init_script(&config);
//! ```

use crate::webview::WebViewConfig;

/// Core event bridge script
///
/// Provides the primary `window.auroraview` API for JavaScript <-> Python communication.
/// This includes:
/// - `auroraview.call(method, params)` - Promise-based RPC
/// - `auroraview.send_event(event, detail)` - Fire-and-forget events
/// - `auroraview.on(event, handler)` - Event listeners
pub const EVENT_BRIDGE: &str = include_str!("../assets/js/core/event_bridge.js");

/// Context menu disable script
///
/// Prevents the native browser context menu from appearing on right-click.
/// This allows applications to implement custom context menus.
pub const CONTEXT_MENU_DISABLE: &str = include_str!("../assets/js/features/context_menu.js");

/// Legacy compatibility script
///
/// Provides backward compatibility with older AuroraView API.
/// Creates the `window.aurora` alias for `window.auroraview`.
pub const LEGACY_COMPAT: &str = include_str!("../assets/js/features/legacy_compat.js");

/// Build complete initialization script based on configuration
///
/// This function assembles the final JavaScript initialization script
/// by combining core scripts and optional feature scripts based on
/// the provided WebViewConfig.
///
/// # Arguments
///
/// * `config` - WebView configuration
///
/// # Returns
///
/// Complete JavaScript initialization script as a String
///
/// # Example
///
/// ```rust
/// use crate::webview::{WebViewConfig, js_assets};
///
/// let mut config = WebViewConfig::default();
/// config.context_menu = false;
///
/// let script = js_assets::build_init_script(&config);
/// // script now contains event_bridge.js + context_menu.js
/// ```
pub fn build_init_script(config: &WebViewConfig) -> String {
    let mut script = String::with_capacity(8192); // Pre-allocate reasonable size

    // Core scripts (always included)
    script.push_str(EVENT_BRIDGE);
    script.push('\n');

    // Optional features based on configuration
    if !config.context_menu {
        tracing::info!("[js_assets] Including context menu disable script");
        script.push_str(CONTEXT_MENU_DISABLE);
        script.push('\n');
    }

    // Legacy compatibility (always included for now)
    // TODO: Make this configurable via WebViewConfig.legacy_compat flag
    script.push_str(LEGACY_COMPAT);
    script.push('\n');

    tracing::debug!(
        "[js_assets] Built initialization script: {} bytes",
        script.len()
    );

    script
}

/// Get event bridge script only
///
/// Returns just the core event bridge without any optional features.
/// Useful for minimal WebView setups.
#[allow(dead_code)]
pub fn get_event_bridge() -> &'static str {
    EVENT_BRIDGE
}

/// Get context menu disable script only
///
/// Returns just the context menu disable script.
/// Useful for dynamic injection after WebView creation.
#[allow(dead_code)]
pub fn get_context_menu_disable() -> &'static str {
    CONTEXT_MENU_DISABLE
}

/// Get legacy compatibility script only
///
/// Returns just the legacy compatibility layer.
#[allow(dead_code)]
pub fn get_legacy_compat() -> &'static str {
    LEGACY_COMPAT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_init_script_default() {
        let config = WebViewConfig::default();
        let script = build_init_script(&config);

        // Should include event bridge
        assert!(script.contains("window.auroraview"));
        // Should include legacy compat
        assert!(script.contains("window.aurora"));
        // Should NOT include context menu disable (default is true)
        assert!(!script.contains("contextmenu"));
    }

    #[test]
    fn test_build_init_script_no_context_menu() {
        let config = WebViewConfig {
            context_menu: false,
            ..Default::default()
        };
        let script = build_init_script(&config);

        // Should include context menu disable
        assert!(script.contains("contextmenu"));
        assert!(script.contains("preventDefault"));
    }

    #[test]
    fn test_individual_scripts() {
        // Test that individual getters work
        assert!(get_event_bridge().contains("window.auroraview"));
        assert!(get_context_menu_disable().contains("contextmenu"));
        assert!(get_legacy_compat().contains("window.aurora"));
    }
}
