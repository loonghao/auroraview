# Maya Integration Guide

## 常见错误诊断

### 错误：`ImportError: cannot import name 'QtWebView' from 'auroraview'`

**症状**：
```python
from auroraview import QtWebView
# ImportError: cannot import name 'QtWebView' from 'auroraview'
```

**原因**：Qt 后端依赖未安装（需要 `qtpy` 和 Qt 绑定）

**诊断**：
```python
import auroraview
print(f"Qt backend available: {auroraview._HAS_QT}")
if not auroraview._HAS_QT:
    print(f"Qt import error: {auroraview._QT_IMPORT_ERROR}")
```

**解决方案 1**：安装 Qt 依赖（如果你想使用 Qt 后端）
```bash
# Windows
"C:\Program Files\Autodesk\Maya2022\bin\mayapy.exe" -m pip install auroraview[qt]

# macOS
/Applications/Autodesk/maya2022/Maya.app/Contents/bin/mayapy -m pip install auroraview[qt]

# Linux
/usr/autodesk/maya2022/bin/mayapy -m pip install auroraview[qt]
```

**解决方案 2**：使用 Native 后端（推荐，无需额外依赖）
```python
from auroraview import NativeWebView  # 而不是 QtWebView
import maya.OpenMayaUI as omui

maya_hwnd = int(omui.MQtUtil.mainWindow())
webview = NativeWebView.embedded(
    parent_hwnd=maya_hwnd,
    title="My Tool",
    mode="owner"
)
webview.show()
```

---

## Threading Model

### [ERROR] WRONG: Using `show_async()`

```python
# DON'T DO THIS - Will cause Maya to freeze!
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="owner")
webview.load_html(html)
webview.show_async()  # [ERROR] Creates window in background thread
```

**Why this fails:**
1. `show_async()` creates the WebView in a **background thread**
2. The window is parented to Maya's main window (via `parent_hwnd`)
3. **Windows GUI thread affinity**: Child/owned windows must be created in the same thread as their parent
4. Background thread creates the window, but Maya's main thread can't properly handle its messages
5. Result: **Maya freezes**

### [OK] CORRECT: Using `show()` with scriptJob

```python
# CORRECT PATTERN
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="owner")
webview.load_html(html)

# Store in __main__ for scriptJob access
import __main__
__main__.my_webview = webview

# Create scriptJob to process events
def process_events():
    if hasattr(__main__, 'my_webview'):
        should_close = __main__.my_webview._core.process_events()
        if should_close:
            # Cleanup
            if hasattr(__main__, 'my_webview_timer'):
                cmds.scriptJob(kill=__main__.my_webview_timer)
                del __main__.my_webview_timer
            del __main__.my_webview

# Create timer BEFORE showing window
timer_id = cmds.scriptJob(event=["idle", process_events])
__main__.my_webview_timer = timer_id

# Show window (non-blocking in embedded mode)
webview.show()
```

**Why this works:**
1. WebView is created in **Maya's main thread** (the thread running this script)
2. `show()` in embedded mode is **non-blocking** - it just creates the window and returns
3. `scriptJob` calls `process_events()` periodically to handle Windows messages
4. `process_events()` is **non-blocking** - it only processes pending messages
5. Result: **Maya stays responsive**

## Key Concepts

### Embedded Mode Behavior

When `parent_hwnd` is set, `show()` behaves differently:

```python
# Standalone mode (no parent_hwnd)
webview = NativeWebView(title="Standalone")
webview.show()  # [ERROR] BLOCKING - runs event loop until window closes

# Embedded mode (with parent_hwnd)
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="owner")
webview.show()  # [OK] NON-BLOCKING - creates window and returns immediately
```

### Parent Modes

```python
# Owner mode (recommended)
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="owner")
# - Uses GWLP_HWNDPARENT (owned window)
# - Safer for cross-thread scenarios
# - Window can be moved independently
# - Recommended for Maya integration

# Child mode (advanced)
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="child")
# - Uses WS_CHILD style
# - Requires same-thread creation
# - Window is clipped to parent bounds
# - Use only if you need true child window behavior
```

### Event Processing

The `process_events()` method:

```python
should_close = webview._core.process_events()
```

- **Non-blocking**: Uses `PeekMessageW` to process pending messages only
- **Returns immediately**: Doesn't wait for new messages
- **Returns bool**: `True` if window should close, `False` otherwise
- **Thread-safe**: Can be called from Maya's main thread

## Complete Example

```python
import maya.cmds as cmds
import maya.OpenMayaUI as omui
from auroraview import NativeWebView
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

# Get Maya main window
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)
hwnd = maya_window.winId()

# Create WebView
webview = NativeWebView(
    title="My Tool",
    width=800,
    height=600,
    parent_hwnd=hwnd,
    parent_mode="owner"
)

# Load content
webview.load_html("<h1>Hello Maya</h1>")

# Register event handlers
@webview.on("my_event")
def handle_event(data):
    print(f"Event received: {data}")

# Store in __main__
import __main__
__main__.my_webview = webview

# Create event processor
def process_events():
    if hasattr(__main__, 'my_webview'):
        should_close = __main__.my_webview._core.process_events()
        if should_close:
            # Cleanup
            if hasattr(__main__, 'my_webview_timer'):
                cmds.scriptJob(kill=__main__.my_webview_timer)
                del __main__.my_webview_timer
            del __main__.my_webview

# Create timer
timer_id = cmds.scriptJob(event=["idle", process_events])
__main__.my_webview_timer = timer_id

# Show window
webview.show()

print(f"[OK] WebView shown (timer ID: {timer_id})")
```

## Cleanup

### Manual Cleanup

```python
# Kill the timer
if hasattr(__main__, 'my_webview_timer'):
    cmds.scriptJob(kill=__main__.my_webview_timer)
    del __main__.my_webview_timer

# Delete the WebView
if hasattr(__main__, 'my_webview'):
    del __main__.my_webview
```

### Automatic Cleanup

The `process_events()` function automatically cleans up when the user closes the window:

```python
def process_events():
    if hasattr(__main__, 'my_webview'):
        should_close = __main__.my_webview._core.process_events()
        if should_close:
            # Window was closed by user - cleanup automatically
            if hasattr(__main__, 'my_webview_timer'):
                cmds.scriptJob(kill=__main__.my_webview_timer)
                del __main__.my_webview_timer
            del __main__.my_webview
```

## Common Issues

### Issue: Maya freezes when opening WebView

**Cause**: Using `show_async()` instead of `show()`

**Solution**: Use `show()` with scriptJob pattern (see above)

### Issue: WebView window doesn't respond to clicks

**Cause**: Not calling `process_events()` periodically

**Solution**: Create scriptJob to call `process_events()` on idle events

### Issue: Window closes immediately

**Cause**: WebView object is garbage collected

**Solution**: Store WebView in `__main__` or a global variable

```python
import __main__
__main__.my_webview = webview  # Keeps it alive
```

### Issue: Events from JavaScript not received in Python

**Cause**: Event handlers registered before `show()` might not work in embedded mode

**Solution**: Register event handlers before calling `show()`

```python
# Register handlers FIRST
@webview.on("my_event")
def handle_event(data):
    print(data)

# THEN show
webview.show()
```

## Performance Tips

### Optimize scriptJob Frequency

```python
# Option 1: Use "idle" event (called very frequently)
timer_id = cmds.scriptJob(event=["idle", process_events])

# Option 2: Use timer with interval (less frequent, lower CPU usage)
# Note: This requires a different approach with QTimer
```

### Batch Maya Commands

When handling events from JavaScript, batch Maya commands:

```python
@webview.on("create_objects")
def handle_create(data):
    def _do_create():
        # Batch all Maya commands here
        for obj_type in data['objects']:
            if obj_type == 'cube':
                cmds.polyCube()
            elif obj_type == 'sphere':
                cmds.polySphere()
    
    import maya.utils as mutils
    mutils.executeDeferred(_do_create)
```

## See Also

- [Architecture Documentation](ARCHITECTURE.md)
- [Examples](../examples/maya/)
- [API Reference](../README.md)
