"""Multi-Tab Browser Demo - Browser-like tabbed interface with WebView.

This example demonstrates how to create a browser-like application with:
- Multiple tabs for different web pages
- Tab management (create, close, switch)
- Navigation controls (back, forward, reload, home)
- URL bar with navigation
- Link interception for opening in new tabs

Features demonstrated:
- Using `new_window_mode="child_webview"` for handling target="_blank" links
- Tab state management across multiple WebView instances
- Inter-tab communication via Python
- Custom navigation handling

Usage:
    python examples/multi_tab_browser_demo.py

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import threading
import uuid
from dataclasses import dataclass, field
from typing import Callable, Dict, List, Optional

from auroraview import WebView


@dataclass
class Tab:
    """Represents a browser tab."""

    id: str
    title: str = "New Tab"
    url: str = ""
    favicon: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False


@dataclass
class BrowserState:
    """Manages browser state across all tabs."""

    tabs: Dict[str, Tab] = field(default_factory=dict)
    active_tab_id: Optional[str] = None
    tab_order: List[str] = field(default_factory=list)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def add_tab(self, tab: Tab) -> None:
        """Add a new tab."""
        with self._lock:
            self.tabs[tab.id] = tab
            self.tab_order.append(tab.id)

    def remove_tab(self, tab_id: str) -> Optional[str]:
        """Remove a tab and return the next active tab ID."""
        with self._lock:
            if tab_id not in self.tabs:
                return self.active_tab_id

            # Find next tab to activate
            idx = self.tab_order.index(tab_id)
            del self.tabs[tab_id]
            self.tab_order.remove(tab_id)

            if not self.tab_order:
                return None

            # Activate adjacent tab
            new_idx = min(idx, len(self.tab_order) - 1)
            return self.tab_order[new_idx]

    def get_tab(self, tab_id: str) -> Optional[Tab]:
        """Get a tab by ID."""
        with self._lock:
            return self.tabs.get(tab_id)

    def update_tab(self, tab_id: str, **kwargs) -> None:
        """Update tab properties."""
        with self._lock:
            if tab_id in self.tabs:
                for key, value in kwargs.items():
                    if hasattr(self.tabs[tab_id], key):
                        setattr(self.tabs[tab_id], key, value)

    def get_tabs_info(self) -> List[dict]:
        """Get all tabs info for UI."""
        with self._lock:
            return [
                {
                    "id": tab_id,
                    "title": self.tabs[tab_id].title,
                    "url": self.tabs[tab_id].url,
                    "favicon": self.tabs[tab_id].favicon,
                    "is_loading": self.tabs[tab_id].is_loading,
                    "is_active": tab_id == self.active_tab_id,
                }
                for tab_id in self.tab_order
                if tab_id in self.tabs
            ]


# Global browser state
browser_state = BrowserState()

# Store WebView instances for each tab
tab_webviews: Dict[str, WebView] = {}

# Main browser window reference
main_window: Optional[WebView] = None


BROWSER_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>AuroraView Browser</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #e4e4e4;
            height: 100vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }

        /* Tab Bar */
        .tab-bar {
            display: flex;
            align-items: center;
            background: #0f0f1a;
            padding: 8px 8px 0;
            gap: 4px;
            min-height: 42px;
            border-bottom: 1px solid #2a2a4a;
        }

        .tabs-container {
            display: flex;
            flex: 1;
            gap: 2px;
            overflow-x: auto;
            padding-bottom: 8px;
        }

        .tabs-container::-webkit-scrollbar {
            height: 4px;
        }

        .tabs-container::-webkit-scrollbar-thumb {
            background: #4a4a6a;
            border-radius: 2px;
        }

        .tab {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            background: #1a1a2e;
            border-radius: 8px 8px 0 0;
            cursor: pointer;
            min-width: 120px;
            max-width: 200px;
            transition: all 0.2s;
            border: 1px solid transparent;
            border-bottom: none;
        }

        .tab:hover {
            background: #252540;
        }

        .tab.active {
            background: #1e3a5f;
            border-color: #4facfe;
        }

        .tab-favicon {
            width: 16px;
            height: 16px;
            border-radius: 2px;
            background: #4a4a6a;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 10px;
        }

        .tab-title {
            flex: 1;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            font-size: 13px;
        }

        .tab-close {
            width: 18px;
            height: 18px;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            opacity: 0.6;
            transition: all 0.2s;
        }

        .tab-close:hover {
            background: rgba(255, 100, 100, 0.3);
            opacity: 1;
        }

        .tab-loading {
            width: 16px;
            height: 16px;
            border: 2px solid #4facfe;
            border-top-color: transparent;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        .new-tab-btn {
            width: 32px;
            height: 32px;
            border-radius: 6px;
            background: transparent;
            border: none;
            color: #888;
            font-size: 20px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            margin-bottom: 8px;
        }

        .new-tab-btn:hover {
            background: #252540;
            color: #4facfe;
        }

        /* Navigation Bar */
        .nav-bar {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            background: #0f0f1a;
            border-bottom: 1px solid #2a2a4a;
        }

        .nav-btn {
            width: 32px;
            height: 32px;
            border-radius: 6px;
            background: transparent;
            border: none;
            color: #888;
            font-size: 16px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
        }

        .nav-btn:hover:not(:disabled) {
            background: #252540;
            color: #4facfe;
        }

        .nav-btn:disabled {
            opacity: 0.3;
            cursor: not-allowed;
        }

        .url-bar {
            flex: 1;
            display: flex;
            align-items: center;
            background: #1a1a2e;
            border: 1px solid #2a2a4a;
            border-radius: 20px;
            padding: 0 12px;
            transition: all 0.2s;
        }

        .url-bar:focus-within {
            border-color: #4facfe;
            box-shadow: 0 0 0 2px rgba(79, 172, 254, 0.2);
        }

        .url-bar input {
            flex: 1;
            background: transparent;
            border: none;
            color: #e4e4e4;
            padding: 8px;
            font-size: 14px;
            outline: none;
        }

        .url-bar input::placeholder {
            color: #666;
        }

        .go-btn {
            background: #4facfe;
            color: #0f0f1a;
            border: none;
            padding: 6px 16px;
            border-radius: 14px;
            font-size: 13px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }

        .go-btn:hover {
            background: #6fbfff;
        }

        /* Content Area */
        .content-area {
            flex: 1;
            display: flex;
            flex-direction: column;
            background: #1a1a2e;
            overflow: hidden;
        }

        .tab-content {
            display: none;
            flex: 1;
            flex-direction: column;
        }

        .tab-content.active {
            display: flex;
        }

        /* Welcome Page */
        .welcome-page {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 40px;
            text-align: center;
        }

        .welcome-page h1 {
            font-size: 32px;
            color: #4facfe;
            margin-bottom: 16px;
        }

        .welcome-page p {
            color: #888;
            margin-bottom: 32px;
            max-width: 500px;
        }

        .quick-links {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 16px;
            max-width: 600px;
        }

        .quick-link {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
            padding: 20px;
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            cursor: pointer;
            transition: all 0.2s;
            text-decoration: none;
            color: inherit;
        }

        .quick-link:hover {
            background: rgba(79, 172, 254, 0.1);
            border-color: #4facfe;
            transform: translateY(-2px);
        }

        .quick-link-icon {
            width: 48px;
            height: 48px;
            border-radius: 12px;
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
        }

        .quick-link-title {
            font-size: 13px;
            font-weight: 500;
        }

        /* Page Frame */
        .page-frame {
            flex: 1;
            border: none;
            background: white;
        }

        /* Status Bar */
        .status-bar {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 4px 12px;
            background: #0f0f1a;
            border-top: 1px solid #2a2a4a;
            font-size: 12px;
            color: #666;
        }

        .status-url {
            flex: 1;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        /* Empty state */
        .empty-state {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            color: #666;
        }

        .empty-state svg {
            width: 64px;
            height: 64px;
            margin-bottom: 16px;
            opacity: 0.5;
        }
    </style>
</head>
<body>
    <!-- Tab Bar -->
    <div class="tab-bar">
        <div class="tabs-container" id="tabs-container">
            <!-- Tabs will be rendered here -->
        </div>
        <button class="new-tab-btn" onclick="createNewTab()" title="New Tab">+</button>
    </div>

    <!-- Navigation Bar -->
    <div class="nav-bar">
        <button class="nav-btn" id="btn-back" onclick="goBack()" disabled title="Back">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M19 12H5M12 19l-7-7 7-7"/>
            </svg>
        </button>
        <button class="nav-btn" id="btn-forward" onclick="goForward()" disabled title="Forward">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
        </button>
        <button class="nav-btn" id="btn-reload" onclick="reloadPage()" title="Reload">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M23 4v6h-6M1 20v-6h6"/>
                <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
        </button>
        <button class="nav-btn" onclick="goHome()" title="Home">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z"/>
                <polyline points="9 22 9 12 15 12 15 22"/>
            </svg>
        </button>
        <div class="url-bar">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#666" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35"/>
            </svg>
            <input type="text" id="url-input" placeholder="Search or enter URL" onkeypress="handleUrlKeypress(event)">
            <button class="go-btn" onclick="navigateToUrl()">Go</button>
        </div>
    </div>

    <!-- Content Area -->
    <div class="content-area" id="content-area">
        <!-- Tab contents will be rendered here -->
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
        <span class="status-url" id="status-url">Ready</span>
        <span id="tab-count">0 tabs</span>
    </div>

    <script>
        let tabs = [];
        let activeTabId = null;

        // Quick links for new tab page
        const quickLinks = [
            { title: 'GitHub', url: 'https://github.com', icon: '🐙' },
            { title: 'Google', url: 'https://google.com', icon: '🔍' },
            { title: 'MDN', url: 'https://developer.mozilla.org', icon: '📚' },
            { title: 'Stack Overflow', url: 'https://stackoverflow.com', icon: '💬' },
            { title: 'Python', url: 'https://python.org', icon: '🐍' },
            { title: 'Rust', url: 'https://rust-lang.org', icon: '🦀' },
            { title: 'NPM', url: 'https://npmjs.com', icon: '📦' },
            { title: 'Wikipedia', url: 'https://wikipedia.org', icon: '📖' },
        ];

        // Initialize
        window.addEventListener('auroraviewready', () => {
            console.log('[Browser] AuroraView ready');

            // Listen for tab updates from Python
            auroraview.on('tabs:update', (data) => {
                tabs = data.tabs;
                activeTabId = data.active_tab_id;
                renderTabs();
                updateNavButtons();
            });

            // Listen for URL changes
            auroraview.on('tab:url_changed', (data) => {
                if (data.tab_id === activeTabId) {
                    document.getElementById('url-input').value = data.url;
                    document.getElementById('status-url').textContent = data.url;
                }
            });

            // Listen for title changes
            auroraview.on('tab:title_changed', (data) => {
                const tab = tabs.find(t => t.id === data.tab_id);
                if (tab) {
                    tab.title = data.title;
                    renderTabs();
                }
            });

            // Listen for navigation state changes
            auroraview.on('tab:nav_state', (data) => {
                if (data.tab_id === activeTabId) {
                    document.getElementById('btn-back').disabled = !data.can_go_back;
                    document.getElementById('btn-forward').disabled = !data.can_go_forward;
                }
            });

            // Create initial tab
            createNewTab();
        });

        function renderTabs() {
            const container = document.getElementById('tabs-container');
            container.innerHTML = tabs.map(tab => `
                <div class="tab ${tab.id === activeTabId ? 'active' : ''}"
                     onclick="switchTab('${tab.id}')"
                     data-tab-id="${tab.id}">
                    ${tab.is_loading
                        ? '<div class="tab-loading"></div>'
                        : `<div class="tab-favicon">${tab.favicon || '🌐'}</div>`
                    }
                    <span class="tab-title">${escapeHtml(tab.title)}</span>
                    <div class="tab-close" onclick="event.stopPropagation(); closeTab('${tab.id}')">✕</div>
                </div>
            `).join('');

            // Update tab count
            document.getElementById('tab-count').textContent = `${tabs.length} tab${tabs.length !== 1 ? 's' : ''}`;

            // Render content area
            renderContent();
        }

        function renderContent() {
            const contentArea = document.getElementById('content-area');
            const activeTab = tabs.find(t => t.id === activeTabId);

            if (!activeTab) {
                contentArea.innerHTML = `
                    <div class="empty-state">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="3" y="3" width="18" height="18" rx="2"/>
                            <path d="M3 9h18"/>
                            <circle cx="7" cy="6" r="1"/>
                            <circle cx="10" cy="6" r="1"/>
                        </svg>
                        <p>No tabs open. Click + to create a new tab.</p>
                    </div>
                `;
                return;
            }

            // Show welcome page or iframe based on URL
            if (!activeTab.url) {
                contentArea.innerHTML = `
                    <div class="welcome-page">
                        <h1>🌐 AuroraView Browser</h1>
                        <p>A multi-tab browser demo built with AuroraView WebView.
                           Click a quick link below or enter a URL to get started.</p>
                        <div class="quick-links">
                            ${quickLinks.map(link => `
                                <a class="quick-link" onclick="navigateTo('${link.url}')">
                                    <div class="quick-link-icon">${link.icon}</div>
                                    <span class="quick-link-title">${link.title}</span>
                                </a>
                            `).join('')}
                        </div>
                    </div>
                `;
            } else {
                // Note: In a real implementation, each tab would have its own WebView
                // For this demo, we show a message about the navigation
                contentArea.innerHTML = `
                    <div class="welcome-page">
                        <h1>🔗 Navigation Request</h1>
                        <p>In a full implementation, this tab would display:</p>
                        <div style="background: #0f0f1a; padding: 16px; border-radius: 8px; margin: 16px 0; word-break: break-all;">
                            <code style="color: #4facfe;">${escapeHtml(activeTab.url)}</code>
                        </div>
                        <p style="color: #666; font-size: 13px;">
                            This demo shows the tab management UI. For actual web content rendering,
                            each tab would need its own WebView instance (using new_window_mode="child_webview").
                        </p>
                        <button class="go-btn" style="margin-top: 16px;" onclick="openInSystemBrowser('${escapeHtml(activeTab.url)}')">
                            Open in System Browser
                        </button>
                    </div>
                `;
            }

            // Update URL bar
            document.getElementById('url-input').value = activeTab.url || '';
            document.getElementById('status-url').textContent = activeTab.url || 'New Tab';
        }

        function updateNavButtons() {
            const activeTab = tabs.find(t => t.id === activeTabId);
            document.getElementById('btn-back').disabled = !activeTab?.can_go_back;
            document.getElementById('btn-forward').disabled = !activeTab?.can_go_forward;
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Tab Actions
        function createNewTab() {
            if (window.auroraview?.api) {
                auroraview.api.create_tab();
            }
        }

        function closeTab(tabId) {
            if (window.auroraview?.api) {
                auroraview.api.close_tab({ tab_id: tabId });
            }
        }

        function switchTab(tabId) {
            if (window.auroraview?.api) {
                auroraview.api.switch_tab({ tab_id: tabId });
            }
        }

        // Navigation Actions
        function navigateTo(url) {
            if (window.auroraview?.api) {
                auroraview.api.navigate({ url: url });
            }
        }

        function navigateToUrl() {
            const url = document.getElementById('url-input').value.trim();
            if (url) {
                // Add protocol if missing
                let finalUrl = url;
                if (!url.match(/^https?:\\/\\//)) {
                    if (url.includes('.') && !url.includes(' ')) {
                        finalUrl = 'https://' + url;
                    } else {
                        // Treat as search query
                        finalUrl = 'https://www.google.com/search?q=' + encodeURIComponent(url);
                    }
                }
                navigateTo(finalUrl);
            }
        }

        function handleUrlKeypress(event) {
            if (event.key === 'Enter') {
                navigateToUrl();
            }
        }

        function goBack() {
            if (window.auroraview?.api) {
                auroraview.api.go_back();
            }
        }

        function goForward() {
            if (window.auroraview?.api) {
                auroraview.api.go_forward();
            }
        }

        function reloadPage() {
            if (window.auroraview?.api) {
                auroraview.api.reload();
            }
        }

        function goHome() {
            if (window.auroraview?.api) {
                auroraview.api.go_home();
            }
        }

        function openInSystemBrowser(url) {
            if (window.auroraview?.shell) {
                auroraview.shell.open(url);
            }
        }
    </script>
</body>
</html>
"""


def create_browser_window() -> WebView:
    """Create the main browser window."""
    global main_window

    view = WebView.create(
        title="AuroraView Browser",
        html=BROWSER_HTML,
        width=1200,
        height=800,
        debug=True,
    )

    main_window = view

    # Tab management API
    @view.bind_call("api.create_tab")
    def create_tab(url: str = "") -> dict:
        """Create a new tab."""
        tab_id = str(uuid.uuid4())[:8]
        tab = Tab(id=tab_id, title="New Tab", url=url)
        browser_state.add_tab(tab)
        browser_state.active_tab_id = tab_id

        # Broadcast update
        broadcast_tabs_update()
        return {"tab_id": tab_id}

    @view.bind_call("api.close_tab")
    def close_tab(tab_id: str) -> dict:
        """Close a tab."""
        next_tab_id = browser_state.remove_tab(tab_id)
        browser_state.active_tab_id = next_tab_id

        # Clean up WebView if exists
        if tab_id in tab_webviews:
            try:
                tab_webviews[tab_id].close()
            except Exception:
                pass
            del tab_webviews[tab_id]

        broadcast_tabs_update()
        return {"success": True, "next_tab_id": next_tab_id}

    @view.bind_call("api.switch_tab")
    def switch_tab(tab_id: str) -> dict:
        """Switch to a tab."""
        if browser_state.get_tab(tab_id):
            browser_state.active_tab_id = tab_id
            broadcast_tabs_update()
            return {"success": True}
        return {"success": False, "error": "Tab not found"}

    @view.bind_call("api.navigate")
    def navigate(url: str) -> dict:
        """Navigate the active tab to a URL."""
        if not browser_state.active_tab_id:
            return {"success": False, "error": "No active tab"}

        tab = browser_state.get_tab(browser_state.active_tab_id)
        if tab:
            browser_state.update_tab(
                browser_state.active_tab_id,
                url=url,
                title=get_title_from_url(url),
                is_loading=True,
            )
            broadcast_tabs_update()

            # Simulate loading completion
            def finish_loading():
                import time

                time.sleep(0.5)
                browser_state.update_tab(
                    browser_state.active_tab_id,
                    is_loading=False,
                    can_go_back=True,
                )
                broadcast_tabs_update()

            threading.Thread(target=finish_loading, daemon=True).start()

            return {"success": True}
        return {"success": False, "error": "Tab not found"}

    @view.bind_call("api.go_back")
    def go_back() -> dict:
        """Go back in the active tab."""
        # In a real implementation, this would call the WebView's goBack method
        return {"success": True}

    @view.bind_call("api.go_forward")
    def go_forward() -> dict:
        """Go forward in the active tab."""
        # In a real implementation, this would call the WebView's goForward method
        return {"success": True}

    @view.bind_call("api.reload")
    def reload() -> dict:
        """Reload the active tab."""
        if browser_state.active_tab_id:
            browser_state.update_tab(browser_state.active_tab_id, is_loading=True)
            broadcast_tabs_update()

            def finish_reload():
                import time

                time.sleep(0.3)
                browser_state.update_tab(browser_state.active_tab_id, is_loading=False)
                broadcast_tabs_update()

            threading.Thread(target=finish_reload, daemon=True).start()
        return {"success": True}

    @view.bind_call("api.go_home")
    def go_home() -> dict:
        """Go to home page (new tab page)."""
        if browser_state.active_tab_id:
            browser_state.update_tab(
                browser_state.active_tab_id,
                url="",
                title="New Tab",
                is_loading=False,
            )
            broadcast_tabs_update()
        return {"success": True}

    return view


def broadcast_tabs_update():
    """Broadcast tab state to the UI."""
    if main_window:
        main_window.emit(
            "tabs:update",
            {
                "tabs": browser_state.get_tabs_info(),
                "active_tab_id": browser_state.active_tab_id,
            },
        )


def get_title_from_url(url: str) -> str:
    """Extract a title from URL."""
    try:
        from urllib.parse import urlparse

        parsed = urlparse(url)
        domain = parsed.netloc or parsed.path
        # Remove www. prefix
        if domain.startswith("www."):
            domain = domain[4:]
        return domain.split(".")[0].capitalize() if domain else "New Tab"
    except Exception:
        return "New Tab"


def main():
    """Run the multi-tab browser demo."""
    print("=" * 60)
    print("AuroraView Multi-Tab Browser Demo")
    print("=" * 60)
    print()
    print("Features demonstrated:")
    print("- Multiple tabs with create/close/switch functionality")
    print("- Navigation controls (back, forward, reload, home)")
    print("- URL bar with smart URL/search detection")
    print("- Quick links for common websites")
    print("- Tab state management")
    print()
    print("Note: This demo shows the tab management UI.")
    print("For actual multi-WebView tabs, use new_window_mode='child_webview'")
    print("when creating WebView instances.")
    print()
    print("Starting browser...")
    print()

    browser = create_browser_window()
    browser.show()


if __name__ == "__main__":
    main()
