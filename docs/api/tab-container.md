# TabContainer API

Foundation for building tabbed browsers and multi-webview applications.

## Overview

`TabContainer` provides tab state management with support for:

- Creating and closing tabs
- Tab activation and switching
- Navigation controls (back, forward, reload)
- Lazy WebView loading
- Event callbacks for UI updates

## Quick Start

```python
from auroraview.browser.tab_container import TabContainer

# Create container with callbacks
container = TabContainer(
    on_tabs_update=lambda tabs: print(f"Tabs: {len(tabs)}"),
    on_tab_change=lambda tab: print(f"Active: {tab.title}"),
    default_url="https://example.com"
)

# Create tabs
tab1 = container.create_tab("https://github.com", "GitHub")
tab2 = container.create_tab("https://google.com", "Google")

# Navigate
container.navigate("https://new-url.com")

# Close tab
container.close_tab(tab1.id)
```

## TabState Class

Represents the state of a single tab.

### Attributes

| Attribute | Type | Description |
|-----------|------|-------------|
| `id` | str | Unique tab identifier |
| `title` | str | Tab title (displayed in tab bar) |
| `url` | str | Current URL |
| `favicon` | str | Favicon URL or data URI |
| `is_loading` | bool | Whether the page is loading |
| `can_go_back` | bool | Whether back navigation is available |
| `can_go_forward` | bool | Whether forward navigation is available |
| `webview_id` | Optional[str] | Reference to WindowManager (None if not loaded) |
| `metadata` | Dict[str, Any] | Custom metadata storage |

### Methods

#### `to_dict()`

Convert to dictionary for JSON serialization.

```python
tab = TabState(
    id="tab_123",
    title="Example",
    url="https://example.com"
)

data = tab.to_dict()
# {
#     "id": "tab_123",
#     "title": "Example",
#     "url": "https://example.com",
#     "favicon": "",
#     "isLoading": False,
#     "canGoBack": False,
#     "canGoForward": False,
#     "metadata": {}
# }
```

**Returns**: `Dict[str, Any]` - JSON-serializable dictionary

## TabContainer Class

### Constructor

```python
container = TabContainer(
    on_tab_change=lambda tab: print(f"Changed: {tab.id}"),
    on_tabs_update=lambda tabs: print(f"Count: {len(tabs)}"),
    default_url="https://example.com",
    webview_factory=None,
    webview_options={"debug": True}
)
```

**Parameters**:
- `on_tab_change` (Optional[Callable[[TabState], None]]): Callback when active tab changes
- `on_tabs_update` (Optional[Callable[[List[TabState]], None]]): Callback when tab list changes
- `default_url` (str): Default URL for new tabs
- `webview_factory` (Optional[Callable[..., WebView]]): Custom factory for creating WebViews
- `webview_options` (Optional[Dict[str, Any]]): Options passed to WebView creation

### Tab Management Methods

#### `create_tab(url, title, activate, load_immediately)`

Create a new tab.

```python
tab = container.create_tab(
    url="https://github.com",
    title="GitHub",
    activate=True,
    load_immediately=True
)
print(f"Created tab: {tab.id}")
```

**Parameters**:
- `url` (str): Initial URL (uses default_url if empty)
- `title` (str): Initial tab title (default: "New Tab")
- `activate` (bool): Whether to activate the new tab (default: True)
- `load_immediately` (bool): Whether to create WebView immediately (default: True)

**Returns**: `TabState` - The created tab state

#### `close_tab(tab_id)`

Close a tab.

```python
new_active_id = container.close_tab("tab_123")
if new_active_id:
    print(f"New active: {new_active_id}")
else:
    print("No tabs remaining")
```

**Parameters**:
- `tab_id` (str): ID of the tab to close

**Returns**: `Optional[str]` - ID of the new active tab, or None if no tabs remain

#### `activate_tab(tab_id)`

Activate a tab.

```python
success = container.activate_tab("tab_456")
if success:
    print("Tab activated")
```

**Parameters**:
- `tab_id` (str): ID of the tab to activate

**Returns**: `bool` - True if tab was found and activated

#### `update_tab(tab_id, **kwargs)`

Update tab properties.

```python
container.update_tab(
    "tab_123",
    title="Updated Title",
    favicon="https://example.com/favicon.ico",
    metadata={"custom": "data"}
)
```

**Parameters**:
- `tab_id` (str): Tab ID
- `**kwargs`: Properties to update (title, favicon, metadata, etc.)

**Returns**: `bool` - True if tab was found and updated

### Navigation Methods

#### `navigate(url, tab_id)`

Navigate a tab to a URL.

```python
container.navigate("https://example.com")  # Active tab
container.navigate("https://example.com", tab_id="tab_123")  # Specific tab
```

**Parameters**:
- `url` (str): URL to navigate to
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

**Returns**: `bool` - True if navigation was initiated

#### `go_back(tab_id)`

Go back in the specified tab.

```python
success = container.go_back()  # Active tab
success = container.go_back("tab_123")  # Specific tab
```

**Parameters**:
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

**Returns**: `bool` - True if navigation succeeded

#### `go_forward(tab_id)`

Go forward in the specified tab.

```python
success = container.go_forward()
```

**Parameters**:
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

**Returns**: `bool` - True if navigation succeeded

#### `reload(tab_id)`

Reload the specified tab.

```python
success = container.reload()
```

**Parameters**:
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

**Returns**: `bool` - True if reload succeeded

### Query Methods

#### `get_tab(tab_id)`

Get a tab by ID.

```python
tab = container.get_tab("tab_123")
if tab:
    print(f"Title: {tab.title}")
```

**Parameters**:
- `tab_id` (str): Tab ID

**Returns**: `Optional[TabState]` - The tab state, or None

#### `get_active_tab()`

Get the active tab.

```python
active = container.get_active_tab()
if active:
    print(f"Active tab: {active.title}")
```

**Returns**: `Optional[TabState]` - The active tab state, or None

#### `get_active_tab_id()`

Get the active tab ID.

```python
active_id = container.get_active_tab_id()
```

**Returns**: `Optional[str]` - The active tab ID, or None

#### `get_all_tabs()`

Get all tabs in order.

```python
tabs = container.get_all_tabs()
for tab in tabs:
    print(f"- {tab.title}: {tab.url}")
```

**Returns**: `List[TabState]` - All tabs in insertion order

#### `get_tab_count()`

Get the number of tabs.

```python
count = container.get_tab_count()
print(f"Total tabs: {count}")
```

**Returns**: `int` - Number of tabs

#### `get_webview(tab_id)`

Get the WebView for a tab.

```python
webview = container.get_webview("tab_123")
if webview:
    webview.eval("console.log('Hello')")
```

**Parameters**:
- `tab_id` (Optional[str]): Tab ID (uses active tab if None)

**Returns**: `Optional[WebView]` - The WebView instance, or None

### Lifecycle Methods

#### `close_all()`

Close all tabs.

```python
container.close_all()
```

## Examples

### Basic Tab Management

```python
from auroraview.browser.tab_container import TabContainer

container = TabContainer(default_url="about:blank")

# Create some tabs
github = container.create_tab("https://github.com", "GitHub")
google = container.create_tab("https://google.com", "Google")
docs = container.create_tab("https://docs.python.org", "Python Docs")

# List all tabs
for tab in container.get_all_tabs():
    marker = ">" if tab.id == container.get_active_tab_id() else " "
    print(f"{marker} [{tab.id}] {tab.title}")

# Switch tabs
container.activate_tab(google.id)

# Navigate active tab
container.navigate("https://google.com/search?q=python")

# Close a tab
container.close_tab(github.id)
```

### With UI Callbacks

```python
from auroraview.browser.tab_container import TabContainer, TabState
from typing import List

def on_tabs_update(tabs: List[TabState]):
    """Called when tab list changes."""
    print(f"Tabs updated: {len(tabs)} total")
    for tab in tabs:
        status = "loading" if tab.is_loading else "ready"
        print(f"  - {tab.title} ({status})")

def on_tab_change(tab: TabState):
    """Called when active tab changes."""
    print(f"Active tab: {tab.title}")
    print(f"  URL: {tab.url}")

container = TabContainer(
    on_tabs_update=on_tabs_update,
    on_tab_change=on_tab_change,
    default_url="https://example.com"
)

# Operations will trigger callbacks
container.create_tab("https://github.com", "GitHub")
container.create_tab("https://google.com", "Google")
container.activate_tab(container.get_all_tabs()[1].id)
```

### Custom WebView Factory

```python
from auroraview.browser.tab_container import TabContainer
from auroraview import create_webview

def custom_factory(**kwargs):
    """Custom WebView factory with additional configuration."""
    webview = create_webview(
        debug=True,
        **kwargs
    )
    # Add custom bindings
    webview.bind_call("custom.action", lambda: print("Custom action"))
    return webview

container = TabContainer(
    webview_factory=custom_factory,
    webview_options={"width": 800, "height": 600}
)

container.create_tab("https://example.com")
```

### Tab Metadata

```python
from auroraview.browser.tab_container import TabContainer

container = TabContainer()

# Create tab with metadata
tab = container.create_tab(
    url="https://example.com",
    title="My Tab"
)

# Update with custom metadata
container.update_tab(
    tab.id,
    metadata={
        "created_at": "2024-01-15T10:00:00Z",
        "category": "work",
        "pinned": True
    }
)

# Access metadata
tab = container.get_tab(tab.id)
if tab and tab.metadata.get("pinned"):
    print(f"Pinned tab: {tab.title}")
```

### Lazy Loading Tabs

```python
from auroraview.browser.tab_container import TabContainer

container = TabContainer(default_url="about:blank")

# Create tabs without loading WebViews
tabs_data = [
    ("https://github.com", "GitHub"),
    ("https://google.com", "Google"),
    ("https://stackoverflow.com", "StackOverflow"),
]

for url, title in tabs_data:
    container.create_tab(
        url=url,
        title=title,
        activate=False,
        load_immediately=False  # Don't create WebView yet
    )

print(f"Created {container.get_tab_count()} tabs (lazy)")

# Activate first tab - this will load its WebView
first_tab = container.get_all_tabs()[0]
container.activate_tab(first_tab.id)
```

## Events

TabContainer sets up the following WebView event handlers for each tab:

| Event | Description |
|-------|-------------|
| `page:load_start` | Page started loading |
| `page:load_finish` | Page finished loading |
| `page:title_changed` | Page title changed |
| `page:favicon_changed` | Page favicon changed |
| `navigation:state_changed` | Back/forward availability changed |
| `closing` | WebView is closing |

## See Also

- [Browser API](./browser.md) - High-level browser API
- [WindowManager API](./window-manager.md) - Global window registry
- [Multi-Tab Browser Guide](../guide/multi-tab-browser.md) - Building browsers
