# API Reference

This section provides detailed API documentation for AuroraView.

## Core Classes

### WebView

The base WebView class for creating web-based UI.

```python
from auroraview import WebView

webview = WebView.create(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

[Full WebView API →](/api/webview)

### QtWebView

Qt widget wrapper for DCC integration.

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=parent_widget,
    url="http://localhost:3000"
)
```

[Full QtWebView API →](/api/qt-webview)

### AuroraView

High-level wrapper with HWND access.

```python
from auroraview import AuroraView

webview = AuroraView(
    url="http://localhost:3000",
    api=MyAPI()
)
```

[Full AuroraView API →](/api/auroraview)

## Convenience Functions

### run_desktop

Launch a standalone desktop application:

```python
from auroraview import run_desktop

run_desktop(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

### run_standalone

Alias for `run_desktop`:

```python
from auroraview import run_standalone

run_standalone(
    title="My App",
    html="<h1>Hello</h1>"
)
```

## Utility Functions

### path_to_file_url

Convert local path to file:// URL:

```python
from auroraview import path_to_file_url

url = path_to_file_url("C:/path/to/file.html")
# Returns: file:///C:/path/to/file.html
```

## Type Definitions

### WindowEventData

```python
from auroraview.core.events import WindowEventData

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"Size: {data.width}x{data.height}")
    print(f"Position: ({data.x}, {data.y})")
```

### Signal

Qt-like signal system:

```python
from auroraview import Signal

class MyTool(WebView):
    selection_changed = Signal(list)
    progress_updated = Signal(int, str)
```

## JavaScript API

### auroraview Object

Available in the browser context:

```javascript
// Call Python methods
const result = await auroraview.call('api.method', { param: 'value' });

// Send events to Python
auroraview.send_event('event_name', { data: 'value' });

// Listen for Python events
auroraview.on('event_name', (data) => {
    console.log(data);
});

// Access shared state
auroraview.state.key = 'value';
```
