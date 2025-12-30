# Unreal Engine Integration

AuroraView integrates with Unreal Engine through Python scripting and native HWND embedding.

## Architecture

```
┌─────────────────────────────────────────────┐
│            Unreal Engine Editor             │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  Slate UI   │ ◄──► │  AuroraView      │ │
│  │  Container  │      │  (WebView2)      │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ HWND                 │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │      Python / Blueprints API        │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## Requirements

| Component | Minimum Version | Recommended |
|-----------|-----------------|-------------|
| Unreal Engine | 5.0 | 5.3+ |
| Python | 3.9 | 3.11+ |
| OS | Windows 10 | Windows 11 |

## Integration Mode

Unreal Engine uses **Native Mode (HWND)** for WebView embedding:

- No Qt dependency required
- Direct HWND embedding into Slate containers
- Uses `register_slate_post_tick_callback()` for main thread execution

## Setup Guide

### Step 1: Enable Python Plugin

1. Open **Edit → Plugins**
2. Search for "Python Editor Script Plugin"
3. Enable the plugin
4. Restart Unreal Editor

### Step 2: Install AuroraView

```python
# In Unreal Python console or startup script
import subprocess
import sys

# Install to Unreal's Python environment
subprocess.check_call([sys.executable, "-m", "pip", "install", "auroraview"])
```

### Step 3: Basic Usage

```python
import unreal
from auroraview import WebView

# Get editor window HWND
def get_editor_hwnd():
    # Platform-specific HWND retrieval
    import ctypes
    return ctypes.windll.user32.GetForegroundWindow()

# Create WebView with Unreal as parent
webview = WebView.create(
    title="My Unreal Tool",
    parent=get_editor_hwnd(),
    mode="owner",
    width=800,
    height=600,
)

webview.load_url("http://localhost:3000")
webview.show()
```

## Thread Dispatcher

AuroraView provides a thread dispatcher backend for Unreal Engine:

```python
from auroraview.utils import run_on_main_thread, ensure_main_thread

@ensure_main_thread
def update_actor_transform(actor_name, location):
    """This function always runs on the game thread."""
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(location, False, False)

# Safe to call from any thread
update_actor_transform("MyActor", unreal.Vector(100, 200, 300))
```

### Unreal Backend Implementation

The Unreal dispatcher backend uses Slate tick callbacks:

```python
from auroraview.utils.thread_dispatcher import (
    ThreadDispatcherBackend,
    register_dispatcher_backend
)

class UnrealDispatcherBackend(ThreadDispatcherBackend):
    """Thread dispatcher for Unreal Engine."""
    
    def is_available(self) -> bool:
        try:
            import unreal
            return True
        except ImportError:
            return False
    
    def run_deferred(self, func, *args, **kwargs):
        import unreal
        unreal.register_slate_post_tick_callback(
            lambda _: func(*args, **kwargs)
        )
    
    def run_sync(self, func, *args, **kwargs):
        import unreal
        import threading
        
        if self.is_main_thread():
            return func(*args, **kwargs)
        
        result = [None]
        error = [None]
        event = threading.Event()
        
        def wrapper(_):
            try:
                result[0] = func(*args, **kwargs)
            except Exception as e:
                error[0] = e
            finally:
                event.set()
        
        unreal.register_slate_post_tick_callback(wrapper)
        event.wait()
        
        if error[0]:
            raise error[0]
        return result[0]
    
    def is_main_thread(self) -> bool:
        import unreal
        return unreal.is_in_game_thread()

# Register with high priority
register_dispatcher_backend(UnrealDispatcherBackend, priority=150)
```

## Editor Utility Widget

Create a custom Editor Utility Widget to host AuroraView:

### Blueprint Setup

1. Create new **Editor Utility Widget**
2. Add a **Named Slot** widget
3. Name it "WebViewContainer"

### Python Integration

```python
import unreal
from auroraview import WebView

@unreal.uclass()
class AuroraViewWidget(unreal.EditorUtilityWidget):
    
    webview = None
    
    @unreal.ufunction(override=True)
    def construct(self):
        # Get the container widget's HWND
        container = self.get_widget_from_name("WebViewContainer")
        if container:
            hwnd = self._get_widget_hwnd(container)
            
            self.webview = WebView.create(
                parent=hwnd,
                mode="child",
            )
            self.webview.load_url("http://localhost:3000")
            self.webview.show()
    
    @unreal.ufunction(override=True)
    def destruct(self):
        if self.webview:
            self.webview.close()
            self.webview = None
```

## API Communication

### Python to JavaScript

```python
from auroraview import WebView

class UnrealAPI:
    def get_selected_actors(self):
        """Get currently selected actors in the editor."""
        import unreal
        actors = unreal.EditorLevelLibrary.get_selected_level_actors()
        return [{"name": a.get_name(), "class": a.get_class().get_name()} 
                for a in actors]
    
    def spawn_actor(self, class_name, location):
        """Spawn an actor at the specified location."""
        import unreal
        actor_class = unreal.load_class(None, class_name)
        loc = unreal.Vector(location['x'], location['y'], location['z'])
        return unreal.EditorLevelLibrary.spawn_actor_from_class(
            actor_class, loc
        ).get_name()

webview = WebView.create(api=UnrealAPI())
```

### JavaScript to Python

```javascript
// Get selected actors
const actors = await auroraview.api.get_selected_actors();
console.log('Selected:', actors);

// Spawn a new actor
const name = await auroraview.api.spawn_actor(
    '/Game/MyBlueprint.MyBlueprint_C',
    { x: 0, y: 0, z: 100 }
);
```

## Troubleshooting

### WebView not displaying

**Cause**: HWND not correctly retrieved or Slate container not ready.

**Solution**: Ensure the widget is fully constructed before creating WebView.

### Python module not found

**Cause**: AuroraView not installed in Unreal's Python environment.

**Solution**: 
```python
import sys
print(sys.executable)  # Check which Python Unreal uses
# Install to that specific Python
```

### Main thread errors

**Cause**: Calling Unreal API from background thread.

**Solution**: Use `@ensure_main_thread` decorator or `run_on_main_thread()`.

## Development Status

| Feature | Status |
|---------|--------|
| Basic Integration | 🚧 In Progress |
| HWND Embedding | 🚧 In Progress |
| Thread Dispatcher | ✅ Supported |
| Editor Utility Widget | 📋 Planned |
| Blueprint Integration | 📋 Planned |

## Resources

- [Unreal Python API](https://docs.unrealengine.com/5.0/en-US/PythonAPI/)
- [Slate UI Framework](https://docs.unrealengine.com/5.0/en-US/slate-ui-framework-in-unreal-engine/)
- [Editor Scripting](https://docs.unrealengine.com/5.0/en-US/scripting-the-unreal-editor-using-python/)
