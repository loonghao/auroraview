//! Browser main structure

use crate::config::BrowserConfig;
use crate::devtools::DevToolsManager;
use crate::extensions::ExtensionRegistry;
use crate::navigation::{BookmarkManager, HistoryManager};
use crate::tab::{TabEvent, TabId, TabManager, TabState};
use crate::{BrowserError, Extension, Result};
use auroraview_core::assets::{get_browser_controller_html, get_event_bridge_js};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::{Window, WindowBuilder};
use wry::WebView as WryWebView;
use wry::WebViewBuilder;
#[cfg(target_os = "windows")]
use wry::WebViewExtWindows;

/// Height of the browser header (tab bar + toolbar)
const HEADER_HEIGHT: u32 = 88;

/// Height with bookmarks bar
const HEADER_HEIGHT_WITH_BOOKMARKS: u32 = 120;

/// Browser events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrowserEvent {
    // Tab events
    TabCreated {
        id: TabId,
        url: String,
    },
    TabClosed {
        id: TabId,
    },
    TabActivated {
        id: TabId,
    },
    TabUpdated {
        id: TabId,
        title: String,
        url: String,
    },
    TabLoading {
        id: TabId,
        is_loading: bool,
    },

    // Navigation events
    NavigationStarted {
        tab_id: TabId,
        url: String,
    },
    NavigationCompleted {
        tab_id: TabId,
        url: String,
    },
    NavigationFailed {
        tab_id: TabId,
        error: String,
    },

    // Bookmark events
    BookmarkAdded {
        id: String,
        url: String,
        title: String,
    },
    BookmarkRemoved {
        id: String,
    },

    // Download events
    DownloadStarted {
        id: String,
        url: String,
        filename: String,
    },
    DownloadProgress {
        id: String,
        received: u64,
        total: u64,
    },
    DownloadCompleted {
        id: String,
        path: String,
    },
}

/// Main browser struct
pub struct Browser {
    config: Rc<BrowserConfig>,
    tabs: Rc<TabManager>,
    bookmarks: BookmarkManager,
    history: HistoryManager,
    extensions: ExtensionRegistry,
    devtools: DevToolsManager,
    controller_webview: Option<WryWebView>,
    window: Option<Window>,
}

impl Browser {
    /// Create a new browser instance
    pub fn new(config: BrowserConfig) -> Self {
        let config = Rc::new(config);

        Self {
            tabs: Rc::new(TabManager::new(config.clone())),
            bookmarks: BookmarkManager::new(config.user_data_dir.as_deref()),
            history: HistoryManager::new(
                config.user_data_dir.as_deref(),
                10000,
                config.features.history,
            ),
            extensions: ExtensionRegistry::new(config.features.extensions),
            devtools: DevToolsManager::new(config.devtools.clone()),
            config,
            controller_webview: None,
            window: None,
        }
    }

    /// Get tab manager
    pub fn tabs(&self) -> &TabManager {
        &self.tabs
    }

    /// Get bookmark manager
    pub fn bookmarks(&self) -> &BookmarkManager {
        &self.bookmarks
    }

    /// Get history manager
    pub fn history(&self) -> &HistoryManager {
        &self.history
    }

    /// Get extension registry
    pub fn extensions(&self) -> &ExtensionRegistry {
        &self.extensions
    }

    /// Get DevTools manager
    pub fn devtools(&self) -> &DevToolsManager {
        &self.devtools
    }

    /// Get mutable DevTools manager
    pub fn devtools_mut(&mut self) -> &mut DevToolsManager {
        &mut self.devtools
    }

    // === Tab Operations ===

    /// Create a new tab
    pub fn new_tab(&self, url: &str) -> Result<TabId> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| BrowserError::WindowCreation("Window not initialized".to_string()))?;

        let header_height = self.header_height();
        self.tabs.create_tab(window, Some(url), header_height)
    }

    /// Close a tab
    pub fn close_tab(&self, id: &TabId) -> Result<()> {
        self.tabs.close_tab(id)
    }

    /// Activate a tab
    pub fn activate_tab(&self, id: &TabId) -> Result<()> {
        self.tabs.activate_tab(id)
    }

    /// Get active tab
    pub fn active_tab(&self) -> Option<TabState> {
        self.tabs.active_tab_state()
    }

    // === Navigation ===

    /// Navigate to URL
    pub fn navigate(&self, url: &str) -> Result<()> {
        self.tabs.navigate(url)
    }

    /// Go back in history
    pub fn go_back(&self) -> Result<()> {
        self.tabs.go_back()
    }

    /// Go forward in history
    pub fn go_forward(&self) -> Result<()> {
        self.tabs.go_forward()
    }

    /// Reload current page
    pub fn reload(&self) -> Result<()> {
        self.tabs.reload()
    }

    // === Bookmarks ===

    /// Add a bookmark
    pub fn add_bookmark(&self, url: &str, title: &str) -> String {
        self.bookmarks.add_bookmark(url, title)
    }

    /// Remove a bookmark
    pub fn remove_bookmark(&self, id: &str) -> bool {
        self.bookmarks.remove(&id.to_string()).is_some()
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> Vec<crate::navigation::Bookmark> {
        self.bookmarks.all()
    }

    // === History ===

    /// Get browsing history
    pub fn get_history(&self, limit: usize) -> Vec<crate::navigation::HistoryEntry> {
        self.history.get(limit)
    }

    /// Clear history
    pub fn clear_history(&self) {
        self.history.clear()
    }

    // === Extensions ===

    /// Register an extension
    pub fn register_extension(&self, ext: Box<dyn Extension>) -> Result<()> {
        self.extensions.register(ext)
    }

    // === Internal ===

    /// Get current header height
    fn header_height(&self) -> u32 {
        if self.config.features.bookmarks_bar && !self.bookmarks.all().is_empty() {
            HEADER_HEIGHT_WITH_BOOKMARKS
        } else {
            HEADER_HEIGHT
        }
    }

    /// Sync tab state to controller WebView
    fn sync_tabs(&self) {
        if let Some(controller) = &self.controller_webview {
            let tabs = self.tabs.tab_states();
            let active_tab_id = self.tabs.active_tab_id();

            let data = serde_json::json!({
                "tabs": tabs,
                "activeTabId": active_tab_id,
            });

            let script = format!(
                r#"(function() {{
                    if (window.auroraview && window.auroraview.trigger) {{
                        window.auroraview.trigger('tabs:update', {});
                    }}
                }})();"#,
                data
            );

            let _ = controller.evaluate_script(&script);
        }
    }

    /// Sync bookmarks to controller WebView
    fn sync_bookmarks(&self) {
        if let Some(controller) = &self.controller_webview {
            let bookmarks = self.bookmarks.bar_items();

            let data = serde_json::json!({
                "bookmarks": bookmarks,
            });

            let script = format!(
                r#"(function() {{
                    if (window.auroraview && window.auroraview.trigger) {{
                        window.auroraview.trigger('bookmarks:update', {});
                    }}
                }})();"#,
                data
            );

            let _ = controller.evaluate_script(&script);
        }
    }

    /// Bring controller WebView to top of z-order
    #[cfg(target_os = "windows")]
    fn bring_controller_to_top(&self) {
        if let Some(controller) = &self.controller_webview {
            let wv2_controller = controller.controller();

            use std::ffi::c_void;

            #[link(name = "user32")]
            extern "system" {
                fn EnumChildWindows(
                    hwnd_parent: *mut c_void,
                    lp_enum_func: unsafe extern "system" fn(*mut c_void, isize) -> i32,
                    l_param: isize,
                ) -> i32;
                fn GetClassNameW(
                    hwnd: *mut c_void,
                    lp_class_name: *mut u16,
                    n_max_count: i32,
                ) -> i32;
                fn SetWindowPos(
                    hwnd: *mut c_void,
                    insert_after: *mut c_void,
                    x: i32,
                    y: i32,
                    cx: i32,
                    cy: i32,
                    flags: u32,
                ) -> i32;
            }

            const HWND_TOP: *mut c_void = std::ptr::null_mut::<c_void>();
            const SWP_NOMOVE: u32 = 0x0002;
            const SWP_NOSIZE: u32 = 0x0001;

            unsafe {
                let mut parent_hwnd: *mut c_void = std::ptr::null_mut();
                if wv2_controller
                    .ParentWindow(&mut parent_hwnd as *mut _ as *mut _)
                    .is_ok()
                    && !parent_hwnd.is_null()
                {
                    unsafe extern "system" fn enum_callback(
                        hwnd: *mut c_void,
                        lparam: isize,
                    ) -> i32 {
                        let result_ptr = lparam as *mut Vec<*mut c_void>;
                        let mut class_name = [0u16; 256];
                        let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
                        if len > 0 {
                            let name = String::from_utf16_lossy(&class_name[..len as usize]);
                            if name.contains("WRY") {
                                (*result_ptr).push(hwnd);
                            }
                        }
                        1
                    }

                    let mut wry_windows: Vec<*mut c_void> = Vec::new();
                    EnumChildWindows(
                        parent_hwnd,
                        enum_callback,
                        &mut wry_windows as *mut Vec<*mut c_void> as isize,
                    );

                    if let Some(&controller_hwnd) = wry_windows.first() {
                        SetWindowPos(
                            controller_hwnd,
                            HWND_TOP,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE,
                        );
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn bring_controller_to_top(&self) {}

    /// Run the browser (blocking)
    pub fn run(&mut self) {
        #[cfg(target_os = "linux")]
        use tao::platform::unix::EventLoopBuilderExtUnix;
        #[cfg(target_os = "windows")]
        use tao::platform::windows::EventLoopBuilderExtWindows;

        tracing::info!("[Browser] Starting with config: {:?}", self.config.title);

        // Create event loop
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let event_loop: EventLoop<TabEvent> = EventLoopBuilder::with_user_event()
            .with_any_thread(true)
            .build();
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let event_loop: EventLoop<TabEvent> = EventLoopBuilder::with_user_event().build();

        self.tabs.set_event_proxy(event_loop.create_proxy());

        // Create window
        let window = WindowBuilder::new()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(true)
            .with_decorations(!self.config.frameless)
            .build(&event_loop)
            .expect("Failed to create window");

        self.window = Some(window);
        let window_ref = self.window.as_ref().unwrap();

        // Create controller WebView
        let controller_html = get_browser_controller_html();
        let event_bridge_script = get_event_bridge_js();

        let tabs_clone = self.tabs.clone();
        let ipc_handler = move |request: wry::http::Request<String>| {
            let body = request.body();
            tracing::debug!("[Browser] IPC message: {}", body);

            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
                    let params = msg.get("params").cloned().unwrap_or(serde_json::json!({}));

                    match method {
                        "browser.new_tab" => {
                            let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::NewTab {
                                    url: if url.is_empty() {
                                        None
                                    } else {
                                        Some(url.to_string())
                                    },
                                });
                            }
                        }
                        "browser.close_tab" => {
                            if let Some(tab_id) = params.get("tabId").and_then(|v| v.as_str()) {
                                if let Some(proxy) = tabs_clone.event_proxy() {
                                    let _ = proxy.send_event(TabEvent::CloseTab {
                                        tab_id: tab_id.to_string(),
                                    });
                                }
                            }
                        }
                        "browser.activate_tab" => {
                            if let Some(tab_id) = params.get("tabId").and_then(|v| v.as_str()) {
                                if let Some(proxy) = tabs_clone.event_proxy() {
                                    let _ = proxy.send_event(TabEvent::ActivateTab {
                                        tab_id: tab_id.to_string(),
                                    });
                                }
                            }
                        }
                        "browser.navigate" => {
                            if let Some(url) = params.get("url").and_then(|v| v.as_str()) {
                                if let Some(proxy) = tabs_clone.event_proxy() {
                                    let _ = proxy.send_event(TabEvent::Navigate {
                                        url: url.to_string(),
                                    });
                                }
                            }
                        }
                        "browser.go_back" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::GoBack);
                            }
                        }
                        "browser.go_forward" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::GoForward);
                            }
                        }
                        "browser.reload" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::Reload);
                            }
                        }
                        "browser.home" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::Home);
                            }
                        }
                        "browser.toggle_devtools" => {
                            let tab_id = params
                                .get("tabId")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::ToggleDevTools { tab_id });
                            }
                        }
                        "browser.open_devtools" => {
                            let tab_id = params
                                .get("tabId")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::OpenDevTools { tab_id });
                            }
                        }
                        "browser.close_devtools" => {
                            let tab_id = params
                                .get("tabId")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::CloseDevTools { tab_id });
                            }
                        }
                        // Window control events (for frameless mode)
                        "browser.minimize" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::Minimize);
                            }
                        }
                        "browser.toggle_maximize" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::ToggleMaximize);
                            }
                        }
                        "browser.close_window" => {
                            if let Some(proxy) = tabs_clone.event_proxy() {
                                let _ = proxy.send_event(TabEvent::Close);
                            }
                        }
                        _ => {
                            tracing::debug!("[Browser] Unknown method: {}", method);
                        }
                    }
                }
            }
        };

        #[cfg(target_os = "windows")]
        let window_size = window_ref.inner_size();
        let header_height = self.header_height();

        let controller_builder = WebViewBuilder::new()
            .with_html(&controller_html)
            .with_initialization_script(&event_bridge_script)
            .with_devtools(self.config.debug)
            .with_ipc_handler(ipc_handler);

        #[cfg(target_os = "windows")]
        let controller_builder = controller_builder.with_bounds(wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
            size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
                window_size.width,
                header_height,
            )),
        });

        let controller = controller_builder
            .build_as_child(window_ref)
            .expect("Failed to create controller WebView");

        self.controller_webview = Some(controller);

        // Create initial tabs
        let initial_urls = if self.config.initial_urls.is_empty() {
            vec![self.config.home_url.clone()]
        } else {
            self.config.initial_urls.clone()
        };

        for url in &initial_urls {
            if let Err(e) = self.tabs.create_tab(window_ref, Some(url), header_height) {
                tracing::error!("[Browser] Failed to create tab: {}", e);
            }
        }

        self.tabs.show_active_tab();
        self.bring_controller_to_top();
        self.sync_tabs();
        self.sync_bookmarks();

        tracing::info!(
            "[Browser] Running event loop with {} tabs...",
            self.tabs.tab_count()
        );

        // Run event loop
        let mut event_loop = event_loop;
        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    tracing::info!("[Browser] Close requested");
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    let header_height = self.header_height();
                    let content_y = header_height as i32;
                    let content_height = size.height.saturating_sub(header_height);

                    #[cfg(target_os = "windows")]
                    if let Some(controller) = &self.controller_webview {
                        let _ = controller.set_bounds(wry::Rect {
                            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                                0.0, 0.0,
                            )),
                            size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
                                size.width,
                                header_height,
                            )),
                        });
                    }

                    self.tabs
                        .resize_tabs(0, content_y, size.width, content_height);
                }
                Event::UserEvent(tab_event) => {
                    self.handle_tab_event(tab_event, control_flow);
                }
                _ => {}
            }
        });

        tracing::info!("[Browser] Event loop exited");
    }

    /// Handle tab events
    fn handle_tab_event(&mut self, event: TabEvent, control_flow: &mut ControlFlow) {
        match event {
            TabEvent::NewTab { url } => {
                if let Some(window) = &self.window {
                    let header_height = self.header_height();
                    if let Ok(tab_id) = self.tabs.create_tab(window, url.as_deref(), header_height)
                    {
                        let _ = self.tabs.activate_tab(&tab_id);
                    }
                }
                self.tabs.show_active_tab();
                self.bring_controller_to_top();
                self.sync_tabs();
            }
            TabEvent::CloseTab { tab_id } => {
                let _ = self.tabs.close_tab(&tab_id);
                self.sync_tabs();

                if self.tabs.is_empty() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            TabEvent::ActivateTab { tab_id } => {
                let _ = self.tabs.activate_tab(&tab_id);
                self.sync_tabs();
            }
            TabEvent::Navigate { url } => {
                let _ = self.tabs.navigate(&url);
                self.sync_tabs();
            }
            TabEvent::GoBack => {
                let _ = self.tabs.go_back();
            }
            TabEvent::GoForward => {
                let _ = self.tabs.go_forward();
            }
            TabEvent::Reload => {
                let _ = self.tabs.reload();
            }
            TabEvent::Home => {
                let _ = self.tabs.home();
                self.sync_tabs();
            }
            TabEvent::TitleChanged { tab_id, title } => {
                self.tabs.update_title(&tab_id, title);
                self.sync_tabs();
            }
            TabEvent::UrlChanged { tab_id, url } => {
                // Add to history
                if let Some(state) = self.tabs.tab_states().iter().find(|t| t.id == tab_id) {
                    self.history.add(&url, &state.title);
                }
                self.tabs.update_url(&tab_id, url);
                self.sync_tabs();
            }
            TabEvent::LoadingChanged { tab_id, is_loading } => {
                self.tabs.update_loading(&tab_id, is_loading);
                self.sync_tabs();
            }
            TabEvent::HistoryChanged {
                tab_id,
                can_go_back,
                can_go_forward,
            } => {
                self.tabs
                    .update_history(&tab_id, can_go_back, can_go_forward);
                self.sync_tabs();
            }
            TabEvent::Close => {
                *control_flow = ControlFlow::Exit;
            }
            TabEvent::Minimize => {
                if let Some(window) = &self.window {
                    window.set_minimized(true);
                }
            }
            TabEvent::ToggleMaximize => {
                if let Some(window) = &self.window {
                    window.set_maximized(!window.is_maximized());
                }
            }
            TabEvent::ToggleDevTools { tab_id } => {
                self.handle_devtools_toggle(tab_id.as_deref());
            }
            TabEvent::OpenDevTools { tab_id } => {
                self.handle_devtools_open(tab_id.as_deref());
            }
            TabEvent::CloseDevTools { tab_id } => {
                self.handle_devtools_close(tab_id.as_deref());
            }
            _ => {}
        }
    }

    /// Handle DevTools toggle for a tab
    fn handle_devtools_toggle(&mut self, tab_id: Option<&str>) {
        if !self.devtools.is_enabled() {
            tracing::debug!("[Browser] DevTools is disabled");
            return;
        }

        let target_tab_id = tab_id
            .map(String::from)
            .or_else(|| self.tabs.active_tab_id());

        if let Some(tab_id) = target_tab_id {
            tracing::info!("[Browser] Toggling DevTools for tab: {}", tab_id);
            self.tabs.toggle_devtools(&tab_id);
            self.devtools.toggle();
        }
    }

    /// Handle DevTools open for a tab
    fn handle_devtools_open(&mut self, tab_id: Option<&str>) {
        if !self.devtools.is_enabled() {
            tracing::debug!("[Browser] DevTools is disabled");
            return;
        }

        let target_tab_id = tab_id
            .map(String::from)
            .or_else(|| self.tabs.active_tab_id());

        if let Some(tab_id) = target_tab_id {
            tracing::info!("[Browser] Opening DevTools for tab: {}", tab_id);
            self.tabs.open_devtools(&tab_id);
            self.devtools.open();
        }
    }

    /// Handle DevTools close for a tab
    fn handle_devtools_close(&mut self, tab_id: Option<&str>) {
        let target_tab_id = tab_id
            .map(String::from)
            .or_else(|| self.tabs.active_tab_id());

        if let Some(tab_id) = target_tab_id {
            tracing::info!("[Browser] Closing DevTools for tab: {}", tab_id);
            self.tabs.close_devtools(&tab_id);
            self.devtools.close();
        }
    }
}
