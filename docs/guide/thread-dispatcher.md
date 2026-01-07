# Thread Dispatcher

The Thread Dispatcher module provides a unified API for executing code on the main/UI thread across different DCC (Digital Content Creation) applications. Many DCC applications require certain operations to be performed on the main thread for thread safety.

## Overview

Different DCC applications have different APIs for main thread execution:

| DCC Application | Deferred Execution | Blocking Execution |
|----------------|-------------------|-------------------|
| **Maya** | `maya.utils.executeDeferred()` | `maya.utils.executeInMainThreadWithResult()` |
| **Houdini** | `hdefereval.executeDeferred()` | `hdefereval.executeInMainThread()` |
| **Blender** | `bpy.app.timers.register()` | Queue-based approach |
| **Nuke** | `nuke.executeInMainThread()` | `nuke.executeInMainThreadWithResult()` |
| **3ds Max** | `MaxPlus.Core.EvalOnMainThread()` | Wrapper with result |
| **Unreal Engine** | `register_slate_post_tick_callback()` | Event-based approach |
| **Qt Applications** | `QTimer.singleShot()` | Event-based approach |

The Thread Dispatcher abstracts these differences, providing a consistent API that automatically detects and uses the appropriate backend.

## Key Features

- **Lazy Loading**: DCC backends are only loaded when needed, preventing import errors
- **String-based Registration**: Register backends by module path for lazy loading
- **Environment Variable Override**: Force a specific backend via `AURORAVIEW_DISPATCHER`
- **Priority-based Selection**: Higher priority backends are tried first
- **Automatic Detection**: Automatically detects the current DCC environment

## Quick Start

### Basic Usage

```python
from auroraview.utils import run_on_main_thread, run_on_main_thread_sync

# Fire-and-forget execution (non-blocking)
def create_cube():
    import maya.cmds as cmds
    cmds.polyCube()

run_on_main_thread(create_cube)

# Blocking execution with return value
def get_selection():
    import maya.cmds as cmds
    return cmds.ls(selection=True)

selected = run_on_main_thread_sync(get_selection)
print(f"Selected: {selected}")
```

### Using Decorators

```python
from auroraview.utils import ensure_main_thread, defer_to_main_thread

# Ensure function always runs on main thread
@ensure_main_thread
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

# Can be called from any thread safely
update_viewport()

# Fire-and-forget decorator
@defer_to_main_thread
def log_message(msg):
    print(f"[Main Thread] {msg}")

# Returns immediately, executes later on main thread
log_message("Hello from background thread!")
```

### Checking Current Thread

```python
from auroraview.utils import is_main_thread

if is_main_thread():
    # Safe to call DCC APIs directly
    do_dcc_operation()
else:
    # Need to dispatch to main thread
    run_on_main_thread(do_dcc_operation)
```

## API Reference

### Functions

#### `run_on_main_thread(func, *args, **kwargs) -> None`

Execute a function on the main thread without waiting for the result (fire-and-forget).

**Parameters:**
- `func`: Function to execute
- `*args`: Positional arguments
- `**kwargs`: Keyword arguments

**Example:**
```python
def create_sphere(radius):
    import maya.cmds as cmds
    cmds.polySphere(radius=radius)

run_on_main_thread(create_sphere, 2.0)
```

#### `run_on_main_thread_sync(func, *args, **kwargs) -> T`

Execute a function on the main thread and wait for the result (blocking).

**Parameters:**
- `func`: Function to execute
- `*args`: Positional arguments
- `**kwargs`: Keyword arguments

**Returns:** The return value of the function

**Raises:** Re-raises any exception from the function

**Example:**
```python
def get_scene_name():
    import maya.cmds as cmds
    return cmds.file(q=True, sceneName=True)

scene = run_on_main_thread_sync(get_scene_name)
```

#### `is_main_thread() -> bool`

Check if the current thread is the main/UI thread.

**Returns:** `True` if on main thread, `False` otherwise

#### `ensure_main_thread(func) -> Callable`

Decorator that ensures a function runs on the main thread. If called from a background thread, the function is dispatched to the main thread and the call blocks until completion.

**Example:**
```python
@ensure_main_thread
def safe_ui_update():
    # This always runs on main thread
    update_ui_elements()
```

#### `defer_to_main_thread(func) -> Callable`

Decorator that defers function execution to the main thread (fire-and-forget). The decorated function returns `None` immediately.

**Example:**
```python
@defer_to_main_thread
def async_log(message):
    print(message)

async_log("This prints later")  # Returns immediately
```

### Backend Management

#### `get_dispatcher_backend() -> ThreadDispatcherBackend`

Get the currently active backend.

#### `list_dispatcher_backends() -> List[Tuple[int, str, bool]]`

List all registered backends with their priority and availability.

**Returns:** List of `(priority, name, is_available)` tuples

**Example:**
```python
for priority, name, available in list_dispatcher_backends():
    status = "+" if available else "-"
    print(f"{status} {name} (priority={priority})")
```

#### `register_dispatcher_backend(backend, priority=0, *, name="")`

Register a custom backend. Supports both class and string-based registration.

**Parameters:**
- `backend`: Either a `ThreadDispatcherBackend` subclass or a string path in `"module:ClassName"` format
- `priority`: Higher values are tried first (default: 0)
- `name`: Optional display name for the backend

**Example - Class registration:**
```python
register_dispatcher_backend(MyDCCBackend, priority=250)
```

**Example - String registration (lazy loading):**
```python
# Only loaded when get_dispatcher_backend() is called
register_dispatcher_backend(
    "my_package.dispatchers:MyDCCBackend",
    priority=250,
    name="MyDCC"
)
```

#### `unregister_dispatcher_backend(backend) -> bool`

Unregister a previously registered backend.

**Parameters:**
- `backend`: The backend class or string path to unregister

**Returns:** `True` if found and removed, `False` otherwise

#### `clear_dispatcher_backends() -> None`

Clear all registered backends and reset to initial state. Mainly useful for testing.

## Environment Variable Override

You can force a specific backend using the `AURORAVIEW_DISPATCHER` environment variable:

```bash
# Force Qt backend
export AURORAVIEW_DISPATCHER=qt

# Force fallback backend
export AURORAVIEW_DISPATCHER=fallback
```

Valid values (case-insensitive): `maya`, `houdini`, `nuke`, `blender`, `max`, `unreal`, `qt`, `fallback`

## Custom Backends

### Class-based Registration

```python
from auroraview.utils.thread_dispatcher import (
    ThreadDispatcherBackend,
    register_dispatcher_backend
)

class MyDCCBackend(ThreadDispatcherBackend):
    """Custom backend for MyDCC application."""
    
    def is_available(self) -> bool:
        try:
            import mydcc
            return True
        except ImportError:
            return False
    
    def run_deferred(self, func, *args, **kwargs):
        import mydcc
        mydcc.execute_deferred(lambda: func(*args, **kwargs))
    
    def run_sync(self, func, *args, **kwargs):
        import mydcc
        return mydcc.execute_in_main_thread(lambda: func(*args, **kwargs))
    
    def is_main_thread(self) -> bool:
        import mydcc
        return mydcc.is_main_thread()

# Register with high priority
register_dispatcher_backend(MyDCCBackend, priority=250, name="MyDCC")
```

### String-based Registration (Lazy Loading)

For external packages or to avoid import errors when the DCC is not available:

```python
from auroraview.utils import register_dispatcher_backend

# Register by module path - only loaded when needed
register_dispatcher_backend(
    "my_package.dispatchers:MyDCCBackend",
    priority=250,
    name="MyDCC"
)
```

This is especially useful when:
- Your backend depends on DCC-specific modules that may not be installed
- You want to distribute a plugin that works across multiple DCCs
- You need to delay import until the DCC environment is fully initialized

## Built-in Backends

The following backends are registered by default (in priority order):

| Backend | Priority | Description |
|---------|----------|-------------|
| Maya | 200 | Uses `maya.utils` module |
| Houdini | 190 | Uses `hdefereval` module |
| Nuke | 180 | Uses `nuke` module |
| Blender | 170 | Uses `bpy.app.timers` |
| 3ds Max | 160 | Uses `MaxPlus` or `pymxs` |
| Unreal | 150 | Uses slate tick callbacks |
| Qt | 100 | Uses `QTimer.singleShot()` |
| Fallback | 0 | Direct execution (warning logged) |

## Use Cases

### Background Processing with UI Updates

```python
import threading
from auroraview.utils import run_on_main_thread

def process_data(data):
    """Process data in background thread."""
    result = heavy_computation(data)
    
    # Update UI on main thread
    run_on_main_thread(update_progress_bar, 100)
    run_on_main_thread(show_result, result)

# Start background thread
thread = threading.Thread(target=process_data, args=(my_data,))
thread.start()
```

### Thread-Safe Event Handlers

```python
from auroraview import WebView
from auroraview.utils import ensure_main_thread

webview = WebView()

@webview.on("export_scene")
@ensure_main_thread
def handle_export(data):
    """Handle export event - always runs on main thread."""
    import maya.cmds as cmds
    cmds.file(data['path'], exportSelected=True, type='mayaAscii')
```

### Async Operations with Results

```python
from concurrent.futures import ThreadPoolExecutor
from auroraview.utils import run_on_main_thread_sync

def get_scene_info():
    """Get scene info from main thread."""
    return run_on_main_thread_sync(lambda: {
        'name': cmds.file(q=True, sceneName=True),
        'objects': len(cmds.ls(dag=True)),
    })

with ThreadPoolExecutor() as executor:
    future = executor.submit(get_scene_info)
    info = future.result()
```

## Best Practices

1. **Minimize main thread work**: Keep main thread operations short to avoid UI freezes.

2. **Batch operations**: Group multiple DCC operations into a single main thread call.

3. **Use deferred for fire-and-forget**: Use `run_on_main_thread` when you don't need the result.

4. **Handle exceptions**: Wrap main thread operations in try/except when using `run_sync`.

5. **Check thread before dispatching**: Use `is_main_thread()` to avoid unnecessary dispatching.

```python
from auroraview.utils import is_main_thread, run_on_main_thread_sync

def safe_operation():
    if is_main_thread():
        return do_operation()
    else:
        return run_on_main_thread_sync(do_operation)
```

6. **Use string registration for plugins**: When distributing plugins, use string-based registration to avoid import errors.

## Troubleshooting

### Function not executing

**Cause**: The DCC application's event loop is not running.

**Solution**: Ensure the application is in an interactive state, not during startup or shutdown.

### Deadlock when using `run_sync`

**Cause**: Calling `run_on_main_thread_sync` from the main thread while the main thread is blocked.

**Solution**: Check `is_main_thread()` before calling, or use `run_on_main_thread` (non-blocking) instead.

### Wrong backend selected

**Cause**: Multiple DCC environments detected.

**Solution**: Register a custom backend with higher priority or set `AURORAVIEW_DISPATCHER` environment variable.

### Import errors for DCC modules

**Cause**: DCC-specific modules are not available in the current environment.

**Solution**: Use string-based registration for lazy loading, or check availability before importing.

## See Also

- [Qt Integration](./qt-integration) - Qt-specific integration guide
- [Maya Integration](../dcc/maya) - Maya-specific guide
- [DCC Overview](../dcc/) - Overview of all DCC integrations

## DCC Thread Safety for WebView

When integrating AuroraView WebView into DCC applications, you need to handle thread safety between the WebView thread and the DCC main thread. AuroraView provides specialized utilities for this purpose.

### The Challenge

- **WebView Thread**: WebView2 runs on its own STA thread
- **DCC Main Thread**: DCC APIs (Maya cmds, Blender bpy, etc.) must be called from the main thread
- **Event Handlers**: `@webview.on()` handlers may be called from the WebView thread

### Using `@dcc_thread_safe` Decorator

The `@dcc_thread_safe` decorator automatically marshals function execution to the DCC main thread:

```python
from auroraview import WebView
from auroraview.utils import dcc_thread_safe

webview = WebView(parent=dcc_hwnd)

@webview.on("create_object")
@dcc_thread_safe  # Ensures this runs on DCC main thread
def handle_create(data):
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### Using `dcc_mode`

Enable `dcc_mode` on WebView to automatically wrap all callbacks:

```python
# All callbacks automatically run on DCC main thread
webview = WebView(parent=dcc_hwnd, dcc_mode=True)

@webview.on("create_object")
def handle_create(data):  # No decorator needed!
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### Thread-Safe Wrapper

Use `thread_safe()` for cross-thread WebView operations:

```python
webview = WebView(parent=dcc_hwnd)

# Get thread-safe wrapper
safe = webview.thread_safe()

# Can be called from any thread:
safe.eval_js("updateStatus('ready')")
safe.emit("data_loaded", {"count": 100})
safe.load_url("https://example.com")
```

### Fire-and-Forget with `@dcc_thread_safe_async`

For operations that don't need a return value:

```python
from auroraview.utils import dcc_thread_safe_async

@dcc_thread_safe_async
def update_viewport():
    import maya.cmds as cmds
    cmds.refresh()

# Returns immediately, executes on main thread later
update_viewport()
```

### See Also

- [RFC 0002: DCC Thread Safety](/rfcs/0002-dcc-thread-safety) - Detailed design document
