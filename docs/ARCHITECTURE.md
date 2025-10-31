# AuroraView Architecture

## Overview

AuroraView is designed with a modular, backend-agnostic architecture that supports multiple window integration modes. This document describes the architectural design and implementation details.

## Design Principles

1. **Modularity**: Clear separation between core logic and platform-specific implementations
2. **Extensibility**: Easy to add new backends and platforms
3. **Type Safety**: Leveraging Rust's type system for reliability
4. **API Consistency**: Unified API across different backends
5. **Performance**: Zero-cost abstractions where possible

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     Python API Layer                        │
│  (WebView, NativeWebView, QtWebView)                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   PyO3 Bindings Layer                       │
│  (AuroraView - Python-facing Rust class)                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend Abstraction Layer                  │
│  (WebViewBackend trait)                                     │
└─────────────────────────────────────────────────────────────┘
                            │
                ┌───────────┴───────────┐
                ▼                       ▼
┌───────────────────────┐   ┌───────────────────────┐
│   Native Backend      │   │    Qt Backend         │
│  (Platform-specific)  │   │  (Qt integration)     │
└───────────────────────┘   └───────────────────────┘
                │                       │
                ▼                       ▼
┌───────────────────────┐   ┌───────────────────────┐
│   Wry WebView         │   │  Qt WebEngine         │
│  (WebView2/WebKit)    │   │  (QWebEngineView)     │
└───────────────────────┘   └───────────────────────┘
```

## Code Structure

### Rust Side (`src/`)

```
src/
├── lib.rs                      # PyO3 module entry point
├── ipc/                        # IPC system for Python ↔ JavaScript
│   ├── mod.rs
│   ├── handler.rs              # IPC message handler
│   ├── message_queue.rs        # Thread-safe message queue
│   └── ...
├── utils/                      # Utilities (logging, etc.)
│   └── mod.rs
└── webview/                    # WebView implementation
    ├── mod.rs                  # Module exports
    ├── aurora_view.rs          # Python-facing class (PyO3)
    ├── config.rs               # Configuration structures
    ├── backend/                # Backend implementations
    │   ├── mod.rs              # Backend trait definition
    │   ├── native.rs           # Native backend (HWND on Windows)
    │   └── qt.rs               # Qt backend (stub)
    ├── event_loop.rs           # Event loop handling
    ├── message_pump.rs         # Windows message pump
    ├── protocol.rs             # Custom protocol handler
    ├── standalone.rs           # Standalone window mode
    ├── embedded.rs             # Embedded mode (legacy, to be removed)
    └── webview_inner.rs        # Core WebView logic
```

### Python Side (`python/auroraview/`)

```
python/auroraview/
├── __init__.py                 # Public API exports
├── webview.py                  # Base WebView class
├── native.py                   # Native backend wrapper
├── qt_integration.py           # Qt backend implementation
├── decorators.py               # Event handler decorators
└── dcc_event_queue.py          # DCC event queue utilities
```

## Backend System

### Backend Trait

The `WebViewBackend` trait defines the common interface that all backends must implement:

```rust
pub trait WebViewBackend {
    fn create(
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn webview(&self) -> Arc<Mutex<WryWebView>>;
    fn message_queue(&self) -> Arc<MessageQueue>;
    fn window(&self) -> Option<&tao::window::Window>;
    fn event_loop(&mut self) -> Option<tao::event_loop::EventLoop<UserEvent>>;
    fn process_events(&self) -> bool;
    fn run_event_loop_blocking(&mut self);
    
    // Default implementations for common operations
    fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn load_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn eval_js(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn emit(&mut self, event_name: &str, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>>;
}
```

### Native Backend

The `NativeBackend` uses platform-specific APIs for window embedding:

**Windows**:
- Uses HWND (window handle) for parenting
- Supports two modes:
  - `Child`: WS_CHILD style (same-thread parenting required)
  - `Owner`: GWLP_HWNDPARENT (safe for cross-thread usage)

**macOS** (planned):
- Uses NSView for embedding
- Integrates with Cocoa event loop

**Linux** (planned):
- Uses X11/Wayland window parenting
- GTK integration

### Qt Backend

The `QtBackend` integrates with Qt's widget system:

**Current Status**: Stub implementation

**Planned Features**:
- QWidget-based WebView
- Uses Qt's event loop (no separate event loop needed)
- Seamless integration with Qt-based DCCs (Maya, Houdini, Nuke)
- Memory-safe Qt ↔ Rust interaction

## Integration Modes

### 1. Standalone Mode

Creates an independent window with its own event loop.

```python
from auroraview import WebView

webview = WebView(title="My App", width=800, height=600)
webview.show()  # Blocking call
```

**Use Cases**:
- Standalone tools
- Desktop applications
- Testing and development

### 2. Native Embedded Mode

Embeds WebView into existing window using platform APIs.

```python
from auroraview import NativeWebView

webview = NativeWebView(
    title="DCC Tool",
    parent_hwnd=parent_window_handle,
    parent_mode="owner"  # Recommended for DCC integration
)
webview.show_async()  # Non-blocking
```

**Use Cases**:
- Maya, 3ds Max, Blender plugins
- Any application with accessible window handles
- Cross-platform DCC integration

### 3. Qt Integration Mode

Integrates as a Qt widget (requires Qt bindings).

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=qt_parent_widget,
    title="Qt Tool",
    width=800,
    height=600
)
webview.show()
```

**Use Cases**:
- Maya (PySide2/PySide6)
- Houdini (PySide2)
- Nuke (PySide2)
- Any Qt-based application

## Event System

### Python → JavaScript

```python
# Python
webview.emit("update_data", {"frame": 120})
```

```javascript
// JavaScript
window.addEventListener('update_data', (event) => {
    console.log(event.detail.frame);  // 120
});
```

### JavaScript → Python

```javascript
// JavaScript
window.dispatchEvent(new CustomEvent('export_scene', {
    detail: { path: '/path/to/file.ma' }
}));
```

```python
# Python
@webview.on('export_scene')
def handle_export(data):
    print(f"Exporting to: {data['path']}")
```

## Thread Safety

### Native Backend

- WebView and EventLoop are **not** `Send` on Windows
- Designed for single-thread usage (UI thread)
- Message queue provides thread-safe communication
- `show_async()` runs event loop in background thread

### Qt Backend

- Uses Qt's thread model
- All Qt operations must be on main thread
- Qt signals/slots handle cross-thread communication

## Future Enhancements

### Short Term

1. [OK] Complete Qt backend implementation
2. [OK] Add macOS support for Native backend
3. [OK] Add Linux support for Native backend
4. [OK] Improve error handling and diagnostics

### Long Term

1. Support for additional backends (Electron, Tauri)
2. Custom protocol handlers for DCC asset access
3. Advanced IPC features (streaming, binary data)
4. Performance optimizations
5. Comprehensive test suite

## Migration Guide

### From Old API to New API

**Before** (v0.0.x):
```python
from auroraview import WebView

webview = WebView(parent_hwnd=hwnd)
```

**After** (v0.1.x):
```python
# Option 1: Use base class (auto-detects backend)
from auroraview import WebView
webview = WebView(parent_hwnd=hwnd)

# Option 2: Explicitly use Native backend
from auroraview import NativeWebView
webview = NativeWebView(parent_hwnd=hwnd, parent_mode="owner")

# Option 3: Use Qt backend (requires qtpy)
from auroraview import QtWebView
webview = QtWebView(parent=qt_widget)
```

The old API remains compatible through aliases (`AuroraView` → `NativeWebView`).

## Contributing

When adding a new backend:

1. Create `src/webview/backend/your_backend.rs`
2. Implement the `WebViewBackend` trait
3. Add Python wrapper in `python/auroraview/your_backend.py`
4. Export from `__init__.py`
5. Update documentation
6. Add tests

See `backend/native.rs` and `backend/qt.rs` for examples.

