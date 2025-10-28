# Maya Embedded WebView Integration Guide

## Overview

This guide explains how to properly integrate AuroraView WebView into Maya using **embedded mode**. This is the recommended approach for creating professional Maya tools.

## Two Integration Approaches

### 1. Standalone Mode (Simple, Non-Blocking)
- WebView runs in a separate window
- Maya main thread is not blocked
- Good for quick prototypes
- **File:** `examples/maya_quick_test.py`

**Pros:**
- Simple to implement
- No need to get window handles
- Works immediately

**Cons:**
- Separate window (not integrated into Maya UI)
- Can be moved outside Maya window
- Not part of Maya's workspace system

### 2. Embedded Mode (Professional, Integrated)
- WebView is embedded directly into Maya's UI
- Appears as a dockable panel
- Part of Maya's workspace system
- **Files:** `examples/maya_embedded_integration.py`, `examples/maya_workspace_control.py`

**Pros:**
- Fully integrated into Maya UI
- Can be docked like any Maya panel
- Professional appearance
- Workspace is saved/restored

**Cons:**
- Requires getting Maya's window handle
- More complex setup
- Platform-specific (Windows HWND)

## Embedded Mode Implementation

### Step 1: Get Maya's Main Window Handle

```python
import maya.OpenMayaUI as omui

def get_maya_main_window_hwnd():
    """Get the HWND of Maya's main window."""
    main_window_ptr = omui.MQtUtil.mainWindow()
    if main_window_ptr is None:
        raise RuntimeError("Could not get Maya main window pointer")
    hwnd = int(main_window_ptr)
    return hwnd
```

### Step 2: Create WebView and Load Content

```python
from auroraview import WebView

webview = WebView(
    title="My Maya Tool",
    width=600,
    height=500
)

# Load HTML content
webview.load_html(html_content)
```

### Step 3: Create Embedded WebView

```python
hwnd = get_maya_main_window_hwnd()
webview._core.create_embedded(hwnd, 600, 500)
```

### Step 4: Register Event Handlers

```python
@webview.on("my_event")
def handle_my_event(data):
    # Handle event from WebView
    result = do_something(data)
    webview.emit("response", result)
```

## Complete Example

See `examples/maya_workspace_control.py` for a complete, production-ready example.

### Key Features:
- ✓ Gets Maya's main window handle
- ✓ Creates embedded WebView
- ✓ Registers event handlers
- ✓ Communicates with Maya via events
- ✓ Creates/deletes objects
- ✓ Displays scene information

### Usage:

1. Open Maya 2022
2. Open Script Editor (Windows > General Editors > Script Editor)
3. Switch to Python tab
4. Copy the entire script from `examples/maya_workspace_control.py`
5. Paste into the Python tab
6. Click "Execute" or press Ctrl+Enter
7. The WebView will appear as an embedded panel!

## Event Communication

### From WebView to Maya

```javascript
// In HTML/JavaScript
window.dispatchEvent(new CustomEvent('my_event', {
    detail: { data: 'value' }
}));
```

### From Maya to WebView

```python
# In Python
webview.emit("response", {"status": "ok"})
```

### Receiving Events in Python

```python
@webview.on("my_event")
def handle_my_event(data):
    print(f"Received: {data}")
```

### Receiving Events in JavaScript

```javascript
window.addEventListener('response', (e) => {
    console.log('Received:', e.detail);
});
```

## Common Tasks

### Create Objects

```python
@webview.on("create_cube")
def handle_create_cube(data):
    size = float(data.get("size", 1.0))
    cube = cmds.polyCube(w=size, h=size, d=size)
    webview.emit("status", {"message": f"Created: {cube[0]}"})
```

### Get Scene Information

```python
@webview.on("get_info")
def handle_get_info(data):
    nodes = cmds.ls()
    meshes = cmds.ls(type="mesh")
    webview.emit("info_response", {
        "nodes": len(nodes),
        "meshes": len(meshes)
    })
```

### Delete Selected Objects

```python
@webview.on("delete_selected")
def handle_delete_selected(data):
    selected = cmds.ls(selection=True)
    if selected:
        cmds.delete(selected)
        webview.emit("status", {"message": f"Deleted {len(selected)} objects"})
```

## Troubleshooting

### WebView doesn't appear

1. Check that you're using `create_embedded()` not `show()`
2. Verify the HWND is correct
3. Check the Script Editor output for errors
4. Make sure AuroraView is installed: `pip install auroraview`

### "Could not get Maya main window pointer"

- This usually means Maya's UI is not fully initialized
- Try running the script after Maya has fully loaded
- Make sure you're using Maya 2022 or later

### Events not working

1. Check that event names match between Python and JavaScript
2. Verify the event handler is registered before emitting
3. Check the Script Editor output for errors
4. Use `logger.info()` to debug

## Platform Support

Currently, embedded mode is supported on:
- ✓ Windows (using HWND)
- ⚠️ macOS (requires native window handle)
- ⚠️ Linux (requires native window handle)

## Performance Tips

1. **Minimize HTML complexity** - Keep UI simple for better performance
2. **Use event batching** - Send multiple updates in one event
3. **Cache scene data** - Don't query Maya every frame
4. **Optimize JavaScript** - Use efficient event handlers

## Next Steps

1. Try the embedded example: `examples/maya_workspace_control.py`
2. Customize the HTML UI for your needs
3. Add more event handlers for your specific workflow
4. Save your workspace to persist the panel layout

## See Also

- `examples/maya_quick_test.py` - Standalone mode example
- `examples/maya_embedded_integration.py` - Basic embedded example
- `docs/ASYNC_DCC_INTEGRATION.md` - Async integration guide
- `MAYA_QUICK_START.md` - Quick start guide

