# RFC 0002: DCC Thread Safety

> **Status**: Implementing (Phase 2 In Progress)
> **Author**: AuroraView Team
> **Created**: 2025-01-07
> **Updated**: 2025-01-10
> **Target Version**: v0.5.0

## Summary

This RFC proposes a comprehensive thread safety solution for AuroraView when integrated with DCC (Digital Content Creation) applications like Maya, Houdini, Blender, Nuke, 3ds Max, and Unreal Engine. The solution addresses the fundamental challenge that both WebView2 and DCC applications have strict threading requirements that must be respected.

**Key Additions in v2:**
- Deadlock prevention strategies and lock ordering specification
- Timeout protection mechanisms for all synchronous operations
- Debug-mode lock order verification
- Enhanced error handling for thread-related failures

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
- [x] Update documentation

### Phase 3: Deadlock Prevention (Week 2)
- [x] Document lock ordering specification
- [x] Add timeout protection to synchronous operations
- [ ] Implement debug-mode lock order verification
- [ ] Add graceful shutdown coordination

### Phase 4: Testing & Documentation (Week 2-3)
- [ ] Unit tests for all new functionality
- [ ] Integration tests with Maya/Blender/Houdini
- [ ] Deadlock detection tests
- [ ] Timeout behavior tests
- [ ] Update DCC integration guides
- [ ] Add examples

## Backward Compatibility

- All changes are additive
- Existing code continues to work without modification
- `dcc_mode=False` by default maintains current behavior
- Thread dispatcher remains available for manual control

## Deadlock Prevention

### Lock Ordering Specification

To prevent deadlocks, all code must acquire locks in a consistent order. The following hierarchy defines the canonical lock acquisition order (lower numbers must be acquired first):

| Level | Lock Type | Examples | Notes |
|-------|-----------|----------|-------|
| 1 | Global/Static | `CLICK_THROUGH_DATA` | Rarely needed, acquire first |
| 2 | Registry/Collection | `ProcessRegistry`, `ChannelRegistry` | Outer container locks |
| 3 | Individual Resource | `ManagedProcess`, `IpcChannelHandle` | Inner resource locks |
| 4 | State | `BridgeState`, `ExtensionsState` | Component state |
| 5 | Callback | `event_callback` | Always acquire last |

**Example - Correct Lock Order:**

```rust
// Good: Acquire registry lock first, then individual resource lock
fn kill_process(&self, pid: u32) -> Result<()> {
    // Level 2: Registry lock
    let processes = self.processes.read().unwrap();
    
    if let Some(proc) = processes.get(&pid) {
        // Level 3: Individual resource lock
        let mut managed = proc.lock().unwrap();
        managed.child.kill()?;
    }
    Ok(())
}
```

**Example - Incorrect Lock Order (Deadlock Risk):**

```rust
// Bad: Holding inner lock while trying to acquire outer lock
fn bad_example(&self, pid: u32) {
    let proc = some_process.lock().unwrap();  // Level 3 first - WRONG!
    let processes = self.processes.write().unwrap();  // Level 2 - DEADLOCK RISK!
}
```

### Nested Lock Guidelines

1. **Minimize Lock Scope**: Release locks as soon as possible
2. **Avoid Callbacks Under Lock**: Never call user callbacks while holding locks
3. **Clone Before Lock**: Clone data if needed outside lock scope
4. **Use Try-Lock for Optional Operations**: Use `try_lock()` when operation can be skipped

```rust
// Good: Release lock before callback
fn emit_event(&self, event: &str, data: Value) {
    // Clone callback outside of lock scope
    let callback = {
        let guard = self.event_callback.read().unwrap();
        guard.clone()
    };
    
    // Call callback without holding lock
    if let Some(cb) = callback {
        cb(event, data);
    }
}
```

### Critical Deadlock Patterns to Avoid

#### Pattern 1: Lock Inversion

```rust
// Thread A                          // Thread B
lock(A);                             lock(B);
lock(B);  // waits for B             lock(A);  // waits for A -> DEADLOCK
```

**Solution**: Always acquire locks in the same order.

#### Pattern 2: Callback Under Lock

```rust
// Bad: User callback might try to acquire same lock
let guard = self.state.write().unwrap();
user_callback();  // If callback calls our API -> DEADLOCK
```

**Solution**: Release lock before calling user code.

#### Pattern 3: Cross-Thread Synchronous Call

```rust
// Main thread                       // WebView thread
run_on_main_thread_sync(|| {         process_messages(|| {
    webview.eval_js_sync(...);       //   run_on_main_thread_sync(...)
});                                  // });
// Both threads waiting for each other -> DEADLOCK
```

**Solution**: Use async patterns or timeouts.

### Debug-Mode Lock Order Verification

In debug builds, enable lock order verification:

```rust
#[cfg(debug_assertions)]
mod lock_order {
    use std::cell::RefCell;
    use std::collections::HashSet;
    
    thread_local! {
        static HELD_LOCKS: RefCell<Vec<LockLevel>> = RefCell::new(Vec::new());
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum LockLevel {
        Global = 1,
        Registry = 2,
        Resource = 3,
        State = 4,
        Callback = 5,
    }
    
    pub fn acquire(level: LockLevel) {
        HELD_LOCKS.with(|locks| {
            let locks = locks.borrow();
            if let Some(&last) = locks.last() {
                assert!(
                    level > last,
                    "Lock order violation: attempting to acquire {:?} while holding {:?}",
                    level, last
                );
            }
        });
        HELD_LOCKS.with(|locks| locks.borrow_mut().push(level));
    }
    
    pub fn release(level: LockLevel) {
        HELD_LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            assert_eq!(locks.pop(), Some(level), "Lock release mismatch");
        });
    }
}
```

## Timeout Protection

### Synchronous Operations

All synchronous cross-thread operations must have timeout protection:

```python
class DCCThreadSafeWrapper:
    def eval_js_sync(
        self,
        script: str,
        timeout_ms: int = 5000,
    ) -> Any:
        """Execute JavaScript and wait for result with timeout protection."""
        result_holder: list = [None]
        error_holder: list = [None]
        event = threading.Event()

        def callback(res: Any, err: Optional[str]) -> None:
            result_holder[0] = res
            error_holder[0] = err
            event.set()

        self._proxy.eval_js_async(script, callback, timeout_ms)

        # Wait with timeout
        timeout_sec = timeout_ms / 1000.0 + 1.0
        if not event.wait(timeout=timeout_sec):
            raise TimeoutError(
                f"JavaScript execution timed out after {timeout_ms}ms. "
                f"Script: {script[:100]}..."
            )

        if error_holder[0]:
            raise RuntimeError(f"JavaScript error: {error_holder[0]}")

        return result_holder[0]
```

### Main Thread Dispatch Timeout

```python
def run_on_main_thread_sync(
    func: Callable[..., T],
    *args: Any,
    timeout: float = 30.0,
    **kwargs: Any,
) -> T:
    """Execute function on main thread with timeout protection.
    
    Args:
        func: Function to execute
        *args: Positional arguments
        timeout: Maximum wait time in seconds (default: 30.0)
        **kwargs: Keyword arguments
        
    Returns:
        Function return value
        
    Raises:
        TimeoutError: If execution doesn't complete within timeout
        RuntimeError: If execution fails
    """
    if is_main_thread():
        return func(*args, **kwargs)
    
    result_holder: list = [None]
    error_holder: list = [None]
    event = threading.Event()
    
    def wrapper():
        try:
            result_holder[0] = func(*args, **kwargs)
        except Exception as e:
            error_holder[0] = e
        finally:
            event.set()
    
    run_on_main_thread(wrapper)
    
    if not event.wait(timeout=timeout):
        raise TimeoutError(
            f"Main thread execution timed out after {timeout}s. "
            f"Function: {func.__name__}"
        )
    
    if error_holder[0]:
        raise error_holder[0]
    
    return result_holder[0]
```

### Rust-Side Timeout Configuration

```rust
/// Configuration for thread-safe operations
#[derive(Debug, Clone)]
pub struct ThreadSafetyConfig {
    /// Default timeout for synchronous JavaScript execution (ms)
    pub js_eval_timeout_ms: u64,
    
    /// Default timeout for main thread dispatch (ms)
    pub main_thread_timeout_ms: u64,
    
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    
    /// Delay between retry attempts (ms)
    pub retry_delay_ms: u64,
    
    /// Enable lock order verification in debug builds
    pub debug_lock_order: bool,
}

impl Default for ThreadSafetyConfig {
    fn default() -> Self {
        Self {
            js_eval_timeout_ms: 5000,
            main_thread_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 100,
            debug_lock_order: cfg!(debug_assertions),
        }
    }
}
```

## Graceful Shutdown

### Shutdown Coordination (powered by ipckit)

The `ShutdownState` from ipckit provides coordinated shutdown across all threads:

```rust
use ipckit::graceful::ShutdownState;

pub struct MessageQueue {
    // ... other fields ...
    shutdown_state: Arc<ShutdownState>,
}

impl MessageQueue {
    /// Signal shutdown - no more messages will be accepted
    pub fn shutdown(&self) {
        self.shutdown_state.shutdown();
        tracing::info!("[MessageQueue] Shutdown signaled");
    }
    
    /// Wait for all in-flight operations to complete
    pub fn wait_for_drain(&self, timeout: Option<Duration>) -> Result<(), DrainError> {
        self.shutdown_state.wait_for_drain(timeout)
    }
    
    /// Push message with shutdown check
    pub fn push(&self, message: WebViewMessage) {
        // Check shutdown flag first
        if self.shutdown_state.is_shutdown() {
            tracing::debug!("[MessageQueue] Dropping message - shutdown in progress");
            return;
        }
        
        // Use operation guard to track in-flight operations
        let _guard = self.shutdown_state.begin_operation();
        
        // ... send message ...
    }
}
```

### Python-Side Shutdown Handling

```python
class WebView:
    def close(self) -> None:
        """Close the WebView with graceful shutdown."""
        # Signal shutdown to prevent new operations
        self._shutting_down = True
        
        # Wait for pending operations with timeout
        try:
            self._drain_pending_operations(timeout=5.0)
        except TimeoutError:
            logger.warning("Timeout waiting for pending operations during shutdown")
        
        # Close the underlying WebView
        self._proxy.close()
    
    def _drain_pending_operations(self, timeout: float) -> None:
        """Wait for pending operations to complete."""
        start = time.time()
        while self._pending_operations > 0:
            if time.time() - start > timeout:
                raise TimeoutError(f"Drain timeout: {self._pending_operations} operations pending")
            time.sleep(0.01)
```

## Error Handling

### Thread-Related Error Types

```python
class ThreadSafetyError(Exception):
    """Base exception for thread safety errors."""
    pass

class DeadlockDetectedError(ThreadSafetyError):
    """Raised when a potential deadlock is detected."""
    pass

class ThreadDispatchTimeoutError(ThreadSafetyError):
    """Raised when main thread dispatch times out."""
    pass

class ShutdownInProgressError(ThreadSafetyError):
    """Raised when operation is attempted during shutdown."""
    pass
```

### Error Recovery Strategies

```python
@dcc_thread_safe
def safe_dcc_operation(data: dict) -> dict:
    """Example of robust error handling in DCC operations."""
    try:
        result = perform_dcc_operation(data)
        return {"success": True, "result": result}
    except TimeoutError as e:
        logger.error(f"Operation timed out: {e}")
        return {"success": False, "error": "timeout", "message": str(e)}
    except ThreadSafetyError as e:
        logger.error(f"Thread safety error: {e}")
        return {"success": False, "error": "thread_safety", "message": str(e)}
    except Exception as e:
        logger.exception(f"Unexpected error in DCC operation")
        return {"success": False, "error": "unknown", "message": str(e)}
```

## References

- [Thread Dispatcher Documentation](../guide/thread-dispatcher.md)
- WebView Proxy Implementation: `src/webview/proxy.rs`
- Message Queue Implementation: `src/ipc/message_queue.rs`
- COM Initialization: `crates/auroraview-core/src/builder/com_init.rs`
- ipckit ShutdownState: External crate for graceful shutdown coordination

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-01-07 | Draft | Initial RFC |
| 2025-01-10 | v2 | Added deadlock prevention, lock ordering, timeout protection |
