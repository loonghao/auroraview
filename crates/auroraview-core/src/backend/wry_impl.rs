//! Wry backend implementation
//!
//! Implements the WebViewBackend trait for the Wry library.
//! This provides a unified interface for platform-specific WebView implementations.

use super::error::{WebViewError, WebViewResult};
use super::settings::{WebViewSettings, WebViewSettingsImpl};
use super::traits::{CookieInfo, JavaScriptCallback, LoadProgress, WebViewBackend};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

/// Wry backend implementation
///
/// Wraps the wry WebView to provide a unified interface.
/// This implementation is designed to work with the existing wry-based code.
pub struct WryBackend {
    /// Current URL (tracked locally since wry doesn't provide direct access)
    current_url: RwLock<Option<String>>,
    /// Current title (tracked locally)
    current_title: RwLock<Option<String>>,
    /// Whether the WebView is loading
    is_loading: AtomicBool,
    /// Load progress percentage
    load_progress: RwLock<u8>,
    /// Whether the WebView is closed
    is_closed: AtomicBool,
    /// Settings (using Box for stable address)
    settings: Box<WebViewSettingsImpl>,
    /// User agent string
    user_agent: String,
}

impl Default for WryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WryBackend {
    /// Create a new Wry backend
    pub fn new() -> Self {
        Self {
            current_url: RwLock::new(None),
            current_title: RwLock::new(None),
            is_loading: AtomicBool::new(false),
            load_progress: RwLock::new(0),
            is_closed: AtomicBool::new(false),
            settings: Box::new(WebViewSettingsImpl::default()),
            user_agent: format!("AuroraView/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Update current URL (called by navigation event handlers)
    pub fn set_current_url(&self, url: Option<String>) {
        if let Ok(mut u) = self.current_url.write() {
            *u = url;
        }
    }

    /// Update current title
    pub fn set_current_title(&self, title: Option<String>) {
        if let Ok(mut t) = self.current_title.write() {
            *t = title;
        }
    }

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        self.is_loading.store(loading, Ordering::SeqCst);
    }

    /// Set load progress
    pub fn set_load_progress(&self, progress: u8) {
        if let Ok(mut p) = self.load_progress.write() {
            *p = progress.min(100);
        }
    }

    /// Apply settings from a WebViewSettingsImpl
    pub fn apply_settings(&mut self, settings: WebViewSettingsImpl) {
        *self.settings = settings;
    }
}

impl WebViewBackend for WryBackend {
    // ========== Navigation ==========

    fn navigate(&self, url: &str) -> WebViewResult<()> {
        // Note: Actual navigation is handled by the main WebView
        // This is a state-tracking implementation
        self.set_current_url(Some(url.to_string()));
        self.set_loading(true);
        self.set_load_progress(0);
        Ok(())
    }

    fn url(&self) -> Option<String> {
        self.current_url.read().ok().and_then(|u| u.clone())
    }

    fn can_go_back(&self) -> bool {
        // Note: Wry doesn't expose history navigation state directly
        // This would need platform-specific implementation
        false
    }

    fn can_go_forward(&self) -> bool {
        false
    }

    fn go_back(&self) -> WebViewResult<()> {
        Err(WebViewError::Unsupported("go_back".to_string()))
    }

    fn go_forward(&self) -> WebViewResult<()> {
        Err(WebViewError::Unsupported("go_forward".to_string()))
    }

    fn reload(&self) -> WebViewResult<()> {
        self.set_loading(true);
        self.set_load_progress(0);
        Ok(())
    }

    fn stop(&self) -> WebViewResult<()> {
        self.set_loading(false);
        Ok(())
    }

    // ========== Content Loading ==========

    fn load_html(&self, _html: &str) -> WebViewResult<()> {
        self.set_loading(true);
        self.set_load_progress(0);
        Ok(())
    }

    fn title(&self) -> Option<String> {
        self.current_title.read().ok().and_then(|t| t.clone())
    }

    fn load_progress(&self) -> LoadProgress {
        let percent = self.load_progress.read().map(|p| *p).unwrap_or(0);
        LoadProgress {
            percent,
            is_complete: !self.is_loading.load(Ordering::SeqCst),
        }
    }

    fn is_loading(&self) -> bool {
        self.is_loading.load(Ordering::SeqCst)
    }

    // ========== JavaScript ==========

    fn eval_js(&self, _script: &str) -> WebViewResult<()> {
        // Note: Actual JS execution is handled by the main WebView
        Ok(())
    }

    fn eval_js_with_callback(
        &self,
        _script: &str,
        callback: JavaScriptCallback,
    ) -> WebViewResult<()> {
        // Note: Wry doesn't support async JS execution with callbacks directly
        // This would need to be implemented via IPC
        callback(Ok(serde_json::Value::Null));
        Ok(())
    }

    // ========== Cookie Management ==========

    fn set_cookie(&self, _cookie: &CookieInfo) -> WebViewResult<()> {
        // Note: Cookie management requires platform-specific implementation
        Err(WebViewError::Unsupported("set_cookie".to_string()))
    }

    fn get_cookie(&self, _domain: &str, _name: &str) -> WebViewResult<Option<CookieInfo>> {
        Err(WebViewError::Unsupported("get_cookie".to_string()))
    }

    fn delete_cookie(&self, _domain: &str, _name: &str) -> WebViewResult<()> {
        Err(WebViewError::Unsupported("delete_cookie".to_string()))
    }

    fn clear_cookies(&self) -> WebViewResult<()> {
        Err(WebViewError::Unsupported("clear_cookies".to_string()))
    }

    // ========== Settings ==========

    fn settings(&self) -> &dyn WebViewSettings {
        self.settings.as_ref()
    }

    fn settings_mut(&mut self) -> &mut dyn WebViewSettings {
        self.settings.as_mut()
    }

    fn http_user_agent(&self) -> String {
        self.user_agent.clone()
    }

    // ========== Lifecycle ==========

    fn close(&self) -> WebViewResult<()> {
        self.is_closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::SeqCst)
    }

    // ========== Window Control ==========

    fn set_bounds(&self, _x: i32, _y: i32, _width: u32, _height: u32) -> WebViewResult<()> {
        // Note: Window bounds are managed by the main window
        Ok(())
    }

    fn set_visible(&self, _visible: bool) -> WebViewResult<()> {
        Ok(())
    }

    fn focus(&self) -> WebViewResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wry_backend_creation() {
        let backend = WryBackend::new();
        assert!(backend.url().is_none());
        assert!(!backend.is_loading());
        assert!(!backend.is_closed());
    }

    #[test]
    fn test_navigate() {
        let backend = WryBackend::new();
        backend.navigate("https://example.com").unwrap();
        assert_eq!(backend.url(), Some("https://example.com".to_string()));
        assert!(backend.is_loading());
    }

    #[test]
    fn test_load_progress() {
        let backend = WryBackend::new();
        backend.set_load_progress(50);
        let progress = backend.load_progress();
        assert_eq!(progress.percent, 50);
    }

    #[test]
    fn test_close() {
        let backend = WryBackend::new();
        assert!(!backend.is_closed());
        backend.close().unwrap();
        assert!(backend.is_closed());
    }

    #[test]
    fn test_user_agent() {
        let backend = WryBackend::new();
        let ua = backend.http_user_agent();
        assert!(ua.starts_with("AuroraView/"));
    }
}
