# Blender Integration

AuroraView integrates with Blender using **Desktop Mode** or **Native Mode (HWND)**, as Blender uses its own UI framework (not Qt).

## Integration Modes for Blender

| Mode | Description | Use Case |
|------|-------------|----------|
| **Desktop Mode** | Independent window with own event loop | Simple tools, quick prototypes |
| **Native Mode (HWND)** | Floating window attached to Blender | Production tools, always-on-top panels |

## Installation

```bash
pip install auroraview
```

## Quick Start

### Desktop Mode

```python
from auroraview import run_desktop

run_desktop(
    title="Blender Tool",
    url="http://localhost:3000",
    width=800,
    height=600
)
```

### Native Mode (HWND) with Window Effects

```python
from auroraview import WebView

# Create floating panel with effects
webview = WebView(
    title="Blender Tool",
    url="http://localhost:3000",
    width=400,
    height=600,
    transparent=True,
    decorations=False,
    always_on_top=True,
    tool_window=True,
)

# Apply window effects (Windows only)
webview.apply_acrylic((30, 30, 30, 180))
webview.enable_click_through()

webview.show()
```

## API Binding Example

```python
from auroraview import AuroraView
import bpy

class BlenderAPI:
    def get_selected_objects(self) -> dict:
        """Get selected objects"""
        selected = bpy.context.selected_objects
        return {
            "objects": [obj.name for obj in selected],
            "count": len(selected)
        }

    def select_object(self, name: str = "") -> dict:
        """Select object by name"""
        bpy.ops.object.select_all(action='DESELECT')
        obj = bpy.data.objects.get(name)
        if obj:
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
            return {"ok": True, "name": name}
        return {"ok": False, "error": "Object not found"}

    def create_cube(self, name: str = "Cube", size: float = 2.0) -> dict:
        """Create a cube"""
        bpy.ops.mesh.primitive_cube_add(size=size)
        obj = bpy.context.active_object
        obj.name = name
        return {"ok": True, "name": obj.name}

    def get_transform(self, name: str = "") -> dict:
        """Get object transform"""
        obj = bpy.data.objects.get(name)
        if obj:
            return {
                "ok": True,
                "location": list(obj.location),
                "rotation": list(obj.rotation_euler),
                "scale": list(obj.scale)
            }
        return {"ok": False, "error": "Object not found"}

    def set_location(self, name: str = "", x: float = 0, y: float = 0, z: float = 0) -> dict:
        """Set object location"""
        obj = bpy.data.objects.get(name)
        if obj:
            obj.location = (x, y, z)
            return {"ok": True}
        return {"ok": False, "error": "Object not found"}

    def render_image(self, filepath: str = "/tmp/render.png") -> dict:
        """Render current scene"""
        bpy.context.scene.render.filepath = filepath
        bpy.ops.render.render(write_still=True)
        return {"ok": True, "filepath": filepath}

# Create WebView with API
webview = AuroraView(
    url="http://localhost:3000",
    api=BlenderAPI()
)
webview.show()
```

```javascript
// JavaScript side
const sel = await auroraview.api.get_selected_objects();
console.log('Selected:', sel.objects);

await auroraview.api.create_cube({ name: 'MyCube', size: 3.0 });
await auroraview.api.set_location({ name: 'MyCube', x: 2, y: 0, z: 1 });

// Render
await auroraview.api.render_image({ filepath: '/tmp/my_render.png' });
```

## Blender Operator

Create a Blender operator to launch the tool:

```python
import bpy
from auroraview import AuroraView

class AURORAVIEW_OT_launch_tool(bpy.types.Operator):
    bl_idname = "auroraview.launch_tool"
    bl_label = "Launch AuroraView Tool"
    bl_description = "Launch the AuroraView web tool"

    _webview = None

    def execute(self, context):
        if AURORAVIEW_OT_launch_tool._webview is None:
            AURORAVIEW_OT_launch_tool._webview = AuroraView(
                url="http://localhost:3000",
                api=BlenderAPI()
            )
        AURORAVIEW_OT_launch_tool._webview.show()
        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(AURORAVIEW_OT_launch_tool.bl_idname)

def register():
    bpy.utils.register_class(AURORAVIEW_OT_launch_tool)
    bpy.types.VIEW3D_MT_view.append(menu_func)

def unregister():
    bpy.types.VIEW3D_MT_view.remove(menu_func)
    bpy.utils.unregister_class(AURORAVIEW_OT_launch_tool)

if __name__ == "__main__":
    register()
```

## Add-on Structure

Create a proper Blender add-on:

```
my_addon/
├── __init__.py
├── api.py
└── operators.py
```

**`__init__.py`:**

```python
bl_info = {
    "name": "AuroraView Tool",
    "author": "Your Name",
    "version": (1, 0, 0),
    "blender": (3, 0, 0),
    "location": "View3D > View > AuroraView Tool",
    "description": "Web-based tool using AuroraView",
    "category": "3D View",
}

from . import operators

def register():
    operators.register()

def unregister():
    operators.unregister()
```

## Timer-based Updates

For real-time sync with Blender:

```python
import bpy
from auroraview import AuroraView

class BlenderSyncTool:
    def __init__(self):
        self.webview = AuroraView(url="http://localhost:3000")
        self._last_selection = []
        bpy.app.timers.register(self._check_selection, persistent=True)

    def _check_selection(self):
        current = [obj.name for obj in bpy.context.selected_objects]
        if current != self._last_selection:
            self._last_selection = current
            self.webview.emit("selection_changed", {"objects": current})
        return 0.1  # Check every 100ms

    def show(self):
        self.webview.show()

# Usage
tool = BlenderSyncTool()
tool.show()
```
