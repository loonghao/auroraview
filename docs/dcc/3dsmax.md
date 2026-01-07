# 3ds Max Integration

AuroraView integrates with Autodesk 3ds Max through QtWebView.

::: info 3ds Max 2020+
MaxPlus is deprecated since 3ds Max 2020. This guide uses `pymxs` which is the recommended API for 3ds Max 2020 and later versions.
:::

## Requirements

| Component | Minimum Version | Recommended |
|-----------|-----------------|-------------|
| 3ds Max | 2020 | 2024+ |
| Python | 3.9 | 3.11+ |
| Qt | PySide2/Qt5 | PySide2/Qt5 |

## Installation

```bash
pip install auroraview[qt]
```

## Quick Start

```python
from auroraview import QtWebView
from qtpy import QtWidgets
from pymxs import runtime as rt

def max_main_window():
    """Get 3ds Max main window as QWidget."""
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

# AuroraView auto-detects 3ds Max and enables thread safety
webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

::: tip Automatic Thread Safety
When running inside 3ds Max, AuroraView automatically enables thread safety (`dcc_mode="auto"`). No additional configuration needed!
:::

## API Binding Example

```python
from auroraview import QtWebView
from pymxs import runtime as rt

class MaxAPI:
    def get_selection(self) -> dict:
        """Get selected objects"""
        sel = list(rt.selection)
        return {
            "selection": [str(obj.name) for obj in sel],
            "count": len(sel)
        }

    def select_by_name(self, names: list = None) -> dict:
        """Select objects by name"""
        names = names or []
        rt.clearSelection()
        for name in names:
            obj = rt.getNodeByName(name)
            if obj:
                rt.selectMore(obj)
        return {"ok": True}

    def create_box(self, name: str = "Box001", size: float = 10.0) -> dict:
        """Create a box primitive"""
        box = rt.Box(
            name=name,
            length=size,
            width=size,
            height=size
        )
        return {"ok": True, "name": str(box.name)}

    def get_transform(self, name: str = "") -> dict:
        """Get object transform"""
        obj = rt.getNodeByName(name)
        if obj:
            pos = obj.position
            return {
                "ok": True,
                "position": [pos.x, pos.y, pos.z],
                "rotation": [obj.rotation.x, obj.rotation.y, obj.rotation.z]
            }
        return {"ok": False, "error": "Object not found"}

    def set_position(self, name: str = "", x: float = 0, y: float = 0, z: float = 0) -> dict:
        """Set object position"""
        obj = rt.getNodeByName(name)
        if obj:
            obj.position = rt.Point3(x, y, z)
            return {"ok": True}
        return {"ok": False, "error": "Object not found"}

# Create WebView with API
webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000"
)
webview.bind_api(MaxAPI())
webview.show()
```

```javascript
// JavaScript side
const sel = await auroraview.api.get_selection();
console.log('Selected:', sel.selection);

await auroraview.api.create_box({ name: 'myBox', size: 20.0 });
await auroraview.api.set_position({ name: 'myBox', x: 10, y: 0, z: 5 });
```

## Thread Safety

AuroraView provides **automatic** thread safety for 3ds Max integration. Since 3ds Max uses Qt internally, AuroraView leverages Qt's event loop (`QTimer.singleShot`) to schedule callbacks on the main thread.

::: info Implementation Note
3ds Max's thread dispatcher uses Qt's `QTimer.singleShot()` since 3ds Max 2020+ runs a Qt event loop. This is more reliable than the deprecated MaxPlus API.
:::

### Automatic Thread Safety (Default)

**No configuration needed!** AuroraView auto-detects 3ds Max and enables thread safety:

```python
from auroraview import QtWebView
from pymxs import runtime as rt
from qtpy import QtWidgets

def max_main_window():
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

# Thread safety is automatically enabled when 3ds Max is detected
webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000",
    # dcc_mode="auto" is the default - no need to specify!
)

@webview.on("create_primitive")
def handle_create(data):
    # Automatically runs on 3ds Max main thread!
    prim_type = data.get("type", "box")
    name = data.get("name", "Object001")
    size = data.get("size", 10.0)

    if prim_type == "box":
        obj = rt.Box(name=name, length=size, width=size, height=size)
    elif prim_type == "sphere":
        obj = rt.Sphere(name=name, radius=size)
    else:
        return {"ok": False, "error": f"Unknown type: {prim_type}"}

    return {"ok": True, "name": str(obj.name)}

@webview.on("get_selection")
def handle_selection(data):
    sel = list(rt.selection)
    return {"selection": [str(obj.name) for obj in sel], "count": len(sel)}

webview.show()
```

### Manual Thread Safety with Decorators

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
from pymxs import runtime as rt

webview = QtWebView(parent=max_main_window(), url="http://localhost:3000")

@webview.on("render_scene")
@dcc_thread_safe  # Blocks until render complete
def handle_render(data):
    output_path = data.get("path", "C:/temp/render.png")
    rt.render(outputFile=output_path)
    return {"ok": True, "path": output_path}

@webview.on("refresh_viewport")
@dcc_thread_safe_async  # Fire-and-forget
def handle_refresh(data):
    rt.redrawViews()

webview.show()
```

### Using `run_on_main_thread` Directly

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
from pymxs import runtime as rt

# Fire-and-forget
def clear_selection():
    rt.clearSelection()

run_on_main_thread(clear_selection)

# Blocking with return value
def get_max_file_path():
    return rt.maxFilePath + rt.maxFileName

file_path = run_on_main_thread_sync(get_max_file_path)
print(f"Current file: {file_path}")
```

## Dockable Panel

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt
from pymxs import runtime as rt
from qtpy import QtWidgets

def max_main_window():
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

# Create dock widget
main_win = max_main_window()
dock = QDockWidget("My Tool", main_win)

# Create WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# Set as dock widget content
dock.setWidget(webview)
main_win.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## Selection Callback

```python
from auroraview import QtWebView
from pymxs import runtime as rt

class SceneBrowser(QtWebView):
    def __init__(self, parent=None):
        super().__init__(parent=parent, width=300, height=600)
        self.load_url("http://localhost:3000")
        self._setup_callbacks()

    def _setup_callbacks(self):
        # Register selection change callback
        rt.callbacks.addScript(
            rt.Name("selectionSetChanged"),
            "python.execute('scene_browser._on_selection_changed()')"
        )

        @self.on("select_object")
        def handle_select(data):
            name = data.get("name", "")
            obj = rt.getNodeByName(name)
            if obj:
                rt.select(obj)

    def _on_selection_changed(self):
        sel = [str(obj.name) for obj in rt.selection]
        self.emit("selection_changed", {"selection": sel})

# Global reference for callback
scene_browser = SceneBrowser(parent=max_main_window())
scene_browser.show()
```

## MAXScript Integration

Launch from MAXScript:

```maxscript
python.Execute "from auroraview import QtWebView; webview = QtWebView(url='http://localhost:3000'); webview.show()"
```
