---
outline: deep
---

# Multi-Tab Browser Demo

This guide demonstrates how to create a browser-like application with multiple tabs using AuroraView WebView.

## Overview

The multi-tab browser demo showcases:

- **Tab Management**: Create, close, and switch between tabs
- **Navigation Controls**: Back, forward, reload, and home buttons
- **URL Bar**: Smart URL/search detection
- **New Window Handling**: Using `new_window_mode` for link interception
- **State Synchronization**: Keeping tab state in sync between Python and JavaScript

## Key Concepts

### New Window Mode

AuroraView provides three modes for handling `window.open()` and `target="_blank"` links:

```python
from auroraview import WebView

# Mode 1: Deny (default) - Block all new window requests
webview = WebView(new_window_mode="deny")

# Mode 2: System Browser - Open links in the default browser
webview = WebView(new_window_mode="system_browser")

# Mode 3: Child WebView - Create new WebView windows
webview = WebView(new_window_mode="child_webview")
```

For a tabbed browser, `child_webview` mode is most appropriate as it allows you to intercept new window requests and handle them within your application.

### Tab State Management

The demo uses a `BrowserState` class to manage tabs:

```python
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import threading

@dataclass
class Tab:
    """Represents a browser tab."""
    id: str
    title: str = "New Tab"
    url: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False

@dataclass
class BrowserState:
    """Thread-safe browser state manager."""
    tabs: Dict[str, Tab] = field(default_factory=dict)
    active_tab_id: Optional[str] = None
    tab_order: List[str] = field(default_factory=list)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def add_tab(self, tab: Tab) -> None:
        with self._lock:
            self.tabs[tab.id] = tab
            self.tab_order.append(tab.id)

    def remove_tab(self, tab_id: str) -> Optional[str]:
        """Remove tab and return next active tab ID."""
        with self._lock:
            if tab_id not in self.tabs:
                return self.active_tab_id
            
            idx = self.tab_order.index(tab_id)
            del self.tabs[tab_id]
            self.tab_order.remove(tab_id)
            
            if not self.tab_order:
                return None
            
            new_idx = min(idx, len(self.tab_order) - 1)
            return self.tab_order[new_idx]
```

### Python-JavaScript Communication

The browser uses AuroraView's event system for bidirectional communication:

**Python → JavaScript (Events)**:
```python
# Broadcast tab updates to UI
def broadcast_tabs_update():
    main_window.emit("tabs:update", {
        "tabs": browser_state.get_tabs_info(),
        "active_tab_id": browser_state.active_tab_id,
    })
```

**JavaScript → Python (API Calls)**:
```javascript
// Create a new tab
auroraview.api.create_tab({ url: "https://example.com" });

// Close a tab
auroraview.api.close_tab({ tab_id: "tab-123" });

// Navigate
auroraview.api.navigate({ url: "https://github.com" });
```

## Running the Examples

### Multi-Tab Browser Demo

```bash
python examples/multi_tab_browser_demo.py
```

This demo provides a complete browser-like UI with:
- Tab bar with create/close/switch functionality
- Navigation bar with back/forward/reload/home buttons
- URL bar with smart URL detection
- Quick links for common websites

### Tabbed WebView Demo

```bash
python examples/tabbed_webview_demo.py
```

A simpler version focusing on tab management concepts and `new_window_mode` usage.

## Implementation Details

### Creating the Main Window

```python
from auroraview import WebView

def create_browser_window() -> WebView:
    view = WebView.create(
        title="AuroraView Browser",
        html=BROWSER_HTML,
        width=1200,
        height=800,
        debug=True,
        # Enable child WebView mode for handling window.open()
        new_window_mode="child_webview",
    )
    
    # Register API handlers
    @view.bind_call("api.create_tab")
    def create_tab(url: str = "") -> dict:
        tab_id = str(uuid.uuid4())[:8]
        tab = Tab(id=tab_id, title="New Tab", url=url)
        browser_state.add_tab(tab)
        browser_state.active_tab_id = tab_id
        broadcast_tabs_update()
        return {"tab_id": tab_id}
    
    @view.bind_call("api.navigate")
    def navigate(url: str) -> dict:
        if browser_state.active_tab_id:
            browser_state.update_tab(
                browser_state.active_tab_id,
                url=url,
                title=get_title_from_url(url),
            )
            broadcast_tabs_update()
            return {"success": True}
        return {"success": False}
    
    return view
```

### Handling Tab Events in JavaScript

```javascript
window.addEventListener('auroraviewready', () => {
    // Listen for tab updates from Python
    auroraview.on('tabs:update', (data) => {
        tabs = data.tabs;
        activeTabId = data.active_tab_id;
        renderTabs();
    });
    
    // Create initial tab
    auroraview.api.create_tab();
});

function renderTabs() {
    const container = document.getElementById('tabs-container');
    container.innerHTML = tabs.map(tab => `
        <div class="tab ${tab.is_active ? 'active' : ''}"
             onclick="switchTab('${tab.id}')">
            <span class="tab-title">${tab.title}</span>
            <span class="tab-close" 
                  onclick="event.stopPropagation(); closeTab('${tab.id}')">×</span>
        </div>
    `).join('');
}
```

## Advanced Topics

### Real Multi-WebView Implementation

For a production browser, each tab would have its own WebView instance. The architecture would look like:

```python
class TabManager:
    def __init__(self):
        self.tab_webviews: Dict[str, WebView] = {}
    
    def create_tab(self, url: str = "") -> str:
        tab_id = str(uuid.uuid4())[:8]
        
        # Create a new WebView for this tab
        webview = WebView.create(
            title=f"Tab {tab_id}",
            url=url,
            new_window_mode="child_webview",
        )
        
        # Handle navigation events from this tab
        @webview.on("navigation")
        def on_navigate(data):
            self.update_tab_url(tab_id, data["url"])
        
        self.tab_webviews[tab_id] = webview
        return tab_id
    
    def close_tab(self, tab_id: str):
        if tab_id in self.tab_webviews:
            self.tab_webviews[tab_id].close()
            del self.tab_webviews[tab_id]
```

### Thread Safety

The `BrowserState` class uses a lock for thread safety, which is important when:
- Multiple tabs may trigger events simultaneously
- Background tasks update tab state
- The main window needs to read state while tabs are being modified

```python
def update_tab(self, tab_id: str, **kwargs) -> None:
    with self._lock:
        if tab_id in self.tabs:
            for key, value in kwargs.items():
                if hasattr(self.tabs[tab_id], key):
                    setattr(self.tabs[tab_id], key, value)
```

## See Also

- [WebView Basics](./webview-basics.md) - Core WebView concepts
- [Communication](./communication.md) - Python ↔ JavaScript communication
- [Child Windows](./child-windows.md) - Managing child windows
- [Examples](./examples.md) - More example applications
