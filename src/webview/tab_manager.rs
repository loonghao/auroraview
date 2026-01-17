//! Tab Manager for Multi-WebView Browser
//!
//! This module implements a multi-tab browser following Microsoft WebView2Browser architecture.
//!
//! ## Architecture Reference
//!
//! Based on the official Microsoft Edge WebView2Browser sample:
//! - Repository: <https://github.com/MicrosoftEdge/WebView2Browser>
//! - Key files: `BrowserWindow.cpp`, `Tab.cpp`, `BrowserWindow.h`
//!
//! ## Key Design Principles (from Microsoft WebView2Browser)
//!
//! 1. **Single UI Thread** - All WebView creation and operations happen on the same thread
//!    - Quote from BrowserWindow.cpp: "CreateCoreWebView2EnvironmentWithOptions" is called once
//!    - All tabs share the same event loop and message pump
//!
//! 2. **Dual Environment Pattern** - Separates UI and content for security/isolation
//!    - `m_uiEnv`: Used for browser UI (tab bar, toolbar) - isolated from user content
//!    - `m_contentEnv`: Shared by all Tab WebViews - enables cookie/cache sharing
//!    - See: `BrowserWindow.h` lines defining `m_uiEnv` and `m_contentEnv`
//!
//! 3. **Shared Content Environment** - All tabs reuse the same `CoreWebView2Environment`
//!    - When creating new tab: `Tab::CreateNewTab(m_hWnd, m_contentEnv.Get(), ...)`
//!    - Benefits: Cookie sharing, reduced memory, faster tab creation
//!    - wry handles this automatically on Windows when WebViews share the same window
//!
//! 4. **Tab Visibility Management** - Show/hide tabs instead of create/destroy
//!    - Active tab: `SetVisible(true)` + `put_Bounds(contentRect)`
//!    - Inactive tabs: `SetVisible(false)`
//!    - Reduces overhead compared to recreating WebViews
//!
//! ## Implementation Notes
//!
//! ### wry Behavior on Windows
//!
//! When multiple `WebView` instances are created in the same process using wry:
//! - wry internally shares the `CoreWebView2Environment` across WebViews
//! - This happens automatically when WebViews are built on the same window
//! - No explicit environment sharing API is needed (unlike raw Win32 WebView2)
//!
//! ### Comparison with Python multi_webview_browser.py
//!
//! The Python example has architectural issues:
//! - Each Tab creates its WebView in a **separate thread** (via `WebView.create_embedded`)
//! - This may create multiple `CoreWebView2Environment` instances
//! - Violates Microsoft's recommended single-thread/shared-environment pattern
//!
//! This Rust implementation follows the correct pattern:
//! - Single event loop owns all WebViews
//! - All WebViews created on the same thread
//! - Automatic environment sharing via wry
//!
//! ## Architecture Diagram
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Main Window (tao Window)                                       │
//! │  ├── Controller WebView (Tab bar + URL bar)                    │
//! │  │   └── Uses m_uiEnv (isolated browser UI environment)        │
//! │  │                                                              │
//! │  └── Content Area                                               │
//! │      ├── Tab 1 WebView (visible when active)  ─┐               │
//! │      ├── Tab 2 WebView (hidden)                ├─ m_contentEnv │
//! │      └── Tab N WebView (hidden)               ─┘  (shared)     │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! Thread Model:
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  UI Thread (main)                                                │
//! │  ├── EventLoop.run() / run_return()                             │
//! │  ├── Window message pump                                         │
//! │  ├── Controller WebView operations                               │
//! │  └── Tab WebView operations (all on same thread!)               │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## References
//!
//! - Microsoft WebView2Browser: <https://github.com/MicrosoftEdge/WebView2Browser>
//! - WebView2 Best Practices: <https://learn.microsoft.com/en-us/microsoft-edge/webview2/>
//! - wry WebView library: <https://github.com/nicholaswilson/wry>

use std::collections::HashMap;

use auroraview_core::assets::get_browser_controller_html;
use auroraview_core::builder::{get_background_color, log_background_color};
use serde::{Deserialize, Serialize};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::{Window, WindowBuilder};
use wry::WebView as WryWebView;
use wry::WebViewBuilder;
#[cfg(target_os = "windows")]
use wry::WebViewExtWindows;

/// Tab state - tracks the current state of a browser tab
///
/// This struct mirrors the state tracking in Microsoft's Tab.cpp/Tab.h:
/// - Title updates from `document.title` changes
/// - URL updates from navigation events
/// - Loading state from `NavigationStarting`/`NavigationCompleted`
/// - History state from `HistoryChanged` events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// Favicon URL (optional, for future enhancement)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// Security state (optional, mirrors Security.securityStateChanged in Tab.cpp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_state: Option<String>,
}

/// Bookmark entry for the browser
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub url: String,
    pub title: String,
}

impl TabState {
    pub fn new(id: String, url: String) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url,
            is_loading: true, // Start as loading
            can_go_back: false,
            can_go_forward: false,
            favicon: None,
            security_state: None,
        }
    }

    /// Update title (called when document.title changes)
    pub fn set_title(&mut self, title: String) {
        if !title.is_empty() {
            self.title = title;
        }
    }

    /// Update URL (called on SourceChanged event)
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    /// Update loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Update history state (called on HistoryChanged event)
    pub fn set_history_state(&mut self, can_go_back: bool, can_go_forward: bool) {
        self.can_go_back = can_go_back;
        self.can_go_forward = can_go_forward;
    }
}

/// A tab with its WebView
struct Tab {
    state: TabState,
    webview: WryWebView,
}

/// User events for the tab manager event loop
///
/// These events are sent from the controller WebView (via IPC) or from
/// tab WebViews (via navigation callbacks) to the main event loop.
///
/// Reference: Similar to WM_USER messages in BrowserWindow.cpp
#[derive(Debug, Clone)]
pub enum TabManagerEvent {
    // === Tab Management Events (from UI) ===
    /// Create a new tab with optional URL
    NewTab { url: String },
    /// Close a specific tab
    CloseTab { tab_id: String },
    /// Activate/switch to a tab (show it, hide others)
    ActivateTab { tab_id: String },

    // === Navigation Events (from UI) ===
    /// Navigate the active tab to a URL
    Navigate { url: String },
    /// Go back in active tab history
    GoBack,
    /// Go forward in active tab history
    GoForward,
    /// Reload the active tab
    Reload,
    /// Stop loading the active tab
    Stop,
    /// Navigate to home page
    Home,

    // === Tab State Update Events (from Tab WebViews) ===
    /// Tab title changed (from document.title)
    TitleChanged { tab_id: String, title: String },
    /// Tab URL changed (from SourceChanged)
    UrlChanged { tab_id: String, url: String },
    /// Tab loading state changed
    LoadingChanged { tab_id: String, is_loading: bool },
    /// Tab history state changed (can go back/forward)
    HistoryChanged {
        tab_id: String,
        can_go_back: bool,
        can_go_forward: bool,
    },
    /// Tab favicon changed (for future use)
    FaviconChanged { tab_id: String, favicon_url: String },

    // === Window Events ===
    /// Close the browser window
    Close,
    /// Minimize the window
    Minimize,
    /// Toggle maximize/restore the window
    ToggleMaximize,
    /// Resize the content area for tab WebViews
    ResizeContent {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },

    // === Bookmark Events ===
    /// Add a bookmark
    AddBookmark { url: String, title: String },
    /// Remove a bookmark by URL
    RemoveBookmark { url: String },
    /// Get all bookmarks (response via IPC)
    GetBookmarks,
}

/// Tab Manager configuration
///
/// Reference: Similar to command-line options and defaults in BrowserWindow.cpp
#[derive(Debug, Clone)]
pub struct TabManagerConfig {
    /// Window title
    pub title: String,
    /// Window width in logical pixels
    pub width: u32,
    /// Window height in logical pixels
    pub height: u32,
    /// Height of the header area (tab bar + toolbar)
    /// Reference: In BrowserWindow, this is calculated from UI WebView layout
    pub header_height: u32,
    /// Home page URL (navigated when opening new tabs or pressing Home)
    pub home_url: String,
    /// Enable DevTools for debugging
    pub debug: bool,
    /// Initial URLs to open as tabs
    pub initial_urls: Vec<String>,
    /// Whether to restore previous session (for future use)
    pub restore_session: bool,
    /// Frameless window (no native title bar)
    pub frameless: bool,
}

impl Default for TabManagerConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView Browser".to_string(),
            width: 1280,
            height: 900,
            header_height: 180, // Tab bar + Toolbar + Bookmarks bar (with DPI scaling margin)
            home_url: "https://www.google.com".to_string(),
            debug: false,
            initial_urls: vec![], // Empty = open home page
            restore_session: false,
            frameless: true, // Default to frameless for modern look
        }
    }
}

impl TabManagerConfig {
    /// Create a new config with custom title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window dimensions
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set home URL
    pub fn with_home_url(mut self, url: impl Into<String>) -> Self {
        self.home_url = url.into();
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set initial URLs to open
    pub fn with_initial_urls(mut self, urls: Vec<String>) -> Self {
        self.initial_urls = urls;
        self
    }

    /// Enable/disable frameless window
    pub fn with_frameless(mut self, frameless: bool) -> Self {
        self.frameless = frameless;
        self
    }
}

/// Tab Manager - manages multiple WebViews in a single window
///
/// This is the Rust equivalent of Microsoft's BrowserWindow class.
/// It follows the same architecture patterns:
///
/// 1. **Single Window with Multiple WebViews**
///    - One main tao Window
///    - One controller WebView for UI (tab bar, toolbar)
///    - Multiple content WebViews (one per tab)
///
/// 2. **Shared Environment (automatic with wry)**
///    - wry shares `CoreWebView2Environment` across WebViews in same process
///    - Enables cookie/cache sharing between tabs
///
/// 3. **Tab Visibility Management**
///    - Active tab: visible, receives input
///    - Inactive tabs: hidden, preserved state
///
/// Reference: `BrowserWindow.cpp` and `BrowserWindow.h` from
/// <https://github.com/MicrosoftEdge/WebView2Browser>
pub struct TabManager {
    config: TabManagerConfig,
    /// All tabs indexed by ID (equivalent to `m_tabs` in BrowserWindow.cpp)
    tabs: HashMap<String, Tab>,
    /// Currently active tab ID (equivalent to `m_activeTabId`)
    active_tab_id: Option<String>,
    /// Order of tabs for UI display
    tab_order: Vec<String>,
    /// Controller WebView for browser UI
    controller_webview: Option<WryWebView>,
    /// Main window handle
    window: Option<Window>,
    /// Event loop proxy for sending events from callbacks
    event_loop_proxy: Option<EventLoopProxy<TabManagerEvent>>,
    /// Counter for generating unique tab IDs
    tab_counter: u32,
    /// Bookmarks storage
    bookmarks: Vec<Bookmark>,
}

impl TabManager {
    /// Create a new Tab Manager
    ///
    /// This initializes the manager but doesn't create any windows yet.
    /// Call `run()` to start the browser.
    pub fn new(config: TabManagerConfig) -> Self {
        Self {
            config,
            tabs: HashMap::new(),
            active_tab_id: None,
            tab_order: Vec::new(),
            controller_webview: None,
            window: None,
            event_loop_proxy: None,
            tab_counter: 0,
            bookmarks: Vec::new(),
        }
    }

    /// Get the event loop proxy for external event injection
    pub fn event_proxy(&self) -> Option<&EventLoopProxy<TabManagerEvent>> {
        self.event_loop_proxy.as_ref()
    }

    /// Get current tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get active tab ID
    pub fn active_tab(&self) -> Option<&str> {
        self.active_tab_id.as_deref()
    }

    /// Generate a unique tab ID
    fn next_tab_id(&mut self) -> String {
        self.tab_counter += 1;
        format!("tab_{}", self.tab_counter)
    }

    /// Get controller HTML with tab bar and toolbar (Edge-style)
    ///
    /// This method loads the browser controller HTML from the assets module.
    /// The HTML provides an Edge-style browser UI with:
    /// - Tab bar with rounded tabs, favicons, and close buttons
    /// - Navigation toolbar (back, forward, reload, home)
    /// - URL/search bar with modern styling
    fn get_controller_html(&self) -> String {
        get_browser_controller_html()
    }

    /// Create a new tab
    ///
    /// This is equivalent to `Tab::CreateNewTab` in Microsoft's Tab.cpp.
    /// Key behaviors:
    /// - Uses shared `CoreWebView2Environment` (automatic with wry)
    /// - Positions WebView in content area below the header
    /// - Starts hidden (caller should activate if needed)
    ///
    /// Reference: Tab.cpp `CreateNewTab` and `Init` methods
    fn create_tab(&mut self, url: &str) -> Option<String> {
        // Check if window exists before proceeding (early return enables borrowing later)
        self.window.as_ref()?;

        // Generate tab ID before borrowing window (requires &mut self)
        let tab_id = self.next_tab_id();
        let actual_url = if url.is_empty() {
            self.config.home_url.clone()
        } else {
            url.to_string()
        };
        let debug = self.config.debug;
        let header_height = self.config.header_height;

        // Get event loop proxy for sending events from callbacks
        let proxy = self.event_loop_proxy.clone()?;

        // Now get window reference for the build phase
        let window = self.window.as_ref()?;

        tracing::info!(
            "[TabManager] Creating tab {} with URL: {}",
            tab_id,
            actual_url
        );

        // Calculate content area bounds
        // Reference: BrowserWindow::ResizeUIWebViews calculates content area
        let size = window.inner_size();
        let content_y = header_height as i32;
        let content_height = size.height.saturating_sub(header_height);

        // Set dark background color to prevent white flash during loading
        let background_color = get_background_color();
        log_background_color(background_color);

        // Create WebView for this tab
        // Note: On Windows, wry automatically shares CoreWebView2Environment
        // across WebViews in the same process, matching the pattern from
        // BrowserWindow.cpp where m_contentEnv is passed to Tab::CreateNewTab
        let mut builder = WebViewBuilder::new()
            .with_url(&actual_url)
            .with_devtools(debug)
            .with_visible(false) // Start hidden (Tab.cpp: initial state)
            .with_background_color(background_color);

        // Add document title changed handler
        // Reference: Tab.cpp - Title updates from document.title changes
        let tab_id_for_title = tab_id.clone();
        let proxy_for_title = proxy.clone();
        builder = builder.with_document_title_changed_handler(move |title| {
            tracing::debug!("[Tab {}] Title changed: {}", tab_id_for_title, title);
            let _ = proxy_for_title.send_event(TabManagerEvent::TitleChanged {
                tab_id: tab_id_for_title.clone(),
                title,
            });
        });

        // Add navigation handler for URL changes and loading state
        // Reference: Tab.cpp - NavigationStarting/NavigationCompleted events
        let tab_id_for_nav = tab_id.clone();
        let proxy_for_nav = proxy.clone();
        builder = builder.with_navigation_handler(move |uri| {
            tracing::debug!("[Tab {}] Navigation to: {}", tab_id_for_nav, uri);
            // Update URL when navigation starts
            let _ = proxy_for_nav.send_event(TabManagerEvent::UrlChanged {
                tab_id: tab_id_for_nav.clone(),
                url: uri.clone(),
            });
            // Mark as loading
            let _ = proxy_for_nav.send_event(TabManagerEvent::LoadingChanged {
                tab_id: tab_id_for_nav.clone(),
                is_loading: true,
            });
            true // Allow navigation
        });

        // Add page load handler for loading state completion
        let tab_id_for_load = tab_id.clone();
        let proxy_for_load = proxy.clone();
        builder = builder.with_on_page_load_handler(move |event, _url| {
            if matches!(event, wry::PageLoadEvent::Finished) {
                tracing::debug!("[Tab {}] Page load finished", tab_id_for_load);
                let _ = proxy_for_load.send_event(TabManagerEvent::LoadingChanged {
                    tab_id: tab_id_for_load.clone(),
                    is_loading: false,
                });
            }
        });

        // Set position within parent window (Windows-specific bounds)
        // IMPORTANT: Use build_as_child() so WRY respects bounds settings
        #[cfg(target_os = "windows")]
        let builder = builder.with_bounds(wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                0.0,
                content_y as f64,
            )),
            size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(size.width, content_height)),
        });

        // Use build_as_child() to ensure bounds are respected
        match builder.build_as_child(window) {
            Ok(webview) => {
                let state = TabState::new(tab_id.clone(), actual_url);
                self.tabs.insert(tab_id.clone(), Tab { state, webview });

                // Add to tab order
                self.tab_order.push(tab_id.clone());

                // If this is the first tab, make it active
                if self.active_tab_id.is_none() {
                    self.active_tab_id = Some(tab_id.clone());
                }

                tracing::info!("[TabManager] Tab {} created successfully", tab_id);
                Some(tab_id)
            }
            Err(e) => {
                tracing::error!("[TabManager] Failed to create tab WebView: {}", e);
                None
            }
        }
    }

    /// Close a tab
    ///
    /// Reference: BrowserWindow::HandleTabClose in BrowserWindow.cpp
    /// - Removes tab from collection
    /// - If closing active tab, switches to adjacent tab
    fn close_tab(&mut self, tab_id: &str) {
        tracing::info!("[TabManager] Closing tab: {}", tab_id);

        // Remove from tab order first
        self.tab_order.retain(|id| id != tab_id);

        if self.tabs.remove(tab_id).is_some() {
            // If closed the active tab, switch to another
            if self.active_tab_id.as_deref() == Some(tab_id) {
                // Try to activate the next tab, or the previous one
                self.active_tab_id = self.tab_order.first().cloned();
                self.show_active_tab();
            }
        }
    }

    /// Activate a tab (show it, hide others)
    fn activate_tab(&mut self, tab_id: &str) {
        if self.tabs.contains_key(tab_id) {
            tracing::info!("[TabManager] Activating tab: {}", tab_id);
            self.active_tab_id = Some(tab_id.to_string());
            self.show_active_tab();
        }
    }

    /// Show the active tab, hide others
    fn show_active_tab(&mut self) {
        let active_id = self.active_tab_id.clone();
        for (id, tab) in &mut self.tabs {
            let is_active = active_id.as_deref() == Some(id.as_str());
            let _ = tab.webview.set_visible(is_active);
        }
    }

    /// Navigate the active tab to a URL
    fn navigate(&mut self, url: &str) {
        if let Some(tab_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get_mut(tab_id) {
                // Normalize URL
                let url = if url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("file://")
                {
                    url.to_string()
                } else if url.contains('.') && !url.contains(' ') {
                    format!("https://{}", url)
                } else {
                    format!("https://www.google.com/search?q={}", url)
                };

                tracing::info!("[TabManager] Navigating to: {}", url);
                tab.state.url = url.clone();
                tab.state.is_loading = true;
                let _ = tab.webview.load_url(&url);
            }
        }
    }

    /// Go back in active tab
    fn go_back(&self) {
        if let Some(tab_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get(tab_id) {
                let _ = tab.webview.evaluate_script("history.back()");
            }
        }
    }

    /// Go forward in active tab
    fn go_forward(&self) {
        if let Some(tab_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get(tab_id) {
                let _ = tab.webview.evaluate_script("history.forward()");
            }
        }
    }

    /// Reload active tab
    fn reload(&self) {
        if let Some(tab_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get(tab_id) {
                let _ = tab.webview.evaluate_script("location.reload()");
            }
        }
    }

    /// Resize all tab WebViews to match content area
    #[cfg(target_os = "windows")]
    fn resize_tabs(&mut self, x: i32, y: i32, width: u32, height: u32) {
        for tab in self.tabs.values() {
            let _ = tab.webview.set_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                    x as f64, y as f64,
                )),
                size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(width, height)),
            });
        }
    }

    /// Resize all tab WebViews to match content area (non-Windows placeholder)
    #[cfg(not(target_os = "windows"))]
    fn resize_tabs(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {
        // TODO: Implement for other platforms
    }

    /// Get current tab states in display order
    ///
    /// Returns tabs in the order they appear in the tab bar
    fn get_tab_states(&self) -> Vec<TabState> {
        self.tab_order
            .iter()
            .filter_map(|id| self.tabs.get(id).map(|t| t.state.clone()))
            .collect()
    }

    /// Update a tab's title
    fn update_tab_title(&mut self, tab_id: &str, title: String) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.state.set_title(title);
        }
    }

    /// Update a tab's URL
    fn update_tab_url(&mut self, tab_id: &str, url: String) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.state.set_url(url);
        }
    }

    /// Update a tab's loading state
    fn update_tab_loading(&mut self, tab_id: &str, is_loading: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.state.set_loading(is_loading);
        }
    }

    /// Update a tab's history state
    fn update_tab_history(&mut self, tab_id: &str, can_go_back: bool, can_go_forward: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.state.set_history_state(can_go_back, can_go_forward);
        }
    }

    /// Sync tab state to controller WebView
    fn sync_tabs(&self) {
        if let Some(controller) = &self.controller_webview {
            let tabs = self.get_tab_states();
            let data = serde_json::json!({
                "tabs": tabs,
                "activeTabId": self.active_tab_id,
            });

            let script = format!(
                r#"(function() {{
                    if (window.auroraview && window.auroraview.trigger) {{
                        window.auroraview.trigger('tabs:update', {data});
                    }}
                }})();"#,
            );

            let _ = controller.evaluate_script(&script);
        }
    }

    /// Bring controller WebView to the top of z-order
    ///
    /// This is necessary because newly created Tab WebViews will be placed
    /// on top of existing WebViews by default. We need to ensure the controller
    /// (tab bar + toolbar) remains visible on top.
    ///
    /// Reference: Windows z-order - later created child windows are on top
    #[cfg(target_os = "windows")]
    fn bring_controller_to_top(&self) {
        if let Some(controller) = &self.controller_webview {
            // Get the controller's ICoreWebView2Controller
            let wv2_controller = controller.controller();

            // Use raw Win32 API calls - use *mut c_void for consistency with webview_inner.rs
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
                // Get parent window from controller
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
                            // WRY creates container windows with "WRY_WEBVIEW" class
                            if name.contains("WRY") {
                                (*result_ptr).push(hwnd);
                            }
                        }
                        1 // Continue enumeration
                    }

                    // Find all WRY WebView child windows
                    let mut wry_windows: Vec<*mut c_void> = Vec::new();
                    EnumChildWindows(
                        parent_hwnd,
                        enum_callback,
                        &mut wry_windows as *mut Vec<*mut c_void> as isize,
                    );

                    // The first WRY window is the controller (created first)
                    // Bring it to the top of z-order
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
                        tracing::debug!(
                            "[TabManager] Brought controller WebView to top: 0x{:X}",
                            controller_hwnd as isize
                        );
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn bring_controller_to_top(&self) {
        // No-op on non-Windows platforms
    }

    /// Run the tab manager (blocking)
    ///
    /// This is the main entry point that creates the window and runs the event loop.
    /// Equivalent to BrowserWindow::LaunchWindow + message loop in BrowserWindow.cpp
    ///
    /// ## Architecture (from Microsoft WebView2Browser)
    ///
    /// 1. Create main window (`BrowserWindow::InitInstance`)
    /// 2. Create UI environment and controller WebView (`BrowserWindow::InitUIWebViews`)
    /// 3. Create content environment for tabs (automatic with wry)
    /// 4. Create initial tabs (`Tab::CreateNewTab`)
    /// 5. Run message loop (this function blocks until window closes)
    ///
    /// Reference: <https://github.com/MicrosoftEdge/WebView2Browser>
    pub fn run(&mut self) {
        #[cfg(target_os = "linux")]
        use tao::platform::unix::EventLoopBuilderExtUnix;
        #[cfg(target_os = "windows")]
        use tao::platform::windows::EventLoopBuilderExtWindows;

        tracing::info!(
            "[TabManager] Starting browser with config: {:?}",
            self.config
        );

        // Step 1: Create event loop on UI thread
        // Reference: BrowserWindow uses single UI thread for all operations
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let event_loop: EventLoop<TabManagerEvent> = EventLoopBuilder::with_user_event()
            .with_any_thread(true)
            .build();
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let event_loop: EventLoop<TabManagerEvent> = EventLoopBuilder::with_user_event().build();
        self.event_loop_proxy = Some(event_loop.create_proxy());

        // Step 2: Create main window
        // Reference: BrowserWindow::InitInstance creates the main HWND
        let window = WindowBuilder::new()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(true)
            .with_decorations(!self.config.frameless)
            .build(&event_loop)
            .expect("Failed to create window");

        self.window = Some(window);
        let window_ref = self.window.as_ref().unwrap();

        // Step 3: Create controller WebView for browser UI
        // Reference: BrowserWindow::CreateBrowserControlsWebView
        // This uses a separate "UI environment" for browser controls
        let controller_html = self.get_controller_html();
        let proxy_clone = self.event_loop_proxy.clone();

        // Set up IPC handler for controller UI
        // Reference: BrowserWindow::SetUIMessageBroker handles UI messages
        let ipc_handler = move |request: wry::http::Request<String>| {
            let body = request.body();
            tracing::debug!("[TabManager] IPC message: {}", body);

            // Parse the message and send appropriate event
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
                    let params = msg.get("params").cloned().unwrap_or(serde_json::json!({}));

                    if let Some(proxy) = &proxy_clone {
                        let event = match method {
                            "browser.new_tab" => {
                                let url = params
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::NewTab { url })
                            }
                            "browser.close_tab" => {
                                let tab_id = params
                                    .get("tabId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::CloseTab { tab_id })
                            }
                            "browser.activate_tab" => {
                                let tab_id = params
                                    .get("tabId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::ActivateTab { tab_id })
                            }
                            "browser.navigate" => {
                                let url = params
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::Navigate { url })
                            }
                            "browser.go_back" => Some(TabManagerEvent::GoBack),
                            "browser.go_forward" => Some(TabManagerEvent::GoForward),
                            "browser.reload" => Some(TabManagerEvent::Reload),
                            "browser.stop" => Some(TabManagerEvent::Stop),
                            "browser.home" => Some(TabManagerEvent::Home),
                            // Window control events (for frameless mode)
                            "browser.minimize" => Some(TabManagerEvent::Minimize),
                            "browser.toggle_maximize" => Some(TabManagerEvent::ToggleMaximize),
                            "browser.close_window" => Some(TabManagerEvent::Close),
                            "browser.add_bookmark" => {
                                let url = params
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let title = params
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::AddBookmark { url, title })
                            }
                            "browser.remove_bookmark" => {
                                let url = params
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                Some(TabManagerEvent::RemoveBookmark { url })
                            }
                            "browser.get_bookmarks" => Some(TabManagerEvent::GetBookmarks),
                            _ => None,
                        };

                        if let Some(e) = event {
                            let _ = proxy.send_event(e);
                        }
                    }
                }
            }
        };

        // Create controller WebView with IPC
        // Reference: Uses m_uiEnv (isolated from content environment)
        //
        // Controller needs the event bridge script to enable:
        // - auroraview.call() for sending commands to Rust
        // - auroraview.on() for receiving tab state updates
        let event_bridge_script = auroraview_core::assets::get_event_bridge_js();

        // Controller WebView only occupies the header area (tab bar + toolbar)
        // This prevents it from overlapping with tab content WebViews
        let window_size = window_ref.inner_size();
        let header_height = self.config.header_height;

        // Set dark background color to prevent white flash during resize
        let background_color = get_background_color();

        let mut controller_builder = WebViewBuilder::new()
            .with_html(&controller_html)
            .with_initialization_script(&event_bridge_script)
            .with_devtools(self.config.debug)
            .with_ipc_handler(ipc_handler)
            .with_background_color(background_color);

        // Set controller bounds to header area only (Windows-specific)
        // IMPORTANT: Use build_as_child() so WRY respects bounds settings
        // (build() ignores bounds and fills the entire parent window)
        #[cfg(target_os = "windows")]
        {
            controller_builder = controller_builder.with_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
                size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
                    window_size.width,
                    header_height,
                )),
            });
        }

        // Use build_as_child() to ensure bounds are respected
        let controller = controller_builder
            .build_as_child(window_ref)
            .expect("Failed to create controller WebView");

        self.controller_webview = Some(controller);

        // Step 4: Create initial tabs
        // Reference: BrowserWindow creates first tab after UI is ready
        let initial_urls = if self.config.initial_urls.is_empty() {
            vec![self.config.home_url.clone()]
        } else {
            self.config.initial_urls.clone()
        };

        for url in &initial_urls {
            if let Some(tab_id) = self.create_tab(url) {
                // Activate the first tab
                if self.active_tab_id.is_none() {
                    self.active_tab_id = Some(tab_id);
                }
            }
        }

        // Show the active tab and sync state to UI
        self.show_active_tab();
        // Bring controller to top after creating initial tabs
        self.bring_controller_to_top();
        self.sync_tabs();

        tracing::info!(
            "[TabManager] Running event loop with {} tabs...",
            self.tabs.len()
        );

        // Step 5: Run event loop
        // Reference: BrowserWindow::WndProc handles all window messages
        let mut event_loop = event_loop;
        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    tracing::info!("[TabManager] Close requested");
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Resize controller WebView to header area
                    // Resize tab WebViews to fill content area below header
                    // Reference: BrowserWindow::ResizeUIWebViews
                    let header_height = self.config.header_height;
                    let content_y = header_height as i32;
                    let content_height = size.height.saturating_sub(header_height);

                    // Update controller bounds (header area)
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

                    // Update tab WebViews bounds (content area)
                    self.resize_tabs(0, content_y, size.width, content_height);
                }
                Event::UserEvent(tab_event) => {
                    self.handle_tab_event(tab_event, control_flow);
                }
                _ => {}
            }
        });

        tracing::info!("[TabManager] Event loop exited, cleaning up...");
    }

    /// Handle tab manager events
    ///
    /// Reference: BrowserWindow::SetUIMessageBroker switch statement
    fn handle_tab_event(&mut self, event: TabManagerEvent, control_flow: &mut ControlFlow) {
        match event {
            // === Tab Management ===
            TabManagerEvent::NewTab { url } => {
                // Reference: MG_CREATE_TAB in BrowserWindow.cpp
                if let Some(tab_id) = self.create_tab(&url) {
                    self.active_tab_id = Some(tab_id);
                }
                self.show_active_tab();
                // Bring controller to top after creating new tab
                // (new WebView is created on top, so we need to restore z-order)
                self.bring_controller_to_top();
                self.sync_tabs();
            }
            TabManagerEvent::CloseTab { tab_id } => {
                // Reference: MG_CLOSE_TAB
                self.close_tab(&tab_id);
                self.sync_tabs();

                // Close window if no tabs left
                if self.tabs.is_empty() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            TabManagerEvent::ActivateTab { tab_id } => {
                // Reference: MG_SWITCH_TAB
                self.activate_tab(&tab_id);
                self.sync_tabs();
            }

            // === Navigation ===
            TabManagerEvent::Navigate { url } => {
                // Reference: MG_NAVIGATE
                self.navigate(&url);
                self.sync_tabs();
            }
            TabManagerEvent::GoBack => {
                // Reference: MG_GO_BACK
                self.go_back();
            }
            TabManagerEvent::GoForward => {
                // Reference: MG_GO_FORWARD
                self.go_forward();
            }
            TabManagerEvent::Reload => {
                // Reference: MG_RELOAD
                self.reload();
            }
            TabManagerEvent::Stop => {
                self.stop();
            }
            TabManagerEvent::Home => {
                self.navigate(&self.config.home_url.clone());
                self.sync_tabs();
            }

            // === Tab State Updates (from Tab WebViews) ===
            TabManagerEvent::TitleChanged { tab_id, title } => {
                // Reference: HandleTabTitleUpdate
                self.update_tab_title(&tab_id, title);
                self.sync_tabs();
            }
            TabManagerEvent::UrlChanged { tab_id, url } => {
                // Reference: HandleTabSourceChanged
                self.update_tab_url(&tab_id, url);
                self.sync_tabs();
            }
            TabManagerEvent::LoadingChanged { tab_id, is_loading } => {
                // Reference: HandleTabNavStarting / HandleTabNavCompleted
                self.update_tab_loading(&tab_id, is_loading);
                self.sync_tabs();
            }
            TabManagerEvent::HistoryChanged {
                tab_id,
                can_go_back,
                can_go_forward,
            } => {
                // Reference: HandleTabHistoryUpdate
                self.update_tab_history(&tab_id, can_go_back, can_go_forward);
                self.sync_tabs();
            }
            TabManagerEvent::FaviconChanged {
                tab_id,
                favicon_url,
            } => {
                // For future use
                if let Some(tab) = self.tabs.get_mut(&tab_id) {
                    tab.state.favicon = Some(favicon_url);
                }
                self.sync_tabs();
            }

            // === Window Events ===
            TabManagerEvent::Close => {
                *control_flow = ControlFlow::Exit;
            }
            TabManagerEvent::Minimize => {
                if let Some(window) = &self.window {
                    window.set_minimized(true);
                }
            }
            TabManagerEvent::ToggleMaximize => {
                if let Some(window) = &self.window {
                    window.set_maximized(!window.is_maximized());
                }
            }
            TabManagerEvent::ResizeContent {
                x,
                y,
                width,
                height,
            } => {
                self.resize_tabs(x, y, width, height);
            }

            // === Bookmark Events ===
            TabManagerEvent::AddBookmark { url, title } => {
                self.add_bookmark(url, title);
                self.sync_bookmarks();
            }
            TabManagerEvent::RemoveBookmark { url } => {
                self.remove_bookmark(&url);
                self.sync_bookmarks();
            }
            TabManagerEvent::GetBookmarks => {
                self.sync_bookmarks();
            }
        }
    }

    /// Add a bookmark
    fn add_bookmark(&mut self, url: String, title: String) {
        // Check if already exists
        if self.bookmarks.iter().any(|b| b.url == url) {
            tracing::debug!("[TabManager] Bookmark already exists: {}", url);
            return;
        }
        self.bookmarks.push(Bookmark { url, title });
        tracing::info!(
            "[TabManager] Added bookmark: {} bookmarks total",
            self.bookmarks.len()
        );
    }

    /// Remove a bookmark by URL
    fn remove_bookmark(&mut self, url: &str) {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.url != url);
        if self.bookmarks.len() < initial_len {
            tracing::info!(
                "[TabManager] Removed bookmark: {} bookmarks remaining",
                self.bookmarks.len()
            );
        }
    }

    /// Sync bookmarks to the controller WebView
    fn sync_bookmarks(&self) {
        if let Some(controller) = &self.controller_webview {
            let bookmarks_json = serde_json::json!({ "bookmarks": self.bookmarks });
            let script = format!(
                "if (window.auroraview && window.auroraview.trigger) {{ window.auroraview.trigger('bookmarks:update', {}); }}",
                bookmarks_json
            );
            let _ = controller.evaluate_script(&script);
        }
    }

    /// Stop loading the active tab
    fn stop(&self) {
        if let Some(tab_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get(tab_id) {
                let _ = tab.webview.evaluate_script("window.stop()");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_state_new() {
        let state = TabState::new("tab_1".to_string(), "https://example.com".to_string());
        assert_eq!(state.id, "tab_1");
        assert_eq!(state.url, "https://example.com");
        assert_eq!(state.title, "New Tab");
        assert!(state.is_loading); // Changed: starts as loading
    }

    #[test]
    fn test_tab_state_setters() {
        let mut state = TabState::new("tab_1".to_string(), "https://example.com".to_string());

        state.set_title("Google".to_string());
        assert_eq!(state.title, "Google");

        state.set_url("https://google.com".to_string());
        assert_eq!(state.url, "https://google.com");

        state.set_loading(false);
        assert!(!state.is_loading);

        state.set_history_state(true, false);
        assert!(state.can_go_back);
        assert!(!state.can_go_forward);
    }

    #[test]
    fn test_tab_manager_config_default() {
        let config = TabManagerConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 900);
        assert_eq!(config.header_height, 180);
        assert_eq!(config.title, "AuroraView Browser");
    }

    #[test]
    fn test_tab_manager_config_builder() {
        let config = TabManagerConfig::default()
            .with_title("My Browser")
            .with_size(800, 600)
            .with_home_url("https://github.com")
            .with_debug(true);

        assert_eq!(config.title, "My Browser");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.home_url, "https://github.com");
        assert!(config.debug);
    }

    #[test]
    fn test_tab_manager_next_tab_id() {
        let mut manager = TabManager::new(TabManagerConfig::default());
        assert_eq!(manager.next_tab_id(), "tab_1");
        assert_eq!(manager.next_tab_id(), "tab_2");
        assert_eq!(manager.next_tab_id(), "tab_3");
    }

    #[test]
    fn test_tab_manager_initial_state() {
        let manager = TabManager::new(TabManagerConfig::default());
        assert_eq!(manager.tab_count(), 0);
        assert!(manager.active_tab().is_none());
        assert!(manager.event_proxy().is_none());
    }
}
