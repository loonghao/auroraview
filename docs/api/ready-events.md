# ReadyEvents API

Lifecycle event system for WebView instances.

## Overview

`ReadyEvents` provides event waiting mechanisms and decorators to ensure WebView operations are executed at the right time. It tracks four lifecycle states:

- **created**: WebView instance created
- **shown**: Window is visible
- **loaded**: Page content loaded
- **bridge_ready**: JS bridge is ready for communication

## Quick Start

```python
from auroraview import create_webview

webview = create_webview(url="https://example.com")

# Wait for page to load
webview.ready_events.wait_loaded(timeout=10)

# Check if bridge is ready
if webview.ready_events.is_bridge_ready():
    webview.emit("ready", {"status": "ok"})
```

## ReadyEvents Class

### Constructor

```python
from auroraview.core.ready_events import ReadyEvents

events = ReadyEvents(webview)
```

**Parameters**:
- `window` (WebView): The WebView instance to track

### Wait Methods

#### `wait_created(timeout=20.0)`

Wait for WebView to be created.

```python
success = events.wait_created(timeout=10.0)
if not success:
    print("WebView creation timed out")
```

**Parameters**:
- `timeout` (float): Maximum time to wait in seconds (default: 20.0)

**Returns**: `bool` - True if event was set, False if timeout occurred

#### `wait_shown(timeout=20.0)`

Wait for window to be shown.

```python
success = events.wait_shown(timeout=10.0)
if success:
    print("Window is now visible")
```

**Parameters**:
- `timeout` (float): Maximum time to wait in seconds (default: 20.0)

**Returns**: `bool` - True if event was set, False if timeout occurred

#### `wait_loaded(timeout=20.0)`

Wait for page to be loaded.

```python
# Wait for page content
if events.wait_loaded(timeout=15.0):
    print("Page loaded successfully")
    webview.eval("initializeApp()")
```

**Parameters**:
- `timeout` (float): Maximum time to wait in seconds (default: 20.0)

**Returns**: `bool` - True if event was set, False if timeout occurred

#### `wait_bridge_ready(timeout=20.0)`

Wait for JS bridge to be ready.

```python
# Wait for JS bridge before calling JS functions
if events.wait_bridge_ready(timeout=10.0):
    webview.call("api.initialize", {})
```

**Parameters**:
- `timeout` (float): Maximum time to wait in seconds (default: 20.0)

**Returns**: `bool` - True if event was set, False if timeout occurred

#### `wait_all(timeout=30.0)`

Wait for all events (created, shown, loaded, bridge_ready).

```python
# Wait for everything to be ready
if events.wait_all(timeout=30.0):
    print("WebView is fully ready")
else:
    print("Timeout waiting for WebView")
```

**Parameters**:
- `timeout` (float): Maximum total time to wait in seconds (default: 30.0)

**Returns**: `bool` - True if all events were set, False if timeout occurred

### Set Methods

These methods are typically called internally by WebView, but can be used for testing.

#### `set_created()`

Mark WebView as created.

```python
events.set_created()
```

#### `set_shown()`

Mark window as shown.

```python
events.set_shown()
```

#### `set_loaded()`

Mark page as loaded.

```python
events.set_loaded()
```

#### `set_bridge_ready()`

Mark JS bridge as ready.

```python
events.set_bridge_ready()
```

### Check Methods

#### `is_created()`

Check if WebView is created.

```python
if events.is_created():
    print("WebView exists")
```

**Returns**: `bool` - True if created event is set

#### `is_shown()`

Check if window is shown.

```python
if events.is_shown():
    print("Window is visible")
```

**Returns**: `bool` - True if shown event is set

#### `is_loaded()`

Check if page is loaded.

```python
if events.is_loaded():
    print("Page is loaded")
```

**Returns**: `bool` - True if loaded event is set

#### `is_bridge_ready()`

Check if JS bridge is ready.

```python
if events.is_bridge_ready():
    print("Bridge is ready")
```

**Returns**: `bool` - True if bridge_ready event is set

#### `is_ready()`

Check if all events are set.

```python
if events.is_ready():
    print("WebView is fully ready")
```

**Returns**: `bool` - True if all events are set

### Utility Methods

#### `status()`

Get status of all events.

```python
status = events.status()
# {
#     "created": True,
#     "shown": True,
#     "loaded": False,
#     "bridge_ready": False
# }
```

**Returns**: `dict` - Dict with event names and their status

#### `reset()`

Reset all events to unset state.

```python
events.reset()
```

## Decorators

### `@require_created`

Ensure WebView is created before executing method.

```python
from auroraview.core.ready_events import require_created

class MyWebView(WebView):
    @require_created
    def setup_ui(self):
        # Called only after WebView is created
        self.eval("setupUI()")
```

### `@require_shown`

Ensure window is shown before executing method.

```python
from auroraview.core.ready_events import require_shown

class MyWebView(WebView):
    @require_shown
    def capture_screenshot(self):
        # Called only after window is visible
        return self.eval("captureScreen()")
```

### `@require_loaded`

Ensure page is loaded before executing method.

```python
from auroraview.core.ready_events import require_loaded

class MyWebView(WebView):
    @require_loaded
    def initialize_app(self):
        # Called only after page is loaded
        self.eval("app.init()")
```

### `@require_bridge_ready`

Ensure JS bridge is ready before executing method.

```python
from auroraview.core.ready_events import require_bridge_ready

class MyWebView(WebView):
    @require_bridge_ready
    def call_api(self, method, params):
        # Called only after JS bridge is ready
        return self.call(method, params)
```

### `@require_ready`

Ensure WebView is fully ready (all events set) before executing method.

```python
from auroraview.core.ready_events import require_ready

class MyWebView(WebView):
    @require_ready
    def start_interaction(self):
        # Called only when everything is ready
        self.emit("app:started", {})
```

## Examples

### Waiting for Page Load

```python
from auroraview import create_webview

webview = create_webview(url="https://example.com")

# Start showing (non-blocking)
webview.show(wait=False)

# Wait for page to load
if webview.ready_events.wait_loaded(timeout=10):
    # Page is ready, safe to interact
    webview.eval("document.querySelector('#app').click()")
else:
    print("Page load timed out")
```

### Custom WebView with Decorators

```python
from auroraview import WebView
from auroraview.core.ready_events import require_bridge_ready, require_loaded

class AppWebView(WebView):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._data = {}
    
    @require_loaded
    def set_content(self, html):
        """Set inner HTML after page is loaded."""
        self.eval(f"document.body.innerHTML = {repr(html)}")
    
    @require_bridge_ready
    def fetch_data(self, endpoint):
        """Fetch data through JS bridge."""
        return self.call("api.fetch", {"endpoint": endpoint})
```

### Monitoring Lifecycle States

```python
from auroraview import create_webview
import threading

webview = create_webview(url="https://example.com")

def monitor_state():
    events = webview.ready_events
    
    print("Waiting for created...")
    events.wait_created()
    print("✓ Created")
    
    print("Waiting for shown...")
    events.wait_shown()
    print("✓ Shown")
    
    print("Waiting for loaded...")
    events.wait_loaded()
    print("✓ Loaded")
    
    print("Waiting for bridge_ready...")
    events.wait_bridge_ready()
    print("✓ Bridge Ready")
    
    print("WebView is fully ready!")

# Monitor in background thread
monitor = threading.Thread(target=monitor_state)
monitor.start()

# Show the webview
webview.show()
monitor.join()
```

### Conditional Execution

```python
from auroraview import create_webview

webview = create_webview(url="https://example.com")
webview.show(wait=False)

# Check state without blocking
if webview.ready_events.is_bridge_ready():
    # Bridge is ready, can use immediately
    webview.emit("quick:update", {"fast": True})
else:
    # Wait for bridge
    webview.ready_events.wait_bridge_ready()
    webview.emit("delayed:update", {"waited": True})
```

## Thread Safety

All ReadyEvents methods are thread-safe. The underlying `threading.Event` objects handle synchronization.

```python
import threading
from auroraview import create_webview

webview = create_webview(url="https://example.com")

def worker(name):
    # Safe to wait from any thread
    webview.ready_events.wait_loaded()
    print(f"Worker {name}: page loaded")

# Multiple threads can wait
threads = [
    threading.Thread(target=worker, args=(f"T{i}",))
    for i in range(3)
]
for t in threads:
    t.start()

webview.show(wait=False)

for t in threads:
    t.join()
```

## See Also

- [WindowManager API](./window-manager.md) - Global window registry
- [WebView API](./webview.md) - Core WebView API
- [Multi-Window Guide](../guide/multi-window.md) - Usage patterns
