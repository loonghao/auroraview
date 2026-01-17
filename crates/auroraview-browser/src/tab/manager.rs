//! Tab Manager implementation

use super::{Tab, TabEvent, TabId, TabState};
use crate::config::BrowserConfig;
use crate::BrowserError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::WebViewBuilder;

/// Tab Manager - manages multiple WebViews in a single window
///
/// This is the Rust equivalent of Microsoft's BrowserWindow class.
/// It follows the same architecture patterns:
///
/// 1. **Single Window with Multiple WebViews**
/// 2. **Shared Environment** (automatic with wry)
/// 3. **Tab Visibility Management**
pub struct TabManager {
    /// All tabs indexed by ID
    tabs: RwLock<HashMap<TabId, Tab>>,
    /// Currently active tab ID
    active_tab_id: RwLock<Option<TabId>>,
    /// Order of tabs for UI display
    tab_order: RwLock<Vec<TabId>>,
    /// Counter for generating unique tab IDs
    tab_counter: AtomicU32,
    /// Browser configuration
    config: Rc<BrowserConfig>,
    /// Event loop proxy for sending events
    event_proxy: RwLock<Option<EventLoopProxy<TabEvent>>>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new(config: Rc<BrowserConfig>) -> Self {
        Self {
            tabs: RwLock::new(HashMap::new()),
            active_tab_id: RwLock::new(None),
            tab_order: RwLock::new(Vec::new()),
            tab_counter: AtomicU32::new(0),
            config,
            event_proxy: RwLock::new(None),
        }
    }

    /// Set the event loop proxy
    pub fn set_event_proxy(&self, proxy: EventLoopProxy<TabEvent>) {
        *self.event_proxy.write() = Some(proxy);
    }

    /// Get the event loop proxy
    pub fn event_proxy(&self) -> Option<EventLoopProxy<TabEvent>> {
        self.event_proxy.read().clone()
    }

    /// Generate a unique tab ID
    fn next_tab_id(&self) -> TabId {
        let id = self.tab_counter.fetch_add(1, Ordering::SeqCst);
        format!("tab_{}", id)
    }

    /// Get current tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.read().len()
    }

    /// Get active tab ID
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.active_tab_id.read().clone()
    }

    /// Get tab order
    pub fn tab_order(&self) -> Vec<TabId> {
        self.tab_order.read().clone()
    }

    /// Get all tab states in display order
    pub fn tab_states(&self) -> Vec<TabState> {
        let tabs = self.tabs.read();
        let order = self.tab_order.read();
        order
            .iter()
            .filter_map(|id| tabs.get(id).map(|t| t.state.clone()))
            .collect()
    }

    /// Create a new tab
    pub fn create_tab(
        &self,
        window: &Window,
        url: Option<&str>,
        header_height: u32,
    ) -> crate::Result<TabId> {
        let tab_id = self.next_tab_id();
        let actual_url = url
            .filter(|u| !u.is_empty())
            .unwrap_or(&self.config.home_url)
            .to_string();

        tracing::info!(
            "[TabManager] Creating tab {} with URL: {}",
            tab_id,
            actual_url
        );

        // Calculate content area bounds
        let size = window.inner_size();
        let content_y = header_height as i32;
        let content_height = size.height.saturating_sub(header_height);

        // Build WebView
        let builder = WebViewBuilder::new()
            .with_url(&actual_url)
            .with_devtools(self.config.debug || self.config.features.dev_tools)
            .with_visible(false);

        // Set position within parent window (Windows-specific bounds)
        #[cfg(target_os = "windows")]
        let builder = builder.with_bounds(wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                0.0,
                content_y as f64,
            )),
            size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(size.width, content_height)),
        });

        let webview = builder
            .build_as_child(window)
            .map_err(|e| BrowserError::WebViewCreation(e.to_string()))?;

        let state = TabState::new(tab_id.clone(), actual_url);
        let tab = Tab::new(state, webview);

        // Insert tab
        {
            let mut tabs = self.tabs.write();
            tabs.insert(tab_id.clone(), tab);
        }

        // Add to tab order
        {
            let mut order = self.tab_order.write();
            order.push(tab_id.clone());
        }

        // If this is the first tab, make it active
        {
            let mut active = self.active_tab_id.write();
            if active.is_none() {
                *active = Some(tab_id.clone());
            }
        }

        tracing::info!("[TabManager] Tab {} created successfully", tab_id);
        Ok(tab_id)
    }

    /// Close a tab
    pub fn close_tab(&self, tab_id: &TabId) -> crate::Result<()> {
        tracing::info!("[TabManager] Closing tab: {}", tab_id);

        // Remove from tab order
        {
            let mut order = self.tab_order.write();
            order.retain(|id| id != tab_id);
        }

        // Remove tab
        {
            let mut tabs = self.tabs.write();
            tabs.remove(tab_id);
        }

        // If closed the active tab, switch to another
        {
            let mut active = self.active_tab_id.write();
            if active.as_ref() == Some(tab_id) {
                let order = self.tab_order.read();
                *active = order.first().cloned();
            }
        }

        // Show the new active tab
        self.show_active_tab();

        Ok(())
    }

    /// Activate a tab
    pub fn activate_tab(&self, tab_id: &TabId) -> crate::Result<()> {
        let tabs = self.tabs.read();
        if !tabs.contains_key(tab_id) {
            return Err(BrowserError::TabNotFound(tab_id.clone()));
        }
        drop(tabs);

        tracing::info!("[TabManager] Activating tab: {}", tab_id);

        {
            let mut active = self.active_tab_id.write();
            *active = Some(tab_id.clone());
        }

        self.show_active_tab();
        Ok(())
    }

    /// Show active tab, hide others
    pub fn show_active_tab(&self) {
        let active_id = self.active_tab_id.read().clone();
        let tabs = self.tabs.read();

        for (id, tab) in tabs.iter() {
            let is_active = active_id.as_ref() == Some(id);
            let _ = tab.set_visible(is_active);
        }
    }

    /// Navigate active tab to URL
    pub fn navigate(&self, url: &str) -> crate::Result<()> {
        let active_id = self.active_tab_id.read().clone();
        if let Some(tab_id) = active_id {
            let mut tabs = self.tabs.write();
            if let Some(tab) = tabs.get_mut(&tab_id) {
                // Normalize URL
                let url = Self::normalize_url(url);
                tracing::info!("[TabManager] Navigating to: {}", url);
                return tab.navigate(&url);
            }
        }
        Ok(())
    }

    /// Normalize a URL (add https:// if needed, or search if not a URL)
    fn normalize_url(url: &str) -> String {
        if url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("file://")
            || url.starts_with("about:")
        {
            url.to_string()
        } else if url.contains('.') && !url.contains(' ') {
            format!("https://{}", url)
        } else {
            format!(
                "https://www.google.com/search?q={}",
                urlencoding_encode(url)
            )
        }
    }

    /// Go back in active tab
    pub fn go_back(&self) -> crate::Result<()> {
        let active_id = self.active_tab_id.read().clone();
        if let Some(tab_id) = active_id {
            let tabs = self.tabs.read();
            if let Some(tab) = tabs.get(&tab_id) {
                return tab.go_back();
            }
        }
        Ok(())
    }

    /// Go forward in active tab
    pub fn go_forward(&self) -> crate::Result<()> {
        let active_id = self.active_tab_id.read().clone();
        if let Some(tab_id) = active_id {
            let tabs = self.tabs.read();
            if let Some(tab) = tabs.get(&tab_id) {
                return tab.go_forward();
            }
        }
        Ok(())
    }

    /// Reload active tab
    pub fn reload(&self) -> crate::Result<()> {
        let active_id = self.active_tab_id.read().clone();
        if let Some(tab_id) = active_id {
            let tabs = self.tabs.read();
            if let Some(tab) = tabs.get(&tab_id) {
                return tab.reload();
            }
        }
        Ok(())
    }

    /// Stop loading active tab
    pub fn stop(&self) -> crate::Result<()> {
        let active_id = self.active_tab_id.read().clone();
        if let Some(tab_id) = active_id {
            let tabs = self.tabs.read();
            if let Some(tab) = tabs.get(&tab_id) {
                return tab.stop();
            }
        }
        Ok(())
    }

    /// Navigate to home page
    pub fn home(&self) -> crate::Result<()> {
        self.navigate(&self.config.home_url.clone())
    }

    /// Resize all tab WebViews
    pub fn resize_tabs(&self, x: i32, y: i32, width: u32, height: u32) {
        let tabs = self.tabs.read();
        for tab in tabs.values() {
            #[cfg(target_os = "windows")]
            {
                let _ = tab.webview.set_bounds(wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                        x as f64, y as f64,
                    )),
                    size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(width, height)),
                });
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = (x, y, width, height); // Suppress unused warnings
            }
        }
    }

    /// Update tab title
    pub fn update_title(&self, tab_id: &TabId, title: String) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_title(title);
        }
    }

    /// Update tab URL
    pub fn update_url(&self, tab_id: &TabId, url: String) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_url(url);
        }
    }

    /// Update tab loading state
    pub fn update_loading(&self, tab_id: &TabId, is_loading: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_loading(is_loading);
        }
    }

    /// Update tab history state
    pub fn update_history(&self, tab_id: &TabId, can_go_back: bool, can_go_forward: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut()
                .set_history_state(can_go_back, can_go_forward);
        }
    }

    /// Update tab favicon
    pub fn update_favicon(&self, tab_id: &TabId, favicon: Option<String>) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_favicon(favicon);
        }
    }

    /// Pin/unpin a tab
    pub fn set_pinned(&self, tab_id: &TabId, pinned: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_pinned(pinned);
        }
    }

    /// Mute/unmute a tab
    pub fn set_muted(&self, tab_id: &TabId, muted: bool) {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state_mut().set_muted(muted);
        }
    }

    /// Reorder tabs
    pub fn reorder_tab(&self, tab_id: &TabId, new_index: usize) {
        let mut order = self.tab_order.write();
        if let Some(old_index) = order.iter().position(|id| id == tab_id) {
            let id = order.remove(old_index);
            let new_index = new_index.min(order.len());
            order.insert(new_index, id);
        }
    }

    /// Duplicate a tab
    pub fn duplicate_tab(
        &self,
        window: &Window,
        tab_id: &TabId,
        header_height: u32,
    ) -> crate::Result<TabId> {
        let url = {
            let tabs = self.tabs.read();
            tabs.get(tab_id)
                .map(|t| t.state.url.clone())
                .ok_or_else(|| BrowserError::TabNotFound(tab_id.clone()))?
        };

        self.create_tab(window, Some(&url), header_height)
    }

    /// Get active tab state
    pub fn active_tab_state(&self) -> Option<TabState> {
        let active_id = self.active_tab_id.read().clone()?;
        let tabs = self.tabs.read();
        tabs.get(&active_id).map(|t| t.state.clone())
    }

    /// Check if any tabs exist
    pub fn is_empty(&self) -> bool {
        self.tabs.read().is_empty()
    }

    // === DevTools Methods ===

    /// Toggle DevTools for a tab
    /// Note: DevTools API requires wry's devtools feature and platform support
    pub fn toggle_devtools(&self, tab_id: &TabId) {
        let tabs = self.tabs.read();
        if let Some(_tab) = tabs.get(tab_id) {
            // TODO: Implement using CDP or WebView2 native DevTools API
            tracing::debug!(
                "[TabManagerNested] DevTools toggle requested for tab {}",
                tab_id
            );
        }
    }

    /// Open DevTools for a tab
    pub fn open_devtools(&self, tab_id: &TabId) {
        let tabs = self.tabs.read();
        if let Some(_tab) = tabs.get(tab_id) {
            // TODO: Implement using CDP or WebView2 native DevTools API
            tracing::debug!(
                "[TabManagerNested] DevTools open requested for tab {}",
                tab_id
            );
        }
    }

    /// Close DevTools for a tab
    pub fn close_devtools(&self, tab_id: &TabId) {
        let tabs = self.tabs.read();
        if let Some(_tab) = tabs.get(tab_id) {
            // TODO: Implement using CDP or WebView2 native DevTools API
            tracing::debug!(
                "[TabManagerNested] DevTools close requested for tab {}",
                tab_id
            );
        }
    }

    /// Check if DevTools is open for a tab
    pub fn is_devtools_open(&self, _tab_id: &TabId) -> bool {
        // TODO: Track DevTools state
        false
    }
}

/// Simple URL encoding function
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}
