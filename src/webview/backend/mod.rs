//! WebView backend abstraction layer for Python bindings
//!
//! This module provides the Python-bindings-specific backend implementations.
//! Core backend abstractions are defined in `auroraview-core`.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │           auroraview-core::backend::traits              │
//! │  (Platform-agnostic WebViewBackend trait)               │
//! └─────────────────────────────────────────────────────────┘
//!                          ↑
//!                    implements
//!                          │
//! ┌─────────────────────────────────────────────────────────┐
//! │              NativeBackend (this module)                │
//! │  (Python bindings WebView with IPC, MessageQueue)       │
//! └─────────────────────────────────────────────────────────┘
//!                          ↓
//!                    uses wry/tao
//!                          ↓
//! ┌─────────────────────────────────────────────────────────┐
//! │     Platform WebView (WebView2/WebKit/WebKitGTK)        │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use super::js_assets;
use crate::ipc::{IpcHandler, MessageQueue};

pub mod native;

// Re-export core backend types for convenience
pub use auroraview_core::backend::{
    BackendFactory, BackendType as CoreBackendType, WebViewBackend as CoreWebViewBackend,
    WebViewSettings, WryBackend,
};

/// Python bindings backend trait
///
/// This trait extends the core WebViewBackend with Python-bindings-specific
/// functionality like IPC handling, message queues, and event loops.
///
/// Note: We don't require `Send` because WebView and EventLoop are not Send on Windows.
/// The backend is designed to be used from a single thread (the UI thread).
#[allow(dead_code)]
pub trait WebViewBackend {
    /// Create a new backend instance
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Get the underlying WebView instance
    fn webview(&self) -> Arc<Mutex<WryWebView>>;

    /// Get the message queue
    fn message_queue(&self) -> Arc<MessageQueue>;

    /// Get the window handle (if available)
    fn window(&self) -> Option<&tao::window::Window>;

    /// Get the native window handle (HWND on Windows)
    ///
    /// Returns the platform-specific window handle for integration with external applications.
    fn get_hwnd(&self) -> Option<u64> {
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

    /// Get the event loop (if available)
    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>>;

    /// Process pending events (for embedded mode)
    /// Returns true if the window should be closed
    fn process_events(&self) -> bool;

    /// Run the event loop (blocking, for standalone mode)
    fn run_event_loop_blocking(&mut self);

    /// Load a URL
    ///
    /// Uses native WebView load_url() for reliable navigation.
    /// This is preferred over JavaScript-based navigation (window.location.href)
    /// as it handles all edge cases including splash screen transitions.
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(webview) = self.webview().lock() {
            webview.load_url(url)?;
        }
        Ok(())
    }

    /// Load HTML content
    fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(webview) = self.webview().lock() {
            webview.load_html(html)?;
        }
        Ok(())
    }

    /// Execute JavaScript
    fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(script)?;
        }
        Ok(())
    }

    /// Emit an event to JavaScript
    fn emit(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
}

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
