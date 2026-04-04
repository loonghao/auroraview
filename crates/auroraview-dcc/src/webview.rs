//! DCC WebView implementation (Windows)
//!
//! Uses WebView2 directly via webview2-com for embedding into Qt widgets.
//!
//! This implementation is designed for DCC environments where:
//! - The host application owns the event loop (Qt in Maya/Houdini/Nuke)
//! - WebView is embedded as a child window into a Qt widget
//! - No blocking event loop - host application calls process_events() periodically

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, error, info, warn};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2Controller, ICoreWebView2Settings,
};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT};

use crate::config::DccConfig;
use crate::error::{DccError, Result};

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
/// Uses lock-free channel for message passing and atomic for initialization state.
pub struct DccWebView {
    config: DccConfig,
    inner: Arc<Mutex<Option<DccWebViewInner>>>,
    /// Lock-free message channel for async operations
    msg_sender: Sender<DccMessage>,
    msg_receiver: Receiver<DccMessage>,
    /// Whether the WebView is initialized (atomic, no lock needed)
    initialized: Arc<AtomicBool>,
}

struct DccWebViewInner {
    /// WebView HWND (created as child of parent)
    webview_hwnd: Option<isize>,
    /// WebView2 controller (owns the WebView lifecycle)
    controller: Option<ICoreWebView2Controller>,
    /// WebView2 core interface (Navigate, ExecuteScript, etc.)
    webview: Option<ICoreWebView2>,
    /// Current visibility state
    visible: bool,
    /// Current size
    width: u32,
    height: u32,
}

// SAFETY: DccWebViewInner contains COM objects (ICoreWebView2Controller, ICoreWebView2)
// that are apartment-threaded (STA). All access is guarded by a Mutex and only performed
// from the UI thread via process_events() called from the DCC host's Qt timer.
// The Arc<Mutex> wrapper ensures exclusive access; the caller must ensure init() and
// process_events() are called from the same STA thread.
unsafe impl Send for DccWebViewInner {}
unsafe impl Sync for DccWebViewInner {}

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

        let (msg_sender, msg_receiver) = crossbeam_channel::unbounded();

        Ok(Self {
            config,
            inner: Arc::new(Mutex::new(None)),
            msg_sender,
            msg_receiver,
            initialized: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Initialize the WebView (must be called from UI thread)
    ///
    /// This sets up WebView2 as a child window of the parent HWND.
    /// Creates a WebView2 environment and controller, then embeds the WebView
    /// as a child of the given parent window.
    pub fn init(&self) -> Result<()> {
        let parent = self.config.parent_hwnd.ok_or(DccError::InvalidParent)?;

        info!("[DCC] Initializing WebView2 on parent 0x{:X}", parent);

        let parent_hwnd = HWND(parent as *mut std::ffi::c_void);
        let width = self.config.width;
        let height = self.config.height;
        let devtools = self.config.devtools;

        // Build user data directory path
        let data_dir = self
            .config
            .data_dir
            .clone()
            .unwrap_or_else(|| {
                let mut dir = std::env::temp_dir();
                dir.push("auroraview_dcc");
                dir
            });
        let _ = std::fs::create_dir_all(&data_dir);

        let data_dir_wide: Vec<u16> = data_dir
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // Create WebView2 environment
        let (env_tx, env_rx) = std::sync::mpsc::channel();

        // SAFETY: WebView2 COM API call. The data_dir_wide ptr is valid for
        // the duration of the call (owned Vec<u16> lives until end of scope).
        // The completion handler is boxed and moved into the COM callback.
        let env_result = unsafe {
            webview2_com::Microsoft::Web::WebView2::Win32::CreateCoreWebView2EnvironmentWithOptions(
                PCWSTR::null(),
                PCWSTR(data_dir_wide.as_ptr()),
                None,
                &webview2_com::CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
                    move |error_code, environment| {
                        error_code?;
                        let _ = env_tx.send(environment);
                        Ok(())
                    },
                )),
            )
        };

        if let Err(e) = env_result {
            return Err(DccError::WebViewCreation(format!(
                "CreateCoreWebView2EnvironmentWithOptions failed: {e}"
            )));
        }

        // Pump Win32 messages while waiting for environment creation callback
        let environment = pump_messages_until_recv(&env_rx).map_err(|e| {
            DccError::WebViewCreation(format!("Failed to receive WebView2 environment: {e}"))
        })?;

        let environment = environment.ok_or_else(|| {
            DccError::WebViewCreation("WebView2 environment callback returned None".to_string())
        })?;

        info!("[DCC] WebView2 environment created");

        // Create WebView2 controller (child of parent HWND)
        let (ctrl_tx, ctrl_rx) = std::sync::mpsc::channel();

        // SAFETY: WebView2 COM API call. environment is a valid COM interface
        // obtained from the previous callback. parent_hwnd is the caller-provided
        // HWND which must be valid for the WebView's lifetime.
        let ctrl_result = unsafe {
            environment.CreateCoreWebView2Controller(
                parent_hwnd,
                &webview2_com::CreateCoreWebView2ControllerCompletedHandler::create(Box::new(
                    move |error_code, controller| {
                        error_code?;
                        let _ = ctrl_tx.send(controller);
                        Ok(())
                    },
                )),
            )
        };

        if let Err(e) = ctrl_result {
            return Err(DccError::WebViewCreation(format!(
                "CreateCoreWebView2Controller failed: {e}"
            )));
        }

        let controller = pump_messages_until_recv(&ctrl_rx).map_err(|e| {
            DccError::WebViewCreation(format!("Failed to receive WebView2 controller: {e}"))
        })?;

        let controller = controller.ok_or_else(|| {
            DccError::WebViewCreation("WebView2 controller callback returned None".to_string())
        })?;

        info!("[DCC] WebView2 controller created");

        // Set initial bounds
        let bounds = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        // SAFETY: controller is a valid COM interface obtained from the
        // creation callback. SetBounds is a safe COM method call.
        unsafe {
            controller.SetBounds(bounds).map_err(|e| {
                DccError::WebViewCreation(format!("SetBounds failed: {e}"))
            })?;
        }

        // Get the core WebView2 interface
        // SAFETY: controller is a valid COM interface. CoreWebView2() returns
        // the associated ICoreWebView2 interface.
        let webview_core: ICoreWebView2 = unsafe {
            controller.CoreWebView2().map_err(|e| {
                DccError::WebViewCreation(format!("CoreWebView2 failed: {e}"))
            })?
        };

        // Configure settings
        // SAFETY: webview_core is a valid COM interface. Settings() and the
        // subsequent Set* calls are safe COM method invocations.
        let settings: ICoreWebView2Settings = unsafe {
            webview_core.Settings().map_err(|e| {
                DccError::WebViewCreation(format!("Settings failed: {e}"))
            })?
        };
        unsafe {
            let _ = settings.SetAreDevToolsEnabled(devtools);
            let _ = settings.SetAreDefaultContextMenusEnabled(devtools);
            let _ = settings.SetIsStatusBarEnabled(false);
            let _ = settings.SetIsZoomControlEnabled(false);
        }

        info!("[DCC] WebView2 settings configured (devtools={})", devtools);

        // Store inner state
        let inner = DccWebViewInner {
            webview_hwnd: None,
            controller: Some(controller),
            webview: Some(webview_core),
            visible: false,
            width,
            height,
        };

        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(inner);
        }

        self.initialized.store(true, Ordering::Release);

        // Load initial content
        if let Some(ref url) = self.config.url {
            self.navigate(url)?;
        } else if let Some(ref html) = self.config.html {
            self.load_html(html)?;
        }

        info!("[DCC] WebView2 initialization complete");
        Ok(())
    }

    /// Check if WebView is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
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
        let mut processed = false;

        while let Ok(msg) = self.msg_receiver.try_recv() {
            processed = true;
            if let Err(e) = self.handle_message(msg) {
                warn!("[DCC] Failed to handle message: {}", e);
            }
        }

        // Return false since we drained all available messages
        // If new messages arrive concurrently, next call will pick them up
        if processed {
            !self.msg_receiver.is_empty()
        } else {
            false
        }
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

    /// Push message to channel
    fn push_message(&self, msg: DccMessage) {
        if let Err(e) = self.msg_sender.send(msg) {
            warn!("[DCC] Failed to send message: {}", e);
        }
    }

    /// Handle a single message
    fn handle_message(&self, msg: DccMessage) -> Result<()> {
        match msg {
            DccMessage::Navigate(url) => {
                debug!("[DCC] Handling Navigate: {}", url);
                if let Ok(inner) = self.inner.lock() {
                    if let Some(ref state) = *inner {
                        if let Some(ref wv) = state.webview {
                            let url_wide: Vec<u16> =
                                url.encode_utf16().chain(std::iter::once(0)).collect();
                            unsafe {
                                wv.Navigate(PCWSTR(url_wide.as_ptr())).map_err(|e| {
                                    DccError::Com(format!("Navigate failed: {e}"))
                                })?;
                            }
                        }
                    }
                }
            }
            DccMessage::LoadHtml(html) => {
                debug!("[DCC] Handling LoadHtml ({} bytes)", html.len());
                if let Ok(inner) = self.inner.lock() {
                    if let Some(ref state) = *inner {
                        if let Some(ref wv) = state.webview {
                            let html_wide: Vec<u16> =
                                html.encode_utf16().chain(std::iter::once(0)).collect();
                            unsafe {
                                wv.NavigateToString(PCWSTR(html_wide.as_ptr())).map_err(
                                    |e| DccError::Com(format!("NavigateToString failed: {e}")),
                                )?;
                            }
                        }
                    }
                }
            }
            DccMessage::EvalJs(script) => {
                debug!("[DCC] Handling EvalJs ({} bytes)", script.len());
                if let Ok(inner) = self.inner.lock() {
                    if let Some(ref state) = *inner {
                        if let Some(ref wv) = state.webview {
                            let script_wide: Vec<u16> =
                                script.encode_utf16().chain(std::iter::once(0)).collect();
                            unsafe {
                                wv.ExecuteScript(
                                    PCWSTR(script_wide.as_ptr()),
                                    &webview2_com::ExecuteScriptCompletedHandler::create(
                                        Box::new(move |error_code, result| {
                                            if let Err(e) = error_code {
                                                error!("[DCC] ExecuteScript error: {e}");
                                            } else {
                                                debug!(
                                                    "[DCC] ExecuteScript result: {:?}",
                                                    result.to_string()
                                                );
                                            }
                                            Ok(())
                                        }),
                                    ),
                                )
                                .map_err(|e| {
                                    DccError::Com(format!("ExecuteScript failed: {e}"))
                                })?;
                            }
                        }
                    }
                }
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
                self.cleanup_webview();
            }
        }
        Ok(())
    }

    /// Apply resize to WebView controller bounds and native window
    fn apply_resize(&self, width: u32, height: u32) -> Result<()> {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                state.width = width;
                state.height = height;

                // Update controller bounds (preferred over raw Win32 SetWindowPos)
                if let Some(ref controller) = state.controller {
                    let bounds = RECT {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    };
                    unsafe {
                        let _ = controller.SetBounds(bounds);
                    }
                }

                // Also update native HWND if available
                if let Some(hwnd) = state.webview_hwnd {
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

    /// Apply visibility to WebView controller
    fn apply_visibility(&self, visible: bool) -> Result<()> {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                state.visible = visible;

                // Use controller visibility (preferred API)
                if let Some(ref controller) = state.controller {
                    unsafe {
                        let _ = controller.SetIsVisible(visible);
                    }
                }

                // Also update native HWND if available
                if let Some(hwnd) = state.webview_hwnd {
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

    /// Release WebView2 controller and core references
    fn cleanup_webview(&self) {
        info!("[DCC] Cleaning up WebView2 resources");
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(ref mut state) = *inner {
                // Close the controller (releases the WebView2 browser process connection)
                if let Some(ref controller) = state.controller {
                    unsafe {
                        let _ = controller.Close();
                    }
                }
                state.controller = None;
                state.webview = None;
                state.webview_hwnd = None;
            }
        }
        self.initialized.store(false, Ordering::Release);
        info!("[DCC] WebView2 resources released");
    }
}

impl Drop for DccWebView {
    fn drop(&mut self) {
        info!("[DCC] WebView dropping");
        self.cleanup_webview();
        if let Ok(mut inner) = self.inner.lock() {
            *inner = None;
        }
    }
}

/// Initialize COM for WebView2
fn init_com() -> Result<()> {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    // SAFETY: CoInitializeEx is a well-defined Win32 COM API.
    // S_FALSE (0x00000001) means already initialized, which is harmless.
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

/// Pump Win32 messages while waiting for an async WebView2 callback.
///
/// WebView2 callbacks are dispatched via the Win32 message loop, so we must
/// keep pumping messages on the current thread until the callback fires.
fn pump_messages_until_recv<T>(
    rx: &std::sync::mpsc::Receiver<T>,
) -> std::result::Result<T, String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    loop {
        // Check for result
        if let Ok(value) = rx.try_recv() {
            return Ok(value);
        }

        // Timeout guard
        if start.elapsed() > timeout {
            return Err("Timed out waiting for WebView2 callback".to_string());
        }

        // Pump Win32 messages to allow WebView2 callbacks to fire
        // SAFETY: PeekMessageW, TranslateMessage, DispatchMessageW are
        // standard Win32 message loop APIs. Called on the current thread
        // with a default MSG struct. No memory safety concerns.
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // Yield to avoid busy-spinning
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
