# Window Effects

AuroraView provides advanced window effects for creating modern, visually appealing applications:

- **Click-through**: Allow mouse events to pass through transparent areas
- **Background Blur**: Apply native Windows blur effects (Acrylic, Mica)

## Click-Through Windows

Click-through mode allows you to create transparent overlay windows where mouse events pass through to underlying windows, while keeping specific interactive regions active.

### Use Cases

- Floating overlay panels
- HUD displays
- Annotation tools
- Desktop widgets

### Basic Usage

```python
from auroraview import AuroraView

# Create a transparent window
webview = AuroraView(
    html="""
    <html>
    <body style="background: transparent;">
        <button data-interactive style="position: fixed; top: 10px; left: 10px;">
            Click Me
        </button>
        <div style="position: fixed; bottom: 10px; right: 10px; opacity: 0.5;">
            This area passes through clicks
        </div>
    </body>
    </html>
    """,
    transparent=True,
    decorations=False,
)

# Enable click-through mode
webview.enable_click_through()

# Define interactive regions (areas that receive mouse events)
webview.update_interactive_regions([
    {"x": 10, "y": 10, "width": 100, "height": 40}  # Button area
])

webview.show()
```

### JavaScript Integration

The `data-interactive` attribute automatically tracks element positions:

```html
<!-- These elements will be interactive -->
<button data-interactive>Button 1</button>
<div data-interactive class="toolbar">Toolbar</div>

<!-- These elements will pass through clicks -->
<div class="overlay">Transparent overlay</div>
```

```javascript
// The SDK automatically monitors data-interactive elements
// and sends their positions to the native layer

// Manual update (if needed)
window.auroraview.interactive.update();

// Get current regions
const regions = window.auroraview.interactive.getRegions();

// Disable tracking
window.auroraview.interactive.setEnabled(false);
```

### Python API

```python
# Enable click-through
webview.enable_click_through()

# Disable click-through
webview.disable_click_through()

# Check if enabled
is_enabled = webview.is_click_through_enabled()

# Update interactive regions manually
webview.update_interactive_regions([
    {"x": 0, "y": 0, "width": 200, "height": 50},
    {"x": 300, "y": 100, "width": 150, "height": 80},
])

# Get current regions
regions = webview.get_interactive_regions()
```

## Background Blur (Vibrancy)

AuroraView supports Windows native background blur effects:

| Effect | Windows 10 | Windows 11 | Description |
|--------|------------|------------|-------------|
| Blur | ✅ 1809+ | ✅ | Basic blur behind window |
| Acrylic | ✅ 1809+ | ✅ | Semi-transparent blur with noise |
| Mica | ❌ | ✅ 22000+ | Desktop wallpaper sampling |
| Mica Alt | ❌ | ✅ 22523+ | Stronger Mica variant |

### Basic Usage

```python
from auroraview import AuroraView

webview = AuroraView(
    html="""
    <html>
    <body style="background: rgba(30, 30, 30, 0.5);">
        <h1>Blurred Background</h1>
    </body>
    </html>
    """,
    transparent=True,
    decorations=False,
)

# Apply blur effect
webview.apply_blur()

# Or with custom tint color (RGBA)
webview.apply_blur((30, 30, 30, 200))

webview.show()
```

### Effect Types

#### Blur

Basic blur effect, works on Windows 10 1809+ and Windows 11.

```python
# Apply blur
webview.apply_blur()

# Blur with dark tint
webview.apply_blur((30, 30, 30, 200))

# Clear blur
webview.clear_blur()
```

#### Acrylic

Semi-transparent blur with noise texture, similar to Windows Fluent Design.

```python
# Apply acrylic
webview.apply_acrylic()

# Acrylic with custom color
webview.apply_acrylic((30, 30, 30, 150))

# Clear acrylic
webview.clear_acrylic()
```

::: warning
Acrylic may have performance issues during window resize on some Windows versions.
:::

#### Mica

Windows 11 material that samples the desktop wallpaper for a personalized backdrop.

```python
# Apply mica (light mode)
webview.apply_mica(dark=False)

# Apply mica (dark mode)
webview.apply_mica(dark=True)

# Clear mica
webview.clear_mica()
```

#### Mica Alt

Stronger variant of Mica, typically used for tabbed windows.

```python
# Apply mica alt
webview.apply_mica_alt(dark=True)

# Clear mica alt
webview.clear_mica_alt()
```

### CSS Integration

For partial region blur, use CSS `backdrop-filter`:

```css
.blurred-panel {
    background: rgba(30, 30, 30, 0.5);
    backdrop-filter: blur(20px);
    border-radius: 10px;
}
```

## Platform Support

| Feature | Windows 10 | Windows 11 | macOS | Linux |
|---------|------------|------------|-------|-------|
| Click-through | ✅ | ✅ | ❌ | ❌ |
| Blur | ✅ 1809+ | ✅ | ❌ | ❌ |
| Acrylic | ✅ 1809+ | ✅ | ❌ | ❌ |
| Mica | ❌ | ✅ 22000+ | ❌ | ❌ |
| Mica Alt | ❌ | ✅ 22523+ | ❌ | ❌ |

## Troubleshooting

### Click-through not working

1. **Check if click-through is enabled**:
   ```python
   print(webview.is_click_through_enabled())  # Should be True
   ```

2. **Verify interactive regions are set**:
   ```python
   regions = webview.get_interactive_regions()
   print(f"Interactive regions: {regions}")
   ```

3. **Ensure window is transparent**:
   ```python
   webview = WebView(transparent=True, ...)
   ```

### Blur effects not visible

1. **Window must be transparent**:
   ```python
   webview = WebView(transparent=True, ...)
   ```

2. **HTML background must be semi-transparent**:
   ```html
   <body style="background: rgba(30, 30, 30, 0.5);">
   ```

3. **Check Windows version compatibility** (see Platform Support table)

### "HWND not available" error

This error occurs when trying to use window effects before the window is fully initialized. Call effect methods after `show()`:

```python
webview = WebView(...)
webview.show()  # Window is now initialized
webview.apply_blur()  # Now safe to call
```

### API calls return errors in JavaScript

When calling window effect APIs from JavaScript via `auroraview.api.*`, use object parameters:

```javascript
// Correct - use object parameter
await auroraview.api.apply_blur({color: [30, 30, 30, 200]});
await auroraview.api.apply_mica({dark: true});
await auroraview.api.update_interactive_regions({regions: [...]});

// Incorrect - array parameters will be expanded
await auroraview.api.apply_blur([30, 30, 30, 200]);  // Error!
```

## Known Limitations

1. **Windows Only**: Click-through and vibrancy effects are only supported on Windows 10/11. macOS and Linux are not supported.

2. **Mica requires Windows 11**: Mica and Mica Alt effects only work on Windows 11 build 22000+.

3. **Acrylic performance**: Acrylic effect may cause performance issues during window resize on some Windows versions.

4. **Click-through and DCC integration**: When using click-through in DCC hosts (Maya, 3ds Max, etc.), interactive regions must be manually updated when the window is moved or resized.

5. **Transparent window artifacts**: On some systems, transparent windows may show rendering artifacts. Use `extend_frame_into_client_area` for better results.

## DCC Integration Notes

When using window effects in DCC applications (Maya, 3ds Max, Houdini, etc.), there are some important considerations.

### Integration Modes

AuroraView provides three integration modes for different scenarios:

| Mode | Class | Description | Use Case |
|------|-------|-------------|----------|
| **Desktop** | `WebView` + `show()` | Independent window with own event loop | Standalone tools, desktop apps |
| **Native (HWND)** | `WebView` + `parent=hwnd` | Embedded via HWND, no Qt dependency | Blender, Unreal Engine, non-Qt apps |
| **Qt** | `QtWebView` | Embedded as Qt widget child | Maya, Houdini, Nuke, 3ds Max |

### Effect Support by Mode

| Feature | Desktop Mode | Native Mode (HWND) | Qt Mode |
|---------|-------------|-------------------|---------|
| Click-through | ✅ Full | ✅ Full | ⚠️ Limited |
| Blur/Acrylic | ✅ Full | ✅ Full | ⚠️ May conflict |
| Mica | ✅ Full | ✅ Full | ❌ Not recommended |
| Transparent window | ✅ Full | ✅ Full | ⚠️ Qt-dependent |

::: tip Mode Selection
- **Desktop Mode**: Best for standalone applications and development/testing
- **Native Mode (HWND)**: Best for non-Qt DCC (Blender, Unreal) with full effect support
- **Qt Mode**: Best for Qt-based DCC (Maya, Houdini, Nuke) where docking is needed
:::

### Qt Mode Limitations

When embedding AuroraView in DCC applications via Qt:

1. **Click-through**: Works but requires careful management of interactive regions when the parent Qt widget is moved or resized.

2. **Vibrancy effects**: May conflict with Qt's own window composition. Test thoroughly in your target DCC.

3. **Window handle access**: Effects require direct HWND access, which is available through `webview._core` but may have timing issues during Qt widget lifecycle.

### Recommended Usage in DCC

```python
from auroraview import WebView
from auroraview.qt import QtWebView

# Desktop Mode - Independent window with full effect support
def create_desktop_panel():
    webview = WebView(
        title="Desktop Panel",
        transparent=True,
        decorations=False,
        always_on_top=True,
        tool_window=True,
    )
    webview.enable_click_through()
    webview.apply_acrylic((30, 30, 30, 180))
    webview.show()  # Blocking, owns event loop
    return webview

# Native Mode (HWND) - For Blender, Unreal, etc. Full effect support
def create_native_panel(parent_hwnd):
    webview = WebView(
        title="Native Panel",
        parent=parent_hwnd,  # HWND from non-Qt app
        transparent=True,
        decorations=False,
    )
    webview.enable_click_through()
    webview.apply_acrylic((30, 30, 30, 180))
    return webview

# Qt Mode - For Maya, Houdini, Nuke. Limited effect support
def create_qt_panel(parent_widget):
    qt_webview = QtWebView(parent=parent_widget)
    # Note: Effects may not work as expected in Qt mode
    # Use CSS backdrop-filter for blur effects instead
    return qt_webview
```

### Best Practices for DCC

1. **Use Desktop or Native mode for overlay panels** that need click-through or vibrancy effects.

2. **Use Qt mode for docked panels** where visual effects are less important but Qt integration is needed.

3. **Test on target DCC versions** as Qt versions vary between DCCs.

4. **Provide fallback styling** for cases where effects don't work.

5. **Handle window lifecycle events** to properly clean up effects when panels are closed.

## Best Practices

1. **Always set `transparent=True`** when using blur effects
2. **Use `decorations=False`** for custom-shaped windows
3. **Combine with `tool_window=True`** to hide from taskbar
4. **Test on target Windows versions** as effects vary
5. **Provide fallback styling** for unsupported platforms
6. **Use object parameters** when calling APIs from JavaScript

## Example: Floating Panel

```python
from auroraview import AuroraView

html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        body {
            margin: 0;
            background: transparent;
            font-family: 'Segoe UI', sans-serif;
        }
        .panel {
            background: rgba(30, 30, 30, 0.7);
            border-radius: 12px;
            padding: 20px;
            margin: 10px;
            color: white;
        }
        button {
            background: #0078d4;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 6px;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <div class="panel" data-interactive>
        <h2>Floating Panel</h2>
        <p>This panel has a blurred background</p>
        <button data-interactive>Action</button>
    </div>
</body>
</html>
"""

webview = AuroraView(
    html=html,
    width=300,
    height=200,
    transparent=True,
    decorations=False,
    always_on_top=True,
    tool_window=True,
)

# Enable click-through for transparent areas
webview.enable_click_through()

# Apply acrylic blur
webview.apply_acrylic((30, 30, 30, 180))

webview.show()
```
