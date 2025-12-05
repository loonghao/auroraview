//! WebView settings abstraction
//!
//! Unified settings interface inspired by Qt WebView's QWebViewSettingsPrivate.

use serde::{Deserialize, Serialize};

/// Unified WebView settings trait
///
/// This trait provides a platform-agnostic interface for WebView settings,
/// similar to Qt WebView's `QWebViewSettingsPrivate`.
pub trait WebViewSettings: Send + Sync {
    /// Check if local storage is enabled
    fn local_storage_enabled(&self) -> bool;

    /// Enable or disable local storage
    fn set_local_storage_enabled(&mut self, enabled: bool);

    /// Check if JavaScript is enabled
    fn javascript_enabled(&self) -> bool;

    /// Enable or disable JavaScript execution
    fn set_javascript_enabled(&mut self, enabled: bool);

    /// Check if developer tools are enabled
    fn dev_tools_enabled(&self) -> bool;

    /// Enable or disable developer tools
    fn set_dev_tools_enabled(&mut self, enabled: bool);

    /// Check if file:// URLs can access local content
    fn allow_file_access(&self) -> bool;

    /// Enable or disable file:// URL access
    fn set_allow_file_access(&mut self, enabled: bool);

    /// Check if context menu is enabled
    fn context_menu_enabled(&self) -> bool;

    /// Enable or disable context menu
    fn set_context_menu_enabled(&mut self, enabled: bool);

    /// Get custom user agent string
    fn user_agent(&self) -> Option<String>;

    /// Set custom user agent string
    fn set_user_agent(&mut self, user_agent: Option<String>);

    /// Get background color
    fn background_color(&self) -> Option<String>;

    /// Set background color (hex format, e.g., "#1e1e1e")
    fn set_background_color(&mut self, color: Option<String>);
}

/// Default implementation of WebViewSettings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewSettingsImpl {
    local_storage: bool,
    javascript: bool,
    dev_tools: bool,
    file_access: bool,
    context_menu: bool,
    user_agent: Option<String>,
    background_color: Option<String>,
}

impl Default for WebViewSettingsImpl {
    fn default() -> Self {
        Self {
            local_storage: true,
            javascript: true,
            dev_tools: false,
            file_access: false,
            context_menu: true,
            user_agent: None,
            background_color: None,
        }
    }
}

impl WebViewSettings for WebViewSettingsImpl {
    fn local_storage_enabled(&self) -> bool {
        self.local_storage
    }

    fn set_local_storage_enabled(&mut self, enabled: bool) {
        self.local_storage = enabled;
    }

    fn javascript_enabled(&self) -> bool {
        self.javascript
    }

    fn set_javascript_enabled(&mut self, enabled: bool) {
        self.javascript = enabled;
    }

    fn dev_tools_enabled(&self) -> bool {
        self.dev_tools
    }

    fn set_dev_tools_enabled(&mut self, enabled: bool) {
        self.dev_tools = enabled;
    }

    fn allow_file_access(&self) -> bool {
        self.file_access
    }

    fn set_allow_file_access(&mut self, enabled: bool) {
        self.file_access = enabled;
    }

    fn context_menu_enabled(&self) -> bool {
        self.context_menu
    }

    fn set_context_menu_enabled(&mut self, enabled: bool) {
        self.context_menu = enabled;
    }

    fn user_agent(&self) -> Option<String> {
        self.user_agent.clone()
    }

    fn set_user_agent(&mut self, user_agent: Option<String>) {
        self.user_agent = user_agent;
    }

    fn background_color(&self) -> Option<String> {
        self.background_color.clone()
    }

    fn set_background_color(&mut self, color: Option<String>) {
        self.background_color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = WebViewSettingsImpl::default();
        assert!(settings.local_storage_enabled());
        assert!(settings.javascript_enabled());
        assert!(!settings.dev_tools_enabled());
        assert!(!settings.allow_file_access());
    }

    #[test]
    fn test_settings_mutation() {
        let mut settings = WebViewSettingsImpl::default();
        settings.set_dev_tools_enabled(true);
        assert!(settings.dev_tools_enabled());

        settings.set_user_agent(Some("CustomAgent/1.0".into()));
        assert_eq!(settings.user_agent(), Some("CustomAgent/1.0".into()));
    }
}
