# Floating Panels

Create floating tool windows for AI assistants, tool palettes, or overlay interfaces.

## Basic Floating Panel

```python
from auroraview import WebView

webview = WebView.create(
    title="AI Assistant",
    html=panel_html,
    width=320,
    height=400,
    frame=False,         # Frameless window
    transparent=True,    # Transparent background
    always_on_top=True,  # Keep on top
)
webview.show()
```

## Tool Window Mode

Hide from taskbar and Alt+Tab:

```python
webview = WebView.create(
    title="Tool Palette",
    html=palette_html,
    width=200,
    height=600,
    frame=False,
    tool_window=True,    # Hide from taskbar/Alt+Tab (WS_EX_TOOLWINDOW)
)
```

## Owner Mode

Window follows parent minimize/restore:

```python
webview = WebView.create(
    title="Floating Tool",
    html=tool_html,
    parent=parent_hwnd,  # Parent window handle
    mode="owner",        # Follow parent minimize/restore
    frame=False,
    always_on_top=True,
)
```

## Complete Example

```python
from auroraview import WebView

panel_html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            background: rgba(30, 30, 30, 0.95);
            color: white;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            border-radius: 12px;
            overflow: hidden;
        }
        .titlebar {
            height: 32px;
            background: rgba(0, 0, 0, 0.3);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 12px;
            -webkit-app-region: drag;
        }
        .titlebar button {
            -webkit-app-region: no-drag;
            background: none;
            border: none;
            color: white;
            cursor: pointer;
            padding: 4px 8px;
        }
        .content {
            padding: 16px;
        }
    </style>
</head>
<body>
    <div class="titlebar">
        <span>AI Assistant</span>
        <button onclick="auroraview.send_event('close')">✕</button>
    </div>
    <div class="content">
        <p>Ask me anything...</p>
    </div>
</body>
</html>
"""

webview = WebView.create(
    title="AI Assistant",
    html=panel_html,
    width=320,
    height=400,
    frame=False,
    transparent=True,
    always_on_top=True,
    tool_window=True,
)

@webview.on("close")
def handle_close(data):
    webview.close()

webview.show()
```

## Configuration Options

| Option | Description | Effect |
|--------|-------------|--------|
| `frame=False` | Remove window frame | Frameless window |
| `transparent=True` | Enable transparency | See-through background |
| `always_on_top=True` | Keep above other windows | Always visible |
| `tool_window=True` | Tool window style | Hidden from taskbar |
| `mode="owner"` | Owner relationship | Follow parent window |

## Custom Dragging

For frameless windows, use CSS `-webkit-app-region`:

```css
/* Make element draggable */
.titlebar {
    -webkit-app-region: drag;
}

/* Exclude interactive elements */
.titlebar button {
    -webkit-app-region: no-drag;
}
```
