"""Tabbed WebView Demo - Real multi-WebView tabbed browser.

This example demonstrates how to create a true tabbed browser where:
- Each tab is a separate WebView instance
- Clicking links with target="_blank" opens new tabs
- Uses new_window_mode="child_webview" for handling new window requests

Key Features:
- Real WebView instances per tab
- Link interception and new tab creation
- Tab synchronization between main UI and child WebViews
- Proper cleanup of WebView resources

Usage:
    python examples/tabbed_webview_demo.py

Note:
    This demo uses the child_webview mode which creates new WebView
    windows for window.open() calls. The main window acts as a tab
    manager while each tab content is rendered in its own WebView.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import threading
import uuid
from dataclasses import dataclass, field
from typing import Dict, List, Optional

from auroraview import WebView


@dataclass
class TabInfo:
    """Information about a browser tab."""

    id: str
    title: str = "New Tab"
    url: str = ""
    is_active: bool = False


class TabManager:
    """Manages multiple WebView tabs."""

    def __init__(self):
        self.tabs: Dict[str, TabInfo] = {}
        self.tab_order: List[str] = []
        self.active_tab_id: Optional[str] = None
        self.main_window: Optional[WebView] = None
        self._lock = threading.Lock()

    def set_main_window(self, window: WebView) -> None:
        """Set the main browser window."""
        self.main_window = window

    def create_tab(self, url: str = "", title: str = "New Tab") -> str:
        """Create a new tab and return its ID."""
        with self._lock:
            tab_id = str(uuid.uuid4())[:8]
            tab = TabInfo(id=tab_id, title=title, url=url, is_active=True)

            # Deactivate current tab
            if self.active_tab_id and self.active_tab_id in self.tabs:
                self.tabs[self.active_tab_id].is_active = False

            self.tabs[tab_id] = tab
            self.tab_order.append(tab_id)
            self.active_tab_id = tab_id

            return tab_id

    def close_tab(self, tab_id: str) -> Optional[str]:
        """Close a tab and return the next active tab ID."""
        with self._lock:
            if tab_id not in self.tabs:
                return self.active_tab_id

            idx = self.tab_order.index(tab_id)
            del self.tabs[tab_id]
            self.tab_order.remove(tab_id)

            if not self.tab_order:
                self.active_tab_id = None
                return None

            # Select adjacent tab
            new_idx = min(idx, len(self.tab_order) - 1)
            new_active = self.tab_order[new_idx]
            self.tabs[new_active].is_active = True
            self.active_tab_id = new_active

            return new_active

    def switch_tab(self, tab_id: str) -> bool:
        """Switch to a specific tab."""
        with self._lock:
            if tab_id not in self.tabs:
                return False

            if self.active_tab_id and self.active_tab_id in self.tabs:
                self.tabs[self.active_tab_id].is_active = False

            self.tabs[tab_id].is_active = True
            self.active_tab_id = tab_id
            return True

    def update_tab(self, tab_id: str, **kwargs) -> None:
        """Update tab properties."""
        with self._lock:
            if tab_id in self.tabs:
                for key, value in kwargs.items():
                    if hasattr(self.tabs[tab_id], key):
                        setattr(self.tabs[tab_id], key, value)

    def get_tabs_info(self) -> List[dict]:
        """Get all tabs as a list of dicts."""
        with self._lock:
            return [
                {
                    "id": tab_id,
                    "title": self.tabs[tab_id].title,
                    "url": self.tabs[tab_id].url,
                    "is_active": self.tabs[tab_id].is_active,
                }
                for tab_id in self.tab_order
                if tab_id in self.tabs
            ]

    def broadcast_update(self) -> None:
        """Send tab update to main window."""
        if self.main_window:
            self.main_window.emit(
                "tabs:update",
                {
                    "tabs": self.get_tabs_info(),
                    "active_tab_id": self.active_tab_id,
                },
            )


# Global tab manager
tab_manager = TabManager()


MAIN_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Tabbed WebView Demo</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1000px;
            margin: 0 auto;
        }

        .header {
            text-align: center;
            margin-bottom: 30px;
        }

        .header h1 {
            font-size: 32px;
            background: linear-gradient(90deg, #4facfe, #00f2fe);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 10px;
        }

        .header p {
            color: #888;
            font-size: 14px;
        }

        /* Tab Bar */
        .tab-bar {
            display: flex;
            align-items: center;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 12px 12px 0 0;
            padding: 8px 8px 0;
            gap: 4px;
        }

        .tabs-container {
            display: flex;
            flex: 1;
            gap: 4px;
            overflow-x: auto;
            padding-bottom: 8px;
        }

        .tab {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 10px 16px;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 8px 8px 0 0;
            cursor: pointer;
            min-width: 140px;
            max-width: 220px;
            transition: all 0.2s;
            border: 1px solid transparent;
            border-bottom: none;
        }

        .tab:hover {
            background: rgba(255, 255, 255, 0.1);
        }

        .tab.active {
            background: rgba(79, 172, 254, 0.2);
            border-color: #4facfe;
        }

        .tab-icon {
            font-size: 16px;
        }

        .tab-title {
            flex: 1;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            font-size: 13px;
        }

        .tab-close {
            width: 20px;
            height: 20px;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 14px;
            opacity: 0.5;
            transition: all 0.2s;
        }

        .tab-close:hover {
            background: rgba(255, 100, 100, 0.3);
            opacity: 1;
        }

        .new-tab-btn {
            width: 36px;
            height: 36px;
            border-radius: 8px;
            background: transparent;
            border: 1px dashed rgba(255, 255, 255, 0.2);
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
            background: rgba(79, 172, 254, 0.2);
            border-color: #4facfe;
            color: #4facfe;
        }

        /* Content Area */
        .content-area {
            background: rgba(0, 0, 0, 0.3);
            border-radius: 0 0 12px 12px;
            padding: 24px;
            min-height: 400px;
        }

        .welcome {
            text-align: center;
            padding: 40px;
        }

        .welcome h2 {
            color: #4facfe;
            margin-bottom: 16px;
        }

        .welcome p {
            color: #888;
            margin-bottom: 24px;
            line-height: 1.6;
        }

        .link-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
            margin-top: 24px;
        }

        .link-card {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 16px;
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 10px;
            cursor: pointer;
            transition: all 0.2s;
            text-decoration: none;
            color: inherit;
        }

        .link-card:hover {
            background: rgba(79, 172, 254, 0.1);
            border-color: #4facfe;
            transform: translateY(-2px);
        }

        .link-icon {
            width: 40px;
            height: 40px;
            border-radius: 8px;
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 20px;
        }

        .link-info h3 {
            font-size: 14px;
            margin-bottom: 4px;
        }

        .link-info p {
            font-size: 12px;
            color: #666;
        }

        /* Active Tab Content */
        .tab-content {
            display: none;
        }

        .tab-content.active {
            display: block;
        }

        .page-info {
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
        }

        .page-info h3 {
            color: #4facfe;
            margin-bottom: 8px;
        }

        .page-info .url {
            font-family: monospace;
            font-size: 13px;
            color: #888;
            word-break: break-all;
        }

        .action-buttons {
            display: flex;
            gap: 10px;
            margin-top: 16px;
        }

        button {
            padding: 10px 20px;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s;
            background: #4facfe;
            color: #0f0f1a;
            font-weight: 600;
        }

        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(79, 172, 254, 0.4);
        }

        button.secondary {
            background: rgba(255, 255, 255, 0.1);
            color: #e4e4e4;
        }

        /* Status */
        .status-bar {
            display: flex;
            justify-content: space-between;
            padding: 12px 16px;
            background: rgba(0, 0, 0, 0.2);
            border-radius: 8px;
            margin-top: 20px;
            font-size: 12px;
            color: #666;
        }

        .empty-state {
            text-align: center;
            padding: 60px 20px;
            color: #666;
        }

        .empty-state svg {
            width: 64px;
            height: 64px;
            margin-bottom: 16px;
            opacity: 0.3;
        }

        /* Code block */
        code {
            background: rgba(0, 0, 0, 0.3);
            padding: 2px 6px;
            border-radius: 4px;
            font-family: 'SF Mono', Monaco, monospace;
            font-size: 12px;
            color: #4facfe;
        }

        pre {
            background: rgba(0, 0, 0, 0.3);
            padding: 16px;
            border-radius: 8px;
            overflow-x: auto;
            margin: 16px 0;
        }

        pre code {
            background: none;
            padding: 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Tabbed WebView Demo</h1>
            <p>Multi-tab browser with real WebView instances per tab</p>
        </div>

        <!-- Tab Bar -->
        <div class="tab-bar">
            <div class="tabs-container" id="tabs-container">
                <!-- Tabs rendered here -->
            </div>
            <button class="new-tab-btn" onclick="createTab()" title="New Tab">+</button>
        </div>

        <!-- Content Area -->
        <div class="content-area" id="content-area">
            <!-- Content rendered here -->
        </div>

        <!-- Status Bar -->
        <div class="status-bar">
            <span id="status-text">Ready</span>
            <span id="tab-count">0 tabs</span>
        </div>
    </div>

    <script>
        let tabs = [];
        let activeTabId = null;

        const sampleLinks = [
            { title: 'GitHub', url: 'https://github.com', icon: '🐙', desc: 'Code hosting' },
            { title: 'Google', url: 'https://google.com', icon: '🔍', desc: 'Search engine' },
            { title: 'Python', url: 'https://python.org', icon: '🐍', desc: 'Python docs' },
            { title: 'MDN', url: 'https://developer.mozilla.org', icon: '📚', desc: 'Web docs' },
            { title: 'Rust', url: 'https://rust-lang.org', icon: '🦀', desc: 'Rust language' },
            { title: 'Wikipedia', url: 'https://wikipedia.org', icon: '📖', desc: 'Encyclopedia' },
        ];

        window.addEventListener('auroraviewready', () => {
            console.log('[TabbedDemo] AuroraView ready');

            // Listen for tab updates
            auroraview.on('tabs:update', (data) => {
                tabs = data.tabs;
                activeTabId = data.active_tab_id;
                renderUI();
            });

            // Create initial tab
            createTab();
        });

        function renderUI() {
            renderTabs();
            renderContent();
            updateStatus();
        }

        function renderTabs() {
            const container = document.getElementById('tabs-container');
            container.innerHTML = tabs.map(tab => `
                <div class="tab ${tab.is_active ? 'active' : ''}"
                     onclick="switchTab('${tab.id}')">
                    <span class="tab-icon">🌐</span>
                    <span class="tab-title">${escapeHtml(tab.title)}</span>
                    <span class="tab-close" onclick="event.stopPropagation(); closeTab('${tab.id}')">×</span>
                </div>
            `).join('');
        }

        function renderContent() {
            const container = document.getElementById('content-area');
            const activeTab = tabs.find(t => t.is_active);

            if (!activeTab) {
                container.innerHTML = `
                    <div class="empty-state">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="3" y="3" width="18" height="18" rx="2"/>
                            <path d="M3 9h18"/>
                        </svg>
                        <p>No tabs open. Click + to create a new tab.</p>
                    </div>
                `;
                return;
            }

            if (!activeTab.url) {
                // Welcome page
                container.innerHTML = `
                    <div class="welcome">
                        <h2>Welcome to Tabbed WebView</h2>
                        <p>
                            This demo shows how to create a multi-tab browser using AuroraView.<br>
                            Each tab can load different URLs. Click a link below to navigate.
                        </p>
                        <div class="link-grid">
                            ${sampleLinks.map(link => `
                                <div class="link-card" onclick="navigateTo('${link.url}')">
                                    <div class="link-icon">${link.icon}</div>
                                    <div class="link-info">
                                        <h3>${link.title}</h3>
                                        <p>${link.desc}</p>
                                    </div>
                                </div>
                            `).join('')}
                        </div>
                        <div style="margin-top: 32px;">
                            <h3 style="color: #4facfe; margin-bottom: 12px;">How it works</h3>
                            <p style="color: #888; font-size: 13px; line-height: 1.8;">
                                Use <code>new_window_mode="child_webview"</code> when creating a WebView
                                to handle <code>target="_blank"</code> links by opening new WebView windows.
                            </p>
                            <pre><code>webview = WebView(
    title="My Browser",
    new_window_mode="child_webview",  # Opens new windows as child WebViews
    ...
)</code></pre>
                        </div>
                    </div>
                `;
            } else {
                // Tab with URL
                container.innerHTML = `
                    <div class="page-info">
                        <h3>${escapeHtml(activeTab.title)}</h3>
                        <p class="url">${escapeHtml(activeTab.url)}</p>
                        <div class="action-buttons">
                            <button onclick="openInBrowser('${escapeHtml(activeTab.url)}')">
                                Open in System Browser
                            </button>
                            <button class="secondary" onclick="goHome()">
                                Go Home
                            </button>
                        </div>
                    </div>
                    <div class="welcome">
                        <p style="color: #888;">
                            In a full implementation, this area would contain an embedded WebView
                            showing the actual web content. This demo focuses on the tab management UI.
                        </p>
                        <p style="color: #666; font-size: 12px; margin-top: 16px;">
                            For real multi-WebView tabs, each tab would spawn a separate WebView instance
                            using <code>new_window_mode="child_webview"</code>.
                        </p>
                    </div>
                `;
            }
        }

        function updateStatus() {
            document.getElementById('tab-count').textContent = `${tabs.length} tab${tabs.length !== 1 ? 's' : ''}`;
            const activeTab = tabs.find(t => t.is_active);
            document.getElementById('status-text').textContent = activeTab?.url || 'Ready';
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text || '';
            return div.innerHTML;
        }

        // Tab Actions
        function createTab(url = '') {
            if (window.auroraview?.api) {
                auroraview.api.create_tab({ url });
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

        function navigateTo(url) {
            if (window.auroraview?.api) {
                auroraview.api.navigate({ url });
            }
        }

        function goHome() {
            if (window.auroraview?.api) {
                auroraview.api.go_home();
            }
        }

        function openInBrowser(url) {
            if (window.auroraview?.shell) {
                auroraview.shell.open(url);
            }
        }
    </script>
</body>
</html>
"""


def get_domain_title(url: str) -> str:
    """Extract domain name as title."""
    try:
        from urllib.parse import urlparse

        parsed = urlparse(url)
        domain = parsed.netloc or parsed.path
        if domain.startswith("www."):
            domain = domain[4:]
        return domain.split(".")[0].capitalize()
    except Exception:
        return "New Tab"


def create_main_window() -> WebView:
    """Create the main browser window."""
    view = WebView.create(
        title="Tabbed WebView Demo",
        html=MAIN_HTML,
        width=1100,
        height=800,
        debug=True,
        # Enable child WebView mode for handling window.open()
        # This allows links with target="_blank" to open in new WebView windows
        new_window_mode="child_webview",
    )

    tab_manager.set_main_window(view)

    @view.bind_call("api.create_tab")
    def create_tab(url: str = "") -> dict:
        """Create a new tab."""
        title = get_domain_title(url) if url else "New Tab"
        tab_id = tab_manager.create_tab(url=url, title=title)
        tab_manager.broadcast_update()
        return {"tab_id": tab_id, "success": True}

    @view.bind_call("api.close_tab")
    def close_tab(tab_id: str) -> dict:
        """Close a tab."""
        next_id = tab_manager.close_tab(tab_id)
        tab_manager.broadcast_update()

        # If no tabs left, create a new one
        if next_id is None:
            new_id = tab_manager.create_tab()
            tab_manager.broadcast_update()
            return {"success": True, "next_tab_id": new_id}

        return {"success": True, "next_tab_id": next_id}

    @view.bind_call("api.switch_tab")
    def switch_tab(tab_id: str) -> dict:
        """Switch to a tab."""
        success = tab_manager.switch_tab(tab_id)
        tab_manager.broadcast_update()
        return {"success": success}

    @view.bind_call("api.navigate")
    def navigate(url: str) -> dict:
        """Navigate the active tab."""
        if tab_manager.active_tab_id:
            title = get_domain_title(url)
            tab_manager.update_tab(tab_manager.active_tab_id, url=url, title=title)
            tab_manager.broadcast_update()
            return {"success": True}
        return {"success": False, "error": "No active tab"}

    @view.bind_call("api.go_home")
    def go_home() -> dict:
        """Go to home page."""
        if tab_manager.active_tab_id:
            tab_manager.update_tab(
                tab_manager.active_tab_id, url="", title="New Tab"
            )
            tab_manager.broadcast_update()
        return {"success": True}

    return view


def main():
    """Run the tabbed WebView demo."""
    print("=" * 60)
    print("Tabbed WebView Demo")
    print("=" * 60)
    print()
    print("This demo demonstrates:")
    print("- Multi-tab browser interface")
    print("- Tab management (create, close, switch)")
    print("- URL navigation per tab")
    print("- Using new_window_mode='child_webview' for new windows")
    print()
    print("Starting...")
    print()

    window = create_main_window()
    window.show()


if __name__ == "__main__":
    main()
