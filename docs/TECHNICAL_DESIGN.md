# AuroraView - Technical Design Document

## Executive Summary

AuroraView is a high-performance, lightweight WebView framework designed specifically for Digital Content Creation (DCC) software. Built with Rust and providing Python bindings through PyO3, it enables modern web-based UIs in professional applications like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine.

## Problem Statement

### Current Challenges

1. **Integration Difficulty**: DCC software environments make it challenging to integrate modern web frameworks
2. **Performance Overhead**: Existing solutions (Electron, Qt WebEngine) have significant memory and size overhead
3. **Version Conflicts**: Qt WebEngine versions often conflict with DCC software's built-in Qt
4. **Limited Python Support**: Most web frameworks lack seamless Python integration
5. **Platform Inconsistency**: Different behavior across Windows, macOS, and Linux

### Target Users

- Pipeline TDs (Technical Directors)
- Tool Developers
- UI/UX Designers
- Plugin Developers

## Solution Architecture

### Technology Stack

#### Core Components

1. **Rust Core Library** (`auroraview_core`)
   - **Wry**: Cross-platform WebView library
   - **Tao**: Window management
   - **Tokio**: Async runtime
   - **Serde**: Serialization

2. **Python Bindings** (`auroraview`)
   - **PyO3**: Rust-Python bindings with ABI3 support
   - **Maturin**: Build system

3. **System WebView**
   - **Windows**: WebView2 (Chromium-based)
   - **macOS**: WKWebView (Safari-based)
   - **Linux**: WebKitGTK

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    DCC Application Layer                     │
│         (Maya/Max/Houdini/Blender/Photoshop/UE)             │
└────────────────────────┬────────────────────────────────────┘
                         │ Python API
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Python Package (auroraview)                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  High-Level API                                       │  │
│  │  - WebView class                                      │  │
│  │  - Event decorators                                   │  │
│  │  - Utility functions                                  │  │
│  └────────────────────┬─────────────────────────────────┘  │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐  │
│  │  PyO3 Bindings (_core)                               │  │
│  │  - Type conversion                                    │  │
│  │  - GIL management                                     │  │
│  │  - Error handling                                     │  │
│  └────────────────────┬─────────────────────────────────┘  │
└───────────────────────┼──────────────────────────────────────┘
                        │ FFI (Foreign Function Interface)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│           Rust Core Library (auroraview_core)              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  WebView Module                                       │  │
│  │  - Window management (Tao)                            │  │
│  │  - WebView creation (Wry)                             │  │
│  │  - Event loop handling                                │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  IPC Module                                           │  │
│  │  - Message serialization                              │  │
│  │  - Event routing                                      │  │
│  │  - Callback management                                │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Protocol Handler Module                              │  │
│  │  - Custom protocol registration                       │  │
│  │  - Resource loading                                   │  │
│  │  - MIME type handling                                 │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              System Native WebView                           │
│  Windows: WebView2 | macOS: WKWebView | Linux: WebKitGTK   │
└─────────────────────────────────────────────────────────────┘
```

## Core Features

### 1. WebView Management

#### Window Creation
- Configurable size, title, and decorations
- Support for transparent windows
- Always-on-top option
- Multi-window support

#### Content Loading
- URL loading (HTTP/HTTPS)
- HTML string loading
- Custom protocol handlers (e.g., `dcc://`)

### 2. Bidirectional Communication (IPC)

#### Python → JavaScript
```python
# Emit events to JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube"]})

# Execute JavaScript directly
webview.eval_js("console.log('Hello from Python')")
```

#### JavaScript → Python
```python
# Register event handlers
@webview.on("export_scene")
def handle_export(data):
    export_to_file(data["path"])
```

#### Message Format
```json
{
  "event": "event_name",
  "data": { "key": "value" },
  "id": "optional_message_id"
}
```

### 3. Custom Protocol Handler

Enables loading resources from DCC projects:

```python
# Register custom protocol
webview.register_protocol("dcc", lambda uri: {
    "dcc://assets/texture.png": load_texture(),
    "dcc://scene/data": get_scene_json()
})
```

### 4. Developer Tools

- Chrome DevTools integration (Windows/Linux)
- Safari Web Inspector (macOS)
- Console logging
- Network inspection

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

### Package Size

- Wheel size: ~5 MB (platform-specific)
- No bundled Chromium

## Platform-Specific Considerations

### Windows

**WebView2 Requirements:**
- Windows 10 1803+ or Windows 11
- WebView2 Runtime (auto-installed on Windows 11)
- Fallback: Download WebView2 installer

**Python Compatibility:**
- Python 3.7+ (including Maya 2022+, 3ds Max 2023+)

### macOS

**WKWebView:**
- macOS 10.13+
- Native Safari engine
- Best performance on Apple Silicon

### Linux

**WebKitGTK:**
- Requires `libwebkit2gtk-4.0-37` or newer
- Install: `sudo apt install libwebkit2gtk-4.0-dev`

## DCC Software Integration

### Supported DCC Applications

| DCC Software | Python Version | Status | Notes |
|--------------|----------------|--------|-------|
| Maya 2022+ | 3.7+ | [OK] Supported | Full support |
| 3ds Max 2023+ | 3.7+ | [OK] Supported | Windows only |
| Houdini 19+ | 3.7+ | [OK] Supported | All platforms |
| Blender 3.0+ | 3.9+ | [OK] Supported | All platforms |
| Photoshop 2024+ | 3.7+ | [CONSTRUCTION] Planned | Via UXP |
| Unreal Engine 5+ | 3.7+ | [CONSTRUCTION] Planned | Via Python API |

### Integration Patterns

#### 1. Standalone Panel
```python
from auroraview import WebView

webview = WebView(title="My Tool", width=800, height=600)
webview.load_url("http://localhost:3000")
webview.show()
```

#### 2. Embedded in DCC UI
```python
# Maya example
import maya.cmds as cmds

def create_docked_panel():
    webview = WebView(title="Docked Tool")
    # Dock into Maya's UI
    # Implementation varies by DCC
```

#### 3. Modal Dialog
```python
with WebView(title="Export Options") as webview:
    webview.load_html(export_dialog_html)
    webview.show()
    # Blocks until closed
```

## Security Considerations

### Content Security Policy (CSP)

Default CSP:
```
default-src 'self';
script-src 'self' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
```

### Custom Protocol Security

- Validate all URIs
- Sanitize file paths
- Prevent directory traversal

### JavaScript Execution

- Sandboxed execution
- No access to file system (except via Python API)
- CORS restrictions apply

## Error Handling

### Rust Layer
- Use `Result<T, E>` for all fallible operations
- Convert to Python exceptions via PyO3

### Python Layer
- Raise descriptive exceptions
- Provide error context
- Log errors for debugging

## Testing Strategy

### Unit Tests

**Rust:**
```bash
cargo test
```

**Python:**
```bash
pytest tests/
```

### Integration Tests

- Test in actual DCC environments
- Automated UI testing
- Cross-platform validation

### Performance Benchmarks

```bash
cargo bench
```

## Build and Distribution

### Build Process

1. **Rust Compilation**
   ```bash
   cargo build --release
   ```

2. **Python Wheel Creation**
   ```bash
   maturin build --release
   ```

3. **Platform-Specific Wheels**
   - Windows: `auroraview-0.1.0-cp37-abi3-win_amd64.whl`
   - macOS: `auroraview-0.1.0-cp37-abi3-macosx_10_13_x86_64.whl`
   - Linux: `auroraview-0.1.0-cp37-abi3-manylinux_2_17_x86_64.whl`

### Distribution Channels

1. **PyPI** (primary)
   ```bash
   pip install dcc-webview
   ```

2. **GitHub Releases** (binaries)

3. **Conda** (future)

## Future Enhancements

### Phase 2 Features

- [ ] Multi-window management
- [ ] Window state persistence
- [ ] Screenshot/recording API
- [ ] Drag-and-drop support
- [ ] File picker integration

### Phase 3 Features

- [ ] WebGL/WebGPU support
- [ ] Video playback
- [ ] Audio support
- [ ] Clipboard integration
- [ ] Native menu integration

### Advanced Features

- [ ] Hot reload for development
- [ ] Remote debugging
- [ ] Performance profiling
- [ ] Memory leak detection

## Appendix

### Glossary

- **DCC**: Digital Content Creation
- **IPC**: Inter-Process Communication
- **FFI**: Foreign Function Interface
- **ABI**: Application Binary Interface
- **GIL**: Global Interpreter Lock

### References

- [Wry Documentation](https://github.com/tauri-apps/wry)
- [PyO3 Guide](https://pyo3.rs/)
- [WebView2 Documentation](https://docs.microsoft.com/en-us/microsoft-edge/webview2/)
- [WKWebView Documentation](https://developer.apple.com/documentation/webkit/wkwebview)

### Version History

- **0.1.0** (2025-01): Initial release
  - Basic WebView functionality
  - Python bindings
  - IPC support
  - Custom protocols

