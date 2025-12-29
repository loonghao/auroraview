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

## Transparent Floating Button

Create a truly transparent circular button:

```python
from auroraview import AuroraView

class TriggerButton(AuroraView):
    def __init__(self):
        super().__init__(
            html=BUTTON_HTML,
            width=48,
            height=48,
            frame=False,
            transparent=True,
            undecorated_shadow=False,  # Default: no shadow (set True to force-enable)

            always_on_top=True,
            tool_window=True,
        )
```

**Key Parameters for Transparent Windows:**
- `frame=False` - Frameless window
- `transparent=True` - Transparent background
- `undecorated_shadow=False` - Default: **no shadow** for frameless windows; set to `True` to force-enable

- `tool_window=True` - Hide from taskbar/Alt+Tab

## Floating Toolbar with GSAP

Create an expandable toolbar with smooth animations:

```python
from auroraview import AuroraView

TOOLBAR_HTML = """
<!DOCTYPE html>
<html>
<head>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
    <style>
        .toolbar { display: flex; gap: 8px; }
        .tool-item { 
            width: 40px; height: 40px;
            opacity: 0; transform: scale(0.5);
        }
    </style>
</head>
<body>
    <button class="trigger" onclick="toggleToolbar()">+</button>
    <div class="toolbar" id="toolbar">
        <div class="tool-item">Maya</div>
        <div class="tool-item">Blender</div>
    </div>
    <script>
        let expanded = false;
        function toggleToolbar() {
            expanded = !expanded;
            const items = document.querySelectorAll('.tool-item');
            if (expanded) {
                gsap.to(items, {
                    opacity: 1,
                    scale: 1,
                    duration: 0.3,
                    stagger: 0.05,
                    ease: 'back.out(1.7)'
                });
            } else {
                gsap.to(items, {
                    opacity: 0,
                    scale: 0.5,
                    duration: 0.2,
                    stagger: 0.02,
                });
            }
        }
    </script>
</body>
</html>
"""

class FloatingToolbar(AuroraView):
    def __init__(self):
        super().__init__(
            html=TOOLBAR_HTML,
            width=64,
            height=64,
            frame=False,
            transparent=True,
            undecorated_shadow=False,
            always_on_top=True,
            tool_window=True,
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
| `undecorated_shadow=False` | Disable shadow | Truly transparent window |
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

## Related Examples

- `examples/floating_panel_demo.py` - Basic floating panel
- `examples/floating_toolbar_demo.py` - Expandable toolbar with GSAP
- `examples/radial_menu_demo.py` - Circular menu
- `examples/dock_launcher_demo.py` - macOS-style dock
- `examples/logo_button_demo.py` - Logo button trigger
