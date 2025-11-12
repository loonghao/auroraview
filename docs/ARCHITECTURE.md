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
│  (WebView, QtWebView)                                      │
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
├── qt_integration.py           # Qt backend implementation
└── event_timer.py              # Event timer for DCC integration
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

### 2. DCC Integration Mode (Experimental - Requires QtPy) ⚠️

**Status**: Experimental - Requires QtPy middleware for Qt version compatibility

This mode creates a WebView that integrates with DCC applications, but requires QtPy to handle different Qt versions across DCC applications.

**Requirements**:
```bash
pip install auroraview[qt]  # Installs QtPy automatically
```

**Example**:
```python
from auroraview import WebView
import hou  # or maya.OpenMayaUI, etc.

# Get DCC main window HWND
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create embedded WebView (auto timer; no manual Qt timer wiring)
webview = WebView.create(
    title="My Tool",
    width=650,
    height=500,
    parent=hwnd,
    mode="owner",
)

# Load content
webview.load_html("<h1>Hello from Houdini!</h1>")

# Show the window (non-blocking, integrated with DCC)
webview.show()
```

**Key Features**:
- ✅ Non-blocking - DCC UI remains fully responsive
- ✅ Uses DCC's Qt message pump for event processing
- 🔸 QtPy recommended for Qt version compatibility (optional)
- 🔸 Depends on DCC's Qt bindings availability (PySide2/PySide6)

**Technical Details**:
- Creates WebView on DCC's main UI thread
- Does NOT create a separate event loop
- Uses an internal event timer in embedded mode (no manual `process_messages` wiring)
- Messages are processed through the DCC's existing message pump

**Limitations**:
- Requires QtPy middleware installation
- Depends on DCC's Qt bindings availability
- May have compatibility issues with future Qt versions

**Use Cases**:
- Maya, Houdini, Nuke, 3ds Max plugins (with QtPy installed)
- Any Qt-based DCC application that supports QtPy

### 3. Native Embedded Mode (Legacy)

Embeds WebView into existing window using platform APIs.

**Note**: This mode creates its own event loop and may cause conflicts with Qt-based DCCs. Use DCC Integration Mode instead for Qt-based applications.

```python
from auroraview import WebView

webview = WebView.create(
    title="DCC Tool",
    parent=parent_window_handle,
    mode="owner",  # Recommended for DCC integration
)
webview.show()  # Embedded mode: non-blocking
```

**Use Cases**:
- Non-Qt applications
- Legacy integrations
- Special cases where DCC Integration Mode is not suitable

### 4. Qt Integration Mode (Deprecated)

Integrates as a Qt widget (requires Qt bindings).

**Note**: This mode has PySide dependency issues and is being phased out in favor of DCC Integration Mode.

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

## DCC Integration Mode - Technical Implementation

### Architecture

The DCC Integration Mode solves the fundamental problem of integrating WebView into Qt-based DCC applications without creating event loop conflicts or requiring PySide dependencies.

#### Problem Statement

Traditional approaches have issues:
1. **Native Embedded Mode**: Creates its own event loop → conflicts with DCC's Qt event loop → UI freezing
2. **Qt Backend**: Requires PySide2/PySide6 → version compatibility issues → breaks with future Qt versions

#### Solution

DCC Integration Mode uses a hybrid approach:
1. Creates WebView on DCC's main UI thread (satisfies WebView2 threading requirements)
2. Does NOT create a separate event loop (avoids conflicts)
3. Relies on DCC's existing Qt message pump (reuses infrastructure)
4. Uses an internal event timer in embedded mode (no manual `process_messages` wiring)

### Implementation Details

#### Rust Layer (conceptual)

```rust
impl NativeBackend {
    /// Create WebView embedded under a parent HWND (no dedicated event loop)
    pub fn create_with_parent(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        /* ... implementation details elided ... */
        Ok(Self { /* ... */ })
    }

    /// Drive message processing (internally called by timer when embedded)
    pub fn tick(&self) -> bool {
        /* ... */
        false
    }
}
```

#### Python Layer (`python/auroraview/webview.py`)

```python
class WebView:
    @classmethod
    def create(cls, title: str, *, parent: int | None = None, mode: str = "auto", **kwargs):
        """Unified factory - supports embedded (owner) and standalone modes."""
        instance = cls.__new__(cls)
        # initialize core with config and optional parent handle
        # instance._core = _CoreWebView.create(...)
        # instance._is_embedded = parent is not None
        return instance
```

#### Usage Pattern

```python
# 1. Create embedded WebView
webview = WebView.create(title="Tool", parent=hwnd, mode="owner")

# 2. Load content and show (internal timer handles message processing)
webview.load_url("http://localhost:3000")
webview.show()

# 3. Keep reference alive if script exits immediately
_webview = webview
```

### Message Flow

```
DCC Qt Event Loop
    │
    ├─> EventTimer (16ms interval)
    │       │
    │       └─> WebView.tick()  (internal)
    │               │
    │               └─> Rust: process_windows_messages_for_hwnd()
    │                       │
    │                       ├─> Process Windows messages
    │                       ├─> Process WebView events
    │                       └─> Process message queue
    │
    └─> Continue DCC event processing
```

### Advantages

1. **No Event Loop Conflicts**: Uses DCC's existing message pump
2. **No PySide Dependency**: Pure Rust implementation, only needs HWND
3. **Non-Blocking**: DCC UI remains fully responsive
4. **Future-Proof**: No Qt version dependencies
5. **WebView2 Compliant**: Runs on UI thread with message pump

### Limitations

1. Windows-only (currently)
2. Slightly more setup code than other modes

## Thread Safety

### Native Backend

- WebView and EventLoop are **not** `Send` on Windows
- Designed for single-thread usage (UI thread)
- Message queue provides thread-safe communication


### DCC Integration Mode

- WebView created on DCC's main UI thread
- No separate event loop (no threading issues)
- Message processing handled by internal EventTimer in embedded mode (no manual Qt timer)
- Thread-safe message queue for cross-thread communication

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
# New unified API
from auroraview import WebView

# Embedded under a DCC main window (recommended)
webview = WebView.create(title="Tool", parent=hwnd, mode="owner")

# Qt backend (widget-based; requires qtpy)
from auroraview import QtWebView
webview_qt = QtWebView(parent=qt_widget)
```

## Contributing

When adding a new backend:

1. Create `src/webview/backend/your_backend.rs`
2. Implement the `WebViewBackend` trait
3. Add Python wrapper in `python/auroraview/your_backend.py`
4. Export from `__init__.py`
5. Update documentation
6. Add tests

See `backend/native.rs` and `backend/qt.rs` for examples.

