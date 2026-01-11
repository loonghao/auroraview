//! WebView backend abstraction layer for Python bindings
//!
//! This module provides the Python-bindings-specific backend implementations.
//! Core backend abstractions are defined in `auroraview-core`.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                    auroraview-core::backend                              │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
//! │  │ WebViewBackend  │  │ AtomicLifecycle │  │ ProcessorConfig │         │
//! │  │ (Core trait)    │  │ (Lock-free)     │  │ (Unified)       │         │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    ↑
//!                              implements
//!                                    │
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │              PyBindingsBackend (this module)                             │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
//! │  │ NativeBackend   │  │ IpcHandler      │  │ MessageQueue    │         │
//! │  │ (wry/tao)       │  │ (Python calls)  │  │ (crossbeam)     │         │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    ↓
//!                              uses wry/tao
//!                                    ↓
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │         Platform WebView (WebView2/WebKit/WebKitGTK)                     │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Design Decisions
//!
//! ### Separation of Concerns
//!
//! - `auroraview-core::backend::WebViewBackend`: Platform-agnostic operations
//! - `PyBindingsBackend`: Python-specific extensions (IPC, MessageQueue)
//! - `NativeBackend`: Actual wry/tao implementation
//!
//! ### Thread Safety
//!
//! The `PyBindingsBackend` trait does NOT require `Send` because:
//! - `WryWebView` is `!Send` on Windows (COM threading model)
//! - `EventLoop` is `!Send` (platform event loop constraints)
//!
//! Thread-safe operations are marshalled through `MessageQueue`.

use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::js_assets;
use crate::ipc::{IpcHandler, MessageQueue};

pub mod native;

// Re-export core backend types for convenience
pub use auroraview_core::backend::{
    // Lifecycle management (P0 fix)
    AtomicLifecycle,
    // Message processing (P2 fix)
    AtomicProcessorStats,
    // Factory and types
    BackendConfig,
    BackendFactory,
    BackendType as CoreBackendType,
    // Core traits and types
    CookieInfo,
    EmbeddableBackend,
    EventLoopBackend,
    JavaScriptCallback,
    LifecycleEvent,
    LifecycleObserver,
    LifecycleState,
    LoadProgress,
    MessagePriority,
    NavigationEvent,
    NavigationState,
    ObservableLifecycle,
    ProcessResult,
    ProcessingMode,
    ProcessorConfig,
    ProcessorStats,
    TransitionResult,
    WakeController,
    WebViewBackend as CoreWebViewBackend,
    WebViewError,
    WebViewResult,
    WebViewSettings,
    WryBackend,
};

/// Python bindings backend trait
///
/// This trait extends the core `WebViewBackend` with Python-bindings-specific
/// functionality like IPC handling, message queues, and event loops.
///
/// ## Design Rationale
///
/// We use a separate trait instead of extending `CoreWebViewBackend` because:
/// 1. `WryWebView` and `EventLoop` are `!Send` on Windows
/// 2. Python bindings need direct access to wry/tao types
/// 3. IPC and MessageQueue are specific to the Python binding layer
///
/// ## Thread Model
///
/// This trait is designed for single-threaded use on the UI thread.
/// Cross-thread operations should go through `MessageQueue`.
#[allow(dead_code)]
pub trait PyBindingsBackend {
    /// Create a new backend instance
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Get the underlying WebView instance
    ///
    /// Note: The Mutex is for interior mutability, not thread safety.
    /// All access should be from the UI thread.
    fn webview(&self) -> Arc<Mutex<WryWebView>>;

    /// Get the message queue for cross-thread communication
    fn message_queue(&self) -> Arc<MessageQueue>;

    /// Get the window handle (if available)
    fn window(&self) -> Option<&tao::window::Window>;

    /// Get the native window handle (HWND on Windows)
    ///
    /// Returns the platform-specific window handle for DCC integration.
    fn native_handle(&self) -> Option<u64> {
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Some(window) = self.window() {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        return Some(handle.hwnd.get() as u64);
                    }
                }
            }
        }
        None
    }

    /// Get the lifecycle state
    fn lifecycle_state(&self) -> LifecycleState;

    /// Check if the backend is closing or closed
    fn is_closing(&self) -> bool {
        matches!(
            self.lifecycle_state(),
            LifecycleState::CloseRequested | LifecycleState::Destroying | LifecycleState::Destroyed
        )
    }

    /// Get the event loop (consumes it, for standalone mode)
    fn take_event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>>;

    /// Process pending events
    ///
    /// This is the unified entry point for event processing.
    /// Returns `ProcessResult` indicating whether to continue or close.
    fn process_events(&self) -> ProcessResult;

    /// Process only IPC messages (skip native message pump)
    ///
    /// Use this in Qt/DCC mode where the host owns the message loop.
    fn process_ipc_only(&self) -> ProcessResult;

    /// Run the event loop (blocking, for standalone mode)
    fn run_blocking(&mut self);

    /// Load a URL using native WebView navigation
    ///
    /// Preferred over JavaScript-based navigation for reliability.
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_closing() {
            return Err("WebView is closing".into());
        }
        if let Ok(webview) = self.webview().lock() {
            webview.load_url(url)?;
        }
        Ok(())
    }

    /// Load HTML content directly
    fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_closing() {
            return Err("WebView is closing".into());
        }
        if let Ok(webview) = self.webview().lock() {
            webview.load_html(html)?;
        }
        Ok(())
    }

    /// Execute JavaScript
    fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_closing() {
            return Err("WebView is closing".into());
        }
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(script)?;
        }
        Ok(())
    }

    /// Emit an event to JavaScript
    ///
    /// Uses the unified `window.auroraview.trigger()` mechanism.
    fn emit(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_closing() {
            return Err("WebView is closing".into());
        }
        let json_str = data.to_string();
        let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
        let script = js_assets::build_emit_event_script(event_name, &escaped_json);
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(&script)?;
        }
        Ok(())
    }

    /// Set window visibility
    fn set_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>>;

    /// Request close
    fn request_close(&self) -> Result<(), Box<dyn std::error::Error>>;
}

// Keep the old trait name as an alias for backwards compatibility
#[allow(dead_code)]
pub trait WebViewBackend: PyBindingsBackend {}
impl<T: PyBindingsBackend> WebViewBackend for T {}

/// Backend type enum for runtime selection
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Native embedding mode (using platform-specific wry/tao)
    Native,
}

#[allow(dead_code)]
impl BackendType {
    /// Create a native backend
    pub fn native() -> Self {
        BackendType::Native
    }

    /// Parse backend type from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" => Some(Self::native()),
            _ => None,
        }
    }

    /// Auto-detect the best backend for the current environment
    pub fn auto_detect() -> Self {
        Self::native()
    }
}
