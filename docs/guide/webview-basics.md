# WebView Basics

This guide covers the fundamental concepts of working with AuroraView's WebView.

## Creating a WebView

### Basic Creation

```python
from auroraview import WebView

webview = WebView.create(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768
)
webview.show()
```

### With HTML Content

```python
html = """
<!DOCTYPE html>
<html>
<body>
    <h1>Hello World!</h1>
</body>
</html>
"""

webview = WebView.create("My App", html=html)
webview.show()
```

## Window Configuration

### Size and Position

```python
webview = WebView.create(
    title="My App",
    width=1024,
    height=768,
    resizable=True,
)
webview.show()

# Runtime control
webview.resize(1280, 720)
webview.move(100, 100)
```

### Window Styles

```python
webview = WebView.create(
    title="My App",
    frame=True,          # Show window frame
    resizable=True,      # Allow resizing
    always_on_top=False, # Normal z-order
    transparent=False,   # Opaque background
)
```

### Frameless Window

```python
webview = WebView.create(
    title="Floating Panel",
    frame=False,         # No title bar
    transparent=True,    # Transparent background
    always_on_top=True,  # Keep on top
)
```

## Window Events

### Event Handlers

```python
from auroraview.core.events import WindowEventData

@webview.on_shown
def on_shown(data: WindowEventData):
    print("Window is now visible")

@webview.on_focused
def on_focused(data: WindowEventData):
    print("Window gained focus")

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"Window resized to {data.width}x{data.height}")

@webview.on_closing
def on_closing(data: WindowEventData):
    # Return True to allow close, False to cancel
    return True
```

### Window Control

```python
webview.minimize()
webview.maximize()
webview.restore()
webview.toggle_fullscreen()
webview.focus()
webview.hide()
```

## Navigation

### Loading Content

```python
# Load URL
webview.load_url("https://example.com")

# Load HTML
webview.load_html("<h1>Hello</h1>")
```

### Navigation Control

```python
webview.go_back()
webview.go_forward()
webview.reload()
webview.stop()

# Check navigation state
if webview.can_go_back():
    webview.go_back()
```

## JavaScript Execution

### Execute JavaScript

```python
# Fire and forget
webview.eval_js("console.log('Hello from Python')")

# With return value
result = webview.eval_js("document.title")
```

### Inject Scripts

```python
webview = WebView.create(
    title="My App",
    url="https://example.com",
)

# Inject after page load
webview.eval_js("""
    const btn = document.createElement('button');
    btn.textContent = 'Custom Button';
    document.body.appendChild(btn);
""")
```

## Custom Icon

```python
webview = WebView.create(
    title="My App",
    icon="path/to/icon.png"  # PNG, ICO, JPEG, BMP, GIF
)
```

**Icon Requirements:**

| Property | Recommendation |
|----------|----------------|
| Format | PNG (recommended), ICO, JPEG, BMP, GIF |
| Size | 32×32 (taskbar), 64×64 (alt-tab), 256×256 (high-DPI) |
| Color Depth | 32-bit RGBA for transparency |

## Debug Mode

```python
webview = WebView.create(
    title="My App",
    debug=True,  # Enable DevTools
)

# Open DevTools programmatically
webview.open_devtools()
```

## Context Menu

```python
# Disable native context menu (for custom menus)
webview = WebView.create(
    title="My App",
    context_menu=False,
)
```
