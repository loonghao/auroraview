# WebView API

The `WebView` class is the core component for creating web-based UI.

## Constructor

```python
WebView.create(
    title: str = "AuroraView",
    url: str = None,
    html: str = None,
    width: int = 800,
    height: int = 600,
    resizable: bool = True,
    frame: bool = True,
    debug: bool = False,
    transparent: bool = False,
    always_on_top: bool = False,
    context_menu: bool = True,
    icon: str = None,
    asset_root: str = None,
    allow_file_protocol: bool = False,
    parent_hwnd: int = None,
    parent_mode: str = None,
    tool_window: bool = False,
) -> WebView
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `title` | `str` | `"AuroraView"` | Window title |
| `url` | `str` | `None` | URL to load |
| `html` | `str` | `None` | HTML content to load |
| `width` | `int` | `800` | Window width |
| `height` | `int` | `600` | Window height |
| `resizable` | `bool` | `True` | Allow window resizing |
| `frame` | `bool` | `True` | Show window frame |
| `debug` | `bool` | `False` | Enable DevTools |
| `transparent` | `bool` | `False` | Transparent background |
| `always_on_top` | `bool` | `False` | Keep window on top |
| `context_menu` | `bool` | `True` | Enable native context menu |
| `icon` | `str` | `None` | Window icon path |
| `asset_root` | `str` | `None` | Root directory for assets |
| `allow_file_protocol` | `bool` | `False` | Allow file:// URLs |
| `parent_hwnd` | `int` | `None` | Parent window handle |
| `parent_mode` | `str` | `None` | Parent relationship mode |
| `tool_window` | `bool` | `False` | Tool window style |

## Methods

### show()

Display the WebView window.

```python
webview.show()
```

### hide()

Hide the WebView window.

```python
webview.hide()
```

### close()

Close the WebView window.

```python
webview.close()
```

### load_url(url: str)

Load a URL.

```python
webview.load_url("https://example.com")
```

### load_html(html: str)

Load HTML content.

```python
webview.load_html("<h1>Hello</h1>")
```

### eval_js(script: str) -> Any

Execute JavaScript and return the result.

```python
result = webview.eval_js("document.title")
```

### emit(event: str, data: Any)

Emit an event to JavaScript.

```python
webview.emit("update", {"value": 42})
```

### bind_call(name: str, func: Callable)

Bind a Python function for JavaScript calls.

```python
@webview.bind_call("api.getData")
def get_data():
    return {"items": [1, 2, 3]}
```

### on(event: str)

Register an event handler.

```python
@webview.on("button_clicked")
def handle_click(data):
    print(f"Button clicked: {data}")
```

### bind_api(obj: object, namespace: str = "api")

Bind all public methods of an object.

```python
class MyAPI:
    def get_data(self):
        return {"value": 42}

webview.bind_api(MyAPI())
```

### register_protocol(scheme: str, handler: Callable)

Register a custom protocol handler.

```python
def handle_custom(uri: str) -> dict:
    return {"data": b"content", "mime_type": "text/plain", "status": 200}

webview.register_protocol("custom", handle_custom)
```

## Navigation Methods

### go_back()

Navigate back in history.

```python
webview.go_back()
```

### go_forward()

Navigate forward in history.

```python
webview.go_forward()
```

### reload()

Reload the current page.

```python
webview.reload()
```

### stop()

Stop loading.

```python
webview.stop()
```

### can_go_back() -> bool

Check if back navigation is possible.

```python
if webview.can_go_back():
    webview.go_back()
```

### can_go_forward() -> bool

Check if forward navigation is possible.

```python
if webview.can_go_forward():
    webview.go_forward()
```

## Window Control Methods

### resize(width: int, height: int)

Resize the window.

```python
webview.resize(1024, 768)
```

### move(x: int, y: int)

Move the window.

```python
webview.move(100, 100)
```

### minimize()

Minimize the window.

```python
webview.minimize()
```

### maximize()

Maximize the window.

```python
webview.maximize()
```

### restore()

Restore the window from minimized/maximized state.

```python
webview.restore()
```

### toggle_fullscreen()

Toggle fullscreen mode.

```python
webview.toggle_fullscreen()
```

### focus()

Focus the window.

```python
webview.focus()
```

### open_devtools()

Open DevTools (requires `debug=True`).

```python
webview.open_devtools()
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `width` | `int` | Window width (read-only) |
| `height` | `int` | Window height (read-only) |
| `x` | `int` | Window X position (read-only) |
| `y` | `int` | Window Y position (read-only) |
| `state` | `dict` | Shared state object |

## Event Decorators

### @on_shown

Window shown event.

```python
@webview.on_shown
def on_shown(data):
    print("Window shown")
```

### @on_focused

Window focused event.

```python
@webview.on_focused
def on_focused(data):
    print("Window focused")
```

### @on_blurred

Window lost focus event.

```python
@webview.on_blurred
def on_blurred(data):
    print("Window blurred")
```

### @on_resized

Window resized event.

```python
@webview.on_resized
def on_resized(data):
    print(f"Resized to {data.width}x{data.height}")
```

### @on_moved

Window moved event.

```python
@webview.on_moved
def on_moved(data):
    print(f"Moved to ({data.x}, {data.y})")
```

### @on_closing

Window closing event. Return `False` to cancel.

```python
@webview.on_closing
def on_closing(data):
    return True  # Allow close
```
