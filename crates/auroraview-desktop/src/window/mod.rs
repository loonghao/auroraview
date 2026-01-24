//! Window creation and management for desktop mode

pub mod builder;
mod state;

pub use builder::{create_window, create_window_with_router};

use crate::config::DesktopConfig;
use crate::error::{DesktopError, Result};
use crate::ipc::IpcRouter;
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::WebView;

use crate::event_loop::UserEvent;

/// Desktop window with WebView
pub struct DesktopWindow {
    /// The WebView instance
    pub(crate) webview: Arc<Mutex<WebView>>,

    /// The window instance
    pub(crate) window: Window,

    /// Event loop proxy for sending events
    pub(crate) event_loop_proxy: EventLoopProxy<UserEvent>,

    /// Window configuration
    pub(crate) config: DesktopConfig,

    /// IPC router for handling JS messages
    pub(crate) router: Arc<IpcRouter>,

    /// Cached HWND (Windows only)
    #[cfg(target_os = "windows")]
    pub(crate) hwnd: Option<u64>,
}

impl DesktopWindow {
    /// Get the window title
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Set the window title
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Show the window
    pub fn show(&self) {
        self.window.set_visible(true);
    }

    /// Hide the window
    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    /// Check if window is visible
    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    /// Set window size
    pub fn set_size(&self, width: u32, height: u32) {
        self.window
            .set_inner_size(tao::dpi::LogicalSize::new(width, height));
    }

    /// Get window size
    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    /// Set window position
    pub fn set_position(&self, x: i32, y: i32) {
        self.window
            .set_outer_position(tao::dpi::LogicalPosition::new(x, y));
    }

    /// Get window position
    pub fn position(&self) -> (i32, i32) {
        let pos = self.window.outer_position().unwrap_or_default();
        (pos.x, pos.y)
    }

    /// Maximize the window
    pub fn maximize(&self) {
        self.window.set_maximized(true);
    }

    /// Minimize the window
    pub fn minimize(&self) {
        self.window.set_minimized(true);
    }

    /// Restore the window
    pub fn restore(&self) {
        self.window.set_maximized(false);
        self.window.set_minimized(false);
    }

    /// Focus the window
    pub fn focus(&self) {
        self.window.set_focus();
    }

    /// Close the window
    pub fn close(&self) {
        let _ = self.event_loop_proxy.send_event(UserEvent::CloseWindow);
    }

    /// Navigate to URL
    pub fn navigate(&self, url: &str) -> Result<()> {
        if let Ok(webview) = self.webview.lock() {
            webview
                .load_url(url)
                .map_err(|e| DesktopError::WebViewCreation(e.to_string()))?;
        }
        Ok(())
    }

    /// Load HTML content
    pub fn load_html(&self, html: &str) -> Result<()> {
        if let Ok(webview) = self.webview.lock() {
            webview
                .load_html(html)
                .map_err(|e| DesktopError::WebViewCreation(e.to_string()))?;
        }
        Ok(())
    }

    /// Evaluate JavaScript
    pub fn eval(&self, script: &str) -> Result<()> {
        if let Ok(webview) = self.webview.lock() {
            webview
                .evaluate_script(script)
                .map_err(|e| DesktopError::WebViewCreation(e.to_string()))?;
        }
        Ok(())
    }

    /// Get the HWND (Windows only)
    #[cfg(target_os = "windows")]
    pub fn hwnd(&self) -> Option<u64> {
        self.hwnd
    }

    /// Get the IPC router
    pub fn router(&self) -> &Arc<IpcRouter> {
        &self.router
    }
}
