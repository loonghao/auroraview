# Cross-Platform Window Lifecycle Management

## Overview

AuroraView now includes a robust, cross-platform window lifecycle management system that elegantly handles window closing across different platforms (Windows, macOS, Linux) and modes (standalone, embedded).

## Architecture

### Core Components

1. **LifecycleManager** (`src/webview/lifecycle.rs`)
   - Manages window lifecycle states
   - Provides event-driven close notifications via `flume` channels
   - Supports cleanup handler registration
   - Thread-safe and platform-agnostic

2. **PlatformWindowManager** (`src/webview/platform/`)
   - Platform-specific window event processing
   - Abstracts platform differences behind a common trait
   - Implementations for Windows, macOS, and Linux

3. **Resource Cleanup** (using `scopeguard`)
   - Guarantees cleanup execution even on panic
   - RAII-style resource management
   - Zero-cost abstraction

## Lifecycle States

```rust
pub enum LifecycleState {
    Creating,        // Window is being created
    Active,          // Window is active and running
    CloseRequested,  // Close has been requested
    Destroying,      // Window is being destroyed
    Destroyed,       // Window has been destroyed
}
```

## Close Reasons

```rust
pub enum CloseReason {
    UserRequest,      // User clicked close button
    AppRequest,       // Application requested close
    ParentClosed,     // Parent window closed (embedded mode)
    SystemShutdown,   // System shutdown
    Error,            // Error occurred
}
```

## Usage Examples

### Embedded Mode (DCC Integration)

```python
from auroraview import AuroraView

# Create embedded webview
view = AuroraView.create_embedded(
    parent_hwnd=maya_window_handle,
    width=800,
    height=600,
    url="https://example.com"
)

# Process events periodically (e.g., from Maya timer)
def on_timer():
    if view.process_events():
        print("Window closed by user")
        view.close()
        return False  # Stop timer
    return True  # Continue timer
```

### Standalone Mode

```python
from auroraview import AuroraView

# Create standalone webview
view = AuroraView(
    title="My App",
    width=1024,
    height=768,
    url="https://example.com"
)

# Run event loop (blocking)
view.run()
```

## Platform-Specific Behavior

### Windows

- Uses `PeekMessageW` for non-blocking message processing
- Detects close from multiple sources:
  - `WM_CLOSE` - Standard close message
  - `WM_SYSCOMMAND` with `SC_CLOSE` - System menu close
  - `WM_NCLBUTTONUP`/`WM_NCLBUTTONDOWN` with `HTCLOSE` - Title bar close button
  - `WM_DESTROY` - Window destruction
- Validates window with `IsWindow()` API

### macOS (Planned)

- Uses NSWindow delegate for close notifications
- Implements `windowShouldClose:` method
- Handles application termination events

### Linux (Planned)

- Uses X11 `WM_DELETE_WINDOW` protocol
- Processes `ClientMessage` and `DestroyNotify` events
- Supports Wayland via appropriate backend

## Benefits

### 1. **Event-Driven Architecture**

Instead of polling, the system uses channels for efficient event notification:

```rust
// Old approach (polling)
loop {
    if is_window_closed() {
        break;
    }
    thread::sleep(Duration::from_millis(100));
}

// New approach (event-driven)
if lifecycle.check_close_requested().is_some() {
    // Handle close immediately
}
```

### 2. **Guaranteed Cleanup**

Using `scopeguard`, cleanup is guaranteed even on panic:

```rust
use scopeguard::defer;

pub fn process_events(&self) -> bool {
    defer! {
        tracing::trace!("Event processing completed");
    }
    
    // Your code here - cleanup guaranteed
}
```

### 3. **Platform Abstraction**

Single API works across all platforms:

```rust
// Same code works on Windows, macOS, Linux
if platform_manager.process_events() {
    println!("Window closed");
}
```

### 4. **Better DCC Integration**

- Non-blocking event processing
- Respects host application's event loop
- Clean separation of concerns
- Proper resource cleanup

## Migration Guide

### For Existing Code

The new system is backward compatible. Existing code continues to work without changes.

### To Use New Features

1. **Access lifecycle state:**
   ```python
   state = view.get_lifecycle_state()
   ```

2. **Request programmatic close:**
   ```python
   view.request_close()
   ```

3. **Register cleanup handlers:**
   ```rust
   lifecycle.register_cleanup(|| {
       println!("Cleanup executed");
   });
   ```

## Performance

- **Zero overhead**: `scopeguard` compiles to zero-cost abstractions
- **Efficient channels**: `flume` is faster than `std::sync::mpsc`
- **Non-blocking**: All operations are non-blocking in embedded mode
- **Minimal allocations**: Uses `Arc` and `Mutex` sparingly

## Future Enhancements

1. Complete macOS implementation
2. Complete Linux implementation
3. Add lifecycle event callbacks to Python API
4. Support for custom close confirmation dialogs
5. Graceful shutdown with timeout

