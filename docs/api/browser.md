# Browser API

High-level browser API with multi-tab support.

## Overview

The `Browser` class provides a complete tabbed browser experience with:

- Tab management (create, close, switch)
- Navigation controls (back, forward, reload)
- URL bar and search
- Built-in customizable UI
- DCC integration support

## Quick Start

```python
from auroraview.browser import Browser

# Create and run a browser
browser = Browser(title="My Browser")
browser.new_tab("https://github.com", "GitHub")
browser.new_tab("https://google.com", "Google")
browser.run()  # Blocking
```

### For DCC Integration

```python
from auroraview.browser import Browser

# Create browser embedded in DCC panel
browser = Browser(parent=maya_widget)
browser.new_tab("https://docs.autodesk.com", "Maya Docs")
browser.show()  # Non-blocking
```

## Browser Class

### Constructor

```python
browser = Browser(
    title="AuroraView Browser",
    width=1200,
    height=800,
    debug=False,
    parent=None,
    default_url="about:blank"
)
```

**Parameters**:
- `title` (str): Window title (default: "AuroraView Browser")
- `width` (int): Window width in pixels (default: 1200)
- `height` (int): Window height in pixels (default: 800)
- `debug` (bool): Enable developer tools (default: False)
- `parent` (Any): Parent widget for DCC integration (default: None)
- `default_url` (str): Default URL for new tabs (default: "about:blank")

### Tab Management Methods

#### `new_tab(url, title)`

Create a new tab.

```python
tab = browser.new_tab("https://github.com", "GitHub")
print(f"Created: {tab['id']}")
```

**Parameters**:
- `url` (str): Initial URL (uses default_url if empty)
- `title` (str): Tab title (default: "New Tab")

**Returns**: `Dict[str, Any]` - The created tab info dict

Tab dict structure:
```python
{
    "id": "tab_1",
    "url": "https://github.com",
    "title": "GitHub",
    "canGoBack": False,
    "canGoForward": False,
    "loading": False
}
```

#### `close_tab(tab_id)`

Close a tab.

```python
browser.close_tab("tab_1")  # Close specific tab
browser.close_tab()  # Close active tab
```

**Parameters**:
- `tab_id` (Optional[str]): Tab ID (closes active tab if None)

#### `activate_tab(tab_id)`

Activate a tab by ID.

```python
browser.activate_tab("tab_2")
```

**Parameters**:
- `tab_id` (str): Tab ID to activate

#### `get_tabs()`

Get all tabs.

```python
tabs = browser.get_tabs()
for tab in tabs:
    print(f"- {tab['title']}: {tab['url']}")
```

**Returns**: `List[Dict[str, Any]]` - List of all tab info dicts

#### `get_active_tab()`

Get the active tab.

```python
active = browser.get_active_tab()
if active:
    print(f"Active: {active['title']}")
```

**Returns**: `Optional[Dict[str, Any]]` - The active tab info, or None

### Navigation Methods

#### `navigate(url, tab_id)`

Navigate a tab to a URL.

```python
browser.navigate("https://example.com")  # Active tab
browser.navigate("https://example.com", tab_id="tab_1")  # Specific tab
```

**Parameters**:
- `url` (str): URL to navigate to
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

#### `go_back()`

Go back in the active tab.

```python
browser.go_back()
```

#### `go_forward()`

Go forward in the active tab.

```python
browser.go_forward()
```

#### `reload()`

Reload the active tab.

```python
browser.reload()
```

### Lifecycle Methods

#### `show(wait)`

Show the browser.

```python
browser.show(wait=True)   # Block until closed
browser.show(wait=False)  # Return immediately
```

**Parameters**:
- `wait` (bool): If True, block until window closes (default: True)

#### `run()`

Run the browser (blocking). Alias for `show(wait=True)`.

```python
browser.run()
```

#### `close()`

Close the browser.

```python
browser.close()
```

### Callback Methods

#### `on_ready(callback)`

Register a callback for when browser is ready.

```python
def setup(browser):
    print("Browser ready!")
    browser.new_tab("https://example.com")

browser.on_ready(setup)
browser.run()
```

**Parameters**:
- `callback` (Callable[[Browser], None]): Function to call when browser is ready

## JavaScript API

The Browser exposes these methods to the frontend via `auroraview.call()`:

| Method | Description |
|--------|-------------|
| `browser.new_tab` | Create a new tab |
| `browser.close_tab` | Close a tab |
| `browser.activate_tab` | Activate a tab |
| `browser.navigate` | Navigate to URL |
| `browser.go_back` | Go back |
| `browser.go_forward` | Go forward |
| `browser.reload` | Reload page |
| `browser.get_tabs` | Get all tabs |
| `browser.get_state` | Get browser state |

### JavaScript Example

```javascript
// Create new tab
await auroraview.call('browser.new_tab', { 
    url: 'https://github.com', 
    title: 'GitHub' 
});

// Get all tabs
const { tabs, activeTabId } = await auroraview.call('browser.get_state');

// Navigate
await auroraview.call('browser.navigate', { url: 'https://example.com' });

// Close tab
await auroraview.call('browser.close_tab', { tabId: 'tab_1' });
```

## Events

### Python → JavaScript

| Event | Payload | Description |
|-------|---------|-------------|
| `browser:tabs_update` | `{tabs, activeTabId}` | Tab list or active tab changed |

### Listening in JavaScript

```javascript
auroraview.on('browser:tabs_update', ({ tabs, activeTabId }) => {
    console.log(`Tabs: ${tabs.length}, Active: ${activeTabId}`);
    renderTabBar(tabs, activeTabId);
});
```

## Examples

### Basic Browser

```python
from auroraview.browser import Browser

browser = Browser(
    title="My Browser",
    width=1280,
    height=720,
    debug=True
)

# Add initial tabs
browser.new_tab("https://github.com", "GitHub")
browser.new_tab("https://google.com", "Google")

# Run browser
browser.run()
```

### With Ready Callback

```python
from auroraview.browser import Browser

def on_browser_ready(browser):
    """Called when browser is ready."""
    print("Browser is ready!")
    
    # Add tabs
    browser.new_tab("https://github.com", "GitHub")
    
    # Navigate after delay (example)
    import threading
    def delayed_nav():
        import time
        time.sleep(2)
        browser.navigate("https://github.com/explore")
    
    threading.Thread(target=delayed_nav).start()

browser = Browser(title="My Browser")
browser.on_ready(on_browser_ready)
browser.run()
```

### DCC Integration (Maya)

```python
from auroraview.browser import Browser
import maya.cmds as cmds
from maya import OpenMayaUI
from shiboken2 import wrapInstance
from PySide2 import QtWidgets

# Get Maya main window
def get_maya_window():
    ptr = OpenMayaUI.MQtUtil.mainWindow()
    return wrapInstance(int(ptr), QtWidgets.QMainWindow)

# Create dock widget
main_window = get_maya_window()
dock = QtWidgets.QDockWidget("Help Browser", main_window)

# Create browser in dock
browser = Browser(
    parent=dock,
    title="Maya Help",
    default_url="https://help.autodesk.com/view/MAYAUL/2024/ENU/"
)

# Add help pages
browser.new_tab("https://help.autodesk.com/view/MAYAUL/2024/ENU/", "Maya Help")
browser.new_tab("https://www.python.org/doc/", "Python Docs")

# Show browser
browser.show(wait=False)

# Add dock to Maya
dock.setWidget(browser._webview.widget())
main_window.addDockWidget(QtCore.Qt.RightDockWidgetArea, dock)
```

### Custom Start Page

```python
from auroraview.browser import Browser

class CustomBrowser(Browser):
    def __init__(self, **kwargs):
        super().__init__(
            default_url="https://start.duckduckgo.com",
            **kwargs
        )
    
    def setup_bookmarks(self):
        """Add bookmarked tabs."""
        bookmarks = [
            ("https://github.com", "GitHub"),
            ("https://stackoverflow.com", "Stack Overflow"),
            ("https://docs.python.org", "Python Docs"),
        ]
        for url, title in bookmarks:
            self.new_tab(url, title)

browser = CustomBrowser(title="Dev Browser")
browser.setup_bookmarks()
browser.run()
```

### Non-Blocking with Event Loop

```python
from auroraview.browser import Browser
import time

browser = Browser(title="Background Browser")
browser.new_tab("https://example.com")

# Show without blocking
browser.show(wait=False)

# Do other work while browser is open
for i in range(10):
    print(f"Working... {i}")
    time.sleep(1)
    
    # Check if browser is still running
    if not browser._running:
        break

print("Done")
```

## Built-in UI

The Browser includes a default UI with:

- **Tab Bar**: Chrome-style tabs with close buttons
- **Navigation Bar**: Back, forward, reload buttons
- **URL Bar**: Address bar with search support
- **Content Frame**: Iframe for displaying web content

### Customizing UI

To customize the UI, subclass Browser and override `_get_browser_html()`:

```python
from auroraview.browser import Browser

class CustomUIBrowser(Browser):
    def _get_browser_html(self):
        """Return custom HTML for browser UI."""
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Custom Browser</title>
            <style>
                /* Custom styles */
            </style>
        </head>
        <body>
            <!-- Custom UI -->
            <script>
                // Custom JavaScript
            </script>
        </body>
        </html>
        """
```

## See Also

- [TabContainer API](./tab-container.md) - Lower-level tab management
- [WindowManager API](./window-manager.md) - Global window registry
- [Multi-Tab Browser Guide](../guide/multi-tab-browser.md) - Building browsers
