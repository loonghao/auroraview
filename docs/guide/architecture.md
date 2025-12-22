# Architecture

AuroraView is built with a modular, backend-agnostic architecture that supports multiple window integration modes.

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

## Core Components

### Rust Core (`src/`)

The Rust core provides:

- **WebView Management**: Window creation, lifecycle, and events
- **IPC System**: Bidirectional communication between Python and JavaScript
- **Plugin System**: High-performance native plugins
- **Custom Protocols**: Secure local resource loading

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
    │   └── qt.rs               # Qt backend
    ├── event_loop.rs           # Event loop handling
    ├── message_pump.rs         # Windows message pump
    ├── protocol.rs             # Custom protocol handler
    ├── standalone.rs           # Standalone window mode
    └── webview_inner.rs        # Core WebView logic
```

### Python Bindings (`python/auroraview/`)

Python bindings via PyO3 provide:

- **WebView API**: High-level Python interface
- **Event System**: Node.js-style EventEmitter
- **Qt Integration**: QtWebView for DCC applications
- **Type Safety**: Full type hints and runtime validation

```
python/auroraview/
├── __init__.py                 # Public API exports
├── webview.py                  # Base WebView class
├── qt_integration.py           # Qt backend implementation
└── event_timer.py              # Event timer for DCC integration
```

## Backend System

### Backend Trait

The `WebViewBackend` trait defines the common interface:

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

| Platform | Technology | Status |
|----------|-----------|--------|
| **Windows** | WebView2 (HWND) | ✅ Supported |
| **macOS** | WKWebView (NSView) | ✅ Supported |
| **Linux** | WebKitGTK | ✅ Supported |

**Windows Modes**:
- `Child`: WS_CHILD style (same-thread parenting required)
- `Owner`: GWLP_HWNDPARENT (safe for cross-thread usage)

### Qt Backend

The `QtBackend` integrates with Qt's widget system for seamless DCC integration.

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

### 2. DCC Integration Mode

Creates a WebView that integrates with DCC applications.

```python
from auroraview import WebView
import hou  # or maya.OpenMayaUI, etc.

# Get DCC main window HWND
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create embedded WebView
webview = WebView.create(
    title="My Tool",
    width=650,
    height=500,
    parent=hwnd,
    mode="owner",
)
webview.load_html("<h1>Hello from Houdini!</h1>")
webview.show()
```

**Key Features**:
- ✅ Non-blocking - DCC UI remains fully responsive
- ✅ Uses DCC's Qt message pump for event processing
- ✅ No separate event loop (avoids conflicts)

### 3. Packed Mode

When packaged as a standalone executable:

```
app.exe (Rust)
    ├── Extracts resources and Python runtime
    ├── Creates WebView
    ├── Loads frontend (from overlay)
    ├── Starts Python backend process
    │       └── Runs as API server (JSON-RPC over stdin/stdout)
    └── Event loop (Rust main thread)
```

Key differences from development mode:
- Rust is the main process (not Python)
- Python runs as a subprocess
- Communication via JSON-RPC over stdin/stdout
- Better process isolation and error handling

## Plugin Architecture

AuroraView includes built-in Rust plugins:

| Plugin | Description |
|--------|-------------|
| **Process** | Run external processes with streaming output |
| **File System** | Native file operations |
| **Dialog** | Native file/folder dialogs |
| **Shell** | Execute commands, open URLs |
| **Clipboard** | System clipboard access |

## Thread Safety

### Native Backend

- WebView and EventLoop are **not** `Send` on Windows
- Designed for single-thread usage (UI thread)
- Message queue provides thread-safe communication

### DCC Integration Mode

- WebView created on DCC's main UI thread
- No separate event loop (no threading issues)
- Message processing handled by internal EventTimer
- Thread-safe message queue for cross-thread communication

## Performance Characteristics

### Memory Footprint

| Component | Memory Usage |
|-----------|--------------|
| Rust Core | ~5 MB |
| System WebView | ~20-30 MB |
| **Total** | **~30 MB** |

Compare to:
- Electron: ~150 MB
- Qt WebEngine: ~100 MB

### Startup Time

- Cold start: ~300ms
- Warm start: ~100ms

## Contributing

When adding a new backend:

1. Create `src/webview/backend/your_backend.rs`
2. Implement the `WebViewBackend` trait
3. Add Python wrapper in `python/auroraview/your_backend.py`
4. Export from `__init__.py`
5. Update documentation
6. Add tests

See `backend/native.rs` for reference implementation.
