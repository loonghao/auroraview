# Unified WebView API

AuroraView provides a unified API that automatically selects the appropriate WebView implementation based on your use case. This simplifies development by eliminating the need to choose between `WebView`, `QtWebView`, and `AuroraView` classes.

## Quick Start

```python
from auroraview import create_webview

# 1. Standalone window (no parent)
webview = create_webview(url="http://localhost:3000")
webview.show()

# 2. Qt integration (pass QWidget parent)
webview = create_webview(parent=maya_main_window(), url="http://localhost:3000")
webview.show()

# 3. HWND integration (pass int HWND)
webview = create_webview(parent=unreal_hwnd, url="http://localhost:3000")
webview.show()
```

## How It Works

The `create_webview()` function automatically detects the parent type and selects the appropriate implementation:

| Parent Type | Implementation | Use Case |
|-------------|----------------|----------|
| `None` | `WebView` | Standalone desktop apps |
| `QWidget` | `QtWebView` | Qt-based DCC (Maya, Houdini, Nuke) |
| `int` (HWND) | `WebView` with embed mode | Unreal Engine, custom apps |

## Unified Parameters

All parameters are normalized across implementations:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `parent` | `QWidget/int/None` | `None` | Parent widget or HWND |
| `title` | `str` | `"AuroraView"` | Window title |
| `width` | `int` | `800` | Window width in pixels |
| `height` | `int` | `600` | Window height in pixels |
| `url` | `str` | `None` | URL to load |
| `html` | `str` | `None` | HTML content to load |
| `debug` | `bool` | `True` | Enable DevTools (F12) |
| `context_menu` | `bool` | `True` | Enable right-click menu |
| `frame` | `bool` | `True` | Show window frame |
| `transparent` | `bool` | `False` | Transparent background |
| `background_color` | `str` | `None` | Background color (CSS) |
| `asset_root` | `str` | `None` | Custom protocol root |
| `allow_file_protocol` | `bool` | `False` | Enable file:// |
| `mode` | `str` | `"auto"` | Embedding mode |
| `api` | `Any` | `None` | API object for JS |

## Embedding Modes

The `mode` parameter controls how the WebView relates to its parent:

| Mode | Description | Best For |
|------|-------------|----------|
| `"none"` | Standalone window | Desktop apps |
| `"child"` | WS_CHILD embedding | Tight Qt integration |
| `"owner"` | GWLP_HWNDPARENT | Cross-thread safe, DCC tools |
| `"auto"` | Auto-detect | Default (recommended) |

When `mode="auto"` (default):
- No parent → `"none"`
- QWidget parent → `"child"`
- HWND parent → `"owner"`

---

## Window Display (show)

AuroraView provides a **unified `show()` method** that works across all scenarios. You don't need to remember different methods for different use cases.

### Basic Usage

```python
from auroraview import create_webview

# All scenarios use the same show() method
webview = create_webview(url="http://localhost:3000")
webview.show()  # That's it!
```

### How show() Works

The `show()` method automatically detects your environment and behaves appropriately:

| Scenario | Behavior | Blocking? |
|----------|----------|-----------|
| Standalone (no parent) | Opens window, runs event loop | Yes (blocks until closed) |
| Qt Widget parent | Shows widget, starts event timer | No (returns immediately) |
| HWND parent | Opens embedded window | No (returns immediately) |
| Packed mode (.exe) | Runs as API server | Yes (blocks for requests) |

### Standalone Mode

For standalone desktop applications, `show()` blocks until the window is closed:

```python
from auroraview import create_webview

webview = create_webview(
    url="http://localhost:3000",
    title="My Desktop App",
    width=1024,
    height=768
)

# Blocks here until user closes the window
webview.show()

print("Window closed!")  # Runs after window closes
```

**Force non-blocking** (advanced):

```python
webview.show(wait=False)  # Returns immediately
# WARNING: Window closes when script exits!
# Keep script alive:
input("Press Enter to exit...")
```

### Qt Integration Mode

When using a Qt parent, `show()` returns immediately and integrates with Qt's event loop:

```python
from auroraview import create_webview
from PySide2.QtWidgets import QMainWindow, QDockWidget

class MyDCCTool(QMainWindow):
    def __init__(self):
        super().__init__()

        # Create WebView with Qt parent
        self.webview = create_webview(
            parent=self,  # Pass Qt widget as parent
            url="http://localhost:3000",
            title="My Tool"
        )

        # Add to dock
        dock = QDockWidget("Web Panel", self)
        dock.setWidget(self.webview)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)

        # No need to call show() - WebView auto-initializes via showEvent
        # when parent window becomes visible
```

**Key points for Qt integration:**

1. **Auto-initialization via `showEvent`** - WebView initializes automatically when the Qt widget becomes visible (standard Qt semantics)
2. **No explicit `show()` needed** - When embedded in a layout/dock, showing the parent triggers `showEvent`
3. **Qt manages lifecycle** - WebView follows Qt widget lifecycle
4. **Event timer** - AuroraView starts a timer to process WebView events

### Embedding in Qt Layouts

When embedding WebView in Qt layouts, you have two options:

**Option 1: Direct embedding (recommended)**

```python
from auroraview import create_webview
from PySide2.QtWidgets import QWidget, QVBoxLayout

class MyPanel(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        layout = QVBoxLayout(self)

        # Create WebView as child widget
        self.webview = create_webview(
            parent=self,
            url="http://localhost:3000"
        )
        layout.addWidget(self.webview)

        # No need to call show() - it happens when parent is shown
```

**Option 2: Explicit show (rarely needed)**

```python
# Only if you need to force initialization before parent is shown:
self.webview = create_webview(parent=self, url="http://localhost:3000")
layout.addWidget(self.webview)
self.webview.show()  # Force immediate initialization (usually not needed)
```

### DCC-Specific Integration

#### Maya

```python
from auroraview import create_webview
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

def get_maya_window():
    ptr = omui.MQtUtil.mainWindow()
    return wrapInstance(int(ptr), QWidget)

# Create as child of Maya main window
webview = create_webview(
    parent=get_maya_window(),
    url="http://localhost:3000",
    title="Maya Tool"
)
# No explicit show() needed - initializes when Maya window layout updates
```

#### Houdini

```python
from auroraview import create_webview
import hou

webview = create_webview(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    title="Houdini Tool"
)
webview.show()  # Non-blocking
```

#### Blender (Floating Window)

```python
from auroraview import create_webview

# Blender doesn't use Qt, so we create a floating window
webview = create_webview(
    url="http://localhost:3000",
    title="Blender Tool",
    always_on_top=True  # Keep above Blender
)
webview.show(wait=False)  # Non-blocking for Blender integration
```

#### Unreal Engine

```python
from auroraview import create_webview
import unreal

hwnd = unreal.get_editor_window_hwnd()

webview = create_webview(
    parent=hwnd,  # Pass HWND directly
    url="http://localhost:3000",
    mode="owner"  # Use owner mode for cross-thread safety
)
webview.show()  # Non-blocking
```

### Summary: One Method, All Scenarios

| Use Case | Code | Notes |
|----------|------|-------|
| Desktop app | `webview.show()` | Blocks until closed |
| Qt dock panel | `webview.show()` | Non-blocking, Qt lifecycle |
| Qt layout child | Add to layout, parent shows | Auto-initializes |
| Maya/Houdini | `webview.show()` | Non-blocking |
| Blender | `webview.show(wait=False)` | Floating window |
| Unreal | `webview.show()` | Non-blocking, HWND embed |

**Remember:** Just use `show()` - AuroraView handles the rest!

---

## API Binding

AuroraView provides two methods for exposing Python functions to JavaScript:

### bind_call - Bind Individual Functions

Use `bind_call()` to bind a single Python function:

```python
from auroraview import create_webview

webview = create_webview(url="http://localhost:3000")

# Method 1: Direct binding
def echo(message: str) -> str:
    return f"Echo: {message}"

webview.bind_call("api.echo", echo)

# Method 2: Decorator style
@webview.bind_call("api.greet")
def greet(name: str) -> str:
    return f"Hello, {name}!"

webview.show()
```

**JavaScript side:**

```javascript
// Call bound functions
const result = await auroraview.api.echo({ message: "Hello" });
console.log(result);  // "Echo: Hello"

const greeting = await auroraview.api.greet({ name: "World" });
console.log(greeting);  // "Hello, World!"
```

### bind_api - Bind Object Methods

Use `bind_api()` to expose all public methods of an object:

```python
from auroraview import create_webview

class MyAPI:
    def echo(self, message: str) -> str:
        return f"Echo: {message}"

    def add(self, a: int, b: int) -> int:
        return a + b

    def get_data(self) -> dict:
        return {"status": "ok", "count": 42}

    def _private_method(self):
        """Methods starting with _ are not exposed"""
        pass

api = MyAPI()
webview = create_webview(url="http://localhost:3000", api=api)
# Or: webview.bind_api(api)
webview.show()
```

**JavaScript side:**

```javascript
const echo = await auroraview.api.echo({ message: "test" });
const sum = await auroraview.api.add({ a: 1, b: 2 });
const data = await auroraview.api.get_data();
```

### Parameter Passing Conventions

| Python Call | params Type | JavaScript Call |
|-------------|-------------|-----------------|
| `func()` | None | `auroraview.api.func()` |
| `func(**params)` | `dict` | `auroraview.api.func({key: value})` |
| `func(*params)` | `list` | `auroraview.api.func([arg1, arg2])` |
| `func(params)` | Other | `auroraview.api.func(value)` |

### Idempotency and Rebinding

```python
# Safe: bind_api is idempotent at namespace level
webview.bind_api(api)  # First binding
webview.bind_api(api)  # Silently skipped (already bound)

# Force rebind if needed
webview.bind_api(api, allow_rebind=True)

# Check binding status
if webview.is_namespace_bound("api"):
    print("API namespace already bound")

if webview.is_method_bound("api.echo"):
    print("echo method already bound")

# List all bound methods
methods = webview.get_bound_methods()
print(methods)  # ["api.echo", "api.add", "api.get_data"]
```

---

## Thread Safety

### DCC Thread Safety Mode

DCC applications (Maya, Houdini, Blender, etc.) require UI operations to run on the main thread. AuroraView provides automatic thread safety:

```python
from auroraview import create_webview

# Auto-detect DCC environment (default)
webview = create_webview(parent=maya_hwnd, dcc_mode="auto")

# Explicit modes
webview = create_webview(parent=maya_hwnd, dcc_mode=True)   # Always enable
webview = create_webview(parent=maya_hwnd, dcc_mode=False)  # Disable (standalone)
```

### Thread-Safe Event Handlers

When `dcc_mode` is enabled, event handlers automatically run on the DCC main thread:

```python
@webview.on("create_object")
def handle_create(data):
    # Automatically runs on Maya main thread!
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### Manual Thread Dispatching

For fine-grained control, use the thread dispatcher utilities:

```python
from auroraview.utils.thread_dispatcher import (
    run_on_main_thread,
    run_on_main_thread_sync,
    dcc_thread_safe,
    is_main_thread,
)

# Fire-and-forget execution
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

run_on_main_thread(update_viewport)

# Blocking execution with return value
def get_selection():
    import maya.cmds as cmds
    return cmds.ls(selection=True)

selected = run_on_main_thread_sync(get_selection)

# Decorator style
@dcc_thread_safe
def safe_operation():
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### Thread-Safe WebView Wrapper

For cross-thread WebView operations:

```python
webview = create_webview(parent=dcc_hwnd)
webview.show()

# Get thread-safe wrapper
safe = webview.thread_safe()

# These can be called from any thread:
safe.eval_js("updateUI()")
safe.emit("status", {"ready": True})
safe.load_url("https://example.com")

# Blocking JavaScript execution
title = safe.eval_js_sync("document.title", timeout_ms=5000)
```

### API Binding Thread Safety

All binding operations are protected by locks:

```python
# Thread-safe: Multiple threads can call bind_call/bind_api
import threading

def bind_in_thread():
    webview.bind_call("api.method", some_function)

threads = [threading.Thread(target=bind_in_thread) for _ in range(10)]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

---

## Advanced Scenarios

### Custom Namespace

```python
class SceneAPI:
    def export(self, path: str) -> bool:
        # Export logic
        return True

class ToolAPI:
    def apply(self, settings: dict) -> dict:
        # Apply tool settings
        return {"status": "applied"}

webview.bind_api(SceneAPI(), namespace="scene")
webview.bind_api(ToolAPI(), namespace="tool")
```

**JavaScript:**

```javascript
await auroraview.scene.export({ path: "/tmp/scene.fbx" });
await auroraview.tool.apply({ settings: { strength: 0.8 } });
```

### Error Handling

Python exceptions are automatically propagated to JavaScript:

```python
@webview.bind_call("api.risky_operation")
def risky_operation():
    raise ValueError("Something went wrong")
```

**JavaScript:**

```javascript
try {
    await auroraview.api.risky_operation();
} catch (error) {
    console.error(error.name);     // "ValueError"
    console.error(error.message);  // "Something went wrong"
}
```

### Async Operations with Channels

For streaming data or long-running operations:

```python
@webview.bind_call("api.process_large_file")
def process_large_file(path: str):
    channel = webview.create_channel()

    def process():
        with open(path, 'rb') as f:
            total = os.path.getsize(path)
            processed = 0
            while chunk := f.read(8192):
                processed += len(chunk)
                channel.send({"progress": processed / total * 100})
            channel.close()

    threading.Thread(target=process).start()
    return {"channel_id": channel.id}
```

**JavaScript:**

```javascript
const { channel_id } = await auroraview.api.process_large_file({ path: "/large/file" });
auroraview.on(`channel:${channel_id}`, (data) => {
    console.log(`Progress: ${data.progress}%`);
});
```

### Hot Reload Support

Bound functions support hot-reload scenarios:

```python
def echo_v1(message: str) -> str:
    return f"v1: {message}"

webview.bind_call("api.echo", echo_v1)

# Later, update the function
def echo_v2(message: str) -> str:
    return f"v2: {message}"

webview.bind_call("api.echo", echo_v2, allow_rebind=True)
# Now JavaScript calls will use echo_v2
```

---

## DCC Integration Examples

### Maya Integration

```python
from auroraview import create_webview
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

def get_maya_window():
    ptr = omui.MQtUtil.mainWindow()
    return wrapInstance(int(ptr), QWidget)

class MayaAPI:
    def create_cube(self) -> str:
        import maya.cmds as cmds
        return cmds.polyCube()[0]

    def get_selection(self) -> list:
        import maya.cmds as cmds
        return cmds.ls(selection=True)

    def export_fbx(self, path: str) -> bool:
        import maya.cmds as cmds
        cmds.file(path, exportSelected=True, type="FBX export")
        return True

webview = create_webview(
    parent=get_maya_window(),
    url="http://localhost:3000",
    title="My Maya Tool",
    api=MayaAPI()
)
webview.show()
```

### Houdini Integration

```python
from auroraview import create_webview
import hou

class HoudiniAPI:
    def create_node(self, node_type: str, name: str) -> str:
        obj = hou.node("/obj")
        node = obj.createNode(node_type, name)
        return node.path()

    def get_selected_nodes(self) -> list:
        return [n.path() for n in hou.selectedNodes()]

webview = create_webview(
    parent=hou.qt.mainWindow(),
    url="http://localhost:3000",
    title="My Houdini Tool",
    api=HoudiniAPI()
)
webview.show()
```

### Unreal Engine Integration

```python
from auroraview import create_webview
import unreal

hwnd = unreal.get_editor_window_hwnd()

class UnrealAPI:
    def spawn_actor(self, class_name: str, location: list) -> str:
        # Spawn actor logic
        return "actor_id"

    def get_selected_actors(self) -> list:
        return [str(a) for a in unreal.EditorLevelLibrary.get_selected_level_actors()]

webview = create_webview(
    parent=hwnd,
    url="http://localhost:3000",
    title="My Unreal Tool",
    mode="owner",
    api=UnrealAPI()
)
webview.show()
```

### Standalone Desktop App

```python
from auroraview import create_webview, run_app

webview = create_webview(
    url="http://localhost:3000",
    title="My App",
    width=1024,
    height=768
)
webview.show()

# Or use convenience function
run_app(url="http://localhost:3000", title="My App")
```

---

## Migration Guide

### From WebView

```python
# Before
from auroraview.core import WebView
webview = WebView(
    title="Tool",
    parent_hwnd=hwnd,
    embed_mode="owner",
    dev_tools=True,
    decorations=False
)

# After
from auroraview import create_webview
webview = create_webview(
    title="Tool",
    parent=hwnd,
    mode="owner",
    debug=True,
    frame=False
)
```

### From QtWebView

```python
# Before
from auroraview import QtWebView
webview = QtWebView(
    parent=widget,
    dev_tools=True,
    frameless=True
)

# After
from auroraview import create_webview
webview = create_webview(
    parent=widget,
    debug=True,
    frame=False
)
```

### From AuroraView

```python
# Before
from auroraview import AuroraView
webview = AuroraView(
    url="http://localhost:3000",
    debug=True,
    api=my_api
)

# After
from auroraview import create_webview
webview = create_webview(
    url="http://localhost:3000",
    debug=True,
    api=my_api
)
```

## Parameter Mapping

The unified API normalizes parameter names:

| Unified | WebView | QtWebView | AuroraView |
|---------|---------|-----------|------------|
| `debug` | `debug` / `dev_tools` | `dev_tools` | `debug` |
| `frame` | `frame` / `decorations` | `frameless` (inverted) | - |
| `parent` | `parent` / `parent_hwnd` | `parent` | `parent` / `parent_hwnd` |
| `mode` | `mode` / `embed_mode` | `embed_mode` | `embed_mode` |

## Backward Compatibility

The legacy APIs (`WebView`, `QtWebView`, `AuroraView`) are still fully supported:

```python
# All of these still work
from auroraview import WebView, QtWebView, AuroraView
from auroraview.core import WebView
from auroraview.integration import QtWebView, AuroraView
```

---

## Best Practices

### API Design

1. **Use descriptive method names** - `export_scene()` not `exp()`
2. **Return structured data** - Return dicts instead of multiple values
3. **Handle errors gracefully** - Raise meaningful exceptions
4. **Document parameters** - Use type hints and docstrings

```python
class WellDesignedAPI:
    def export_scene(self, path: str, format: str = "fbx") -> dict:
        """Export the current scene.

        Args:
            path: Export file path
            format: Export format (fbx, obj, gltf)

        Returns:
            dict with keys: success, path, size
        """
        # Implementation
        return {"success": True, "path": path, "size": 1024}
```

### Thread Safety

1. **Use `dcc_mode="auto"`** - Let AuroraView detect the environment
2. **Avoid blocking the main thread** - Use channels for long operations
3. **Use `thread_safe()` wrapper** - For cross-thread WebView access

### Performance

1. **Batch API calls** - Reduce round-trips between JS and Python
2. **Use channels for streaming** - Don't poll for progress
3. **Lazy load heavy modules** - Import DCC modules inside functions

```python
class PerformantAPI:
    def batch_operation(self, items: list) -> list:
        """Process multiple items in one call"""
        return [self._process_item(item) for item in items]

    def _process_item(self, item):
        # Lazy import
        import maya.cmds as cmds
        return cmds.polyCube()[0]
```

### Maintainability

1. **Separate API classes by domain** - `SceneAPI`, `ToolAPI`, `RenderAPI`
2. **Use namespaces** - `bind_api(api, namespace="scene")`
3. **Version your API** - Consider backward compatibility

```python
# Organized API structure
class SceneAPI:
    """Scene management operations"""
    pass

class ToolAPI:
    """Tool operations"""
    pass

class RenderAPI:
    """Rendering operations"""
    pass

webview.bind_api(SceneAPI(), namespace="scene")
webview.bind_api(ToolAPI(), namespace="tool")
webview.bind_api(RenderAPI(), namespace="render")
```

---

## Troubleshooting

### Common Issues

**Q: JavaScript calls return undefined**

Check that:
1. The method is properly bound: `webview.is_method_bound("api.method")`
2. The page has loaded: `webview.is_loaded()`
3. The method name matches exactly (case-sensitive)

**Q: Thread safety errors in DCC**

Ensure:
1. `dcc_mode="auto"` or `dcc_mode=True` is set
2. Use `@dcc_thread_safe` decorator for manual handlers
3. Use `thread_safe()` wrapper for cross-thread access

**Q: API not available in JavaScript**

Verify:
1. `bind_api()` or `bind_call()` was called before `show()`
2. Check browser console for errors
3. Ensure `window.auroraview` is ready (use `auroraviewready` event)

```javascript
window.addEventListener('auroraviewready', () => {
    // Safe to use auroraview.api.*
    console.log('AuroraView ready');
});
```

**Q: Duplicate binding warnings**

Use idempotent binding:
```python
webview.bind_api(api)  # Safe to call multiple times
# Or check first:
if not webview.is_namespace_bound("api"):
    webview.bind_api(api)
```
