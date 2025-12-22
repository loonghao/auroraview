# Maya Integration

AuroraView provides seamless integration with Autodesk Maya through both Native WebView and QtWebView backends.

## Installation

```bash
# For Qt backend (recommended for Maya)
pip install auroraview[qt]

# Or install with mayapy
"C:\Program Files\Autodesk\Maya2024\bin\mayapy.exe" -m pip install auroraview[qt]
```

## Quick Start

### Using QtWebView (Recommended)

```python
from auroraview import QtWebView
import maya.OpenMayaUI as omui
from qtpy import QtWidgets
import shiboken2

def maya_main_window():
    ptr = omui.MQtUtil.mainWindow()
    return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)

# Create WebView
webview = QtWebView(
    parent=maya_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

### Using Native WebView

```python
from auroraview import WebView
import maya.OpenMayaUI as omui

# Get Maya main window handle
maya_hwnd = int(omui.MQtUtil.mainWindow())

# Create WebView
webview = WebView.create(
    "My Tool",
    width=800,
    height=600,
    parent=maya_hwnd,
    mode="owner"
)
webview.load_html("<h1>Hello Maya</h1>")
webview.show()
```

## Common Errors

### ImportError: cannot import name 'QtWebView'

**Symptom**:
```python
from auroraview import QtWebView
# ImportError: cannot import name 'QtWebView' from 'auroraview'
```

**Cause**: Qt backend dependencies not installed

**Diagnosis**:
```python
import auroraview
print(f"Qt backend available: {auroraview._HAS_QT}")
if not auroraview._HAS_QT:
    print(f"Qt import error: {auroraview._QT_IMPORT_ERROR}")
```

**Solution 1**: Install Qt dependencies
```bash
# Windows
"C:\Program Files\Autodesk\Maya2024\bin\mayapy.exe" -m pip install auroraview[qt]

# macOS
/Applications/Autodesk/maya2024/Maya.app/Contents/bin/mayapy -m pip install auroraview[qt]

# Linux
/usr/autodesk/maya2024/bin/mayapy -m pip install auroraview[qt]
```

**Solution 2**: Use Native backend (no extra dependencies)
```python
from auroraview import WebView
import maya.OpenMayaUI as omui

maya_hwnd = int(omui.MQtUtil.mainWindow())
webview = WebView.create("My Tool", parent=maya_hwnd, mode="owner")
webview.show()
```

## Threading Model

### ❌ WRONG: Using `show_async()`

```python
# DON'T DO THIS - Will cause Maya to freeze!
webview = WebView.create("My Tool", parent=hwnd, mode="owner")
webview.load_html(html)
webview.show_async()  # Creates window in background thread
```

**Why this fails:**
1. `show_async()` creates the WebView in a **background thread**
2. The window is parented to Maya's main window
3. **Windows GUI thread affinity**: Child/owned windows must be created in the same thread as their parent
4. Result: **Maya freezes**

### ✅ CORRECT: Using `show()` with scriptJob

```python
import maya.cmds as cmds
import __main__

webview = WebView.create("My Tool", parent=hwnd, mode="owner")
webview.load_html(html)

# Store in __main__ for scriptJob access
__main__.my_webview = webview

# Create scriptJob to process events
def process_events():
    if hasattr(__main__, 'my_webview'):
        should_close = __main__.my_webview.process_events()
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
1. WebView is created in **Maya's main thread**
2. `show()` in embedded mode is **non-blocking**
3. `scriptJob` calls `process_events()` periodically
4. Result: **Maya stays responsive**

## Parent Modes

```python
# Owner mode (recommended)
webview = WebView.create("My Tool", parent=hwnd, mode="owner")
# - Uses GWLP_HWNDPARENT (owned window)
# - Safer for cross-thread scenarios
# - Window can be moved independently

# Child mode (advanced)
webview = WebView.create("My Tool", parent=hwnd, mode="child")
# - Uses WS_CHILD style
# - Requires same-thread creation
# - Window is clipped to parent bounds
```

## Dockable Panel

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt
import maya.OpenMayaUI as omui
from qtpy import QtWidgets
import shiboken2

def maya_main_window():
    ptr = omui.MQtUtil.mainWindow()
    return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)

# Create dock widget
main_win = maya_main_window()
dock = QDockWidget("My Tool", main_win)

# Create WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# Set as dock widget content
dock.setWidget(webview)
main_win.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## API Binding Example

```python
from auroraview import QtWebView
import maya.cmds as cmds

class MayaAPI:
    def get_selection(self) -> dict:
        """Get current selection"""
        sel = cmds.ls(selection=True)
        return {"selection": sel, "count": len(sel)}

    def select_objects(self, names: list = None) -> dict:
        """Select objects by name"""
        names = names or []
        cmds.select(names, replace=True)
        return {"ok": True, "selected": names}

    def create_cube(self, name: str = "cube") -> dict:
        """Create a polygon cube"""
        result = cmds.polyCube(name=name)
        return {"ok": True, "name": result[0]}

# Create WebView with API
webview = QtWebView(
    parent=maya_main_window(),
    url="http://localhost:3000"
)
webview.bind_api(MayaAPI())
webview.show()
```

```javascript
// JavaScript side
const sel = await auroraview.api.get_selection();
console.log('Selected:', sel.selection);

await auroraview.api.create_cube({ name: 'myCube' });
await auroraview.api.select_objects({ names: ['myCube'] });
```

## Selection Sync Example

```python
from auroraview import QtWebView
import maya.cmds as cmds
from maya.api import OpenMaya as om

class OutlinerTool(QtWebView):
    def __init__(self, parent=None):
        super().__init__(parent=parent, width=300, height=600)
        self.load_url("http://localhost:3000")
        self._setup_callbacks()

    def _setup_callbacks(self):
        # Maya selection changed callback
        self.callback_id = om.MEventMessage.addEventCallback(
            "SelectionChanged",
            self._on_maya_selection_changed
        )

        # JavaScript selection event
        @self.on("select_objects")
        def handle_select(data):
            names = data.get("names", [])
            cmds.select(names, replace=True)

    def _on_maya_selection_changed(self, *args):
        sel = cmds.ls(selection=True)
        self.emit("selection_changed", {"selection": sel})

    def closeEvent(self, event):
        om.MMessage.removeCallback(self.callback_id)
        super().closeEvent(event)

# Usage
tool = OutlinerTool(parent=maya_main_window())
tool.show()
```

## Complete Example

```python
import maya.cmds as cmds
import maya.OpenMayaUI as omui
from auroraview import WebView
from shiboken2 import wrapInstance
from qtpy.QtWidgets import QWidget

# Get Maya main window
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)
hwnd = int(maya_window.winId())

# Create WebView
webview = WebView.create(
    "My Tool",
    width=800,
    height=600,
    parent=hwnd,
    mode="owner"
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
        should_close = __main__.my_webview.process_events()
        if should_close:
            if hasattr(__main__, 'my_webview_timer'):
                cmds.scriptJob(kill=__main__.my_webview_timer)
                del __main__.my_webview_timer
            del __main__.my_webview

# Create timer
timer_id = cmds.scriptJob(event=["idle", process_events])
__main__.my_webview_timer = timer_id

# Show window
webview.show()
print(f"WebView shown (timer ID: {timer_id})")
```

## userSetup.py Integration

Add to your `userSetup.py`:

```python
import maya.cmds as cmds

def launch_my_tool():
    from auroraview import QtWebView
    import maya.OpenMayaUI as omui
    from qtpy import QtWidgets
    import shiboken2

    def maya_main_window():
        ptr = omui.MQtUtil.mainWindow()
        return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)

    webview = QtWebView(
        parent=maya_main_window(),
        url="http://localhost:3000"
    )
    webview.show()
    return webview

# Register menu item
cmds.evalDeferred("""
import maya.cmds as cmds
if cmds.menu('myToolMenu', exists=True):
    cmds.deleteUI('myToolMenu')
cmds.menu('myToolMenu', label='My Tool', parent='MayaWindow')
cmds.menuItem(label='Launch Tool', command='launch_my_tool()')
""")
```

## WebView2 Pre-warming

QtWebView automatically pre-warms WebView2 for faster subsequent creation:

```python
from auroraview.integration.qt import WebViewPool

# Explicit pre-warm at Maya startup (optional)
WebViewPool.prewarm()

# Check status
if WebViewPool.has_prewarmed():
    print(f"Pre-warm took {WebViewPool.get_prewarm_time():.2f}s")
```

## Performance Tips

### Optimize scriptJob Frequency

```python
# Option 1: Use "idle" event (called very frequently)
timer_id = cmds.scriptJob(event=["idle", process_events])

# Option 2: Use QTimer for more control
from qtpy.QtCore import QTimer
timer = QTimer()
timer.timeout.connect(process_events)
timer.start(16)  # ~60 FPS
```

### Batch Maya Commands

When handling events from JavaScript, batch Maya commands:

```python
@webview.on("create_objects")
def handle_create(data):
    def _do_create():
        for obj_type in data['objects']:
            if obj_type == 'cube':
                cmds.polyCube()
            elif obj_type == 'sphere':
                cmds.polySphere()
    
    import maya.utils as mutils
    mutils.executeDeferred(_do_create)
```

## Troubleshooting

### Maya freezes when opening WebView

**Cause**: Using `show_async()` instead of `show()`

**Solution**: Use `show()` with scriptJob pattern

### WebView window doesn't respond to clicks

**Cause**: Not calling `process_events()` periodically

**Solution**: Create scriptJob to call `process_events()` on idle events

### Window closes immediately

**Cause**: WebView object is garbage collected

**Solution**: Store WebView in `__main__` or a global variable

```python
import __main__
__main__.my_webview = webview  # Keeps it alive
```

### Events from JavaScript not received

**Cause**: Event handlers registered after `show()`

**Solution**: Register event handlers before calling `show()`

```python
# Register handlers FIRST
@webview.on("my_event")
def handle_event(data):
    print(data)

# THEN show
webview.show()
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

The `process_events()` function automatically cleans up when the user closes the window.
