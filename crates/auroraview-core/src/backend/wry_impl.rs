//! Wry backend implementation
//!
//! Implements the WebViewBackend trait for the Wry library.
//! This provides a unified interface for platform-specific WebView implementations.
//!
//! ## Architecture
//!
//! This implementation uses:
//! - `AtomicLifecycle` for lock-free state management
//! - `RwLock` for URL/title tracking (read-heavy workload)
//! - `AtomicBool` for simple boolean flags
//!
//! The actual WebView operations are delegated to the main WebView instance.
//! This backend primarily tracks state for the trait interface.

use super::error::{WebViewError, WebViewResult};
use super::lifecycle::{AtomicLifecycle, LifecycleState};
use super::settings::{WebViewSettings, WebViewSettingsImpl};
use super::traits::{CookieInfo, JavaScriptCallback, LoadProgress, WebViewBackend};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::RwLock;

/// Wry backend implementation
///
/// Wraps the wry WebView to provide a unified interface.
/// This implementation is designed to work with the existing wry-based code.
///
/// ## Thread Safety
///
/// This struct is `Send + Sync` and uses lock-free atomics where possible.
/// The lifecycle state machine (`AtomicLifecycle`) ensures safe state transitions
/// without the risk of deadlocks.
pub struct WryBackend {
    /// Lifecycle state machine (lock-free)
    lifecycle: AtomicLifecycle,
    /// Current URL (tracked locally since wry doesn't provide direct access)
    current_url: RwLock<Option<String>>,
    /// Current title (tracked locally)
    current_title: RwLock<Option<String>>,
    /// Whether the WebView is loading
    is_loading: AtomicBool,
    /// Load progress percentage (0-100)
    load_progress: AtomicU8,
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
    /// Create a new Wry backend in Active state
    pub fn new() -> Self {
        Self {
            lifecycle: AtomicLifecycle::new_active(),
            current_url: RwLock::new(None),
            current_title: RwLock::new(None),
            is_loading: AtomicBool::new(false),
            load_progress: AtomicU8::new(0),
            settings: Box::new(WebViewSettingsImpl::default()),
            user_agent: format!("AuroraView/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Create a new Wry backend in Creating state
    pub fn new_creating() -> Self {
        Self {
            lifecycle: AtomicLifecycle::new(),
            current_url: RwLock::new(None),
            current_title: RwLock::new(None),
            is_loading: AtomicBool::new(false),
            load_progress: AtomicU8::new(0),
            settings: Box::new(WebViewSettingsImpl::default()),
            user_agent: format!("AuroraView/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Activate the backend (transition from Creating to Active)
    pub fn activate(&self) -> bool {
        self.lifecycle.activate().is_success()
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
        self.is_loading.store(loading, Ordering::Release);
        if !loading {
            self.load_progress.store(100, Ordering::Release);
        }
    }

    /// Set load progress
    pub fn set_load_progress(&self, progress: u8) {
        self.load_progress
            .store(progress.min(100), Ordering::Release);
    }

    /// Apply settings from a WebViewSettingsImpl
    pub fn apply_settings(&mut self, settings: WebViewSettingsImpl) {
        *self.settings = settings;
    }

    /// Get the lifecycle state machine
    pub fn lifecycle(&self) -> &AtomicLifecycle {
        &self.lifecycle
    }

    /// Mark navigation as complete
    pub fn navigation_complete(&self) {
        self.set_loading(false);
        self.set_load_progress(100);
    }

    /// Mark navigation as failed
    pub fn navigation_failed(&self) {
        self.set_loading(false);
    }
}

impl WebViewBackend for WryBackend {
    // ========== Navigation ==========

    fn navigate(&self, url: &str) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
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
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        Err(WebViewError::Unsupported("go_back".to_string()))
    }

    fn go_forward(&self) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        Err(WebViewError::Unsupported("go_forward".to_string()))
    }

    fn reload(&self) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
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
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        self.set_loading(true);
        self.set_load_progress(0);
        Ok(())
    }

    fn title(&self) -> Option<String> {
        self.current_title.read().ok().and_then(|t| t.clone())
    }

    fn load_progress(&self) -> LoadProgress {
        LoadProgress {
            percent: self.load_progress.load(Ordering::Acquire),
            is_complete: !self.is_loading.load(Ordering::Acquire),
        }
    }

    fn is_loading(&self) -> bool {
        self.is_loading.load(Ordering::Acquire)
    }

    // ========== JavaScript ==========

    fn eval_js(&self, _script: &str) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        // Note: Actual JS execution is handled by the main WebView
        Ok(())
    }

    fn eval_js_with_callback(
        &self,
        _script: &str,
        callback: JavaScriptCallback,
    ) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
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

    fn lifecycle_state(&self) -> LifecycleState {
        self.lifecycle.state()
    }

    fn close(&self) -> WebViewResult<()> {
        // Request close through the state machine
        let _ = self.lifecycle.request_close();
        let _ = self.lifecycle.begin_destroy();
        let _ = self.lifecycle.finish_destroy();
        Ok(())
    }

    // ========== Window Control ==========

    fn set_bounds(&self, _x: i32, _y: i32, _width: u32, _height: u32) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        // Note: Window bounds are managed by the main window
        Ok(())
    }

    fn set_visible(&self, _visible: bool) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        Ok(())
    }

    fn focus(&self) -> WebViewResult<()> {
        if self.lifecycle.is_closing() {
            return Err(WebViewError::Closed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wry_backend_creation() {
        let backend = WryBackend::new();
        assert!(backend.lifecycle.is_active());
        assert!(!backend.is_loading());
        assert!(backend.url().is_none());
    }

    #[test]
    fn test_wry_backend_navigation() {
        let backend = WryBackend::new();

        backend.navigate("https://example.com").unwrap();
        assert_eq!(backend.url(), Some("https://example.com".to_string()));
        assert!(backend.is_loading());

        backend.navigation_complete();
        assert!(!backend.is_loading());
        assert_eq!(backend.load_progress().percent, 100);
    }

    #[test]
    fn test_wry_backend_lifecycle() {
        let backend = WryBackend::new_creating();
        assert_eq!(backend.lifecycle_state(), LifecycleState::Creating);

        backend.activate();
        assert_eq!(backend.lifecycle_state(), LifecycleState::Active);

        backend.close().unwrap();
        assert!(backend.is_closed());
    }

    #[test]
    fn test_wry_backend_closed_operations() {
        let backend = WryBackend::new();
        backend.close().unwrap();

        // Operations should fail when closed
        assert!(backend.navigate("https://example.com").is_err());
        assert!(backend.eval_js("console.log('test')").is_err());
        assert!(backend.set_visible(true).is_err());
    }
}
