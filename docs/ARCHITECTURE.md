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
from qtpy.QtCore import QTimer  # QtPy handles PySide2/PySide6/PyQt5/PyQt6

# Get DCC main window HWND
import hou  # or maya.OpenMayaUI, etc.
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create WebView for DCC integration
webview = WebView.for_dcc(
    parent_hwnd=hwnd,
    title="My Tool",
    width=650,
    height=500
)

# Load content
webview.load_html("<h1>Hello from Houdini!</h1>")

# Setup Qt timer to process messages (REQUIRED!)
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)  # 60 FPS

# Keep reference to prevent garbage collection
_webview_instance = webview
_timer_instance = timer
```

**Key Features**:
- ✅ Non-blocking - DCC UI remains fully responsive
- ✅ Uses DCC's Qt message pump for event processing
- ⚠️ Requires QtPy for Qt version compatibility
- ⚠️ Depends on DCC's Qt bindings (PySide2/PySide6)

**Technical Details**:
- Creates WebView on DCC's main UI thread
- Does NOT create a separate event loop
- Relies on periodic `process_messages()` calls from Qt timer
- Messages are processed through DCC's existing message pump
- **Requires QtPy** to abstract Qt version differences

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
from auroraview import NativeWebView

webview = NativeWebView(
    title="DCC Tool",
    parent_hwnd=parent_window_handle,
    parent_mode="owner"  # Recommended for DCC integration
)
webview.show_async()  # Non-blocking
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
4. Periodic `process_messages()` calls from Qt timer (integrates with Qt event loop)

### Implementation Details

#### Rust Layer (`src/webview/backend/native.rs`)

```rust
impl NativeBackend {
    /// Create WebView for DCC integration (no event loop)
    pub fn create_for_dcc(
        parent_hwnd: u64,
        config: WebViewConfig,
        ipc_handler: Arc<IpcHandler>,
        message_queue: Arc<MessageQueue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Create temporary event loop (only for window creation)
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build();

        // 2. Create window as child of DCC window
        let window = WindowBuilder::new()
            .with_parent_window(parent_hwnd as isize)
            .build(&event_loop)?;

        // 3. Create WebView
        let webview = WryWebViewBuilder::new()
            .with_url(&config.url)
            .build(&window)?;

        // 4. CRITICAL: Drop event loop (don't run it!)
        drop(event_loop);

        Ok(Self {
            webview: Arc::new(Mutex::new(webview)),
            window: Some(window),
            event_loop: None,  // No event loop stored
            message_queue,
        })
    }

    /// Process messages (called from Qt timer)
    pub fn process_messages(&self) -> bool {
        // Process Windows messages for this window
        let hwnd = get_window_hwnd(&self.window);
        message_pump::process_messages_for_hwnd(hwnd)
    }
}
```

#### Python Layer (`python/auroraview/webview.py`)

```python
class WebView:
    @classmethod
    def for_dcc(cls, parent_hwnd: int, title: str, width: int, height: int):
        """Create WebView for DCC integration."""
        # Create WebView using Rust implementation
        core = _CoreWebView.create_for_dcc(
            parent_hwnd=parent_hwnd,
            title=title,
            width=width,
            height=height,
        )

        # Wrap in Python class
        instance = cls.__new__(cls)
        instance._core = core
        # ... initialize other fields

        return instance

    def process_messages(self) -> bool:
        """Process messages (called from Qt timer)."""
        return self._core.process_messages()
```

#### Usage Pattern

```python
# 1. Create WebView
webview = WebView.for_dcc(parent_hwnd=hwnd, title="Tool")

# 2. Setup Qt timer (integrates with DCC's event loop)
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)  # 60 FPS

# 3. Keep references alive
_webview = webview
_timer = timer
```

### Message Flow

```
DCC Qt Event Loop
    │
    ├─> Qt Timer (16ms interval)
    │       │
    │       └─> webview.process_messages()
    │               │
    │               └─> Rust: process_messages_for_hwnd()
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

1. Requires periodic `process_messages()` calls (Qt timer needed)
2. Windows-only (currently)
3. Slightly more setup code than other modes

## Thread Safety

### Native Backend

- WebView and EventLoop are **not** `Send` on Windows
- Designed for single-thread usage (UI thread)
- Message queue provides thread-safe communication
- `show_async()` runs event loop in background thread

### DCC Integration Mode

- WebView created on DCC's main UI thread
- No separate event loop (no threading issues)
- Message processing happens on UI thread via Qt timer
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

