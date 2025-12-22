# 3ds Max Integration

AuroraView integrates with Autodesk 3ds Max through QtWebView.

## Installation

```bash
pip install auroraview[qt]
```

## Quick Start

```python
from auroraview import QtWebView
from qtpy import QtWidgets
import MaxPlus

def max_main_window():
    return QtWidgets.QWidget.find(MaxPlus.GetQMaxMainWindow())

webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

## Alternative: pymxs

```python
from auroraview import QtWebView
from qtpy import QtWidgets
from pymxs import runtime as rt

def max_main_window():
    # Get 3ds Max main window using pymxs
    import ctypes
    hwnd = rt.windows.getMAXHWND()
    return QtWidgets.QWidget.find(hwnd)

webview = QtWebView(
    parent=max_main_window(),
    url="http://localhost:3000"
)
webview.show()
```

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
