//! DCC WebView implementation (Windows)
//!
//! Uses WebView2 directly via webview2-com for embedding into Qt widgets.
//!
//! This implementation is designed for DCC environments where:
//! - The host application owns the event loop (Qt in Maya/Houdini/Nuke)
//! - WebView is embedded as a child window into a Qt widget
//! - No blocking event loop - host application calls process_events() periodically

use crate::config::DccConfig;
use crate::error::{DccError, Result};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Message types for internal message queue
#[derive(Debug, Clone)]
pub enum DccMessage {
    Navigate(String),
    LoadHtml(String),
    EvalJs(String),
    Resize { width: u32, height: u32 },
    SetVisible(bool),
    Close,
}

/// DCC-embedded WebView
///
/// Thread-safe wrapper around WebView2 for DCC integration.
pub struct DccWebView {
    config: DccConfig,
    inner: Arc<Mutex<Option<DccWebViewInner>>>,
    /// Message queue for async operations
    messages: Arc<Mutex<VecDeque<DccMessage>>>,
    /// Whether the WebView is initialized
    initialized: Arc<Mutex<bool>>,
}

struct DccWebViewInner {
    /// Parent HWND
    #[allow(dead_code)]
    parent_hwnd: isize,
    /// WebView HWND (created as child of parent)
    webview_hwnd: Option<isize>,
    /// Current visibility state
    visible: bool,
    /// Current size
    width: u32,
    height: u32,
}

impl DccWebView {
    /// Create new DCC WebView
    ///
    /// This creates the WebView wrapper but does not initialize WebView2 yet.
    /// Call `init()` from the UI thread to complete initialization.
    pub fn new(config: DccConfig) -> Result<Self> {
        // Clean up stale WebView user data directories from crashed processes
        // This runs once per process and prevents initialization issues
        match auroraview_core::cleanup::cleanup_stale_webview_dirs() {
            Ok(count) if count > 0 => {
                info!("[DCC] Cleaned up {} stale WebView directories", count);
            }
            Err(e) => {
                debug!("[DCC] Cleanup warning: {}", e);
            }
            _ => {}
        }

        let parent = config.parent_hwnd.ok_or(DccError::InvalidParent)?;

        info!(
            "[DCC] Creating WebView: title='{}', parent=0x{:X}, dcc={}, size={}x{}",
            config.title,
            parent,
            config.dcc_type.name(),
            config.width,
            config.height
        );

        // Initialize COM
        init_com()?;

        Ok(Self {
            config,
            inner: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize the WebView (must be called from UI thread)
    ///
    /// This sets up WebView2 as a child window of the parent HWND.
    pub fn init(&self) -> Result<()> {
        let parent = self.config.parent_hwnd.ok_or(DccError::InvalidParent)?;

        info!("[DCC] Initializing WebView2 on parent 0x{:X}", parent);

        // Create inner state
        let inner = DccWebViewInner {
            parent_hwnd: parent,
            webview_hwnd: None,
            visible: false,
            width: self.config.width,
            height: self.config.height,
        };

        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(inner);
        }

        if let Ok(mut init) = self.initialized.lock() {
            *init = true;
        }

        // Load initial content
        if let Some(ref url) = self.config.url {
            self.navigate(url)?;
        } else if let Some(ref html) = self.config.html {
            self.load_html(html)?;
        }

        Ok(())
    }

    /// Check if WebView is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
            .lock()
            .map(|g| *g)
            .unwrap_or(false)
    }

    /// Navigate to URL
    pub fn navigate(&self, url: &str) -> Result<()> {
        debug!("[DCC] Navigate to: {}", url);
        self.push_message(DccMessage::Navigate(url.to_string()));
        Ok(())
    }

    /// Load HTML content
    pub fn load_html(&self, html: &str) -> Result<()> {
        debug!("[DCC] Load HTML ({} bytes)", html.len());
        self.push_message(DccMessage::LoadHtml(html.to_string()));
        Ok(())
    }

    /// Evaluate JavaScript
    pub fn eval(&self, script: &str) -> Result<()> {
        debug!("[DCC] Eval JS ({} bytes)", script.len());
        self.push_message(DccMessage::EvalJs(script.to_string()));
        Ok(())
    }

    /// Resize WebView
    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        debug!("[DCC] Resize to {}x{}", width, height);
        self.push_message(DccMessage::Resize { width, height });

        // Update cached size
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                state.width = width;
                state.height = height;
            }
        }

        Ok(())
    }

    /// Show WebView
    pub fn show(&self) -> Result<()> {
        debug!("[DCC] Show");
        self.push_message(DccMessage::SetVisible(true));
        Ok(())
    }

    /// Hide WebView
    pub fn hide(&self) -> Result<()> {
        debug!("[DCC] Hide");
        self.push_message(DccMessage::SetVisible(false));
        Ok(())
    }

    /// Request close
    pub fn close(&self) -> Result<()> {
        debug!("[DCC] Close");
        self.push_message(DccMessage::Close);
        Ok(())
    }

    /// Process pending messages
    ///
    /// Call this from the UI thread periodically (e.g., in a Qt timer).
    /// Returns `true` if there are more messages to process.
    pub fn process_events(&self) -> bool {
        let messages: Vec<DccMessage> = {
            let mut queue = match self.messages.lock() {
                Ok(q) => q,
                Err(_) => return false,
            };
            queue.drain(..).collect()
        };

        if messages.is_empty() {
            return false;
        }

        for msg in messages {
            if let Err(e) = self.handle_message(msg) {
                warn!("[DCC] Failed to handle message: {}", e);
            }
        }

        // Check if there are more messages
        self.messages
            .lock()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Get config
    pub fn config(&self) -> &DccConfig {
        &self.config
    }

    /// Get current size
    pub fn size(&self) -> (u32, u32) {
        if let Ok(inner) = self.inner.lock() {
            if let Some(ref state) = *inner {
                return (state.width, state.height);
            }
        }
        (self.config.width, self.config.height)
    }

    /// Get parent HWND
    pub fn parent_hwnd(&self) -> Option<isize> {
        self.config.parent_hwnd
    }

    /// Push message to queue
    fn push_message(&self, msg: DccMessage) {
        if let Ok(mut queue) = self.messages.lock() {
            queue.push_back(msg);
        }
    }

    /// Handle a single message
    fn handle_message(&self, msg: DccMessage) -> Result<()> {
        match msg {
            DccMessage::Navigate(url) => {
                debug!("[DCC] Handling Navigate: {}", url);
                // TODO: Call WebView2 Navigate
            }
            DccMessage::LoadHtml(html) => {
                debug!("[DCC] Handling LoadHtml ({} bytes)", html.len());
                // TODO: Call WebView2 NavigateToString
            }
            DccMessage::EvalJs(script) => {
                debug!("[DCC] Handling EvalJs ({} bytes)", script.len());
                // TODO: Call WebView2 ExecuteScript
            }
            DccMessage::Resize { width, height } => {
                debug!("[DCC] Handling Resize {}x{}", width, height);
                self.apply_resize(width, height)?;
            }
            DccMessage::SetVisible(visible) => {
                debug!("[DCC] Handling SetVisible({})", visible);
                self.apply_visibility(visible)?;
            }
            DccMessage::Close => {
                debug!("[DCC] Handling Close");
                // TODO: Clean up WebView2 resources
            }
        }
        Ok(())
    }

    /// Apply resize to WebView window
    fn apply_resize(&self, width: u32, height: u32) -> Result<()> {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                state.width = width;
                state.height = height;

                // Update window size via Win32 API
                if let Some(hwnd) = state.webview_hwnd {
                    use windows::Win32::Foundation::HWND;
                    use windows::Win32::UI::WindowsAndMessaging::{
                        SetWindowPos, SWP_NOMOVE, SWP_NOZORDER,
                    };

                    unsafe {
                        let _ = SetWindowPos(
                            HWND(hwnd as *mut std::ffi::c_void),
                            None,
                            0,
                            0,
                            width as i32,
                            height as i32,
                            SWP_NOMOVE | SWP_NOZORDER,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply visibility to WebView window
    fn apply_visibility(&self, visible: bool) -> Result<()> {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                state.visible = visible;

                // Update window visibility via Win32 API
                if let Some(hwnd) = state.webview_hwnd {
                    use windows::Win32::Foundation::HWND;
                    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};

                    unsafe {
                        let cmd = if visible { SW_SHOW } else { SW_HIDE };
                        let _ = ShowWindow(HWND(hwnd as *mut std::ffi::c_void), cmd);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for DccWebView {
    fn drop(&mut self) {
        info!("[DCC] WebView dropping");
        // Clean up WebView2 resources
        if let Ok(mut inner) = self.inner.lock() {
            *inner = None;
        }
    }
}

/// Initialize COM for WebView2
fn init_com() -> Result<()> {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            // S_FALSE means already initialized, which is fine
            let code = hr.0 as u32;
            if code != 0x00000001 {
                // S_FALSE
                return Err(DccError::Com(format!(
                    "CoInitializeEx failed: 0x{:08X}",
                    code
                )));
            }
        }
    }
    Ok(())
}
