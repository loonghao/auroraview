//! Custom rendering backend interface
//!
//! This module allows users to provide their own rendering engine implementation.
//! It is feature-gated and only available when the `custom-backend` feature is enabled.
//!
//! ## Use Cases
//!
//! - Integrating proprietary rendering engines
//! - Experimenting with new rendering technologies
//! - Custom DCC-specific rendering pipelines
//! - Research and development
//!
//! ## Example
//!
//! ```rust,ignore
//! use auroraview::backend::{WebViewBackend, CustomBackend};
//!
//! struct MyCustomRenderer {
//!     // Your custom rendering implementation
//! }
//!
//! impl CustomRenderer for MyCustomRenderer {
//!     fn render_html(&mut self, html: &str) {
//!         // Your rendering logic
//!     }
//!     
//!     fn execute_script(&mut self, script: &str) {
//!         // Your JavaScript execution logic
//!     }
//! }
//!
//! // Use your custom renderer
//! let backend = CustomBackend::new(MyCustomRenderer::new());
//! ```

use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

use super::super::config::WebViewConfig;
use super::super::event_loop::UserEvent;
use super::WebViewBackend;
use crate::ipc::{IpcHandler, MessageQueue};

/// Custom renderer trait
///
/// Implement this trait to provide your own rendering engine.
pub trait CustomRenderer: Send + Sync {
    /// Initialize the renderer
    fn initialize(&mut self, config: &WebViewConfig) -> Result<(), Box<dyn std::error::Error>>;

    /// Render HTML content
    fn render_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Load a URL
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Execute JavaScript
    fn execute_script(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Process events (return true if should close)
    fn process_events(&mut self) -> bool;

    /// Run event loop (blocking)
    fn run_event_loop(&mut self);

    /// Get window handle (if available)
    fn window_handle(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
}

/// Custom backend wrapper
///
/// Wraps a user-provided CustomRenderer and implements WebViewBackend.
pub struct CustomBackend<R: CustomRenderer> {
    /// Custom renderer implementation
    renderer: Arc<Mutex<R>>,

    /// IPC handler
    ipc_handler: Arc<IpcHandler>,

    /// Message queue
    message_queue: Arc<MessageQueue>,

    /// Configuration
    config: WebViewConfig,
}

impl<R: CustomRenderer> CustomBackend<R> {
    /// Create a new custom backend
    pub fn new(
        renderer: R,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut renderer = renderer;
        renderer.initialize(&config)?;

        Ok(Self {
            renderer: Arc::new(Mutex::new(renderer)),
            ipc_handler,
            message_queue,
            config,
        })
    }
}

impl<R: CustomRenderer + 'static> WebViewBackend for CustomBackend<R> {
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        Err("CustomBackend cannot be created via WebViewBackend::create(). Use CustomBackend::new() instead.".into())
    }

    fn webview(&self) -> Arc<Mutex<WryWebView>> {
        // Custom backends don't use WryWebView
        unimplemented!("Custom backend does not use WryWebView")
    }

    fn message_queue(&self) -> Arc<MessageQueue> {
        self.message_queue.clone()
    }

    fn window(&self) -> Option<&tao::window::Window> {
        // Custom backends may not use tao windows
        None
    }

    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>> {
        // Custom backends may not use tao event loops
        None
    }

    fn process_events(&self) -> bool {
        if let Ok(mut renderer) = self.renderer.lock() {
            renderer.process_events()
        } else {
            false
        }
    }

    fn run_event_loop_blocking(&mut self) {
        if let Ok(mut renderer) = self.renderer.lock() {
            renderer.run_event_loop();
        }
    }

    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut renderer) = self.renderer.lock() {
            renderer.load_url(url)
        } else {
            Err("Failed to lock renderer".into())
        }
    }

    fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut renderer) = self.renderer.lock() {
            renderer.render_html(html)
        } else {
            Err("Failed to lock renderer".into())
        }
    }

    fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut renderer) = self.renderer.lock() {
            renderer.execute_script(script)
        } else {
            Err("Failed to lock renderer".into())
        }
    }

    fn emit(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Emit event by executing JavaScript
        // Properly escape JSON data to avoid JavaScript syntax errors
        let json_str = data.to_string();
        let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}))",
            event_name, escaped_json
        );
        self.eval_js(&script)
    }
}

/// Example custom renderer (for testing/demonstration)
#[cfg(test)]
pub struct DummyRenderer {
    html: String,
    scripts: Vec<String>,
}

#[cfg(test)]
impl DummyRenderer {
    pub fn new() -> Self {
        Self {
            html: String::new(),
            scripts: Vec::new(),
        }
    }
}

#[cfg(test)]
impl CustomRenderer for DummyRenderer {
    fn initialize(&mut self, _config: &WebViewConfig) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn render_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.html = html.to_string();
        Ok(())
    }

    fn load_url(&mut self, _url: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn execute_script(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.scripts.push(script.to_string());
        Ok(())
    }

    fn process_events(&mut self) -> bool {
        false
    }

    fn run_event_loop(&mut self) {
        // No-op for dummy renderer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_renderer() {
        let mut renderer = DummyRenderer::new();

        renderer.render_html("<h1>Test</h1>").unwrap();
        assert_eq!(renderer.html, "<h1>Test</h1>");

        renderer.execute_script("console.log('test')").unwrap();
        assert_eq!(renderer.scripts.len(), 1);
    }
}
