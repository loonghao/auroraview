---
outline: deep
---

# Agent Browser - Multi-Tab Browser Demo

This guide demonstrates how to create a browser-like application with multiple tabs using AuroraView WebView.

## Overview

The Agent Browser demo showcases:

- **Tab Management**: Create, close, and switch between tabs
- **Navigation Controls**: Back, forward, reload, and home buttons
- **URL Bar**: Smart URL/search detection
- **File Drag & Drop**: Open PDF, images, and text files by dragging
- **Keyboard Shortcuts**: Ctrl+T, Ctrl+W, Ctrl+L, F5, etc.
- **State Synchronization**: Keeping tab state in sync between Python and JavaScript

## Running the Browser

```bash
# Run as module
python -m examples.agent_browser

# Or run directly
python examples/agent_browser/browser.py
```

## Key Concepts

### Using the Unified API

Agent Browser uses AuroraView's unified `create_webview()` API:

```python
from auroraview import create_webview

# Create a standalone browser window
webview = create_webview(
    title="Agent Browser",
    html=browser_html,
    width=1280,
    height=900,
    debug=True,
    allow_file_protocol=True,  # Enable file:// for local files
    context_menu=True,
)
```

### Tab State Management

The demo uses a `TabManager` class to manage tabs:

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
    history: List[str] = field(default_factory=list)
    history_index: int = -1

class TabManager:
    """Thread-safe tab manager with history support."""
    
    def __init__(self):
        self.tabs: Dict[str, Tab] = {}
        self.tab_order: List[str] = []
        self.active_tab_id: Optional[str] = None
        self._lock = threading.Lock()

    def navigate(self, tab_id: str, url: str) -> None:
        """Navigate a tab to a URL, updating history."""
        with self._lock:
            if tab_id not in self.tabs:
                return
            tab = self.tabs[tab_id]
            
            # Truncate forward history
            if tab.history_index < len(tab.history) - 1:
                tab.history = tab.history[:tab.history_index + 1]
            
            tab.history.append(url)
            tab.history_index = len(tab.history) - 1
            tab.url = url
            tab.can_go_back = tab.history_index > 0
            tab.can_go_forward = False
```

### Python-JavaScript Communication

The browser uses AuroraView's event system for bidirectional communication:

**Python → JavaScript (Events)**:
```python
# Broadcast tab updates to UI
def broadcast_tabs_update():
    webview.emit("tabs:update", {
        "tabs": tab_manager.get_tabs_info(),
        "active_tab_id": tab_manager.active_tab_id,
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

### File Drag & Drop

The browser supports opening files by dragging them onto the window:

```python
from auroraview.core.events import WindowEvent

@webview.on(WindowEvent.FILE_DROP)
def on_file_drop(data):
    """Handle file drop events."""
    paths = data.get("paths", [])
    if paths:
        result = handle_file_open(paths[0])
        if result.get("success"):
            webview.emit("file:opened", result)
```

Supported file types:
- **PDF**: Opens directly in browser
- **Images**: PNG, JPG, GIF, WebP, SVG, etc.
- **Text**: TXT, JSON, XML, MD, CSS, JS
- **HTML**: Opens as web page

## Implementation Details

### Creating the Browser

```python
from auroraview import create_webview
from auroraview.core.events import WindowEvent

class AgentBrowser:
    def __init__(self):
        self.tab_manager = TabManager()
        self.webview = None

    def run(self):
        self.webview = create_webview(
            title="Agent Browser",
            html=self._load_html(),
            width=1280,
            height=900,
            debug=True,
            allow_file_protocol=True,
        )
        
        # Register API handlers
        @self.webview.bind_call("api.create_tab")
        def create_tab(url: str = "") -> dict:
            tab_id = self.tab_manager.create_tab(url=url)
            self._broadcast_tabs_update()
            if url:
                self.webview.load_url(url)
            return {"tab_id": tab_id, "success": True}
        
        @self.webview.bind_call("api.navigate")
        def navigate(url: str) -> dict:
            final_url = self._process_url(url)
            self.tab_manager.navigate(
                self.tab_manager.active_tab_id, 
                final_url
            )
            self._broadcast_tabs_update()
            self.webview.load_url(final_url)
            return {"success": True, "url": final_url}
        
        # Create initial tab
        self.tab_manager.create_tab()
        self.webview.show()
```

### Handling Tab Events in JavaScript

```javascript
window.addEventListener('auroraviewready', () => {
    // Listen for tab updates from Python
    auroraview.on('tabs:update', (data) => {
        tabs = data.tabs;
        activeTabId = data.active_tab_id;
        renderTabs();
        updateNavButtons();
    });
    
    // Get initial tabs state
    auroraview.api.get_tabs().then(data => {
        tabs = data.tabs || [];
        activeTabId = data.active_tab_id;
        renderTabs();
    });
});

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 't') { e.preventDefault(); createNewTab(); }
    if (e.ctrlKey && e.key === 'w') { e.preventDefault(); closeTab(activeTabId); }
    if (e.ctrlKey && e.key === 'l') { e.preventDefault(); focusUrlBar(); }
    if (e.key === 'F5') { e.preventDefault(); reloadPage(); }
});
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+L` | Focus URL bar |
| `F5` | Reload page |
| `Alt+←` | Go back |
| `Alt+→` | Go forward |
| `F12` | Developer tools |

## Project Structure

```
examples/agent_browser/
├── __init__.py      # Package exports
├── __main__.py      # Entry point for -m
├── browser.py       # Main browser implementation
└── ui.html          # Browser UI template
```

## See Also

- [WebView Basics](./webview-basics.md) - Core WebView concepts
- [Communication](./communication.md) - Python ↔ JavaScript communication
- [Child Windows](./child-windows.md) - Managing child windows
- [Examples](./examples.md) - More example applications
