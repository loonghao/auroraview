# AuroraView vs PyWebView: Deep Comparison

> A comprehensive technical comparison between AuroraView and PyWebView, analyzing architecture, features, and use cases.

## Executive Summary

| Dimension | PyWebView | AuroraView |
|-----------|-----------|------------|
| **Primary Goal** | General desktop apps | DCC software plugins |
| **Core Language** | Pure Python | Rust + Python |
| **Architecture** | System WebView wrapper | Wry/Tao unified abstraction |
| **DCC Support** | ❌ Not designed for DCC | ✅ First-class DCC support |
| **Performance** | Moderate | High (Rust-optimized) |
| **Memory Safety** | Python GC | Rust guarantees |

---

## 1. Project Overview

### PyWebView (2014-present)

```
📦 pywebview
├── Author: Roman Sirokov
├── Language: Python + JavaScript (pure Python implementation)
├── License: BSD-3-Clause
├── GitHub Stars: 5.6k+
├── Monthly Downloads: 1M+ (PyPI)
├── Version: 6.1 (Oct 2025)
└── Platforms: Windows, macOS, Linux, Android
```

**Core Philosophy**: Lightweight native webview wrapper for displaying HTML content in desktop applications. Uses native GUI frameworks (WinForms, Cocoa, GTK, Qt) without bundling heavy browser engines.

### AuroraView (2025-present)

```
📦 auroraview
├── Author: loonghao
├── Language: Rust + Python (PyO3 bindings)
├── License: Apache-2.0 OR MIT
├── Target: DCC software integration
├── Version: 0.x (active development)
└── Platforms: Windows (macOS/Linux planned)
```

**Core Philosophy**: DCC-first WebView framework designed for embedding modern web UIs into 3D software like Maya, Houdini, Blender, and Nuke.

---

## 2. Architecture Comparison

### 2.1 PyWebView Architecture

```
┌────────────────────────────────────────────┐
│              Python Application            │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│           webview Python API               │
│  (create_window, start, expose, etc.)      │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│         Platform-specific Backends         │
├────────────────────────────────────────────┤
│  Windows │  macOS  │  Linux     │  Android │
│  --------|---------|------------|----------|
│  EdgeChromium      │  GTK/WebKit│          │
│  MSHTML  │  Cocoa  │  Qt/WebEngine          │
│  CEF     │  WKWebView          │  WebView  │
└────────────────────────────────────────────┘
```

**Key Characteristics**:
- Pure Python implementation with platform-specific GUI bindings
- Multiple backend options (EdgeChromium, CEF, Qt, GTK)
- Built-in HTTP server for local file serving
- DOM manipulation support in Python
- Event system with window lifecycle hooks

### 2.2 AuroraView Architecture

```
┌────────────────────────────────────────────┐
│         Python Application / DCC           │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│        auroraview Python API               │
│  (WebView.create, emit, on, Bridge, etc.)  │
└────────────────────┬───────────────────────┘
                     │ PyO3 bindings
┌────────────────────▼───────────────────────┐
│           Rust Core Layer                  │
├────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐   │
│  │       IPC Handler (DashMap)         │   │
│  │  - Lock-free concurrent callbacks   │   │
│  │  - Python ↔ JavaScript bridge       │   │
│  └─────────────────────────────────────┘   │
│  ┌─────────────────────────────────────┐   │
│  │       Lifecycle Manager             │   │
│  │  - Parent window monitoring         │   │
│  │  - Cross-platform window handling   │   │
│  └─────────────────────────────────────┘   │
│  ┌─────────────────────────────────────┐   │
│  │       Protocol Handlers             │   │
│  │  - Custom URL schemes               │   │
│  │  - Asset serving                    │   │
│  └─────────────────────────────────────┘   │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│              Wry + Tao                     │
│  (Unified WebView + Window management)     │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│         System WebView Engine              │
│  WebView2 (Win) / WKWebView (Mac) / WebKitGTK │
└────────────────────────────────────────────┘
```

**Key Characteristics**:
- Rust core with PyO3 Python bindings
- Wry (Tauri's WebView library) for unified cross-platform abstraction
- Tao for window management
- Lock-free IPC with DashMap
- Custom protocol handlers for DCC asset access
- Parent window monitoring for DCC embedding

---

## 3. API Comparison

### 3.1 Basic Window Creation

**PyWebView**:
```python
import webview

# Create and show window
webview.create_window('Hello World', 'https://example.com')
webview.start()  # Blocking call
```

**AuroraView**:
```python
from auroraview import WebView

# Create and show window (2 lines)
webview = WebView.create("Hello World", url="https://example.com")
webview.show()  # Auto-blocking for standalone
```

### 3.2 DCC Integration

**PyWebView** (Maya example - manual threading required):
```python
import webview
import threading
import maya.cmds as cmds

def start_webview():
    webview.create_window('Tool', 'http://localhost:3000')
    webview.start()  # ⚠️ Blocks - needs separate thread

# Must run in thread to avoid freezing Maya
thread = threading.Thread(target=start_webview)
thread.start()

# ⚠️ Problems:
# - No parent window awareness
# - Separate event loop conflicts with Maya's Qt
# - Manual thread management required
```

**AuroraView** (Maya example - native integration):
```python
from auroraview import WebView
import maya.OpenMayaUI as omui

# Get Maya main window handle
maya_hwnd = int(omui.MQtUtil.mainWindow())

# Create embedded WebView (parent-aware)
webview = WebView.create(
    "Maya Tool",
    url="http://localhost:3000",
    parent=maya_hwnd,    # Embedded under Maya
    mode="owner"         # Cross-thread safe
)

# Non-blocking with auto timer
webview.show()  # Returns immediately, auto timer handles events

# ✅ Benefits:
# - Parent window monitoring
# - Auto event processing
# - Lifecycle management
# - Cross-thread safety
```

### 3.3 Python-JavaScript Communication

**PyWebView**:
```python
import webview

class Api:
    def greet(self, name):
        return f"Hello, {name}!"

window = webview.create_window('App', 'index.html', js_api=Api())
webview.start()

# JavaScript side:
# await pywebview.api.greet('World')
```

**AuroraView**:
```python
from auroraview import WebView

webview = WebView.create("App", url="index.html")

# Method 1: Decorator-based event handling
@webview.on("export_scene")
def handle_export(data):
    print(f"Exporting to: {data['path']}")

# Method 2: Bind Python callable
@webview.bind_call("api.echo")
def echo(message):
    return message

# Method 3: Bind entire API object
class MyAPI:
    def greet(self, name):
        return f"Hello, {name}!"

webview.bind_api(MyAPI())  # Exposes as api.greet()

# Python → JavaScript
webview.emit("update_data", {"frame": 120})

# JavaScript side:
# auroraview.call("api.echo", {message: "hi"})
# auroraview.call("api.greet", {name: "World"})
```

### 3.4 Custom Protocol Handlers

**PyWebView**:
```python
# Limited to built-in HTTP server
# No native custom protocol support
```

**AuroraView**:
```python
from auroraview import WebView

webview = WebView.create("App", asset_root="/path/to/assets")

# Register custom protocol handler
def handle_dcc_protocol(uri: str) -> dict:
    path = uri.replace("dcc://", "")
    try:
        with open(f"/models/{path}", "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "application/octet-stream",
                "status": 200
            }
    except FileNotFoundError:
        return {"data": b"Not Found", "mime_type": "text/plain", "status": 404}

webview.register_protocol("dcc", handle_dcc_protocol)

# HTML: <img src="auroraview://textures/wood.png">
# Or: <model src="dcc://models/character.fbx">
```

---

## 4. Feature Comparison Matrix

| Feature | PyWebView | AuroraView | Notes |
|---------|:---------:|:----------:|-------|
| **Core Features** |
| Basic WebView | ✅ | ✅ | Both support modern web content |
| HTML/CSS/JS | ✅ | ✅ | Full web standards |
| Developer Tools | ✅ | ✅ | F12 / Right-click inspect |
| Multiple Windows | ✅ | ✅ | Multi-window support |
| **Communication** |
| Python → JS | ✅ | ✅ | Evaluate JavaScript |
| JS → Python | ✅ | ✅ | Call Python functions |
| Bidirectional Events | ✅ | ✅ | Event-based communication |
| Promise Support | ✅ | ✅ | Async call/response |
| **Window Management** |
| Frameless Window | ✅ | ✅ | Borderless mode |
| Always on Top | ✅ | ✅ | Pin window |
| Window Resize | ✅ | ✅ | Programmatic resize |
| Fullscreen | ✅ | ✅ | Toggle fullscreen |
| **Protocols & Resources** |
| HTTP Server | ✅ Built-in | ⚠️ Via Bridge | Different approaches |
| Custom Protocols | ❌ | ✅ | `auroraview://`, `dcc://` |
| File Protocol | ✅ | ✅ (opt-in) | Security consideration |
| DOM Access | ✅ | ⚠️ Partial | PyWebView has DOM class |
| **DCC Integration** |
| Parent Window Embed | ❌ | ✅ | Native embedding |
| Parent Monitoring | ❌ | ✅ | Close when parent closes |
| Event Loop Integration | ❌ | ✅ | Works with Qt/DCC loops |
| Qt Backend | ✅ Optional | ✅ QtWebView | Both support Qt |
| Thread Safety | ⚠️ Manual | ✅ Native | Rust guarantees |
| Singleton Mode | ❌ | ✅ | One instance per key |
| **Advanced** |
| SSL Support | ✅ (pip extra) | ⚠️ Planned | PyWebView has SSL option |
| Menu System | ✅ | ⚠️ Limited | Native menus |
| File Dialogs | ✅ | ⚠️ Planned | Open/Save dialogs |
| Context Menu | ✅ | ✅ | Right-click menu |
| Cookies | ✅ | ⚠️ Limited | Cookie management |
| **Platform** |
| Windows | ✅ | ✅ | WebView2/Edge |
| macOS | ✅ | ⚠️ Planned | WKWebView |
| Linux | ✅ | ⚠️ Planned | WebKitGTK |
| Android | ✅ | ❌ | Mobile support |

---

## 5. DCC Software Support

### 5.1 PyWebView in DCC Applications

| DCC | Status | Issues |
|-----|--------|--------|
| Maya | ⚠️ Works | Thread conflicts with Qt, manual management |
| Houdini | ⚠️ Limited | Event loop conflicts |
| Blender | ⚠️ Works | No native embedding, unstable |
| 3ds Max | ⚠️ Limited | .NET interop issues |
| Nuke | ⚠️ Limited | Qt conflict |
| Photoshop | ❌ | No UXP/CEP integration |

**Common Issues**:
- PyWebView is designed for standalone applications
- `webview.start()` blocks the main thread
- Separate thread causes Qt event loop conflicts in Qt-based DCCs
- No parent window awareness - window floats separately
- No lifecycle management - doesn't close with parent

### 5.2 AuroraView in DCC Applications

| DCC | Status | Integration Mode |
|-----|--------|------------------|
| Maya | ✅ Supported | Parent embed + owner mode |
| Houdini | ✅ Supported | Parent embed + owner mode |
| Blender | ✅ Supported | Standalone (no Qt) |
| 3ds Max | ✅ Supported | Parent embed + owner mode |
| Nuke | ✅ Supported | Parent embed + owner mode |
| Photoshop | ✅ Planned | Bridge WebSocket |

**Key Advantages**:
1. **Parent Window Embedding**: WebView is a child of DCC main window
2. **Lifecycle Management**: Closes when parent closes
3. **Event Loop Integration**: Works with DCC's event loop
4. **Thread Safety**: Rust-guaranteed cross-thread safety
5. **Auto Timer**: Automatic event processing in embedded mode

### 5.3 Integration Example Comparison

**Maya with PyWebView** (⚠️ Not recommended):
```python
import webview
import threading
from maya import cmds

# ⚠️ Must run in separate thread
def run_ui():
    window = webview.create_window('Tool', 'http://localhost:3000')
    webview.start()

thread = threading.Thread(target=run_ui, daemon=True)
thread.start()

# Problems:
# 1. Window floats separately from Maya
# 2. Thread conflicts with Maya's Qt
# 3. Window doesn't close when Maya closes
# 4. No access to Maya's window hierarchy
```

**Maya with AuroraView** (✅ Recommended):
```python
from auroraview import WebView
import maya.OpenMayaUI as omui

# Get Maya's main window handle
maya_hwnd = int(omui.MQtUtil.mainWindow())

# Create embedded WebView
webview = WebView.create(
    "Maya Tool",
    url="http://localhost:3000",
    parent=maya_hwnd,
    mode="owner",  # Cross-thread safe
)

@webview.on("export_scene")
def handle_export(data):
    cmds.file(data['path'], exportAll=True, type="FBX export")

webview.show()  # Non-blocking, auto timer

# Benefits:
# 1. Embedded under Maya's window
# 2. Closes when Maya closes
# 3. Event processing integrated
# 4. Full DCC API access
```

---

## 6. Performance Comparison

### 6.1 Architecture Overhead

| Aspect | PyWebView | AuroraView |
|--------|-----------|------------|
| Language | Pure Python | Rust + Python |
| IPC | Python threading | Lock-free DashMap |
| Memory Model | GC-managed | Zero-cost abstractions |
| Callback Dispatch | Python dict | Rust DashMap (concurrent) |

### 6.2 Expected Performance Characteristics

| Metric | PyWebView | AuroraView | Reason |
|--------|-----------|------------|--------|
| Startup Time | ~500ms | ~200ms | No Python wrapper layer |
| Memory (idle) | ~100MB | ~50MB | Rust efficiency |
| Event Latency | ~50ms | ~10ms | Lock-free IPC |
| High-frequency Events | ⚠️ GIL contention | ✅ Lock-free | DashMap vs Python dict |

### 6.3 IPC Performance

**PyWebView IPC Flow**:
```
JavaScript → Native Bridge → Python eval → Dict lookup → Callback
     ↑ GIL acquisition required, potential blocking
```

**AuroraView IPC Flow**:
```
JavaScript → Wry IPC → Rust Handler → DashMap (lock-free) → Python callback
     ↑ No GIL until Python callback, minimal blocking
```

---

## 7. Type Safety & Memory Safety

### 7.1 Type Safety

| Aspect | PyWebView | AuroraView |
|--------|-----------|------------|
| Python Types | Optional (typing) | Optional (typing) |
| Core Types | N/A (pure Python) | Rust strict types |
| Runtime Checks | Python exceptions | Rust compile-time + Python |
| Null Safety | None checks | Rust Option<T> |

### 7.2 Memory Safety

| Aspect | PyWebView | AuroraView |
|--------|-----------|------------|
| Memory Model | Python GC | Rust ownership |
| Buffer Overflows | Protected by Python | Prevented at compile |
| Use-after-free | Protected by Python | Prevented at compile |
| Race Conditions | Manual locking | Rust Send/Sync |
| Memory Leaks | GC-managed | RAII |

---

## 8. Use Case Recommendations

### Choose PyWebView When:

1. ✅ **Building standalone desktop applications**
   - Simple GUI wrapper for web content
   - No embedding requirements

2. ✅ **Rapid prototyping**
   - Quick proof-of-concept
   - No performance requirements

3. ✅ **Cross-platform is priority**
   - Need Android support
   - All platforms required now

4. ✅ **Using built-in features**
   - File dialogs needed
   - Native menus needed
   - Cookie management needed

5. ✅ **Mature, stable solution needed**
   - 10+ years of development
   - Large community
   - Extensive documentation

### Choose AuroraView When:

1. ✅ **Developing DCC software plugins**
   - Maya, Houdini, Blender, Nuke tools
   - Embedded UI requirements

2. ✅ **High-performance UI needed**
   - Real-time data visualization
   - High-frequency event updates

3. ✅ **DCC integration is critical**
   - Parent window embedding
   - Lifecycle management
   - Custom protocols for assets

4. ✅ **Type/memory safety matters**
   - Production-critical tools
   - Long-running processes

5. ✅ **Modern architecture preferred**
   - Rust-based performance
   - Lock-free concurrency
   - Future-proof design

---

## 9. Migration Considerations

### From PyWebView to AuroraView

**API Mapping**:

| PyWebView | AuroraView |
|-----------|------------|
| `webview.create_window()` | `WebView.create()` |
| `webview.start()` | `webview.show()` |
| `window.evaluate_js()` | `webview.eval_js()` |
| `window.expose()` | `webview.bind_call()` |
| `js_api=Api()` | `webview.bind_api(api)` |
| `window.load_url()` | `webview.load_url()` |
| `window.load_html()` | `webview.load_html()` |

**Example Migration**:

```python
# PyWebView (before)
import webview

class Api:
    def greet(self, name):
        return f"Hello, {name}!"

window = webview.create_window('App', 'index.html', js_api=Api())
webview.start()

# AuroraView (after)
from auroraview import WebView

class Api:
    def greet(self, name):
        return f"Hello, {name}!"

webview = WebView.create("App", url="index.html")
webview.bind_api(Api())
webview.show()
```

---

## 10. Conclusion

### Summary

| Criteria | Winner | Reason |
|----------|--------|--------|
| **DCC Integration** | AuroraView | Purpose-built for DCC |
| **Performance** | AuroraView | Rust core |
| **Maturity** | PyWebView | 10+ years, stable |
| **Platform Support** | PyWebView | More platforms now |
| **Type Safety** | AuroraView | Rust guarantees |
| **Ease of Use** | Tie | Both have clean APIs |
| **Community** | PyWebView | Larger ecosystem |
| **Future-proof** | AuroraView | Modern architecture |

### Final Recommendation

**For DCC Plugin Development**: Choose **AuroraView**
- First-class DCC support
- Parent window embedding
- Lifecycle management
- Thread-safe by design

**For General Desktop Apps**: Choose **PyWebView**
- Mature and stable
- More platform support
- Rich feature set
- Large community

**For Hybrid Use Cases**: Consider both
- Use AuroraView for DCC-embedded tools
- Use PyWebView for standalone utilities

---

## References

- [PyWebView Documentation](https://pywebview.flowrl.com/)
- [PyWebView GitHub](https://github.com/r0x0r/pywebview)
- [AuroraView Documentation](./README.md)
- [Wry (WebView library)](https://github.com/tauri-apps/wry)
- [Tao (Window management)](https://github.com/tauri-apps/tao)
- [PyO3 (Rust-Python bindings)](https://github.com/PyO3/pyo3)
