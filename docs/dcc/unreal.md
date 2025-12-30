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

## Threading Model

### Understanding Unreal's Threading

Unreal Engine has strict threading requirements:

| Thread Type | Description | Safe Operations |
|-------------|-------------|-----------------|
| **Game Thread** | Main thread for gameplay logic | All Unreal API calls |
| **Render Thread** | GPU operations | Rendering only |
| **Background Threads** | Async tasks | Non-Unreal operations |

**Critical**: Most Unreal Python API calls must be made from the **Game Thread**.

### ❌ WRONG: Calling Unreal API from Background Thread

```python
import threading
import unreal

def background_task():
    # DON'T DO THIS - Will crash or cause undefined behavior!
    actors = unreal.EditorLevelLibrary.get_selected_level_actors()
    
thread = threading.Thread(target=background_task)
thread.start()
```

### ✅ CORRECT: Using Thread Dispatcher

AuroraView provides a thread dispatcher backend for Unreal Engine:

```python
from auroraview.utils import run_on_main_thread, ensure_main_thread

@ensure_main_thread
def update_actor_transform(actor_name, location):
    """This function always runs on the game thread."""
    import unreal
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(location, False, False)

# Safe to call from any thread
update_actor_transform("MyActor", unreal.Vector(100, 200, 300))
```

### Fire-and-Forget vs Blocking

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync

# Fire-and-forget (non-blocking)
def create_actor():
    import unreal
    unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.StaticMeshActor,
        unreal.Vector(0, 0, 0)
    )

run_on_main_thread(create_actor)  # Returns immediately

# Blocking with return value
def get_selection():
    import unreal
    return unreal.EditorLevelLibrary.get_selected_level_actors()

actors = run_on_main_thread_sync(get_selection)  # Waits for result
print(f"Selected {len(actors)} actors")
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

### Thread-Safe Event Handlers

When handling events from JavaScript, ensure thread safety:

```python
from auroraview import WebView
from auroraview.utils import ensure_main_thread

webview = WebView.create(api=UnrealAPI())

@webview.on("transform_actor")
@ensure_main_thread
def handle_transform(data):
    """Handle transform event - always runs on game thread."""
    import unreal
    actor_name = data['actor']
    location = data['location']
    
    actor = unreal.EditorLevelLibrary.get_actor_reference(actor_name)
    if actor:
        actor.set_actor_location(
            unreal.Vector(location['x'], location['y'], location['z']),
            False, False
        )
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

### Main thread errors / Crashes

**Cause**: Calling Unreal API from background thread.

**Solution**: Use `@ensure_main_thread` decorator or `run_on_main_thread()`.

```python
# Check if you're on the game thread
import unreal
if unreal.is_in_game_thread():
    # Safe to call Unreal API
    do_unreal_operation()
else:
    # Must dispatch to game thread
    run_on_main_thread(do_unreal_operation)
```

### Deadlock when using `run_on_main_thread_sync`

**Cause**: Calling `run_on_main_thread_sync` from the game thread while it's blocked.

**Solution**: Check thread before calling:

```python
from auroraview.utils import is_main_thread, run_on_main_thread_sync

def safe_get_selection():
    def _get():
        import unreal
        return unreal.EditorLevelLibrary.get_selected_level_actors()
    
    if is_main_thread():
        return _get()
    else:
        return run_on_main_thread_sync(_get)
```

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

## See Also

- [Thread Dispatcher](../guide/thread-dispatcher) - Unified thread dispatch API
- [DCC Overview](./index) - Overview of all DCC integrations
