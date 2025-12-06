//! WebView backend trait definitions
//!
//! Core trait defining the unified interface for WebView backends,
//! inspired by Qt WebView's `QWebViewPrivate` abstract class.

use super::error::WebViewResult;
use super::settings::WebViewSettings;

/// Navigation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationState {
    /// Navigation started
    Started,
    /// Navigation in progress
    InProgress,
    /// Navigation completed successfully
    Completed,
    /// Navigation failed
    Failed,
}

/// Navigation event
#[derive(Debug, Clone)]
pub struct NavigationEvent {
    /// URL being navigated to
    pub url: String,
    /// Current navigation state
    pub state: NavigationState,
    /// Error message if navigation failed
    pub error: Option<String>,
}

/// Load progress information
#[derive(Debug, Clone, Copy)]
pub struct LoadProgress {
    /// Progress percentage (0-100)
    pub percent: u8,
    /// Whether loading is complete
    pub is_complete: bool,
}

/// Cookie information
#[derive(Debug, Clone)]
pub struct CookieInfo {
    /// Cookie domain
    pub domain: String,
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Cookie path
    pub path: Option<String>,
    /// Expiration timestamp (Unix epoch)
    pub expires: Option<i64>,
    /// HTTP-only flag
    pub http_only: bool,
    /// Secure flag
    pub secure: bool,
}

/// JavaScript execution callback type
pub type JavaScriptCallback = Box<dyn FnOnce(WebViewResult<serde_json::Value>) + Send + 'static>;

/// Unified WebView backend trait
///
/// This trait provides a platform-agnostic interface for WebView operations,
/// inspired by Qt WebView's `QWebViewPrivate` abstract class.
///
/// ## Design Philosophy
///
/// - All methods return `WebViewResult` for consistent error handling
/// - Async operations use callbacks instead of futures for compatibility
/// - Settings are accessed through a separate trait for clean separation
pub trait WebViewBackend: Send + Sync {
    // ========== Navigation ==========

    /// Navigate to a URL
    fn navigate(&self, url: &str) -> WebViewResult<()>;

    /// Get current URL
    fn url(&self) -> Option<String>;

    /// Check if can go back in history
    fn can_go_back(&self) -> bool;

    /// Check if can go forward in history
    fn can_go_forward(&self) -> bool;

    /// Go back in history
    fn go_back(&self) -> WebViewResult<()>;

    /// Go forward in history
    fn go_forward(&self) -> WebViewResult<()>;

    /// Reload current page
    fn reload(&self) -> WebViewResult<()>;

    /// Stop loading
    fn stop(&self) -> WebViewResult<()>;

    // ========== Content Loading ==========

    /// Load HTML content directly
    fn load_html(&self, html: &str) -> WebViewResult<()>;

    /// Get current page title
    fn title(&self) -> Option<String>;

    /// Get load progress
    fn load_progress(&self) -> LoadProgress;

    /// Check if currently loading
    fn is_loading(&self) -> bool;

    // ========== JavaScript ==========

    /// Execute JavaScript and discard result
    fn eval_js(&self, script: &str) -> WebViewResult<()>;

    /// Execute JavaScript with result callback
    fn eval_js_with_callback(
        &self,
        script: &str,
        callback: JavaScriptCallback,
    ) -> WebViewResult<()>;

    // ========== Cookie Management ==========

    /// Set a cookie
    fn set_cookie(&self, cookie: &CookieInfo) -> WebViewResult<()>;

    /// Get a cookie by domain and name
    fn get_cookie(&self, domain: &str, name: &str) -> WebViewResult<Option<CookieInfo>>;

    /// Delete a cookie
    fn delete_cookie(&self, domain: &str, name: &str) -> WebViewResult<()>;

    /// Delete all cookies
    fn clear_cookies(&self) -> WebViewResult<()>;

    // ========== Settings ==========

    /// Get settings reference
    fn settings(&self) -> &dyn WebViewSettings;

    /// Get mutable settings reference
    fn settings_mut(&mut self) -> &mut dyn WebViewSettings;

    /// Get HTTP user agent
    fn http_user_agent(&self) -> String;

    // ========== Lifecycle ==========

    /// Close the WebView
    fn close(&self) -> WebViewResult<()>;

    /// Check if WebView is closed
    fn is_closed(&self) -> bool;

    // ========== Window Control ==========

    /// Set window bounds
    fn set_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> WebViewResult<()>;

    /// Set window visibility
    fn set_visible(&self, visible: bool) -> WebViewResult<()>;

    /// Focus the WebView
    fn focus(&self) -> WebViewResult<()>;
}
