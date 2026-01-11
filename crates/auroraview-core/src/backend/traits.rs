//! WebView backend trait definitions
//!
//! Core trait defining the unified interface for WebView backends,
//! inspired by Qt WebView's `QWebViewPrivate` abstract class.
//!
//! ## Architecture
//!
//! This module provides a layered trait system:
//!
//! - `WebViewBackend`: Core WebView operations (navigation, JS, cookies)
//! - `EmbeddableBackend`: Extended trait for DCC/Qt embedding scenarios
//! - `EventLoopBackend`: Trait for backends that own their event loop
//!
//! ## Thread Safety
//!
//! The `WebViewBackend` trait requires `Send + Sync` for the base operations.
//! However, actual WebView instances (like WryWebView) are typically `!Send`.
//!
//! The pattern is:
//! - State tracking (URLs, loading status) is thread-safe via atomics/RwLock
//! - Actual WebView operations are marshalled to the UI thread via message queue

use super::error::WebViewResult;
use super::lifecycle::LifecycleState;
use super::message_processor::ProcessResult;
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
#[derive(Debug, Clone, Copy, Default)]
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

impl CookieInfo {
    /// Create a simple session cookie
    pub fn session(
        domain: impl Into<String>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            name: name.into(),
            value: value.into(),
            path: Some("/".to_string()),
            expires: None,
            http_only: false,
            secure: false,
        }
    }
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
/// - Lifecycle management uses lock-free atomics (see `AtomicLifecycle`)
///
/// ## Implementation Notes
///
/// Implementors should:
/// 1. Use `AtomicLifecycle` for state management to avoid deadlocks
/// 2. Marshal UI operations to the appropriate thread
/// 3. Track state (URL, title, loading) separately from the actual WebView
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

    /// Get current lifecycle state
    fn lifecycle_state(&self) -> LifecycleState;

    /// Close the WebView
    fn close(&self) -> WebViewResult<()>;

    /// Check if WebView is closed
    fn is_closed(&self) -> bool {
        self.lifecycle_state() == LifecycleState::Destroyed
    }

    /// Check if close has been requested
    fn is_closing(&self) -> bool {
        matches!(
            self.lifecycle_state(),
            LifecycleState::CloseRequested | LifecycleState::Destroying | LifecycleState::Destroyed
        )
    }

    // ========== Window Control ==========

    /// Set window bounds
    fn set_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> WebViewResult<()>;

    /// Set window visibility
    fn set_visible(&self, visible: bool) -> WebViewResult<()>;

    /// Focus the WebView
    fn focus(&self) -> WebViewResult<()>;
}

/// Extended backend trait for embeddable WebViews
///
/// This trait extends `WebViewBackend` with functionality needed for
/// embedding WebViews into DCC applications (Maya, Houdini, etc.) or Qt hosts.
///
/// ## Design Rationale
///
/// DCC/Qt embedding requires:
/// - Platform-specific window handle access (HWND on Windows)
/// - Message processing that cooperates with the host's event loop
/// - Control over whether to pump messages or let the host do it
pub trait EmbeddableBackend: WebViewBackend {
    /// Get the native window handle (HWND on Windows, NSView on macOS)
    fn native_handle(&self) -> Option<u64>;

    /// Process pending events/messages
    ///
    /// This is the unified entry point for message processing.
    /// The behavior depends on the processing mode configured.
    ///
    /// Returns `ProcessResult::CloseRequested` if the window should close.
    fn process_events(&self) -> ProcessResult;

    /// Process only IPC messages, skip native message pump
    ///
    /// Use this in Qt/DCC mode where the host owns the message loop.
    /// This only processes the internal message queue.
    fn process_ipc_only(&self) -> ProcessResult;

    /// Emit an event to JavaScript
    ///
    /// This is a convenience method that builds and executes the
    /// appropriate JavaScript to trigger an event.
    fn emit_event(&self, event_name: &str, data: serde_json::Value) -> WebViewResult<()>;
}

/// Backend trait for standalone mode with owned event loop
///
/// This trait is for backends that own and manage their own event loop,
/// typically used in standalone/desktop mode.
pub trait EventLoopBackend: WebViewBackend {
    /// Run the event loop (blocking)
    ///
    /// This takes ownership of the current thread and runs until
    /// the window is closed.
    fn run_blocking(&mut self) -> WebViewResult<()>;

    /// Poll the event loop once (non-blocking)
    ///
    /// Processes any pending events and returns immediately.
    /// Returns `ProcessResult::CloseRequested` if the window should close.
    fn poll_once(&mut self) -> ProcessResult;

    /// Check if the event loop is running
    fn is_running(&self) -> bool;

    /// Request the event loop to exit
    fn request_exit(&self);
}
