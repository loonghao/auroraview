# AuroraView API

`AuroraView` is a high-level wrapper that provides HWND access and automatic API binding.

## Import

```python
from auroraview import AuroraView
```

## Constructor

```python
AuroraView(
    url: str = None,
    html: str = None,
    title: str = "AuroraView",
    width: int = 800,
    height: int = 600,
    api: object = None,
    **kwargs
)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `url` | `str` | `None` | URL to load |
| `html` | `str` | `None` | HTML content to load |
| `title` | `str` | `"AuroraView"` | Window title |
| `width` | `int` | `800` | Window width |
| `height` | `int` | `600` | Window height |
| `api` | `object` | `None` | API object to auto-bind |
| `**kwargs` | | | Additional WebView options |

## Basic Usage

```python
from auroraview import AuroraView

class MyAPI:
    def get_data(self) -> dict:
        return {"items": [1, 2, 3]}

    def save_file(self, path: str = "", content: str = "") -> dict:
        with open(path, "w") as f:
            f.write(content)
        return {"ok": True}

# Create with auto-bound API
view = AuroraView(
    url="http://localhost:3000",
    api=MyAPI()
)
view.show()
```

## Methods

### show()

Display the window.

```python
view.show()
```

### hide()

Hide the window.

```python
view.hide()
```

### close()

Close the window.

```python
view.close()
```

### get_hwnd() -> int | None

Get the window handle (Windows only).

```python
hwnd = view.get_hwnd()
if hwnd:
    # Use HWND for external integration
    print(f"Window handle: {hwnd}")
```

### emit(event: str, data: Any)

Emit an event to JavaScript.

```python
view.emit("update", {"value": 42})
```

### on(event: str)

Register an event handler.

```python
@view.on("button_clicked")
def handle_click(data):
    print(f"Clicked: {data}")
```

## HWND Integration

`AuroraView` provides direct HWND access for integration with non-Qt applications:

```python
from auroraview import AuroraView

view = AuroraView(url="http://localhost:3000")
view.show()

# Get HWND
hwnd = view.get_hwnd()

# Use with Unreal Engine
if hwnd:
    import unreal
    unreal.parent_external_window_to_slate(hwnd)
```

## API Auto-Binding

When you pass an `api` object, all public methods are automatically bound:

```python
class MyAPI:
    def get_items(self) -> dict:
        """Available as auroraview.api.get_items()"""
        return {"items": [1, 2, 3]}

    def create_item(self, name: str = "") -> dict:
        """Available as auroraview.api.create_item({name: "..."})"""
        return {"ok": True, "name": name}

    def _private_method(self):
        """Not exposed (starts with _)"""
        pass

view = AuroraView(url="http://localhost:3000", api=MyAPI())
```

```javascript
// JavaScript side
const items = await auroraview.api.get_items();
const result = await auroraview.api.create_item({ name: "New Item" });
```

## Parent Window Integration

```python
from auroraview import AuroraView

# Create with parent window
view = AuroraView(
    url="http://localhost:3000",
    parent_hwnd=parent_window_handle,
    parent_mode="owner"  # Follow parent minimize/restore
)
view.show()
```

### Parent Modes

| Mode | Description |
|------|-------------|
| `"owner"` | Window follows parent minimize/restore |
| `"child"` | Window is embedded in parent |
| `None` | Independent window |

## Example: Unreal Engine Integration

```python
from auroraview import AuroraView

class UnrealAPI:
    def get_actors(self) -> dict:
        import unreal
        actors = unreal.EditorLevelLibrary.get_all_level_actors()
        return {"actors": [str(a.get_name()) for a in actors]}

    def select_actor(self, name: str = "") -> dict:
        import unreal
        # Selection logic
        return {"ok": True}

# Create WebView
view = AuroraView(
    url="http://localhost:3000",
    api=UnrealAPI()
)
view.show()

# Parent to Unreal window
hwnd = view.get_hwnd()
if hwnd:
    import unreal
    unreal.parent_external_window_to_slate(hwnd)
```

## Example: Standalone Tool

```python
from auroraview import AuroraView
import os

class FileAPI:
    def list_files(self, path: str = ".") -> dict:
        files = os.listdir(path)
        return {"files": files, "count": len(files)}

    def read_file(self, path: str = "") -> dict:
        try:
            with open(path, "r") as f:
                return {"ok": True, "content": f.read()}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def write_file(self, path: str = "", content: str = "") -> dict:
        try:
            with open(path, "w") as f:
                f.write(content)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

view = AuroraView(
    title="File Manager",
    url="http://localhost:3000",
    width=1024,
    height=768,
    api=FileAPI()
)
view.show()
```

## Comparison with Other Classes

| Feature | `AuroraView` | `QtWebView` | `WebView` |
|---------|-------------|-------------|-----------|
| HWND Access | ✅ | ❌ | ❌ |
| Auto API Binding | ✅ | ❌ | ❌ |
| Qt Widget | ❌ | ✅ | ❌ |
| Docking Support | ❌ | ✅ | ❌ |
| Low-level Control | ❌ | ❌ | ✅ |
