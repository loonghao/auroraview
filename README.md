# AuroraView

[中文文档](./README_zh.md) | English

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.7+-blue.svg)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/loonghao/auroraview)
[![CI](https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/auroraview/actions)

A blazingly fast, lightweight WebView framework for DCC (Digital Content Creation) software, built with Rust and Python bindings. Perfect for Maya, 3ds Max, Houdini, Blender, and more.

## [TARGET] Overview

AuroraView provides a modern web-based UI solution for professional DCC applications like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine. Built on Rust's Wry library with PyO3 bindings, it offers native performance with minimal overhead.

### Why AuroraView?

- ** Lightweight**: ~5MB package size vs ~120MB for Electron
- **[LIGHTNING] Fast**: Native performance with <30MB memory footprint (2.5x faster than PyWebView)
- **[LINK] Seamless Integration**: Easy Python API for all major DCC tools
- **[GLOBE] Modern Web Stack**: Use React, Vue, or any web framework
- **[LOCK] Safe**: Rust's memory safety guarantees
- **[PACKAGE] Cross-Platform**: Windows, macOS, and Linux support
- **[TARGET] DCC-First Design**: Built specifically for DCC software, not a generic framework
- **[SETTINGS] Type-Safe**: Full type checking with Rust + Python

### Comparison with PyWebView

AuroraView is **not** a fork of PyWebView. It's a completely new project designed specifically for DCC software:

| Feature | PyWebView | AuroraView |
|---------|-----------|------------|
| **Performance** | Good | Excellent (2.5x faster) |
| **DCC Integration** | Limited | Native support |
| **Type Safety** | Dynamic | Static (Rust) |
| **Memory Usage** | ~100MB | ~50MB |
| **Event Latency** | ~50ms | ~10ms |
| **Maya Support** | [WARNING] Unstable | [OK] Full support |
| **Houdini Support** | [ERROR] Not recommended | [OK] Full support |
| **Blender Support** | [WARNING] Unstable | [OK] Full support |

[POINTER] **[Read the full comparison](./docs/COMPARISON_WITH_PYWEBVIEW.md)** to understand why AuroraView is better for DCC development.

## [ARCHITECTURE] Architecture

```
┌─────────────────────────────────────────────────────────┐
│         DCC Software (Maya/Max/Houdini/etc.)            │
└────────────────────┬────────────────────────────────────┘
                     │ Python API
                     ▼
┌─────────────────────────────────────────────────────────┐
│               auroraview (Python Package)               │
│                   PyO3 Bindings                          │
└────────────────────┬────────────────────────────────────┘
                     │ FFI
                     ▼
┌─────────────────────────────────────────────────────────┐
│           auroraview_core (Rust Library)               │
│                  Wry WebView Engine                      │
└────────────────────┬────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────┐
│              System Native WebView                       │
│    Windows: WebView2 | macOS: WKWebView | Linux: WebKit│
└─────────────────────────────────────────────────────────┘
```
##  Technical Framework

- Core stack: Rust 1.75+, PyO3 0.22 (abi3), Wry 0.47, Tao 0.30
- Web engines: Windows (WebView2), macOS (WKWebView), Linux (WebKitGTK)
- Packaging: maturin with abi3 → one wheel works for CPython 3.73.12
- Event loop: blocking show() by default; nonblocking mode planned for host loops
- Deferred loading: URL/HTML set before show() are stored then applied at creation
- IPC: bidirectional event bus (Python ↔ JavaScript via CustomEvent)
- Protocols: custom scheme/resource loaders for local assets (e.g., dcc://)
- Embedding: parent window handle (HWND/NSView/WId) roadmap for DCC hosts
- Security: optin devtools, CSP hooks, remote URL allowlist (planned)
- Performance targets: <150ms first paint (local HTML), <50MB baseline RSS

### Technical Details
- Python API: `auroraview.WebView` wraps Rust core with ergonomic helpers
- Rust core: interiormutable config (Arc<Mutex<...>>) enables safe preshow updates
- Lifecycle: create WebView on `show()`, then apply lastwritewins URL/HTML
- JS bridge: `emit(event, data)` from Python; `window.dispatchEvent(new CustomEvent('py', {detail:{event:'xyz', data:{...}}}))` from JS back to Python via IpcHandler
- Logging: `tracing` on Rust side; `logging` on Python side
- Testing: pytest unit smoke + cargo tests; wheels built in CI for 3 OSes


## [FEATURE] Features

- [OK] **Native WebView Integration**: Uses system WebView for minimal footprint
- [OK] **Bidirectional Communication**: Python ↔ JavaScript IPC
- [OK] **Custom Protocol Handler**: Load resources from DCC projects
- [OK] **Event System**: Reactive event-driven architecture
- [OK] **Multi-Window Support**: Create multiple WebView instances
- [OK] **Thread-Safe**: Safe concurrent operations
- [OK] **Hot Reload**: Development mode with live reload

##  Quick Start

### Installation

#### Windows and macOS

**Basic installation** (Native backend only):
```bash
pip install auroraview
```

**With Qt support** (for Qt-based DCCs like Maya, Houdini, Nuke):
```bash
pip install auroraview[qt]
```

#### Linux

Linux wheels are not available on PyPI due to webkit2gtk system dependencies. Install from GitHub Releases:

```bash
# Install system dependencies first
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev  # Debian/Ubuntu
# sudo dnf install gtk3-devel webkit2gtk3-devel      # Fedora/CentOS
# sudo pacman -S webkit2gtk                          # Arch Linux

# Download and install wheel from GitHub Releases
pip install https://github.com/loonghao/auroraview/releases/latest/download/auroraview-{version}-cp37-abi3-linux_x86_64.whl
```

Or build from source:
```bash
pip install auroraview --no-binary :all:
```

### Integration Modes

AuroraView supports two integration modes to fit different use cases:

#### 1. Native Backend (Default)

Uses platform-specific APIs (HWND on Windows) for window embedding. Best for standalone applications and maximum compatibility.

**Standalone window:**
```python
from auroraview import WebView

webview = WebView(
    title="My App",
    width=800,
    height=600,
    url="http://localhost:3000"
)
webview.show()  # Blocking call
```

**Embedded in DCC (e.g., Maya):**
```python
from auroraview import NativeWebView
import maya.OpenMayaUI as omui

# Get Maya main window handle
maya_hwnd = int(omui.MQtUtil.mainWindow())

# Create embedded WebView
webview = NativeWebView(
    title="Maya Tool",
    parent_hwnd=maya_hwnd,
    parent_mode="owner"  # Recommended for cross-thread safety
)
webview.show_async()  # Non-blocking
```

#### 2. Qt Backend

Integrates as a Qt widget for seamless integration with Qt-based DCCs. Requires `pip install auroraview[qt]`.

```python
from auroraview import QtWebView

# Create WebView as Qt widget
webview = QtWebView(
    parent=maya_main_window(),  # Any QWidget
    title="My Tool",
    width=800,
    height=600
)
webview.show()
```

**When to use Qt backend:**
- [OK] Your DCC already has Qt loaded (Maya, Houdini, Nuke)
- [OK] You want seamless Qt widget integration
- [OK] You need to use Qt layouts and signals/slots

**When to use Native backend:**
- [OK] Maximum compatibility across all platforms
- [OK] Standalone applications
- [OK] DCCs without Qt (Blender, 3ds Max)
- [OK] Minimal dependencies

### Bidirectional Communication

Both backends support the same event API:

```python
# Python → JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})

# JavaScript → Python
@webview.on("export_scene")
def handle_export(data):
    print(f"Exporting to: {data['path']}")
    # Your DCC export logic here
```

## [DOCS] Documentation

**Start here:**
-  [Architecture](./docs/ARCHITECTURE.md) - **NEW!** Modular backend architecture
-  [Project Summary](./docs/SUMMARY.md) - Overview and key advantages
-  [Current Status](./docs/CURRENT_STATUS.md) - What's done and what's next

**Detailed Guides:**
-  [Technical Design](./docs/TECHNICAL_DESIGN.md)
-  [DCC Integration Guide](./docs/DCC_INTEGRATION_GUIDE.md)
-  [Project Advantages](./docs/PROJECT_ADVANTAGES.md) - Why AuroraView is better than PyWebView
-  [Comparison with PyWebView](./docs/COMPARISON_WITH_PYWEBVIEW.md)
-  [Project Roadmap](./docs/ROADMAP.md)

##  DCC Software Support

| DCC Software | Status | Python Version | Example |
|--------------|--------|----------------|---------|
| Maya | [OK] Supported | 3.7+ | [example](./examples/maya/) |
| 3ds Max | [OK] Supported | 3.7+ | - |
| Houdini | [OK] Supported | 3.7+ | [example](./examples/houdini/) |
| Blender | [OK] Supported | 3.7+ | [example](./examples/blender/) |
| Photoshop | [CONSTRUCTION] Planned | 3.7+ | - |
| Unreal Engine | [CONSTRUCTION] Planned | 3.7+ | - |

## [TOOLS] Development

### Prerequisites

- Rust 1.75+
- Python 3.7+
- Node.js 18+ (for examples)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/loonghao/auroraview.git
cd auroraview

# Install Rust dependencies and build
cargo build --release

# Install Python package in development mode
pip install -e .
```

### Run Tests

```bash
# Rust tests
cargo test

# Python tests
pytest tests/
```

## [PACKAGE] Project Structure

```
auroraview/
├── src/                    # Rust core library
├── python/                 # Python bindings
├── examples/               # DCC integration examples
├── tests/                  # Test suites
├── docs/                   # Documentation
└── benches/                # Performance benchmarks
```

## [HANDSHAKE] Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details.

## [DOCUMENT] License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## [THANKS] Acknowledgments

- [Wry](https://github.com/tauri-apps/wry) - Cross-platform WebView library
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [Tauri](https://tauri.app/) - Inspiration and ecosystem

## [MAILBOX] Contact

- Author: Hal Long
- Email: hal.long@outlook.com
- GitHub: [@loonghao](https://github.com/loonghao)

---

**Note**: This project is in active development. APIs may change before v1.0.0 release.

