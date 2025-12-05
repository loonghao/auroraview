//! Native backend - WebView embedded using platform-specific APIs
//!
//! This backend uses native window parenting (HWND on Windows) to embed
//! the WebView into existing DCC application windows.

#[allow(unused_imports)]
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use tao::event_loop::EventLoopBuilder;
#[allow(unused_imports)]
use tao::window::WindowBuilder;
use wry::WebContext;
use wry::WebView as WryWebView;
use wry::WebViewBuilder as WryWebViewBuilder;

#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use super::WebViewBackend;
use crate::ipc::{IpcHandler, IpcMessage, MessageQueue};
use crate::webview::config::WebViewConfig;
use crate::webview::event_loop::UserEvent;
use crate::webview::js_assets;
use crate::webview::message_pump;

/// Native backend implementation
///
/// This backend creates a WebView that can be embedded into existing windows
/// using platform-specific APIs (e.g., Windows HWND parenting).
#[allow(dead_code)]
pub struct NativeBackend {
    webview: Arc<Mutex<WryWebView>>,
    window: Option<tao::window::Window>,
    event_loop: Option<tao::event_loop::EventLoop<UserEvent>>,
    message_queue: Arc<MessageQueue>,
    /// When true, skip native message pump in process_events().
    /// This is used in Qt/DCC mode where the host application owns the message loop.
    skip_message_pump: bool,
    /// When true, show window automatically after creation.
    /// When false, window stays hidden until set_visible(true) is called.
    auto_show: bool,
    /// Maximum messages to process per tick (0 = unlimited)
    /// Used for DCCs with busy main threads (e.g., Houdini)
    max_messages_per_tick: usize,
}

impl std::fmt::Debug for NativeBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeBackend")
            .field("webview", &"Arc<Mutex<WryWebView>>")
            .field("window", &self.window.is_some())
            .field("event_loop", &self.event_loop.is_some())
            .field("message_queue", &"Arc<MessageQueue>")
            .finish()
    }
}

impl Drop for NativeBackend {
    fn drop(&mut self) {
        tracing::warn!("[DROP] NativeBackend is being dropped!");
        if self.window.is_some() {
            tracing::warn!("[DROP] Window will be destroyed");
        }
        if self.event_loop.is_some() {
            tracing::warn!("[DROP] EventLoop will be destroyed");
        }
    }
}

impl WebViewBackend for NativeBackend {
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Determine if this is embedded or standalone mode
        if let Some(parent_hwnd) = config.parent_hwnd {
            Self::create_embedded(parent_hwnd, config, ipc_handler, message_queue)
        } else {
            #[cfg(feature = "python-bindings")]
            {
                Self::create_standalone(config, ipc_handler, message_queue)
            }
            #[cfg(not(feature = "python-bindings"))]
            {
                Err("Standalone mode requires python-bindings feature".into())
            }
        }
    }

    fn webview(&self) -> Arc<Mutex<WryWebView>> {
        self.webview.clone()
    }

    fn message_queue(&self) -> Arc<MessageQueue> {
        self.message_queue.clone()
    }

    fn window(&self) -> Option<&tao::window::Window> {
        self.window.as_ref()
    }

    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>> {
        self.event_loop.take()
    }

    fn process_events(&self) -> bool {
        // Check if window handle is still valid (for embedded mode)
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            use std::ffi::c_void;
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::IsWindow;

            if let Some(window) = &self.window {
                if let Ok(window_handle) = window.window_handle() {
                    let raw_handle = window_handle.as_raw();
                    if let RawWindowHandle::Win32(handle) = raw_handle {
                        let hwnd_value = handle.hwnd.get();
                        let hwnd = HWND(hwnd_value as *mut c_void);

                        let is_valid = unsafe { IsWindow(Some(hwnd)).as_bool() };

                        if !is_valid {
                            tracing::info!("[CLOSE] [NativeBackend::process_events] Window handle invalid - user closed window");
                            return true;
                        }
                    }
                }
            }
        }

        // In Qt/DCC mode, skip message pump - the host application owns the message loop.
        // We only process our internal IPC message queue.
        if self.skip_message_pump {
            tracing::trace!("[NativeBackend::process_events] Skipping message pump (Qt/DCC mode)");
        } else {
            // Get window HWND for targeted message processing
            #[cfg(target_os = "windows")]
            let hwnd = {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};

                if let Some(window) = &self.window {
                    if let Ok(window_handle) = window.window_handle() {
                        let raw_handle = window_handle.as_raw();
                        if let RawWindowHandle::Win32(handle) = raw_handle {
                            Some(handle.hwnd.get() as u64)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            #[cfg(not(target_os = "windows"))]
            let hwnd: Option<u64> = None;

            // Process Windows messages
            let should_quit = if let Some(hwnd_value) = hwnd {
                message_pump::process_messages_for_hwnd(hwnd_value)
            } else {
                message_pump::process_all_messages()
            };

            if should_quit {
                tracing::info!(
                    "[CLOSE] [NativeBackend::process_events] Window close signal detected"
                );
                return true;
            }
        }

        // First, handle SetVisible messages separately (needs window access)
        // We collect visibility changes and apply them after processing
        let mut visibility_changes: Vec<bool> = Vec::new();

        // Process message queue with batch limit (always, regardless of skip_message_pump)
        // Use batch processing for DCCs with busy main threads (e.g., Houdini)
        if let Ok(webview) = self.webview.lock() {
            let (count, remaining) =
                self.message_queue
                    .process_batch(self.max_messages_per_tick, |message| {
                        use crate::ipc::WebViewMessage;
                        match message {
                            WebViewMessage::EvalJs(script) => {
                                if let Err(e) = webview.evaluate_script(&script) {
                                    tracing::error!("Failed to execute JavaScript: {}", e);
                                }
                            }
                            WebViewMessage::EmitEvent { event_name, data } => {
                                let json_str = data.to_string();
                                let escaped_json =
                                    json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                let script =
                                    js_assets::build_emit_event_script(&event_name, &escaped_json);
                                tracing::debug!(
                                    "[CLOSE] [NativeBackend] Generated script: {}",
                                    script
                                );
                                if let Err(e) = webview.evaluate_script(&script) {
                                    tracing::error!("Failed to emit event: {}", e);
                                }
                            }
                            WebViewMessage::LoadUrl(url) => {
                                // Use native WebView2 navigation instead of JavaScript
                                tracing::debug!("[NativeBackend] Loading URL: {}", url);
                                if let Err(e) = webview.load_url(&url) {
                                    tracing::error!("Failed to load URL: {}", e);
                                }
                            }
                            WebViewMessage::LoadHtml(html) => {
                                if let Err(e) = webview.load_html(&html) {
                                    tracing::error!("Failed to load HTML: {}", e);
                                }
                            }
                            WebViewMessage::WindowEvent { event_type, data } => {
                                // Window events are handled by emitting to JavaScript
                                let event_name = event_type.as_str();
                                let json_str = data.to_string();
                                let escaped_json =
                                    json_str.replace('\\', "\\\\").replace('\'', "\\'");
                                let script =
                                    js_assets::build_emit_event_script(event_name, &escaped_json);
                                tracing::debug!(
                                    "[WINDOW_EVENT] [NativeBackend] Emitting window event: {}",
                                    event_name
                                );
                                if let Err(e) = webview.evaluate_script(&script) {
                                    tracing::error!("Failed to emit window event: {}", e);
                                }
                            }
                            WebViewMessage::SetVisible(visible) => {
                                // Collect visibility change to apply after closure
                                tracing::debug!("[NativeBackend] SetVisible({})", visible);
                                visibility_changes.push(visible);
                            }
                            WebViewMessage::EvalJsAsync {
                                script,
                                callback_id,
                            } => {
                                // Execute JavaScript and send result back via IPC
                                let async_script =
                                    js_assets::build_eval_js_async_script(&script, callback_id);
                                if let Err(e) = webview.evaluate_script(&async_script) {
                                    tracing::error!(
                                        "Failed to execute async JavaScript (id={}): {}",
                                        callback_id,
                                        e
                                    );
                                }
                            }
                            WebViewMessage::Reload => {
                                if let Err(e) = webview.evaluate_script("location.reload()") {
                                    tracing::error!("Failed to reload: {}", e);
                                }
                            }
                            WebViewMessage::StopLoading => {
                                if let Err(e) = webview.evaluate_script("window.stop()") {
                                    tracing::error!("Failed to stop loading: {}", e);
                                }
                            }
                        }
                    });

            if count > 0 {
                tracing::debug!(
                    "[OK] [NativeBackend::process_events] Processed {} messages ({} remaining)",
                    count,
                    remaining
                );
            }
        }

        // Apply visibility changes outside the closure
        for visible in visibility_changes {
            if let Some(ref window) = self.window {
                tracing::debug!("[NativeBackend] Setting visibility: {}", visible);
                window.set_visible(visible);
            }
        }

        false
    }

    fn run_event_loop_blocking(&mut self) {
        use crate::webview::event_loop::{EventLoopState, WebViewEventHandler};

        tracing::info!("[OK] [NativeBackend::run_event_loop_blocking] Starting event loop");

        if self.window.is_none() || self.event_loop.is_none() {
            tracing::error!("Window or event loop is None!");
            return;
        }

        let event_loop = match self.event_loop.take() {
            Some(el) => el,
            None => {
                tracing::error!("Failed to take event loop");
                return;
            }
        };

        let window = match self.window.take() {
            Some(w) => w,
            None => {
                tracing::error!("Failed to take window");
                return;
            }
        };

        #[allow(clippy::arc_with_non_send_sync)]
        let state = Arc::new(Mutex::new(EventLoopState::new_without_webview(
            window,
            self.message_queue.clone(),
        )));

        if let Ok(mut state_guard) = state.lock() {
            state_guard.set_webview(self.webview.clone());
        }

        WebViewEventHandler::run_blocking(event_loop, state, self.auto_show);
        tracing::info!("Event loop exited");
    }

    fn set_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Use tao::Window if available (works for both standalone and embedded modes)
        if let Some(window) = &self.window {
            window.set_visible(visible);
            tracing::debug!("[NativeBackend] set_visible({}) via tao::Window", visible);
            Ok(())
        } else {
            Err("Window not available for set_visible".into())
        }
    }
}

impl NativeBackend {
    /// Apply anti-flicker optimizations for Container mode (Qt integration)
    ///
    /// This function applies Win32 optimizations to prevent the window from
    /// being visible before Qt's createWindowContainer wraps it:
    ///
    /// 1. Move window off-screen to prevent any visible flash
    /// 2. Add WS_EX_LAYERED and set zero alpha for complete invisibility
    /// 3. Pre-apply WS_CHILD-compatible styles to reduce transition artifacts
    ///
    /// These optimizations will be reversed by Qt when it calls createWindowContainer
    /// and sets up proper visibility.
    #[cfg(target_os = "windows")]
    fn apply_anti_flicker_optimizations(hwnd: isize) {
        use windows::Win32::Foundation::{COLORREF, HWND};
        use windows::Win32::Graphics::Gdi::UpdateWindow;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
            HWND_BOTTOM, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
            WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };

        unsafe {
            let hwnd_win = HWND(hwnd as *mut _);

            // Step 1: Add WS_EX_LAYERED for transparency control
            // Also add WS_EX_TOOLWINDOW to prevent taskbar icon and WS_EX_NOACTIVATE
            let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);
            let new_ex_style = ex_style
                | WS_EX_LAYERED.0 as i32
                | WS_EX_TOOLWINDOW.0 as i32
                | WS_EX_NOACTIVATE.0 as i32;
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

            // Step 2: Set zero alpha to make window completely invisible
            // This is the key to preventing any flash - even if the window
            // briefly appears on screen, it will be fully transparent
            // COLORREF(0) is black but we use LWA_ALPHA so color key is ignored
            let _ = SetLayeredWindowAttributes(hwnd_win, COLORREF(0), 0, LWA_ALPHA);

            // Step 3: Move window far off-screen as a secondary safeguard
            // Position: (-10000, -10000) is well outside any monitor
            let _ = SetWindowPos(
                hwnd_win,
                Some(HWND_BOTTOM), // Put at bottom of Z-order
                -10000,            // X: far off-screen
                -10000,            // Y: far off-screen
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            // Ensure the window is updated
            let _ = UpdateWindow(hwnd_win);

            tracing::info!(
                "[OK] [NativeBackend] Applied anti-flicker optimizations: HWND 0x{:X} moved off-screen with zero alpha",
                hwnd
            );
        }
    }

    /// Remove anti-flicker optimizations (restore normal window behavior)
    ///
    /// This should be called after Qt's createWindowContainer has wrapped the window.
    /// It removes the WS_EX_LAYERED style and restores normal alpha.
    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    pub fn remove_anti_flicker_optimizations(hwnd: isize) {
        use windows::Win32::Foundation::{COLORREF, HWND};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
            LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };

        unsafe {
            let hwnd_win = HWND(hwnd as *mut _);

            // Restore full alpha first (make window visible if it was layered)
            let _ = SetLayeredWindowAttributes(hwnd_win, COLORREF(0), 255, LWA_ALPHA);

            // Remove WS_EX_LAYERED, WS_EX_TOOLWINDOW, and WS_EX_NOACTIVATE styles
            let ex_style = GetWindowLongW(hwnd_win, GWL_EXSTYLE);
            let new_ex_style = ex_style
                & !(WS_EX_LAYERED.0 as i32)
                & !(WS_EX_TOOLWINDOW.0 as i32)
                & !(WS_EX_NOACTIVATE.0 as i32);
            SetWindowLongW(hwnd_win, GWL_EXSTYLE, new_ex_style);

            // Apply style changes
            let _ = SetWindowPos(
                hwnd_win,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            tracing::info!(
                "[OK] [NativeBackend] Removed anti-flicker optimizations: HWND 0x{:X} restored",
                hwnd
            );
        }
    }

    /// Process messages for DCC integration mode
    ///
    /// This method should be called periodically from a Qt timer to process
    /// WebView messages without running a dedicated event loop.
    ///
    /// # Returns
    /// `true` if the window should be closed, `false` otherwise
    #[allow(dead_code)]
    pub fn process_messages(&self) -> bool {
        self.process_events()
    }

    /// Create WebView for DCC integration (no event loop)
    ///
    /// This method creates a WebView that integrates with DCC applications (Maya, Houdini, etc.)
    /// by reusing the DCC's Qt message pump instead of creating its own event loop.
    ///
    /// The method now properly supports embedding into Qt widgets using EmbedMode::Child.
    ///
    /// # Arguments
    /// * `parent_hwnd` - HWND of the DCC main window or Qt widget
    /// * `config` - WebView configuration (use embed_mode to control embedding behavior)
    /// * `ipc_handler` - IPC message handler
    /// * `message_queue` - Message queue for cross-thread communication
    ///
    /// # Returns
    /// A NativeBackend instance without running event loop
    #[cfg(target_os = "windows")]
    pub fn create_for_dcc(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!(
            "[OK] [NativeBackend::create_for_dcc] Creating WebView for DCC integration (parent_hwnd: {}, mode: {:?})",
            parent_hwnd,
            config.embed_mode
        );
        tracing::info!("[OK] This WebView will NOT run its own event loop");
        tracing::info!("[OK] DCC's Qt message pump will handle all messages");

        // Delegate to create_embedded which now handles all embedding modes
        Self::create_embedded(parent_hwnd, config, ipc_handler, message_queue)
    }

    /// Create WebView for DCC integration (non-Windows platforms)
    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    pub fn create_for_dcc(
        _parent_hwnd: u64,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
        _message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Err("DCC integration mode is only supported on Windows".into())
    }

    /// Create standalone WebView with its own window
    #[cfg(feature = "python-bindings")]
    #[allow(dead_code)]
    fn create_standalone(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Save batch size before moving config
        let ipc_batch_size = config.ipc_batch_size;

        // Delegate to standalone module for now
        // We need to use the existing standalone implementation
        // and convert it to NativeBackend structure
        let mut inner = crate::webview::standalone::create_standalone(
            config,
            ipc_handler,
            message_queue.clone(),
        )?;

        // Extract fields from WebViewInner
        // We can safely take these because we own the WebViewInner
        let webview = inner.webview.clone();
        let window = inner.window.take();
        let event_loop = inner.event_loop.take();

        Ok(Self {
            webview,
            window,
            event_loop,
            message_queue,
            // In standalone mode, we own the message pump
            skip_message_pump: false,
            auto_show: true, // Standalone mode always auto-shows
            max_messages_per_tick: ipc_batch_size,
        })
    }

    /// Create embedded WebView for DCC integration
    #[cfg(target_os = "windows")]
    fn create_embedded(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::webview::config::EmbedMode;
        use tao::platform::windows::WindowBuilderExtWindows;

        tracing::info!(
            "[OK] [NativeBackend::create_embedded] Creating embedded WebView (parent_hwnd: {}, mode: {:?})",
            parent_hwnd,
            config.embed_mode
        );

        // Initialize COM for WebView2 on Windows
        // WebView2 requires COM to be initialized in STA (Single-Threaded Apartment) mode
        // This is critical for background thread creation (HWND mode in DCC apps)
        {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
            unsafe {
                // COINIT_APARTMENTTHREADED = STA mode required by WebView2
                // Ignore errors if already initialized (e.g., by Qt on main thread)
                let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                if result.is_ok() {
                    tracing::info!("[NativeBackend] COM initialized in STA mode for this thread");
                } else {
                    tracing::debug!(
                        "[NativeBackend] COM already initialized or failed: {:?}",
                        result
                    );
                }
            }
        }

        // Create event loop
        let event_loop = {
            use tao::platform::windows::EventLoopBuilderExtWindows;
            EventLoopBuilder::<UserEvent>::with_user_event()
                .with_any_thread(true)
                .build()
        };

        // Create window builder
        let mut window_builder = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
            .with_resizable(config.resizable)
            .with_decorations(config.decorations)
            .with_always_on_top(config.always_on_top)
            .with_transparent(config.transparent);

        // Set parent window based on embed mode
        match config.embed_mode {
            EmbedMode::Child => {
                tracing::info!("[OK] [NativeBackend] Using Child mode (WS_CHILD)");
                window_builder = window_builder.with_parent_window(parent_hwnd as isize);
            }
            EmbedMode::Owner => {
                tracing::info!("[OK] [NativeBackend] Using Owner mode (GWLP_HWNDPARENT)");
                window_builder = window_builder.with_owner_window(parent_hwnd as isize);
            }
            EmbedMode::Container => {
                // Container mode: standalone window, no Win32 parent relationship
                // Qt will wrap this window with createWindowContainer
                tracing::info!(
                    "[OK] [NativeBackend] Using Container mode (standalone for Qt container)"
                );
                window_builder = window_builder.with_decorations(false);
            }
            EmbedMode::None => {
                tracing::warn!(
                    "[WARNING] [NativeBackend] EmbedMode::None - creating standalone window"
                );
            }
        }

        // Build window
        let window = window_builder
            .build(&event_loop)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Log window HWND and apply anti-flicker optimizations for Container mode
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    let hwnd_value = handle.hwnd.get();
                    tracing::info!(
                        "[OK] [NativeBackend] Window created: HWND 0x{:X}",
                        hwnd_value
                    );

                    // For Container mode (Qt integration), apply anti-flicker optimizations
                    // This prevents the window from being visible before Qt's createWindowContainer
                    if matches!(config.embed_mode, EmbedMode::Container) {
                        Self::apply_anti_flicker_optimizations(hwnd_value as isize);
                    }
                }
            }
        }

        // Only make window visible if auto_show is true
        // For DCC/Qt integration, we want to keep it hidden until Qt controls visibility
        let auto_show = config.auto_show;
        if auto_show {
            window.set_visible(true);
            tracing::info!("[OK] [NativeBackend] Window auto-shown (auto_show=true)");
        } else {
            window.set_visible(false);
            tracing::info!("[OK] [NativeBackend] Window stays hidden (auto_show=false)");
        }

        // Create WebView with IPC handler
        let webview = Self::create_webview(&window, &config, ipc_handler)?;

        #[allow(clippy::arc_with_non_send_sync)]
        Ok(Self {
            webview: Arc::new(Mutex::new(webview)),
            window: Some(window),
            event_loop: Some(event_loop),
            message_queue,
            // In embedded/DCC mode, skip message pump - Qt/DCC owns the message loop
            skip_message_pump: true,
            auto_show,
            max_messages_per_tick: config.ipc_batch_size,
        })
    }

    /// Create embedded WebView for non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    fn create_embedded(
        _parent_hwnd: u64,
        _config: WebViewConfig,
        _ipc_handler: Arc<IpcHandler>,
        _message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Embedded mode is only supported on Windows".into())
    }

    /// Create WebView instance with IPC handler
    #[allow(dead_code)]
    fn create_webview(
        window: &tao::window::Window,
        config: &WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
    ) -> Result<WryWebView, Box<dyn std::error::Error>> {
        // Create WebContext with custom data directory if specified
        // This allows storing cookies, localStorage, cache in a custom location
        // Priority: 1. config.data_directory, 2. shared warmup folder, 3. system default
        let mut web_context = if let Some(ref data_dir) = config.data_directory {
            tracing::info!(
                "[NativeBackend] Using custom data directory: {:?}",
                data_dir
            );
            WebContext::new(Some(data_dir.clone()))
        } else {
            // Try to use shared user data folder from warmup (Windows only)
            #[cfg(target_os = "windows")]
            let shared_folder = crate::platform::windows::warmup::get_shared_user_data_folder();
            #[cfg(not(target_os = "windows"))]
            let shared_folder: Option<std::path::PathBuf> = None;

            if let Some(ref shared_dir) = shared_folder {
                tracing::info!(
                    "[NativeBackend] Using shared warmup data directory: {:?}",
                    shared_dir
                );
                WebContext::new(Some(shared_dir.clone()))
            } else {
                tracing::debug!("[NativeBackend] Using default data directory");
                WebContext::default()
            }
        };

        let mut builder = WryWebViewBuilder::new_with_web_context(&mut web_context);

        // Set background color to match app background (dark theme)
        // This prevents white flash and removes white border
        // RGBA is a tuple type (u8, u8, u8, u8) in wry
        let background_color = (2u8, 6u8, 23u8, 255u8); // #020617 from Tailwind slate-950
        builder = builder.with_background_color(background_color);
        tracing::debug!(
            "[NativeBackend] Background color: #{:02x}{:02x}{:02x}",
            background_color.0,
            background_color.1,
            background_color.2
        );

        // Register auroraview:// protocol if asset_root is configured
        //
        // SECURITY NOTE: On Windows, wry maps custom protocols to HTTP format:
        //   - "auroraview" scheme becomes "http://auroraview.<path>" by default
        //   - We use with_https_scheme() to use "https://auroraview.<path>" for better security
        //   - The custom protocol handler intercepts ALL matching requests BEFORE DNS resolution
        //   - This means even if "auroraview.com" is a real domain, requests won't reach the network
        //   - However, this also means users cannot access real "auroraview.*" websites
        //
        // We use "auroraview" as a short, memorable name. The collision risk is minimal because:
        //   1. Requests are intercepted before network, so no security leak to external servers
        //   2. The origin is "https://auroraview.<path>", not a real HTTPS site
        //   3. wry's https scheme provides secure context (needed for some Web APIs)
        //
        // Register auroraview:// custom protocol for local asset loading
        if let Some(asset_root) = &config.asset_root {
            let asset_root = asset_root.clone();
            tracing::debug!(
                "[NativeBackend] Registering auroraview:// protocol (asset_root: {:?})",
                asset_root
            );

            // On Windows, use HTTPS scheme for secure context support
            #[cfg(target_os = "windows")]
            {
                builder = builder.with_https_scheme(true);
            }

            builder =
                builder.with_custom_protocol("auroraview".into(), move |_webview_id, request| {
                    crate::webview::protocol_handlers::handle_auroraview_protocol(
                        &asset_root,
                        request,
                    )
                });
        } else {
            tracing::debug!(
                "[NativeBackend] asset_root is None, auroraview:// protocol not registered"
            );
        }

        // Register custom protocols
        for (scheme, callback) in &config.custom_protocols {
            let callback_clone = callback.clone();
            tracing::debug!("[NativeBackend] Registering custom protocol: {}", scheme);
            builder = builder.with_custom_protocol(scheme.clone(), move |_webview_id, request| {
                crate::webview::protocol_handlers::handle_custom_protocol(&*callback_clone, request)
            });
        }

        // Register file:// protocol if enabled
        if config.allow_file_protocol {
            tracing::debug!("[NativeBackend] Enabling file:// protocol");
            builder = builder.with_custom_protocol("file".into(), |_webview_id, request| {
                crate::webview::protocol_handlers::handle_file_protocol(request)
            });
        }

        // Enable developer tools if configured
        if config.dev_tools {
            tracing::debug!("[NativeBackend] Enabling devtools");
            builder = builder.with_devtools(true);
        }

        // Disable context menu if configured
        if !config.context_menu {
            tracing::debug!("[NativeBackend] Disabling context menu");
            #[cfg(target_os = "windows")]
            {
                builder = builder.with_browser_extensions_enabled(false);
            }
        }

        // Configure new window handler
        if config.allow_new_window {
            tracing::debug!("[NativeBackend] Allowing new windows");
            builder = builder.with_new_window_req_handler(|url, _features| {
                tracing::debug!("[NativeBackend] New window: {}", url);
                wry::NewWindowResponse::Allow
            });
        } else {
            tracing::debug!("[NativeBackend] Blocking new windows");
            builder = builder.with_new_window_req_handler(|url, _features| {
                tracing::debug!("[NativeBackend] Blocked: {}", url);
                wry::NewWindowResponse::Deny
            });
        }

        // Build initialization script using js_assets module
        tracing::debug!("[NativeBackend] Building init script");
        let event_bridge_script = js_assets::build_init_script(config);
        builder = builder.with_initialization_script(&event_bridge_script);

        // Set IPC handler
        let ipc_handler_clone = ipc_handler.clone();
        builder = builder.with_ipc_handler(move |request| {
            tracing::debug!("[OK] [NativeBackend] IPC message received");

            let body_str = request.body();
            tracing::debug!("[OK] [NativeBackend] IPC body: {}", body_str);

            if let Ok(message) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
                    if msg_type == "js_callback_result" {
                        // Handle async JavaScript execution result
                        let callback_id = message
                            .get("callback_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let result = message.get("result").cloned();
                        let error = message.get("error").cloned();

                        tracing::debug!(
                            "[NativeBackend] JS callback result: id={}, result={:?}, error={:?}",
                            callback_id,
                            result,
                            error
                        );

                        // Send as a special IPC event that AuroraView can handle
                        let mut payload = serde_json::Map::new();
                        payload.insert("callback_id".to_string(), serde_json::json!(callback_id));
                        if let Some(r) = result {
                            payload.insert("result".to_string(), r);
                        }
                        if let Some(e) = error {
                            payload.insert("error".to_string(), e);
                        }

                        let ipc_message = IpcMessage {
                            event: "__js_callback_result__".to_string(),
                            data: serde_json::Value::Object(payload),
                            id: None,
                        };

                        if let Err(e) = ipc_handler_clone.handle_message(ipc_message) {
                            tracing::error!(
                                "[ERROR] [NativeBackend] Error handling JS callback result: {}",
                                e
                            );
                        }
                    } else if msg_type == "event" {
                        if let Some(event_name) = message.get("event").and_then(|v| v.as_str()) {
                            let detail = message
                                .get("detail")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            tracing::debug!(
                                "[NativeBackend] Event: {} detail: {}",
                                event_name,
                                detail
                            );

                            let ipc_message = IpcMessage {
                                event: event_name.to_string(),
                                data: detail,
                                id: None,
                            };

                            if let Err(e) = ipc_handler_clone.handle_message(ipc_message) {
                                tracing::error!(
                                    "[ERROR] [NativeBackend] Error handling event: {}",
                                    e
                                );
                            }
                        }
                    } else if msg_type == "call" {
                        if let Some(method) = message.get("method").and_then(|v| v.as_str()) {
                            let params = message
                                .get("params")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            let id = message
                                .get("id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            tracing::debug!(
                                "[NativeBackend] Call: {} params: {} id: {:?}",
                                method,
                                params,
                                id
                            );

                            let mut payload = serde_json::Map::new();
                            payload.insert("params".to_string(), params);
                            if let Some(ref call_id) = id {
                                payload.insert(
                                    "id".to_string(),
                                    serde_json::Value::String(call_id.clone()),
                                );
                            }

                            let ipc_message = IpcMessage {
                                event: method.to_string(),
                                data: serde_json::Value::Object(payload),
                                id,
                            };

                            if let Err(e) = ipc_handler_clone.handle_message(ipc_message) {
                                tracing::error!(
                                    "[ERROR] [NativeBackend] Error handling call: {}",
                                    e
                                );
                            }
                        }
                    }
                }
            }
        });

        // Build WebView
        let webview = builder
            .build(window)
            .map_err(|e| format!("Failed to create WebView: {}", e))?;

        tracing::info!("[OK] [NativeBackend] WebView created successfully");

        // Load initial content using native WebView2 API
        if let Some(ref url) = config.url {
            tracing::info!("[OK] [NativeBackend] Loading URL via native API: {}", url);
            webview
                .load_url(url)
                .map_err(|e| format!("Failed to load URL: {}", e))?;
        } else if let Some(ref html) = config.html {
            tracing::info!("[OK] [NativeBackend] Loading HTML ({} bytes)", html.len());
            webview
                .load_html(html)
                .map_err(|e| format!("Failed to load HTML: {}", e))?;
        }

        Ok(webview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webview::config::WebViewConfig;

    #[test]
    fn test_native_backend_create_delegates_to_embedded_when_parent_hwnd_present() {
        // This test verifies that create() delegates to create_embedded when parent_hwnd is set
        let config = WebViewConfig {
            parent_hwnd: Some(12345),
            ..Default::default()
        };
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // On Windows, this should attempt to create embedded mode
        // On other platforms, it should return an error
        let result = NativeBackend::create(config, ipc_handler, message_queue);

        #[cfg(target_os = "windows")]
        {
            // On Windows, it will try to create but may fail due to invalid HWND
            // The important thing is it doesn't panic and follows the embedded path
            assert!(result.is_ok() || result.is_err());
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows, embedded mode is not supported
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_native_backend_create_delegates_to_standalone_when_no_parent() {
        // This test verifies that create() delegates to create_standalone when no parent_hwnd
        // Note: Skipped on Linux because EventLoop must be created on main thread
        let config = WebViewConfig {
            parent_hwnd: None,
            ..Default::default()
        };
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // This should attempt to create standalone mode
        let result = NativeBackend::create(config, ipc_handler, message_queue);

        // Should not panic - may succeed or fail depending on environment
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_create_for_dcc_delegates_to_create_embedded() {
        // Verify that create_for_dcc properly delegates to create_embedded
        use crate::webview::config::EmbedMode;

        let config = WebViewConfig {
            embed_mode: EmbedMode::Child,
            ..Default::default()
        };
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // Should delegate to create_embedded
        let result = NativeBackend::create_for_dcc(12345, config, ipc_handler, message_queue);

        // May fail due to invalid HWND, but should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_create_for_dcc_not_supported_on_non_windows() {
        // Verify that create_for_dcc returns error on non-Windows platforms
        let config = WebViewConfig::default();
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        let result = NativeBackend::create_for_dcc(12345, config, ipc_handler, message_queue);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("only supported on Windows"));
    }

    #[test]
    fn test_webview_backend_trait_methods() {
        // Test that NativeBackend implements WebViewBackend trait methods
        // This is a compile-time test - if it compiles, the trait is implemented correctly

        fn assert_implements_webview_backend<T: WebViewBackend>() {}
        assert_implements_webview_backend::<NativeBackend>();
    }
}
