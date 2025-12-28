# Design Philosophy

AuroraView is built with specific design principles that guide its architecture and API decisions. Understanding these principles will help you make the most of the framework.

## Core Principles

### 1. DCC-First Design

Unlike general-purpose WebView frameworks, AuroraView is specifically designed for Digital Content Creation software integration.

**What this means:**
- Qt widget integration for Maya, Houdini, Nuke, 3ds Max
- HWND-based embedding for Unreal Engine and non-Qt applications
- Lifecycle management that respects DCC application patterns
- Non-blocking operations that don't freeze the host application

**Design decisions influenced by this:**
- `QtWebView` creates a true Qt widget, not a wrapper around a native window
- Event processing integrates with Qt's event loop, not a separate thread
- Parent window monitoring for automatic cleanup

### 2. Zero Python Dependencies

The core `auroraview` package has no Python dependencies beyond the standard library.

**Why:**
- DCC applications often have constrained Python environments
- Avoid version conflicts with DCC-bundled packages
- Minimize installation complexity
- Single `.pyd` file distribution

**Exception:** The `[qt]` extra installs `QtPy` for Qt version abstraction.

### 3. Rust for Performance and Safety

The core is written in Rust with PyO3 bindings.

**Benefits:**
- Memory safety without garbage collection pauses
- ~5MB package size vs ~120MB for Electron
- Native performance for IPC and event handling
- Thread-safe message passing

### 4. Backend Abstraction

AuroraView uses a backend abstraction layer that allows different implementations:

```
┌─────────────────────────────────────────┐
│           Python API Layer              │
│  (WebView, QtWebView, AuroraView)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│        Backend Abstraction Layer        │
│         (WebViewBackend trait)          │
└─────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│ Native Backend│       │  Qt Backend   │
│ (Wry/WebView2)│       │ (Qt Widget)   │
└───────────────┘       └───────────────┘
```

**This enables:**
- Platform-specific optimizations
- Future backend additions (CEF, WebKitGTK, etc.)
- Consistent API across backends

### 5. Convention over Configuration

AuroraView provides sensible defaults while allowing customization.

**Examples:**

```python
# Minimal configuration - works out of the box
webview = WebView.create("My App", url="http://localhost:3000")
webview.show()

# Full customization when needed
webview = WebView.create(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768,
    resizable=True,
    frame=True,
    debug=True,
    context_menu=False,
    asset_root="./assets",
)
```

## API Design Patterns

### 1. Multiple API Styles

AuroraView supports different API styles for different use cases:

| Pattern | Best For | Complexity |
|---------|----------|------------|
| `AuroraView` with `api=` | Quick prototypes | Simple |
| `QtWebView` subclass | DCC integration | Medium |
| `WebView` with `bind_call` | Advanced control | Advanced |

**Example: Simple API Object**
```python
class MyAPI:
    def get_data(self) -> dict:
        return {"items": [1, 2, 3]}

view = AuroraView(url="...", api=MyAPI())
```

**Example: Qt-style Class**
```python
class MyTool(QtWebView):
    selection_changed = Signal(list)
    
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.bind_api(self)
```

### 2. Explicit over Implicit

API methods clearly indicate their behavior:

```python
# Clear distinction between blocking and non-blocking
webview.show()           # Blocking in standalone mode
webview.show(wait=False) # Non-blocking

# Clear distinction between event types
auroraview.send_event()  # Fire-and-forget (JS → Python)
auroraview.call()        # Request-response (JS → Python)
webview.emit()           # Push notification (Python → JS)
```

### 3. Qt-Inspired Signal System

For developers familiar with Qt:

```python
from auroraview import Signal

class MyTool(WebView):
    # Signal definitions
    selection_changed = Signal(list)
    progress_updated = Signal(int, str)
    
    def _on_selection(self, items):
        self.selection_changed.emit(items)
```

## JavaScript API Design

### Unified Namespace

All JavaScript APIs are under `window.auroraview`:

```javascript
// RPC calls
await auroraview.call('api.method', params);

// Events
auroraview.send_event('event_name', data);
auroraview.on('event_name', handler);

// API proxy (pywebview-style)
await auroraview.api.method(params);

// Shared state
auroraview.state.key = value;
```

### Protocol Design

The IPC protocol follows a request/response pattern:

**Request (JS → Python):**
```json
{
  "type": "call",
  "id": "unique-id",
  "method": "api.get_data",
  "params": {"key": "value"}
}
```

**Response (Python → JS):**
```json
{
  "type": "call_result",
  "id": "unique-id",
  "ok": true,
  "result": {"data": "..."}
}
```

## Security Considerations

### Custom Protocol Security

The `auroraview://` protocol uses `.localhost` TLD for security:

1. **IANA Reserved** - Cannot be registered by anyone
2. **Local Only** - Treated as 127.0.0.1
3. **Pre-DNS Interception** - Requests intercepted before DNS
4. **No Network Traffic** - Never leaves local machine

### Resource Access Control

```python
# Secure: Only assets/ directory accessible
webview = WebView.create(
    title="My App",
    asset_root="./assets",  # Restricted access
)

# Less secure: Full filesystem access
webview = WebView.create(
    title="My App",
    allow_file_protocol=True,  # Use with caution
)
```

## Performance Philosophy

### Lazy Initialization

WebView is only created when `show()` is called:

```python
webview = WebView.create("My App")  # No WebView created yet
webview.load_url("...")             # URL stored, not loaded
webview.show()                      # WebView created and URL loaded
```

### Message Batching

IPC messages are processed in batches:

```rust
let messages = message_queue.drain();
for message in messages {
    // Process each message
}
```

### Lock-Free Data Structures

- **DashMap**: Concurrent HashMap for callbacks
- **crossbeam-channel**: Lock-free MPMC for message queue

## Extensibility

### Plugin Architecture

Built-in Rust plugins provide native performance:

| Plugin | Description |
|--------|-------------|
| Process | Run external processes with streaming |
| File System | Native file operations |
| Dialog | Native file/folder dialogs |
| Shell | Execute commands, open URLs |
| Clipboard | System clipboard access |

### Custom Protocols

Register custom protocol handlers:

```python
def handle_maya_protocol(uri: str) -> dict:
    path = uri.replace("maya://", "")
    return {
        "data": load_maya_resource(path),
        "mime_type": "application/octet-stream",
        "status": 200
    }

webview.register_protocol("maya", handle_maya_protocol)
```

## Migration Paths

### From pywebview

AuroraView provides pywebview-compatible APIs:

```python
# pywebview style
class Api:
    def get_data(self):
        return {"items": [1, 2, 3]}

# Works with AuroraView
view = AuroraView(url="...", api=Api())
```

```javascript
// Same JavaScript API
const data = await auroraview.api.get_data();
```

### From Electron

Key differences:
- Single process (no separate main/renderer)
- Python backend instead of Node.js
- Native system WebView instead of bundled Chromium
- ~5MB vs ~120MB package size
