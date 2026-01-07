# RFC 0002: DCC Thread Safety

> **Status**: Implementing (Phase 1 Complete)
> **Author**: AuroraView Team
> **Created**: 2025-01-07
> **Updated**: 2025-01-07
> **Target Version**: v0.5.0

## Summary

This RFC proposes a comprehensive thread safety solution for AuroraView when integrated with DCC (Digital Content Creation) applications like Maya, Houdini, Blender, Nuke, 3ds Max, and Unreal Engine. The solution addresses the fundamental challenge that both WebView2 and DCC applications have strict threading requirements that must be respected.

## Motivation

### The Thread Safety Challenge

When integrating AuroraView into DCC applications, we face a multi-threading challenge:

1. **WebView2 Requirements**:
   - WebView2 COM objects must be created and used on an STA (Single-Threaded Apartment) thread
   - All WebView2 operations (navigation, JavaScript execution, etc.) must happen on the thread that created the controller
   - The message pump must be running on the creation thread

2. **DCC Application Requirements**:
   - Most DCC APIs (Maya cmds, Blender bpy, etc.) can only be called from the main/UI thread
   - UI updates must happen on the main thread
   - Background threads cannot directly manipulate scene data

3. **AuroraView Integration Pattern**:
   - In HWND/Child mode, WebView runs in a background thread to avoid blocking the DCC main thread
   - Event handlers registered via `@webview.on()` may be called from the WebView thread
   - Python callbacks need to safely call DCC APIs

### Current State

The project already has:
- `WebViewProxy` (Rust): Thread-safe proxy for cross-thread WebView operations
- `MessageQueue` (Rust): Thread-safe message queue using crossbeam-channel
- `thread_dispatcher.py` (Python): Unified API for main thread execution across DCCs
- COM initialization utilities for STA mode

However, these components are not well-integrated, making it difficult for users to write thread-safe code.

## Design Proposal

### 1. Thread-Safe Event Handlers

Add a decorator that automatically marshals event handler execution to the DCC main thread:

```python
from auroraview import WebView
from auroraview.utils import dcc_thread_safe

webview = WebView(parent=dcc_hwnd)

@webview.on("save_scene")
@dcc_thread_safe  # Automatically runs on DCC main thread
def handle_save(data):
    import maya.cmds as cmds
    path = data.get("path")
    cmds.file(rename=path)
    cmds.file(save=True)
    return {"status": "saved", "path": path}
```

### 2. Enhanced WebView API Integration

Add `dcc_mode` option to WebView that automatically wraps all callbacks:

```python
# Enable DCC mode - all callbacks automatically run on main thread
webview = WebView(
    parent=dcc_hwnd,
    dcc_mode=True,  # Enable automatic thread marshaling
)

# No decorator needed - automatically thread-safe
@webview.on("create_object")
def handle_create(data):
    import maya.cmds as cmds
    return cmds.polyCube()[0]
```

### 3. Thread-Safe API Methods

Provide decorated versions of common WebView methods for cross-thread usage:

```python
from auroraview import WebView

webview = WebView(...)

# Get a thread-safe wrapper for cross-thread calls
safe_webview = webview.thread_safe()

# Can be called from any thread
safe_webview.eval_js("console.log('Hello')")  # Thread-safe
safe_webview.emit("status", {"ready": True})  # Thread-safe
safe_webview.load_url("https://example.com")  # Thread-safe
```

### 4. DCC-Aware Proxy

Enhance the `WebViewProxy` to integrate with the thread dispatcher:

```python
proxy = webview.get_proxy()

# Register a callback that runs on DCC main thread
proxy.on_dcc_main_thread("scene_changed", handle_scene_change)

# Emit event and get result on DCC main thread
result = proxy.call_on_dcc_main_thread(get_scene_info)
```

## Implementation Details

### New Python Module: `auroraview/utils/dcc_thread_safe.py`

```python
"""DCC Thread Safety utilities.

This module provides decorators and utilities for thread-safe DCC integration.
"""

from functools import wraps
from typing import Callable, TypeVar, Any
from .thread_dispatcher import (
    is_main_thread,
    run_on_main_thread,
    run_on_main_thread_sync,
)

T = TypeVar("T")


def dcc_thread_safe(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to ensure function runs on DCC main thread.
    
    This is an alias for ensure_main_thread with DCC-specific documentation.
    Use this for WebView event handlers that need to call DCC APIs.
    
    Example:
        @webview.on("export")
        @dcc_thread_safe
        def handle_export(data):
            import maya.cmds as cmds
            cmds.file(exportSelected=True)
    """
    @wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> T:
        if is_main_thread():
            return func(*args, **kwargs)
        return run_on_main_thread_sync(func, *args, **kwargs)
    return wrapper


def dcc_thread_safe_async(func: Callable[..., None]) -> Callable[..., None]:
    """Decorator for fire-and-forget execution on DCC main thread.
    
    The decorated function will be queued for execution on the main thread
    and returns immediately. Use this when you don't need the return value.
    
    Example:
        @dcc_thread_safe_async
        def update_viewport():
            import maya.cmds as cmds
            cmds.refresh()
    """
    @wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> None:
        run_on_main_thread(func, *args, **kwargs)
    return wrapper


class DCCThreadSafeWrapper:
    """Wrapper that makes WebView methods thread-safe for DCC environments.
    
    All method calls are automatically marshaled to the correct thread.
    """
    
    def __init__(self, webview: "WebView"):
        self._webview = webview
        self._proxy = webview.get_proxy()
    
    def eval_js(self, script: str) -> None:
        """Execute JavaScript (thread-safe, fire-and-forget)."""
        self._proxy.eval_js(script)
    
    def eval_js_sync(self, script: str, timeout_ms: int = 5000) -> Any:
        """Execute JavaScript and wait for result (blocking, thread-safe)."""
        import threading
        result = [None]
        error = [None]
        event = threading.Event()
        
        def callback(res, err):
            result[0] = res
            error[0] = err
            event.set()
        
        self._proxy.eval_js_async(script, callback, timeout_ms)
        event.wait(timeout=timeout_ms / 1000 + 1)
        
        if error[0]:
            raise RuntimeError(f"JavaScript error: {error[0]}")
        return result[0]
    
    def emit(self, event_name: str, data: dict = None) -> None:
        """Emit event to JavaScript (thread-safe)."""
        self._proxy.emit(event_name, data or {})
    
    def load_url(self, url: str) -> None:
        """Load URL (thread-safe)."""
        self._proxy.load_url(url)
    
    def load_html(self, html: str) -> None:
        """Load HTML content (thread-safe)."""
        self._proxy.load_html(html)
    
    def reload(self) -> None:
        """Reload page (thread-safe)."""
        self._proxy.reload()
    
    def close(self) -> None:
        """Close WebView (thread-safe)."""
        self._proxy.close()
```

### WebView Class Enhancement

Add `dcc_mode` parameter and `thread_safe()` method:

```python
class WebView:
    def __init__(
        self,
        ...,
        dcc_mode: bool = False,  # New parameter
    ):
        self._dcc_mode = dcc_mode
        ...
    
    def thread_safe(self) -> DCCThreadSafeWrapper:
        """Get a thread-safe wrapper for cross-thread operations.
        
        Returns:
            DCCThreadSafeWrapper with thread-safe methods
        """
        from auroraview.utils.dcc_thread_safe import DCCThreadSafeWrapper
        return DCCThreadSafeWrapper(self)
    
    def on(self, event_name: str):
        """Register event handler with optional DCC thread safety."""
        def decorator(func):
            if self._dcc_mode:
                # Auto-wrap with thread safety
                from auroraview.utils import ensure_main_thread
                func = ensure_main_thread(func)
            self._event_handlers[event_name] = func
            return func
        return decorator
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DCC Main Thread                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     Qt Event Loop                            │   │
│  │  ┌───────────────────────────────────────────────────────┐  │   │
│  │  │              AuroraView Python API                     │  │   │
│  │  │  - @dcc_thread_safe decorators                        │  │   │
│  │  │  - webview.thread_safe() wrapper                      │  │   │
│  │  │  - dcc_mode=True auto-wrapping                        │  │   │
│  │  └───────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
                    Thread Dispatcher
                    (run_on_main_thread)
                              │
┌─────────────────────────────┼───────────────────────────────────────┐
│                        WebView Thread                                │
│  ┌──────────────────────────┼──────────────────────────────────┐   │
│  │                  Message Queue                               │   │
│  │            (crossbeam-channel)                               │   │
│  │                          │                                   │   │
│  │  ┌───────────────────────┴─────────────────────────────┐   │   │
│  │  │              WebView2 Controller                     │   │   │
│  │  │  - STA COM Thread                                    │   │   │
│  │  │  - JavaScript execution                               │   │   │
│  │  │  - Event loop                                         │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## API Reference

### Decorators

| Decorator | Description | Use Case |
|-----------|-------------|----------|
| `@dcc_thread_safe` | Ensures function runs on DCC main thread (blocking) | Event handlers that need return values |
| `@dcc_thread_safe_async` | Fire-and-forget execution on main thread | UI updates, logging |

### WebView Methods

| Method | Description |
|--------|-------------|
| `webview.thread_safe()` | Returns `DCCThreadSafeWrapper` for cross-thread ops |
| `webview.get_proxy()` | Returns `WebViewProxy` for low-level cross-thread ops |

### Constructor Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `dcc_mode` | bool | False | Auto-wrap all callbacks with thread safety |

## Migration Guide

### Before (Manual Thread Safety)

```python
from auroraview import WebView
from auroraview.utils import ensure_main_thread
import threading

webview = WebView(parent=hwnd)
proxy = webview.get_proxy()

# Must manually ensure thread safety
@webview.on("create_object")
def handle_create(data):
    # This might be called from WebView thread!
    # Need to manually dispatch to main thread
    def _create():
        import maya.cmds as cmds
        return cmds.polyCube()[0]
    
    return run_on_main_thread_sync(_create)

# Background thread must use proxy
def background_work():
    proxy.eval_js("updateStatus('working')")
```

### After (Automatic Thread Safety)

```python
from auroraview import WebView

# Option 1: Enable dcc_mode
webview = WebView(parent=hwnd, dcc_mode=True)

@webview.on("create_object")
def handle_create(data):
    # Automatically runs on main thread!
    import maya.cmds as cmds
    return cmds.polyCube()[0]

# Option 2: Use thread_safe() wrapper
safe = webview.thread_safe()
safe.eval_js("updateStatus('working')")  # Thread-safe!

# Option 3: Use decorator explicitly
from auroraview.utils import dcc_thread_safe

@webview.on("export")
@dcc_thread_safe
def handle_export(data):
    import maya.cmds as cmds
    cmds.file(save=True)
```

## Test Plan

### Unit Tests

1. `test_dcc_thread_safe_decorator` - Verify decorator marshals to main thread
2. `test_dcc_mode_auto_wrapping` - Verify dcc_mode wraps callbacks
3. `test_thread_safe_wrapper` - Verify all wrapper methods are thread-safe
4. `test_proxy_cross_thread` - Verify proxy works from any thread

### Integration Tests

1. Maya integration with @dcc_thread_safe
2. Blender integration with dcc_mode
3. Houdini multi-threaded callback test
4. Cross-thread emit/eval_js stress test

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)
- [x] Create `dcc_thread_safe.py` module
- [x] Implement `@dcc_thread_safe` decorator
- [x] Implement `@dcc_thread_safe_async` decorator
- [x] Implement `DCCThreadSafeWrapper` class
- [x] Add unit tests

### Phase 2: WebView Integration (Week 1)
- [x] Add `dcc_mode` parameter to WebView
- [x] Implement `thread_safe()` method
- [x] Auto-wrap callbacks when dcc_mode=True
- [ ] Update documentation

### Phase 3: Testing & Documentation (Week 2)
- [ ] Unit tests for all new functionality
- [ ] Integration tests with Maya/Blender/Houdini
- [ ] Update DCC integration guides
- [ ] Add examples

## Backward Compatibility

- All changes are additive
- Existing code continues to work without modification
- `dcc_mode=False` by default maintains current behavior
- Thread dispatcher remains available for manual control

## References

- [Thread Dispatcher Documentation](../guide/thread-dispatcher.md)
- WebView Proxy Implementation: `src/webview/proxy.rs`
- Message Queue Implementation: `src/ipc/message_queue.rs`
- COM Initialization: `crates/auroraview-core/src/builder/com_init.rs`

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-01-07 | Draft | Initial RFC |
