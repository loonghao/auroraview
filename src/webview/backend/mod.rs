//! WebView backend abstraction layer
//!
//! This module defines the trait-based architecture for supporting multiple
//! rendering engines and window integration modes.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              WebViewBackend Trait                       │
//! │  (Common interface for all rendering engines)           │
//! └─────────────────────────────────────────────────────────┘
//!                          ↓
//!     ┌────────────────────┼────────────────────┐
//!     ↓                    ↓                    ↓
//! ┌─────────┐      ┌──────────┐        ┌──────────┐
//! │  Wry    │      │  Servo   │        │   Qt     │
//! │ Backend │      │ Backend  │        │ Backend  │
//! │(Current)│      │ (Future) │        │ (Future) │
//! └─────────┘      └──────────┘        └──────────┘
//!     ↓                    ↓                    ↓
//! WebView2/          WebRender/           Qt WebEngine
//! WebKit             Stylo
//! ```
//!
//! ## Supported Backends
//!
//! - **Native (Wry)**: Current implementation using system WebView
//!   - Windows: WebView2 (Chromium-based)
//!   - macOS: WebKit
//!   - Linux: WebKitGTK
//!
//! - **Servo**: Future implementation using Servo rendering engine
//!   - Pure Rust implementation
//!   - High-performance parallel rendering
//!   - Full control over rendering pipeline
//!
//! - **Qt**: Future implementation for Qt-based DCC applications
//!   - Qt WebEngine integration
//!   - Native Qt widget embedding

use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;
use crate::ipc::{IpcHandler, MessageQueue};

pub mod native;
pub mod qt;

// Future backends (currently disabled)
// #[cfg(feature = "servo-backend")]
// pub mod servo;

// #[cfg(feature = "custom-backend")]
// pub mod custom;

/// Backend trait that all WebView implementations must implement
///
/// This trait defines the common interface for different window integration modes.
/// Each backend is responsible for creating and managing the WebView in its specific context.
///
/// Note: We don't require `Send` because WebView and EventLoop are not Send on Windows.
/// The backend is designed to be used from a single thread (the UI thread).
#[allow(dead_code)]
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
        // Properly escape JSON data to avoid JavaScript syntax errors
        let json_str = data.to_string();
        let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: Object.assign({{}}, {{__aurora_from_python: true}}, JSON.parse('{}')) }}))",
            event_name, escaped_json
        );
        if let Ok(webview) = self.webview().lock() {
            webview.evaluate_script(&script)?;
        }
        Ok(())
    }
}

/// Rendering engine type for backend selection
///
/// This enum allows runtime selection of different rendering engines.
/// New engines can be added without breaking existing code.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingEngine {
    /// System WebView (WebView2/WebKit/WebKitGTK)
    ///
    /// **Pros**: Small binary, system integration, mature
    /// **Cons**: Version inconsistency, limited control
    SystemWebView,
    // Servo rendering engine (future - currently disabled)
    //
    // **Pros**: Full control, high performance, pure Rust
    // **Cons**: Large binary, experimental, limited compatibility
    // #[cfg(feature = "servo-backend")]
    // Servo,

    // Custom rendering engine (future - currently disabled)
    //
    // Allows users to provide their own rendering implementation
    // #[cfg(feature = "custom-backend")]
    // Custom,
}

/// Backend type enum for runtime selection
///
/// Combines rendering engine with integration mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Native embedding mode (using platform-specific APIs)
    ///
    /// Uses system WebView by default, but can be configured
    /// to use other rendering engines.
    Native {
        /// Rendering engine to use
        engine: RenderingEngine,
    },

    /// Qt integration mode (for DCC environments with Qt)
    Qt {
        /// Rendering engine to use
        engine: RenderingEngine,
    },
}

#[allow(dead_code)]
impl BackendType {
    /// Create a native backend with system WebView
    pub fn native() -> Self {
        BackendType::Native {
            engine: RenderingEngine::SystemWebView,
        }
    }

    // Create a native backend with Servo (currently disabled)
    // #[cfg(feature = "servo-backend")]
    // pub fn native_servo() -> Self {
    //     BackendType::Native {
    //         engine: RenderingEngine::Servo,
    //     }
    // }

    /// Create a Qt backend with system WebView
    pub fn qt() -> Self {
        BackendType::Qt {
            engine: RenderingEngine::SystemWebView,
        }
    }

    /// Parse backend type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" => Some(Self::native()),
            "qt" => Some(Self::qt()),
            // #[cfg(feature = "servo-backend")]
            // "native-servo" | "servo" => Some(Self::native_servo()),
            _ => None,
        }
    }

    /// Auto-detect the best backend for the current environment
    pub fn auto_detect() -> Self {
        // TODO: Implement Qt detection logic
        // For now, always use native backend with system WebView
        Self::native()
    }

    /// Get the rendering engine
    pub fn engine(&self) -> RenderingEngine {
        match self {
            BackendType::Native { engine } => *engine,
            BackendType::Qt { engine } => *engine,
        }
    }
}
