# Timer Architecture

## Overview

AuroraView provides two timer implementations for different use cases:

1. **EventTimer (Python)** - High-level timer for WebView event processing
2. **NativeTimer (Rust)** - Low-level Windows SetTimer wrapper

## EventTimer (Python)

### Location
- `python/auroraview/event_timer.py`

### Purpose
Provides a cross-platform, DCC-aware timer for periodic WebView event processing.

### Supported Backends (Priority Order)

1. **Qt QTimer** - Most precise, works in Maya/Houdini/Nuke/3ds Max/Unreal
2. **Maya scriptJob** - Maya-specific, runs on idle events
3. **Blender bpy.app.timers** - Blender-specific, precise timing
4. **Houdini event loop callbacks** - Houdini-specific
5. **Thread-based timer** - Fallback for all platforms

### Features

- Automatic backend selection
- Multiple callback support (on_tick, on_close)
- Window validity checking
- Context manager support
- Error recovery in callbacks
- Throttling for Maya scriptJob

### Usage Example

```python
from auroraview import WebView
from auroraview.event_timer import EventTimer

# Create WebView
webview = WebView(parent=maya_hwnd, mode="owner")

# Create timer with 16ms interval (60 FPS)
timer = EventTimer(webview, interval_ms=16)

# Register callbacks
@timer.on_close
def handle_close():
    print("WebView closed")
    timer.stop()

@timer.on_tick
def handle_tick():
    print("Tick")

# Start timer (auto-detects best backend)
timer.start()

# Or use as context manager
with EventTimer(webview, interval_ms=16) as timer:
    # Timer automatically starts and stops
    pass
```

## NativeTimer (Rust)

### Location
- `src/webview/timer.rs` - Core implementation
- `src/webview/timer_bindings.rs` - Python bindings

### Purpose
Provides a low-level, high-performance Windows SetTimer wrapper for advanced use cases.

### Features

- Windows SetTimer API integration
- Thread-based fallback
- Tick throttling
- Message processing
- Context manager support

### Usage Example

```python
from auroraview._auroraview import NativeTimer

# Create timer
timer = NativeTimer(16)  # 16ms interval

# Set callback
def on_tick():
    print("Tick")

timer.set_callback(on_tick)

# Start Windows timer (requires HWND)
timer.start_windows(hwnd)

# Process messages in main loop
while running:
    timer.process_messages()

# Stop timer
timer.stop()
```

## Architecture Decisions

### Why Two Implementations?

1. **EventTimer (Python)**
   - High-level, user-friendly API
   - DCC integration (Maya, Blender, Houdini)
   - Automatic backend selection
   - Recommended for most use cases

2. **NativeTimer (Rust)**
   - Low-level Windows API access
   - Maximum performance
   - Advanced use cases only
   - Direct control over timer behavior

### Removed: Python ctypes Windows Timer

Previously, EventTimer included a ctypes-based Windows SetTimer implementation. This has been removed because:

1. Rust provides a safer, more performant implementation
2. EventTimer primarily relies on Qt and DCC-specific timers
3. Windows SetTimer was rarely used (low priority backend)
4. Reduces code duplication and maintenance burden

## Testing

### Python Tests
- `tests/test_event_timer.py` - EventTimer unit tests
- `tests/test_native_timer.py` - NativeTimer unit tests
- `tests/test_timer_integration.py` - Integration tests

### Rust Tests
- `src/webview/timer.rs` - Inline unit tests

### Running Tests

```bash
# Python tests
pytest tests/test_event_timer.py tests/test_native_timer.py tests/test_timer_integration.py -v

# Rust tests
cargo test timer

# All tests
uvx nox -s pytest
```

## Performance Characteristics

### EventTimer
- **Qt QTimer**: ~1ms precision, main thread
- **Maya scriptJob**: Variable (throttled to interval_ms)
- **Blender timer**: ~1ms precision, main thread
- **Thread-based**: ~10ms precision, background thread

### NativeTimer
- **Windows SetTimer**: ~15ms precision (Windows limitation)
- **Thread-based**: ~1ms precision

## Best Practices

1. **Use EventTimer for most cases**
   - Automatic backend selection
   - DCC integration
   - Error recovery

2. **Use NativeTimer only when**
   - You need direct Windows API control
   - You're implementing custom message loops
   - You need maximum performance

3. **Interval Selection**
   - 16ms (60 FPS) - Smooth UI updates
   - 33ms (30 FPS) - Balanced performance
   - 100ms+ - Background tasks

4. **Error Handling**
   - EventTimer automatically catches callback errors
   - Always use try-except in NativeTimer callbacks

## Future Improvements

- [ ] macOS/Linux native timer support
- [ ] Adaptive interval adjustment
- [ ] Performance profiling tools
- [ ] Timer pooling for multiple WebViews

