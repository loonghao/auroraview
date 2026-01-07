# Houdini Integration

AuroraView integrates with SideFX Houdini through QtWebView.

## Installation

```bash
pip install auroraview[qt]
```

## Quick Start

```python
from auroraview import QtWebView
import hou

webview = QtWebView(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

## Dockable Panel

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt
import hou

# Create dock widget
main_win = hou.qt.mainWindow()
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
import hou

class HoudiniAPI:
    def get_selected_nodes(self) -> dict:
        """Get selected nodes"""
        nodes = hou.selectedNodes()
        return {
            "nodes": [n.path() for n in nodes],
            "count": len(nodes)
        }

    def select_node(self, path: str = "") -> dict:
        """Select a node by path"""
        node = hou.node(path)
        if node:
            node.setSelected(True, clear_all_selected=True)
            return {"ok": True, "path": path}
        return {"ok": False, "error": "Node not found"}

    def create_node(self, parent: str = "/obj", type: str = "geo", name: str = None) -> dict:
        """Create a new node"""
        parent_node = hou.node(parent)
        if parent_node:
            new_node = parent_node.createNode(type, name)
            return {"ok": True, "path": new_node.path()}
        return {"ok": False, "error": "Parent not found"}

    def get_parm_value(self, node_path: str = "", parm_name: str = "") -> dict:
        """Get parameter value"""
        node = hou.node(node_path)
        if node:
            parm = node.parm(parm_name)
            if parm:
                return {"ok": True, "value": parm.eval()}
        return {"ok": False, "error": "Parameter not found"}

    def set_parm_value(self, node_path: str = "", parm_name: str = "", value=None) -> dict:
        """Set parameter value"""
        node = hou.node(node_path)
        if node:
            parm = node.parm(parm_name)
            if parm:
                parm.set(value)
                return {"ok": True}
        return {"ok": False, "error": "Parameter not found"}

# Create WebView with API
webview = QtWebView(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000"
)
webview.bind_api(HoudiniAPI())
webview.show()
```

```javascript
// JavaScript side
const nodes = await auroraview.api.get_selected_nodes();
console.log('Selected:', nodes.nodes);

await auroraview.api.create_node({
    parent: '/obj',
    type: 'geo',
    name: 'myGeo'
});

await auroraview.api.set_parm_value({
    node_path: '/obj/myGeo',
    parm_name: 'tx',
    value: 5.0
});
```

## Thread Safety

AuroraView provides **automatic** thread safety for Houdini integration. Houdini requires all `hou` operations to run on the main thread, which AuroraView handles via `hdefereval`.

::: tip Zero Configuration
Since `dcc_mode="auto"` is the default, AuroraView automatically detects Houdini and enables thread safety. No configuration needed!
:::

### Automatic Thread Safety (Default)

Just use AuroraView normally - thread safety is automatic:

```python
from auroraview import QtWebView
import hou

# Thread safety is automatically enabled when Houdini is detected
webview = QtWebView(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    # dcc_mode="auto" is the default - no need to specify!
)

@webview.on("create_node")
def handle_create(data):
    # Automatically runs on Houdini main thread!
    parent_path = data.get("parent", "/obj")
    node_type = data.get("type", "geo")
    name = data.get("name")

    parent = hou.node(parent_path)
    new_node = parent.createNode(node_type, name)
    return {"ok": True, "path": new_node.path()}

@webview.on("get_selected_nodes")
def handle_selection(data):
    nodes = hou.selectedNodes()
    return {"nodes": [n.path() for n in nodes], "count": len(nodes)}

webview.show()
```

### Manual Thread Safety with Decorators

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
import hou

webview = QtWebView(parent=hou.qt.mainWindow(), url="http://localhost:3000")

@webview.on("cook_node")
@dcc_thread_safe  # Blocks until cook complete
def handle_cook(data):
    node_path = data.get("path")
    node = hou.node(node_path)
    if node:
        node.cook(force=True)
        return {"ok": True, "cooked": node_path}
    return {"ok": False, "error": "Node not found"}

@webview.on("update_display")
@dcc_thread_safe_async  # Fire-and-forget
def handle_update(data):
    hou.ui.triggerUpdate()

webview.show()
```

### Using `run_on_main_thread` Directly

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
import hou

# Fire-and-forget
def select_node(path):
    node = hou.node(path)
    if node:
        node.setSelected(True, clear_all_selected=True)

run_on_main_thread(select_node, "/obj/geo1")

# Blocking with return value
def get_hip_path():
    return hou.hipFile.path()

hip_path = run_on_main_thread_sync(get_hip_path)
print(f"Current HIP: {hip_path}")
```

## Node Selection Sync

```python
from auroraview import QtWebView
import hou

class NodeBrowser(QtWebView):
    def __init__(self, parent=None):
        super().__init__(parent=parent, width=300, height=600)
        self.load_url("http://localhost:3000")
        self._setup_callbacks()

    def _setup_callbacks(self):
        # Houdini selection callback
        hou.ui.addEventLoopCallback(self._check_selection)
        self._last_selection = []

        @self.on("select_node")
        def handle_select(data):
            path = data.get("path", "")
            node = hou.node(path)
            if node:
                node.setSelected(True, clear_all_selected=True)

    def _check_selection(self):
        current = [n.path() for n in hou.selectedNodes()]
        if current != self._last_selection:
            self._last_selection = current
            self.emit("selection_changed", {"nodes": current})

# Usage
tool = NodeBrowser(parent=hou.qt.mainWindow())
tool.show()
```

## Python Panel Integration

Create a Python Panel in Houdini:

```python
# In Houdini Python Panel
from auroraview import QtWebView

def createInterface():
    webview = QtWebView()
    webview.load_url("http://localhost:3000")
    return webview
```

## Shelf Tool

Create a shelf tool:

```python
from auroraview import QtWebView
import hou

webview = QtWebView(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```
