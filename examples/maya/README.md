# Maya Integration Examples

This directory contains examples for integrating AuroraView with Autodesk Maya.

## 📋 Examples Overview

### 01: Basic Integration (Native Backend)
**File**: `01_basic_integration.py`

Basic Maya panel using the Native backend with HWND parenting.

**Features**:
- Native backend integration
- HWND-based window parenting
- Scene information display
- Object creation from UI

**Usage**:
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.01_basic_integration as example
example.show()
```

### 02: Scene Outliner (Native Backend)
**File**: `02_outliner_native.py`

Advanced example showing a real-time scene hierarchy viewer.

**Features**:
- Real-time scene hierarchy display
- Right-click context menu (rename, delete)
- Auto-refresh on scene changes
- Object selection synchronization

**Usage**:
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.02_outliner_native as example
# The outliner will appear automatically
```

### 03: Qt Integration (Qt Backend)
**File**: `03_qt_integration.py`

Maya panel using the Qt backend for seamless Qt widget integration.

**Requirements**:
```bash
pip install auroraview[qt]
```

**Features**:
- Qt backend integration
- Seamless Qt widget parenting
- No HWND handling required
- Better integration with Maya's Qt UI

**Usage**:
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.03_qt_integration as example
example.show()
```

## 🔧 Backend Comparison

### Native Backend (`01_basic_integration.py`, `02_outliner_native.py`)

**Pros**:
- ✅ No additional dependencies
- ✅ Works in all Maya versions
- ✅ Maximum compatibility
- ✅ Lightweight

**Cons**:
- ⚠️ Requires HWND handling
- ⚠️ Need to use `show_async()` for non-blocking display

**Code Pattern**:
```python
from auroraview import NativeWebView
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

# Get Maya main window handle
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)
hwnd = maya_window.winId()

# Create WebView
webview = NativeWebView(
    title="My Tool",
    parent_hwnd=hwnd,
    parent_mode="owner"  # Recommended
)
webview.show_async()  # Non-blocking
```

### Qt Backend (`03_qt_integration.py`)

**Pros**:
- ✅ Seamless Qt integration
- ✅ No HWND handling
- ✅ Can use Qt layouts/signals/slots
- ✅ Better UI integration

**Cons**:
- ⚠️ Requires `pip install auroraview[qt]`
- ⚠️ Additional dependency (qtpy)

**Code Pattern**:
```python
from auroraview import QtWebView
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

# Get Maya main window as QWidget
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)

# Create WebView as Qt widget
webview = QtWebView(
    parent=maya_window,
    title="My Tool"
)
webview.show()  # Standard Qt show()
```

## 🎯 Choosing the Right Backend

### Use Native Backend When:
- You want minimal dependencies
- You need maximum compatibility
- You're building a standalone tool
- You don't need Qt-specific features

### Use Qt Backend When:
- You're already using Qt in your tool
- You want seamless Qt widget integration
- You need to use Qt layouts
- You want to connect Qt signals/slots

## 💡 Best Practices

### 1. Thread Safety
Always use `maya.utils.executeDeferred()` for Maya commands triggered from JavaScript:

```python
@webview.on('create_cube')
def handle_create_cube(data):
    def _do_create():
        cmds.polyCube()
    
    import maya.utils as mutils
    mutils.executeDeferred(_do_create)
```

### 2. Window Parenting
For Native backend, use `parent_mode="owner"` for better cross-thread safety:

```python
webview = NativeWebView(
    parent_hwnd=hwnd,
    parent_mode="owner"  # Safer than "child"
)
```

### 3. Non-Blocking Display
Use `show_async()` for Native backend to avoid blocking Maya:

```python
webview.show_async()  # Non-blocking
# NOT: webview.show()  # This would block Maya!
```

### 4. Resource Cleanup
Clean up resources when closing:

```python
# For Native backend with scriptJob
if hasattr(__main__, 'my_webview_timer'):
    cmds.scriptJob(kill=__main__.my_webview_timer)
    del __main__.my_webview_timer
```

### 5. Event Processing
For Native backend, use scriptJob for event processing:

```python
def process_events():
    if hasattr(__main__, 'my_webview'):
        should_close = __main__.my_webview._core.process_events()
        if should_close:
            # Cleanup
            pass

timer_id = cmds.scriptJob(event=["idle", process_events])
__main__.my_webview_timer = timer_id
```

## 🐛 Common Issues

### Issue: Window doesn't appear
**Solution**: Make sure you're using `show_async()` for Native backend:
```python
webview.show_async()  # Correct
# NOT: webview.show()
```

### Issue: Maya freezes
**Solution**: Use `maya.utils.executeDeferred()` for Maya commands:
```python
import maya.utils as mutils
mutils.executeDeferred(lambda: cmds.polyCube())
```

### Issue: Qt backend not available
**Solution**: Install Qt support:
```bash
pip install auroraview[qt]
```

### Issue: Import errors
**Solution**: Add the correct path:
```python
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview')
from auroraview import NativeWebView
```

## 📚 Additional Resources

- [Main Examples README](../README.md)
- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [API Reference](../../README.md)

## 🎓 Learning Path

1. **Start with `01_basic_integration.py`**
   - Understand basic Maya integration
   - Learn HWND parenting
   - Practice event communication

2. **Try `03_qt_integration.py`**
   - Compare with Native backend
   - Understand Qt widget integration
   - Choose your preferred approach

3. **Study `02_outliner_native.py`**
   - See a real-world example
   - Learn advanced patterns
   - Understand event processing

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!
