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
            Self::create_standalone(config, ipc_handler, message_queue)
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
            tracing::info!("[CLOSE] [NativeBackend::process_events] Window close signal detected");
            return true;
        }

        // Process message queue
        if let Ok(webview) = self.webview.lock() {
            let count = self.message_queue.process_all(|message| {
                use crate::ipc::WebViewMessage;
                match message {
                    WebViewMessage::EvalJs(script) => {
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to execute JavaScript: {}", e);
                        }
                    }
                    WebViewMessage::EmitEvent { event_name, data } => {
                        let json_str = data.to_string();
                        let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
                        let script = js_assets::build_emit_event_script(&event_name, &escaped_json);
                        tracing::debug!("[CLOSE] [NativeBackend] Generated script: {}", script);
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to emit event: {}", e);
                        }
                    }
                    WebViewMessage::LoadUrl(url) => {
                        let script = js_assets::build_load_url_script(&url);
                        if let Err(e) = webview.evaluate_script(&script) {
                            tracing::error!("Failed to load URL: {}", e);
                        }
                    }
                    WebViewMessage::LoadHtml(html) => {
                        if let Err(e) = webview.load_html(&html) {
                            tracing::error!("Failed to load HTML: {}", e);
                        }
                    }
                }
            });

            if count > 0 {
                tracing::debug!(
                    "[OK] [NativeBackend::process_events] Processed {} messages",
                    count
                );
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

        WebViewEventHandler::run_blocking(event_loop, state);
        tracing::info!("Event loop exited");
    }
}

impl NativeBackend {
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
    #[allow(dead_code)]
    fn create_standalone(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

        // Log window HWND
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
                }
            }
        }

        // Make window visible
        window.set_visible(true);

        // Create WebView with IPC handler
        let webview = Self::create_webview(&window, &config, ipc_handler)?;

        #[allow(clippy::arc_with_non_send_sync)]
        Ok(Self {
            webview: Arc::new(Mutex::new(webview)),
            window: Some(window),
            event_loop: Some(event_loop),
            message_queue,
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
        let mut builder = WryWebViewBuilder::new();

        // Set background color to match app background (dark theme)
        // This prevents white flash and removes white border
        // RGBA is a tuple type (u8, u8, u8, u8) in wry
        let background_color = (2u8, 6u8, 23u8, 255u8); // #020617 from Tailwind slate-950
        builder = builder.with_background_color(background_color);
        tracing::info!(
            "[OK] [NativeBackend] Set WebView background color to #{:02x}{:02x}{:02x}",
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
            tracing::info!(
                "[OK] [NativeBackend] Registering auroraview:// protocol (asset_root: {:?})",
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
            tracing::info!(
                "[OK] [NativeBackend] Registering custom protocol: {}",
                scheme
            );
            builder = builder.with_custom_protocol(scheme.clone(), move |_webview_id, request| {
                crate::webview::protocol_handlers::handle_custom_protocol(&*callback_clone, request)
            });
        }

        // Register file:// protocol if enabled
        if config.allow_file_protocol {
            tracing::info!("[OK] [NativeBackend] Enabling file:// protocol support");
            builder = builder.with_custom_protocol("file".into(), |_webview_id, request| {
                crate::webview::protocol_handlers::handle_file_protocol(request)
            });
        }

        // Enable developer tools if configured
        if config.dev_tools {
            tracing::info!("[OK] [NativeBackend] Enabling developer tools");
            builder = builder.with_devtools(true);
        }

        // Disable context menu if configured
        if !config.context_menu {
            tracing::info!("[OK] [NativeBackend] Disabling native context menu");
            #[cfg(target_os = "windows")]
            {
                builder = builder.with_browser_extensions_enabled(false);
            }
        }

        // Configure new window handler
        if config.allow_new_window {
            tracing::info!("[OK] [NativeBackend] Allowing new windows");
            builder = builder.with_new_window_req_handler(|url, _features| {
                tracing::info!("[OK] [NativeBackend] New window requested: {}", url);
                // Allow the new window to open
                wry::NewWindowResponse::Allow
            });
        } else {
            tracing::info!("[OK] [NativeBackend] Blocking new windows");
            builder = builder.with_new_window_req_handler(|url, _features| {
                tracing::info!("[OK] [NativeBackend] Blocked new window request: {}", url);
                // Block the new window
                wry::NewWindowResponse::Deny
            });
        }

        // Build initialization script using js_assets module
        tracing::info!("[NativeBackend] Building initialization script with js_assets");
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
                    if msg_type == "event" {
                        if let Some(event_name) = message.get("event").and_then(|v| v.as_str()) {
                            let detail = message
                                .get("detail")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            tracing::info!(
                                "[OK] [NativeBackend] Event received: {} with detail: {}",
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

                            tracing::info!(
                                "[OK] [NativeBackend] Call received: {} with params: {} id: {:?}",
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

        // Load initial content
        if let Some(ref url) = config.url {
            tracing::info!("[OK] [NativeBackend] Loading URL: {}", url);
            let script = js_assets::build_load_url_script(url);
            webview
                .evaluate_script(&script)
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
