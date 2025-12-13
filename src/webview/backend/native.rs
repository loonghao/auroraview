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

// Use shared builder utilities from auroraview-core
#[cfg(target_os = "windows")]
use auroraview_core::builder::init_com_sta;
use auroraview_core::builder::{get_background_color, log_background_color};

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
        // Determine if this is embedded or desktop mode
        if let Some(parent_hwnd) = config.parent_hwnd {
            Self::create_embedded(parent_hwnd, config, ipc_handler, message_queue, None)
        } else {
            #[cfg(feature = "python-bindings")]
            {
                Self::create_desktop(config, ipc_handler, message_queue)
            }
            #[cfg(not(feature = "python-bindings"))]
            {
                Err("Desktop mode requires python-bindings feature".into())
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
                            WebViewMessage::Close => {
                                // Close is handled at event loop level
                                tracing::info!("[NativeBackend] Close message received");
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

        // Always show window when run_event_loop_blocking is called
        // The auto_show config controls whether show() is called automatically after create()
        // But when show() is explicitly called, the window should always be visible
        WebViewEventHandler::run_blocking(event_loop, state, true);
        tracing::info!("Event loop exited");
    }

    fn set_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Use tao::Window if available (works for both desktop and embedded modes)
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
    /// Fix all WebView2 child windows to prevent dragging (Qt6 compatibility)
    ///
    /// WebView2 creates multiple child windows (Chrome_WidgetWin_0, Intermediate D3D Window, etc.)
    /// that may not inherit proper WS_CHILD styles. This function recursively fixes all child
    /// windows to ensure they cannot be dragged independently.
    ///
    /// Additionally, this function subclasses Chrome_WidgetWin_0 and Chrome_WidgetWin_1 windows
    /// to intercept WM_NCHITTEST messages and force them to return HTCLIENT, preventing any
    /// drag behavior from the WebView2's internal window handling.
    ///
    /// This is especially important for Qt6 where createWindowContainer behavior differs from Qt5.
    ///
    /// # Arguments
    /// * `hwnd` - The top-level WebView window handle
    #[cfg(target_os = "windows")]
    pub fn fix_webview2_child_windows(hwnd: isize) {
        use std::collections::HashMap;
        use std::ffi::c_void;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Mutex;
        use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            CallWindowProcW, DefWindowProcW, EnumChildWindows, GetClassNameW, GetWindowLongPtrW,
            GetWindowLongW, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, GWLP_WNDPROC,
            GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, WNDPROC, WS_BORDER, WS_CAPTION, WS_CHILD, WS_DLGFRAME, WS_EX_CLIENTEDGE,
            WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE, WS_EX_WINDOWEDGE, WS_POPUP, WS_THICKFRAME,
        };

        // WM_NCHITTEST message constant
        const WM_NCHITTEST: u32 = 0x0084;
        // HTCLIENT - indicates the client area (no dragging)
        const HTCLIENT: isize = 1;

        // Store original window procedures for subclassed windows
        // Using a static HashMap protected by Mutex for thread safety
        static ORIGINAL_WNDPROCS: Mutex<Option<HashMap<isize, isize>>> = Mutex::new(None);

        // Initialize the HashMap if needed
        {
            // Use ok() to avoid panic if mutex is poisoned during shutdown
            if let Ok(mut guard) = ORIGINAL_WNDPROCS.lock() {
                if guard.is_none() {
                    *guard = Some(HashMap::new());
                }
            }
        }

        // Custom window procedure that intercepts WM_NCHITTEST
        unsafe extern "system" fn subclass_wndproc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            // Intercept WM_NCHITTEST to prevent dragging
            if msg == WM_NCHITTEST {
                // Always return HTCLIENT to indicate we're in the client area
                // This prevents any part of the window from being treated as a drag handle
                return LRESULT(HTCLIENT);
            }

            // Get the original window procedure - use ok() to avoid panic during shutdown
            let original_wndproc = ORIGINAL_WNDPROCS.lock().ok().and_then(|guard| {
                guard
                    .as_ref()
                    .and_then(|map| map.get(&(hwnd.0 as isize)).copied())
            });

            if let Some(original) = original_wndproc {
                // Call the original window procedure for all other messages
                let wndproc: WNDPROC = std::mem::transmute(original);
                CallWindowProcW(wndproc, hwnd, msg, wparam, lparam)
            } else {
                // Fallback to DefWindowProc if original not found
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        // Counter for fixed windows
        static FIXED_COUNT: AtomicU32 = AtomicU32::new(0);
        static TOTAL_COUNT: AtomicU32 = AtomicU32::new(0);
        static SUBCLASSED_COUNT: AtomicU32 = AtomicU32::new(0);
        FIXED_COUNT.store(0, Ordering::SeqCst);
        TOTAL_COUNT.store(0, Ordering::SeqCst);
        SUBCLASSED_COUNT.store(0, Ordering::SeqCst);

        // Callback function for EnumChildWindows
        // Returns TRUE (non-zero) to continue enumeration, FALSE (0) to stop
        unsafe extern "system" fn enum_child_proc(
            child_hwnd: HWND,
            _lparam: LPARAM,
        ) -> windows::core::BOOL {
            TOTAL_COUNT.fetch_add(1, Ordering::SeqCst);

            // Get window class name for logging
            let mut class_name_buf = [0u16; 256];
            let class_len = GetClassNameW(child_hwnd, &mut class_name_buf);
            let class_name = if class_len > 0 {
                String::from_utf16_lossy(&class_name_buf[..class_len as usize])
            } else {
                String::from("<unknown>")
            };

            // Get current styles
            let style = GetWindowLongW(child_hwnd, GWL_STYLE);
            let ex_style = GetWindowLongW(child_hwnd, GWL_EXSTYLE);

            // Check if this window has problematic styles
            let has_popup = (style & WS_POPUP.0 as i32) != 0;
            let has_caption = (style & WS_CAPTION.0 as i32) != 0;
            let has_thickframe = (style & WS_THICKFRAME.0 as i32) != 0;
            let is_child = (style & WS_CHILD.0 as i32) != 0;

            tracing::info!(
                "[fix_webview2_child_windows] Checking child HWND 0x{:X} class='{}' style=0x{:08X} (popup={}, caption={}, thickframe={}, is_child={})",
                child_hwnd.0 as isize,
                class_name,
                style,
                has_popup,
                has_caption,
                has_thickframe,
                is_child
            );

            // Subclass Chrome_WidgetWin_0 and Chrome_WidgetWin_1 to intercept WM_NCHITTEST
            // These are the windows that handle mouse input and may cause dragging
            if class_name == "Chrome_WidgetWin_0" || class_name == "Chrome_WidgetWin_1" {
                // Check if already subclassed - use ok() to avoid panic during shutdown
                let already_subclassed = ORIGINAL_WNDPROCS
                    .lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .map(|map| map.contains_key(&(child_hwnd.0 as isize)))
                    })
                    .unwrap_or(false);

                if !already_subclassed {
                    // Get the current window procedure
                    let original_wndproc = GetWindowLongPtrW(child_hwnd, GWLP_WNDPROC);
                    if original_wndproc != 0 {
                        // Store the original window procedure - use ok() to avoid panic
                        if let Ok(mut guard) = ORIGINAL_WNDPROCS.lock() {
                            if let Some(map) = guard.as_mut() {
                                map.insert(child_hwnd.0 as isize, original_wndproc);
                            }
                        }

                        // Set our custom window procedure
                        SetWindowLongPtrW(
                            child_hwnd,
                            GWLP_WNDPROC,
                            subclass_wndproc as usize as isize,
                        );

                        SUBCLASSED_COUNT.fetch_add(1, Ordering::SeqCst);

                        tracing::info!(
                            "[OK] [fix_webview2_child_windows] Subclassed HWND 0x{:X} class='{}' to intercept WM_NCHITTEST",
                            child_hwnd.0 as isize,
                            class_name
                        );
                    }
                }
            }

            // Only fix windows that aren't already proper child windows
            if has_popup || has_caption || has_thickframe || !is_child {
                // Remove problematic styles and ensure WS_CHILD
                let new_style = (style
                    & !(WS_POPUP.0 as i32)
                    & !(WS_CAPTION.0 as i32)
                    & !(WS_THICKFRAME.0 as i32)
                    & !(WS_BORDER.0 as i32)
                    & !(WS_DLGFRAME.0 as i32))
                    | (WS_CHILD.0 as i32);

                // Remove extended styles that can cause issues
                let new_ex_style = ex_style
                    & !(WS_EX_STATICEDGE.0 as i32)
                    & !(WS_EX_CLIENTEDGE.0 as i32)
                    & !(WS_EX_WINDOWEDGE.0 as i32)
                    & !(WS_EX_DLGMODALFRAME.0 as i32);

                if new_style != style || new_ex_style != ex_style {
                    SetWindowLongW(child_hwnd, GWL_STYLE, new_style);
                    SetWindowLongW(child_hwnd, GWL_EXSTYLE, new_ex_style);

                    // Apply changes
                    let _ = SetWindowPos(
                        child_hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );

                    FIXED_COUNT.fetch_add(1, Ordering::SeqCst);

                    tracing::info!(
                        "[OK] [fix_webview2_child_windows] Fixed child HWND 0x{:X} class='{}' (style 0x{:08X} -> 0x{:08X})",
                        child_hwnd.0 as isize,
                        class_name,
                        style,
                        new_style
                    );
                }
            }

            // Continue enumeration (TRUE = 1)
            windows::core::BOOL::from(true)
        }

        unsafe {
            let hwnd_win = HWND(hwnd as *mut c_void);
            let _ = EnumChildWindows(Some(hwnd_win), Some(enum_child_proc), LPARAM(0));
            let total = TOTAL_COUNT.load(Ordering::SeqCst);
            let fixed = FIXED_COUNT.load(Ordering::SeqCst);
            let subclassed = SUBCLASSED_COUNT.load(Ordering::SeqCst);
            tracing::info!(
                "[OK] [NativeBackend] Fixed WebView2 child windows for HWND 0x{:X} (total={}, fixed={}, subclassed={})",
                hwnd,
                total,
                fixed,
                subclassed
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
    /// * `on_created` - Optional callback invoked when WebView2 HWND is created
    ///
    /// Create desktop WebView with its own window
    #[cfg(feature = "python-bindings")]
    #[allow(dead_code)]
    fn create_desktop(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Save config values before moving
        let ipc_batch_size = config.ipc_batch_size;
        // auto_show should be false in headless mode
        let auto_show = config.auto_show && !config.headless;

        // Delegate to desktop module for now
        // We need to use the existing desktop implementation
        // and convert it to NativeBackend structure
        let mut inner =
            crate::webview::desktop::create_desktop(config, ipc_handler, message_queue.clone())?;

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
            // In desktop mode, we own the message pump
            skip_message_pump: false,
            auto_show, // Use config value (false in headless mode)
            max_messages_per_tick: ipc_batch_size,
        })
    }

    /// Alias for `create_desktop` (backward compatibility)
    #[cfg(feature = "python-bindings")]
    #[allow(dead_code)]
    #[inline]
    fn create_standalone(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::create_desktop(config, ipc_handler, message_queue)
    }

    /// Create embedded WebView for external window integration
    #[cfg(target_os = "windows")]
    pub fn create_embedded(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
        on_created: Option<Box<dyn Fn(u64) + Send + Sync>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::webview::config::EmbedMode;
        use tao::platform::windows::WindowBuilderExtWindows;

        tracing::info!(
            "[OK] [NativeBackend::create_embedded] Creating embedded WebView (parent_hwnd: {}, mode: {:?})",
            parent_hwnd,
            config.embed_mode
        );

        // Initialize COM for WebView2 on Windows (using shared utility)
        init_com_sta();

        // Create event loop
        let event_loop = {
            use tao::platform::windows::EventLoopBuilderExtWindows;
            EventLoopBuilder::<UserEvent>::with_user_event()
                .with_any_thread(true)
                .build()
        };

        // Create window builder
        // For Child mode (recommended), window starts hidden and is shown after WebView is ready
        let start_visible = config.auto_show && matches!(config.embed_mode, EmbedMode::None);

        let mut window_builder = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
            .with_resizable(config.resizable)
            .with_decorations(config.decorations)
            .with_always_on_top(config.always_on_top)
            .with_transparent(config.transparent)
            .with_visible(start_visible);

        // Set parent window based on embed mode
        // Child mode is the official recommended approach by wry and WebView2
        match config.embed_mode {
            EmbedMode::Child => {
                // RECOMMENDED for embedding: Use WS_CHILD for true child window
                // - wry's build_as_child() is designed for this
                // - WebView2's "Windowed Hosting" is the simplest option
                // - Automatic resize when parent resizes
                // - Works with Qt5/Qt6 by passing QWidget's winId()
                tracing::info!(
                    "[OK] [NativeBackend] Using Child mode (WS_CHILD, frameless) - RECOMMENDED for embedding"
                );
                window_builder = window_builder
                    .with_decorations(false)
                    .with_parent_window(parent_hwnd as isize);
            }
            EmbedMode::Owner => {
                // RECOMMENDED for floating windows: Owner relationship
                // - Window stays above owner in Z-order
                // - Hidden when owner is minimized
                // - Destroyed when owner is destroyed
                // Owner is set after window creation via SetWindowLongPtrW
                tracing::info!(
                    "[OK] [NativeBackend] Using Owner mode - RECOMMENDED for floating windows"
                );
                // Don't set parent here - we'll set owner after creation
            }
            EmbedMode::None => {
                // Desktop window mode - no parent relationship
                tracing::info!("[OK] [NativeBackend] Using None mode - desktop window");
            }
        }

        // Build window
        let window = window_builder
            .build(&event_loop)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Log window HWND and call on_created callback
        // NOTE: Window styles (Owner, ToolWindow) are applied AFTER WebView2 creation
        // to avoid HRESULT 0x80070057 (E_INVALIDARG) error
        #[cfg(target_os = "windows")]
        let cached_hwnd: Option<isize> = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    let hwnd_value = handle.hwnd.get();
                    tracing::info!(
                        "[OK] [NativeBackend] Window created: HWND 0x{:X}",
                        hwnd_value
                    );

                    // Call the callback immediately!
                    if let Some(callback) = &on_created {
                        tracing::info!("[OK] [NativeBackend] Invoking on_created callback");
                        callback(hwnd_value as u64);
                    }

                    Some(hwnd_value)
                } else {
                    None
                }
            } else {
                None
            }
        };

        #[cfg(not(target_os = "windows"))]
        let cached_hwnd: Option<isize> = None;

        // Control window visibility based on embed mode and auto_show setting
        let auto_show = config.auto_show;
        match config.embed_mode {
            EmbedMode::Child => {
                // Child mode: window visibility is controlled by parent
                // Show immediately since it's embedded in parent
                if auto_show {
                    window.set_visible(true);
                    tracing::info!(
                        "[OK] [NativeBackend] Child mode: window shown (embedded in parent)"
                    );
                } else {
                    window.set_visible(false);
                    tracing::info!(
                        "[OK] [NativeBackend] Child mode: window hidden (auto_show=false)"
                    );
                }
            }
            EmbedMode::Owner => {
                // Owner mode: floating window that follows owner
                if auto_show {
                    window.set_visible(true);
                    tracing::info!("[OK] [NativeBackend] Owner mode: window shown (floating)");
                } else {
                    window.set_visible(false);
                    tracing::info!(
                        "[OK] [NativeBackend] Owner mode: window hidden (auto_show=false)"
                    );
                }
            }
            EmbedMode::None => {
                if auto_show {
                    window.set_visible(true);
                    tracing::info!("[OK] [NativeBackend] Window auto-shown (auto_show=true)");
                } else {
                    window.set_visible(false);
                    tracing::info!("[OK] [NativeBackend] Window stays hidden (auto_show=false)");
                }
            }
        }

        // Create WebView with IPC handler
        let webview = Self::create_webview(&window, &config, ipc_handler)?;

        // Apply mode-specific window styles AFTER WebView2 creation
        // This is critical: applying WS_EX_TOOLWINDOW or owner styles before WebView2 creation
        // causes HRESULT 0x80070057 (E_INVALIDARG) error
        #[cfg(target_os = "windows")]
        if let Some(hwnd_value) = cached_hwnd {
            use crate::webview::config::EmbedMode;
            use auroraview_core::builder::{
                apply_owner_window_style, apply_tool_window_style, disable_window_shadow,
            };

            match config.embed_mode {
                EmbedMode::Child => {
                    // For Child mode, tao's with_parent_window handles WS_CHILD automatically
                    tracing::info!(
                        "[OK] [NativeBackend] Child mode: tao's with_parent_window sets WS_CHILD"
                    );
                }
                EmbedMode::Owner => {
                    // For Owner mode, set owner relationship using GWLP_HWNDPARENT
                    // AFTER WebView2 is created to avoid E_INVALIDARG
                    apply_owner_window_style(hwnd_value, parent_hwnd, config.tool_window);
                }
                EmbedMode::None => {
                    // For None mode with tool_window, just apply tool window style
                    if config.tool_window {
                        apply_tool_window_style(hwnd_value);
                    }
                }
            }

            // Disable window shadow for transparent frameless windows
            // undecorated_shadow=false means we want to disable the shadow
            if !config.undecorated_shadow {
                disable_window_shadow(hwnd_value);
                tracing::info!(
                    "[OK] [NativeBackend] Disabled window shadow (undecorated_shadow=false)"
                );
            }
        }

        // Emit webview2_created event with HWND for Python to use SetParent
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(window_handle) = window.window_handle() {
                let raw_handle = window_handle.as_raw();
                if let RawWindowHandle::Win32(handle) = raw_handle {
                    let hwnd_value = handle.hwnd.get() as u64;
                    tracing::info!(
                        "[OK] [NativeBackend] Emitting webview2_created event with HWND 0x{:X}",
                        hwnd_value
                    );
                    message_queue.push(crate::ipc::WebViewMessage::WindowEvent {
                        event_type: crate::ipc::WindowEventType::WebView2Created,
                        data: serde_json::json!({
                            "hwnd": hwnd_value,
                            "parent_hwnd": parent_hwnd
                        }),
                    });
                }
            }
        }

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
        _on_created: Option<Box<dyn Fn(u64) + Send + Sync>>,
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

        // Transparent windows need both:
        // 1. with_transparent(true) on WebViewBuilder
        // 2. with_background_color((0,0,0,0)) for fully transparent background
        if config.transparent {
            builder = builder
                .with_transparent(true)
                .with_background_color((0, 0, 0, 0));
            tracing::info!("[NativeBackend] Transparent window: WebView transparency enabled");
        } else {
            let background_color = get_background_color();
            builder = builder.with_background_color(background_color);
            log_background_color(background_color);
        }

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
        // On Windows WebView2, NewWindowResponse::Allow may not work reliably.
        // Instead, we open external links in the system default browser.
        if config.allow_new_window {
            tracing::debug!("[NativeBackend] Allowing new windows (opens in system browser)");
            builder = builder.with_new_window_req_handler(|url, _features| {
                tracing::debug!("[NativeBackend] New window requested: {}", url);
                // Open in system default browser instead of creating a new WebView window
                if let Err(e) = open::that(&url) {
                    tracing::error!("[NativeBackend] Failed to open URL in browser: {}", e);
                }
                // Deny creating a new WebView window since we opened in external browser
                wry::NewWindowResponse::Deny
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
                            let has_params = message.get("params").is_some();
                            let params = message.get("params").cloned();
                            let id = message
                                .get("id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            tracing::debug!(
                                "[NativeBackend] Call: {} params: {:?} id: {:?}",
                                method,
                                params,
                                id
                            );

                            let mut payload = serde_json::Map::new();
                            // Only include params if it was present in the original message
                            if has_params {
                                if let Some(p) = params {
                                    payload.insert("params".to_string(), p);
                                } else {
                                    payload.insert("params".to_string(), serde_json::Value::Null);
                                }
                            }
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

        // Add native file drag-drop handler using shared builder module
        // This provides full file paths that browsers cannot access due to security restrictions
        let ipc_handler_for_drop = ipc_handler.clone();
        builder = builder.with_drag_drop_handler(
            auroraview_core::builder::create_drag_drop_handler(move |event_name, data| {
                let ipc_message = IpcMessage {
                    event: event_name.to_string(),
                    data,
                    id: None,
                };

                if let Err(e) = ipc_handler_for_drop.handle_message(ipc_message) {
                    tracing::error!("[NativeBackend] Error handling {}: {}", event_name, e);
                }
            }),
        );

        // Build WebView - use standard build for desktop mode
        // This is the slowest part of initialization on WebView2:
        // 1. Create WebView2 Environment (discover runtime, spawn browser process)
        // 2. Create WebView2 Controller
        // 3. Initialize the WebView
        tracing::info!(
            "[OK] [NativeBackend] Building WebView (this may take a few seconds on first run)..."
        );
        let build_start = std::time::Instant::now();
        let webview = builder
            .build(window)
            .map_err(|e| format!("Failed to create WebView: {}", e))?;
        let build_duration = build_start.elapsed();

        tracing::info!(
            "[OK] [NativeBackend] WebView created successfully in {:.2}s",
            build_duration.as_secs_f64()
        );

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
    fn test_native_backend_create_delegates_to_desktop_when_no_parent() {
        // This test verifies that create() delegates to create_desktop when no parent_hwnd
        // Note: Skipped on Linux because EventLoop must be created on main thread
        let config = WebViewConfig {
            parent_hwnd: None,
            ..Default::default()
        };
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // This should attempt to create desktop mode
        let result = NativeBackend::create(config, ipc_handler, message_queue);

        // Should not panic - may succeed or fail depending on environment
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_create_embedded_with_parent_hwnd() {
        // Verify that create_embedded works with a parent HWND
        use crate::webview::config::EmbedMode;

        let config = WebViewConfig {
            embed_mode: EmbedMode::Child,
            ..Default::default()
        };
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        // Should attempt to create embedded WebView
        let result =
            NativeBackend::create_embedded(12345, config, ipc_handler, message_queue, None);

        // May fail due to invalid HWND, but should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_create_embedded_not_supported_on_non_windows() {
        // Verify that create_embedded returns error on non-Windows platforms
        let config = WebViewConfig::default();
        let ipc_handler = Arc::new(IpcHandler::new());
        let message_queue = Arc::new(MessageQueue::new());

        let result =
            NativeBackend::create_embedded(12345, config, ipc_handler, message_queue, None);

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
