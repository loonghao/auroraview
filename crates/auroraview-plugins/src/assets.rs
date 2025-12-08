//! Plugin Assets
//!
//! This module provides embedded JavaScript files for plugin APIs.
//! These scripts should be injected into the WebView to enable
//! JavaScript access to plugin functionality.

use rust_embed::Embed;

/// Embedded plugin JavaScript files
#[derive(Embed)]
#[folder = "src/assets/js"]
#[include = "*.js"]
pub struct PluginAssets;

impl PluginAssets {
    /// Get the file system plugin JavaScript
    pub fn fs_js() -> &'static str {
        include_str!("assets/js/fs.js")
    }

    /// Get the dialog plugin JavaScript
    pub fn dialog_js() -> &'static str {
        include_str!("assets/js/dialog.js")
    }

    /// Get the clipboard plugin JavaScript
    pub fn clipboard_js() -> &'static str {
        include_str!("assets/js/clipboard.js")
    }

    /// Get the shell plugin JavaScript
    pub fn shell_js() -> &'static str {
        include_str!("assets/js/shell.js")
    }

    /// Get all plugin JavaScript concatenated
    pub fn all_plugins_js() -> String {
        format!(
            "{}\n{}\n{}\n{}",
            Self::fs_js(),
            Self::dialog_js(),
            Self::clipboard_js(),
            Self::shell_js()
        )
    }

    /// Get plugin JavaScript by name
    pub fn get_plugin_js(name: &str) -> Option<&'static str> {
        match name {
            "fs" => Some(Self::fs_js()),
            "dialog" => Some(Self::dialog_js()),
            "clipboard" => Some(Self::clipboard_js()),
            "shell" => Some(Self::shell_js()),
            _ => None,
        }
    }

    /// List available plugin names
    pub fn plugin_names() -> &'static [&'static str] {
        &["fs", "dialog", "clipboard", "shell"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_js_not_empty() {
        assert!(!PluginAssets::fs_js().is_empty());
        assert!(PluginAssets::fs_js().contains("auroraview.fs"));
    }

    #[test]
    fn test_dialog_js_not_empty() {
        assert!(!PluginAssets::dialog_js().is_empty());
        assert!(PluginAssets::dialog_js().contains("auroraview.dialog"));
    }

    #[test]
    fn test_clipboard_js_not_empty() {
        assert!(!PluginAssets::clipboard_js().is_empty());
        assert!(PluginAssets::clipboard_js().contains("auroraview.clipboard"));
    }

    #[test]
    fn test_shell_js_not_empty() {
        assert!(!PluginAssets::shell_js().is_empty());
        assert!(PluginAssets::shell_js().contains("auroraview.shell"));
    }

    #[test]
    fn test_get_plugin_js() {
        assert!(PluginAssets::get_plugin_js("fs").is_some());
        assert!(PluginAssets::get_plugin_js("dialog").is_some());
        assert!(PluginAssets::get_plugin_js("unknown").is_none());
    }

    #[test]
    fn test_all_plugins_js() {
        let all = PluginAssets::all_plugins_js();
        assert!(all.contains("auroraview.fs"));
        assert!(all.contains("auroraview.dialog"));
        assert!(all.contains("auroraview.clipboard"));
        assert!(all.contains("auroraview.shell"));
    }
}
