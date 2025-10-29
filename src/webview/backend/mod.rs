//! WebView backend abstraction layer
//!
//! This module defines the trait-based architecture for supporting multiple
//! window integration modes (native embedding, Qt integration, etc.).

use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use crate::ipc::{IpcHandler, MessageQueue};
use super::config::WebViewConfig;
use super::event_loop::UserEvent;

pub mod native;
pub mod qt;

/// Backend trait that all WebView implementations must implement
///
/// This trait defines the common interface for different window integration modes.
/// Each backend is responsible for creating and managing the WebView in its specific context.
///
/// Note: We don't require `Send` because WebView and EventLoop are not Send on Windows.
/// The backend is designed to be used from a single thread (the UI thread).
pub trait WebViewBackend {
    /// Create a new backend instance
    ///
    /// # Arguments
    /// * `config` - WebView configuration
    /// * `ipc_handler` - IPC message handler
    /// * `message_queue` - Thread-safe message queue
    ///
    /// # Returns
    /// A new backend instance or an error
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

    /// Get the event loop (if available)
    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>>;

    /// Process pending events (for embedded mode)
    ///
    /// Returns true if the window should be closed
    fn process_events(&self) -> bool;

    /// Run the event loop (blocking, for standalone mode)
    fn run_event_loop_blocking(&mut self);

    /// Load a URL
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let script = format!("window.location.href = '{}';", url);
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(&script)?;
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
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: Object.assign({{}}, {{__aurora_from_python: true}}, {}) }}))",
            event_name, data
        );
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(&script)?;
        }
        Ok(())
    }
}

/// Backend type enum for runtime selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Native embedding mode (using platform-specific APIs)
    Native,
    /// Qt integration mode (for DCC environments with Qt)
    Qt,
}

impl BackendType {
    /// Parse backend type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" => Some(BackendType::Native),
            "qt" => Some(BackendType::Qt),
            _ => None,
        }
    }

    /// Auto-detect the best backend for the current environment
    pub fn auto_detect() -> Self {
        // TODO: Implement Qt detection logic
        // For now, always use native backend
        BackendType::Native
    }
}

