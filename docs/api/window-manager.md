# WindowManager API

Global window registry for managing multiple WebView instances.

## Overview

`WindowManager` provides a centralized way to track, access, and manage multiple WebView windows across an application. It implements a singleton pattern for global access and uses weak references to prevent memory leaks.

## Quick Start

```python
from auroraview.core.window_manager import get_window_manager, broadcast_event

# Get the global manager
wm = get_window_manager()

# Register a window
uid = wm.register(webview)

# Get all windows
windows = wm.get_all()

# Broadcast event to all windows
broadcast_event("app:theme_changed", {"theme": "dark"})
```

## Functions

### `get_window_manager()`

Get the global WindowManager singleton instance.

```python
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()
```

**Returns**: `WindowManager` - The global WindowManager instance

### `get_windows()`

Get all registered WebView windows.

```python
from auroraview.core.window_manager import get_windows

windows = get_windows()
for window in windows:
    print(window.title)
```

**Returns**: `List[WebView]` - List of all registered WebView instances

### `get_active_window()`

Get the currently active WebView window.

```python
from auroraview.core.window_manager import get_active_window

active = get_active_window()
if active:
    active.eval("console.log('Active window!')")
```

**Returns**: `Optional[WebView]` - The active window, or None

### `broadcast_event(event, data)`

Broadcast an event to all registered windows.

```python
from auroraview.core.window_manager import broadcast_event

# Notify all windows of a theme change
count = broadcast_event("app:theme_changed", {"theme": "dark"})
print(f"Notified {count} windows")
```

**Parameters**:
- `event` (str): Event name
- `data` (Any): Event payload

**Returns**: `int` - Number of windows that received the event

## WindowManager Class

### Methods

#### `register(window, uid=None)`

Register a window and return its unique ID.

```python
uid = wm.register(webview)
# Or with custom ID
uid = wm.register(webview, uid="my_panel")
```

**Parameters**:
- `window` (WebView): The WebView instance to register
- `uid` (Optional[str]): Custom unique ID. Auto-generated if not provided.

**Returns**: `str` - The unique ID assigned to this window

#### `unregister(uid)`

Unregister a window by ID.

```python
success = wm.unregister("wv_12345678")
```

**Parameters**:
- `uid` (str): The window's unique ID

**Returns**: `bool` - True if window was found and removed

#### `get(uid)`

Get a window by ID.

```python
window = wm.get("wv_12345678")
if window:
    window.show()
```

**Parameters**:
- `uid` (str): The window's unique ID

**Returns**: `Optional[WebView]` - The WebView instance, or None if not found

#### `get_active()`

Get the currently active window.

```python
active = wm.get_active()
```

**Returns**: `Optional[WebView]` - The active window, or None

#### `get_active_id()`

Get the ID of the currently active window.

```python
active_id = wm.get_active_id()
```

**Returns**: `Optional[str]` - The active window's ID, or None

#### `set_active(uid)`

Set the active window by ID.

```python
success = wm.set_active("wv_12345678")
```

**Parameters**:
- `uid` (str): The window's unique ID

**Returns**: `bool` - True if window exists and was set as active

#### `get_all()`

Get all registered windows.

```python
windows = wm.get_all()
for window in windows:
    print(window.title)
```

**Returns**: `List[WebView]` - List of all registered WebView instances

#### `get_all_ids()`

Get all registered window IDs.

```python
ids = wm.get_all_ids()
# ['wv_12345678', 'wv_87654321', ...]
```

**Returns**: `List[str]` - List of all window IDs

#### `count()`

Get the number of registered windows.

```python
n = wm.count()
print(f"Managing {n} windows")
```

**Returns**: `int` - Number of registered windows

#### `has(uid)`

Check if a window with the given ID exists.

```python
if wm.has("my_panel"):
    print("Panel exists")
```

**Parameters**:
- `uid` (str): The window's unique ID

**Returns**: `bool` - True if window exists

#### `on_change(callback)`

Register a callback for window changes.

```python
def on_windows_changed():
    print(f"Windows changed: {wm.count()} total")

unsubscribe = wm.on_change(on_windows_changed)

# Later, to stop receiving notifications:
unsubscribe()
```

**Parameters**:
- `callback` (Callable[[], None]): Function to call when windows change

**Returns**: `Callable[[], None]` - Function to unregister the callback

#### `broadcast(event, data)`

Broadcast an event to all windows.

```python
count = wm.broadcast("app:refresh", {"force": True})
print(f"Sent to {count} windows")
```

**Parameters**:
- `event` (str): Event name
- `data` (Any): Event payload

**Returns**: `int` - Number of windows that received the event

#### `close_all()`

Close all registered windows.

```python
closed = wm.close_all()
print(f"Closed {closed} windows")
```

**Returns**: `int` - Number of windows closed

#### `find_by_title(title)`

Find a window by its title.

```python
settings = wm.find_by_title("Settings")
if settings:
    settings.show()
```

**Parameters**:
- `title` (str): Window title to search for

**Returns**: `Optional[WebView]` - The first matching WebView, or None

#### `reset()`

Reset the WindowManager (for testing).

> **Warning**: This will clear all registered windows without closing them.

```python
wm.reset()
```

## Examples

### Managing Multiple Tool Panels

```python
from auroraview import create_webview
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

# Create inspector panel
inspector = create_webview(
    url="http://localhost:3000/inspector",
    title="Inspector"
)
wm.register(inspector, uid="panel_inspector")

# Create outliner panel
outliner = create_webview(
    url="http://localhost:3000/outliner",
    title="Outliner"
)
wm.register(outliner, uid="panel_outliner")

# Broadcast selection change to all panels
def on_selection_changed(items):
    wm.broadcast("selection:changed", {"items": items})
```

### State Synchronization

```python
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

# Track window changes
def sync_state():
    # Update app state when windows change
    window_count = wm.count()
    active_id = wm.get_active_id()
    print(f"Active: {active_id}, Total: {window_count}")

wm.on_change(sync_state)
```

### Targeted Communication

```python
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

# Send event to specific window
def notify_inspector(data):
    inspector = wm.get("panel_inspector")
    if inspector:
        inspector.emit("data:update", data)

# Send to windows by pattern
def notify_panels(event, data):
    for uid in wm.get_all_ids():
        if uid.startswith("panel_"):
            window = wm.get(uid)
            if window:
                window.emit(event, data)
```

## Thread Safety

WindowManager is thread-safe. All operations are protected by internal locks.

```python
import threading
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

def worker(thread_id):
    # Safe to call from any thread
    wm.broadcast("worker:progress", {"thread": thread_id})

threads = [
    threading.Thread(target=worker, args=(i,))
    for i in range(5)
]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

## See Also

- [ReadyEvents API](./ready-events.md) - Lifecycle event management
- [TabContainer API](./tab-container.md) - Multi-tab management
- [Qt Integration Guide](../guide/qt-integration.md) - DCC integration patterns
