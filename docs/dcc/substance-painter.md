# Substance Painter Integration

AuroraView integrates with Adobe Substance Painter through its Python scripting API.

## Architecture

```
┌─────────────────────────────────────────────┐
│           Substance Painter                  │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  Qt Window  │ ◄──► │  AuroraView      │ │
│  │  Container  │      │  (WebView2)      │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ Qt Parent            │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │    Substance Painter Python API     │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## Requirements

| Component | Minimum Version | Recommended |
|-----------|-----------------|-------------|
| Substance Painter | 2022.1 | 2024.1+ |
| Python | 3.9 | 3.11+ |
| OS | Windows 10, macOS 11 | Windows 11, macOS 14+ |

## Installation

```bash
# Install to Substance Painter's Python environment
pip install auroraview[qt]
```

## Quick Start

### Using QtWebView

```python
from auroraview import QtWebView
import substance_painter.ui as ui

# Get Substance Painter main window
main_window = ui.get_main_window()

webview = QtWebView(
    parent=main_window,
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

### Dockable Panel

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget
from qtpy.QtCore import Qt
import substance_painter.ui as ui

main_window = ui.get_main_window()

# Create dock widget
dock = QDockWidget("My Tool", main_window)

# Create WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# Set as dock widget content
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## Thread Safety

AuroraView provides **automatic** thread safety for Substance Painter integration.

::: tip Zero Configuration
Since `dcc_mode="auto"` is the default, AuroraView automatically detects Substance Painter and enables thread safety. No configuration needed!
:::

### Automatic Thread Safety (Default)

Just use AuroraView normally - thread safety is automatic:

```python
from auroraview import QtWebView
import substance_painter.ui as ui
import substance_painter.project as project
import substance_painter.textureset as textureset

main_window = ui.get_main_window()

# Thread safety is automatically enabled when Substance Painter is detected
webview = QtWebView(
    parent=main_window,
    url="http://localhost:3000",
    # dcc_mode="auto" is the default - no need to specify!
)

@webview.on("get_project_info")
def handle_project_info(data):
    # Automatically runs on main thread!
    if not project.is_open():
        return {"ok": False, "error": "No project open"}
    
    return {
        "ok": True,
        "name": project.name(),
        "file_path": project.file_path(),
        "texture_sets": [ts.name() for ts in textureset.all_texture_sets()]
    }

@webview.on("export_textures")
def handle_export(data):
    export_path = data.get("path", "C:/temp/export")
    # Export logic here
    return {"ok": True, "path": export_path}

webview.show()
```

### Manual Thread Safety with Decorators

```python
from auroraview import QtWebView
from auroraview.utils import dcc_thread_safe, dcc_thread_safe_async
import substance_painter.ui as ui
import substance_painter.project as project

webview = QtWebView(parent=ui.get_main_window(), url="http://localhost:3000")

@webview.on("save_project")
@dcc_thread_safe  # Blocks until save complete
def handle_save(data):
    if project.is_open():
        project.save()
        return {"ok": True}
    return {"ok": False, "error": "No project open"}

@webview.on("refresh_ui")
@dcc_thread_safe_async  # Fire-and-forget
def handle_refresh(data):
    ui.get_main_window().update()

webview.show()
```

### Using `run_on_main_thread` Directly

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync
import substance_painter.project as project

# Fire-and-forget
def close_project():
    if project.is_open():
        project.close()

run_on_main_thread(close_project)

# Blocking with return value
def get_project_name():
    if project.is_open():
        return project.name()
    return None

name = run_on_main_thread_sync(get_project_name)
print(f"Current project: {name}")
```

## API Binding Example

```python
from auroraview import QtWebView
import substance_painter.ui as ui
import substance_painter.project as project
import substance_painter.textureset as textureset
import substance_painter.layerstack as layerstack

class SubstancePainterAPI:
    def get_texture_sets(self) -> dict:
        """Get all texture sets in the project"""
        if not project.is_open():
            return {"ok": False, "error": "No project open"}
        
        sets = textureset.all_texture_sets()
        return {
            "ok": True,
            "texture_sets": [{"name": ts.name()} for ts in sets]
        }
    
    def get_layers(self, texture_set: str = "") -> dict:
        """Get layers in a texture set"""
        ts = textureset.TextureSet.from_name(texture_set)
        if not ts:
            return {"ok": False, "error": "Texture set not found"}
        
        stack = layerstack.get_layer_stack(ts)
        layers = []
        for layer in stack.all_layers():
            layers.append({
                "name": layer.name(),
                "visible": layer.is_visible(),
                "locked": layer.is_locked()
            })
        return {"ok": True, "layers": layers}
    
    def set_layer_visibility(self, texture_set: str = "", layer_name: str = "", 
                             visible: bool = True) -> dict:
        """Toggle layer visibility"""
        ts = textureset.TextureSet.from_name(texture_set)
        if not ts:
            return {"ok": False, "error": "Texture set not found"}
        
        stack = layerstack.get_layer_stack(ts)
        for layer in stack.all_layers():
            if layer.name() == layer_name:
                layer.set_visible(visible)
                return {"ok": True}
        
        return {"ok": False, "error": "Layer not found"}

# Create WebView with API
webview = QtWebView(
    parent=ui.get_main_window(),
    url="http://localhost:3000",
    dcc_mode=True
)
webview.bind_api(SubstancePainterAPI())
webview.show()
```

```javascript
// JavaScript side
const textureSets = await auroraview.api.get_texture_sets();
console.log('Texture sets:', textureSets.texture_sets);

const layers = await auroraview.api.get_layers({ texture_set: 'DefaultMaterial' });
console.log('Layers:', layers.layers);

await auroraview.api.set_layer_visibility({
    texture_set: 'DefaultMaterial',
    layer_name: 'Layer 1',
    visible: false
});
```

## Plugin Structure

Create a proper Substance Painter plugin:

```
my_plugin/
├── __init__.py
├── api.py
└── plugin.py
```

**`__init__.py`:**

```python
from . import plugin

def start_plugin():
    plugin.start()

def close_plugin():
    plugin.close()
```

**`plugin.py`:**

```python
from auroraview import QtWebView
import substance_painter.ui as ui

_webview = None

def start():
    global _webview
    _webview = QtWebView(
        parent=ui.get_main_window(),
        url="http://localhost:3000",
        dcc_mode=True
    )
    _webview.show()

def close():
    global _webview
    if _webview:
        _webview.close()
        _webview = None
```

## Troubleshooting

### WebView not displaying

**Cause**: Parent widget not correctly obtained.

**Solution**: Ensure `substance_painter.ui.get_main_window()` returns a valid QWidget.

### Python module not found

**Cause**: AuroraView not installed in Substance Painter's Python environment.

**Solution**:
```python
import sys
print(sys.executable)  # Check which Python Substance Painter uses
# Install to that specific Python
```

### Events from JavaScript not received

**Cause**: Event handlers registered after `show()`.

**Solution**: Register event handlers before calling `show()`.

## Development Status

| Feature | Status |
|---------|--------|
| Basic Integration | 🚧 In Progress |
| Layer Management | 📋 Planned |
| Export Automation | 📋 Planned |
| Material Sync | 📋 Planned |

## Resources

- [Substance Painter Python API](https://substance3d.adobe.com/documentation/spdoc/python-api-194216357.html)
- [Qt Integration Guide](../guide/qt-integration)
- [DCC Overview](./index) - Overview of all DCC integrations

