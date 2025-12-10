<p align="center">
  <img src="assets/icons/auroraview-logo-text.png" alt="AuroraView Logo" width="400">
</p>

<p align="center">
  <a href="./README_zh.md">中文文档</a> | English
</p>

<p align="center">
  <a href="https://pypi.org/project/auroraview/"><img src="https://img.shields.io/pypi/v/auroraview.svg" alt="PyPI Version"></a>
  <a href="https://pypi.org/project/auroraview/"><img src="https://img.shields.io/pypi/pyversions/auroraview.svg" alt="Python Versions"></a>
  <a href="https://pepy.tech/project/auroraview"><img src="https://static.pepy.tech/badge/auroraview" alt="Downloads"></a>
  <a href="https://codecov.io/gh/loonghao/auroraview"><img src="https://codecov.io/gh/loonghao/auroraview/branch/main/graph/badge.svg" alt="Codecov"></a>
  <a href="https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml/badge.svg" alt="PR Checks"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust"></a>
  <a href="https://github.com/loonghao/auroraview"><img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg" alt="Platform"></a>
  <a href="https://github.com/loonghao/auroraview/actions/workflows/ci.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml/badge.svg?branch=main" alt="Build Wheels"></a>
  <a href="https://github.com/loonghao/auroraview/actions/workflows/release.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/release.yml/badge.svg?branch=main" alt="Release"></a>
</p>

<p align="center">
  <a href="https://github.com/loonghao/auroraview/actions/workflows/codeql.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/codeql.yml/badge.svg?branch=main" alt="CodeQL"></a>
  <a href="https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml"><img src="https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml/badge.svg?branch=main" alt="Security Audit"></a>
  <a href="https://github.com/loonghao/auroraview/releases"><img src="https://img.shields.io/github/v/release/loonghao/auroraview?display_name=tag" alt="Latest Release"></a>
  <a href="https://pre-commit.com/"><img src="https://img.shields.io/badge/pre--commit-enabled-brightgreen.svg" alt="pre-commit"></a>
</p>

<p align="center">
  <a href="https://github.com/loonghao/auroraview/stargazers"><img src="https://img.shields.io/github/stars/loonghao/auroraview?style=social" alt="GitHub Stars"></a>
  <a href="https://github.com/loonghao/auroraview/releases"><img src="https://img.shields.io/github/downloads/loonghao/auroraview/total" alt="GitHub Downloads"></a>
  <a href="https://github.com/loonghao/auroraview/commits/main"><img src="https://img.shields.io/github/last-commit/loonghao/auroraview" alt="Last Commit"></a>
  <a href="https://github.com/loonghao/auroraview/graphs/commit-activity"><img src="https://img.shields.io/github/commit-activity/m/loonghao/auroraview" alt="Commit Activity"></a>
</p>

<p align="center">
  <a href="https://github.com/loonghao/auroraview/issues"><img src="https://img.shields.io/github/issues/loonghao/auroraview" alt="Open Issues"></a>
  <a href="https://github.com/loonghao/auroraview/pulls"><img src="https://img.shields.io/github/issues-pr/loonghao/auroraview" alt="Open PRs"></a>
  <a href="https://github.com/loonghao/auroraview/graphs/contributors"><img src="https://img.shields.io/github/contributors/loonghao/auroraview" alt="Contributors"></a>
  <a href="https://conventionalcommits.org"><img src="https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg" alt="Conventional Commits"></a>
</p>

<p align="center">
  <a href="https://github.com/googleapis/release-please"><img src="https://img.shields.io/badge/release--please-enabled-blue" alt="release-please"></a>
  <a href="./.github/dependabot.yml"><img src="https://img.shields.io/badge/dependabot-enabled-025E8C?logo=dependabot" alt="Dependabot"></a>
  <a href="https://docs.astral.sh/ruff/"><img src="https://img.shields.io/badge/code%20style-ruff-000000.svg" alt="Code Style: ruff"></a>
  <a href="http://mypy-lang.org/"><img src="https://img.shields.io/badge/type%20checked-mypy-2A6DB0.svg" alt="Type Checked: mypy"></a>
</p>

<p align="center">
  <a href="./CODE_OF_CONDUCT.md">Code of Conduct</a> •
  <a href="./SECURITY.md">Security Policy</a> •
  <a href="https://github.com/loonghao/auroraview/issues">Issue Tracker</a>
</p>


A lightweight WebView framework for DCC (Digital Content Creation) software, built with Rust and Python bindings. Perfect for Maya, 3ds Max, Houdini, Blender, and more.

> **⚠️ Development Status**: This project is under active development. APIs may change before v1.0.0 release. The project has not been extensively tested on Linux and macOS platforms.

## Overview

AuroraView provides a modern web-based UI solution for professional DCC applications like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine. Built on Rust's Wry library with PyO3 bindings, it offers native performance with minimal overhead.

### Key Features

- **Lightweight**: ~5MB package size vs ~120MB for Electron
- **Fast**: Native Rust performance with minimal memory footprint
- **Seamless Integration**: Easy Python API for all major DCC tools
- **Modern Web Stack**: Use React, Vue, or any web framework
- **Safe**: Rust's memory safety guarantees
- **Cross-Platform**: Windows, macOS, and Linux support
- **DCC-First Design**: Built specifically for DCC software integration
- **Type-Safe**: Full type checking with Rust + Python

[POINTER] **[DCC Integration Guide](./docs/DCC_INTEGRATION.md)** - Learn how to integrate AuroraView into Maya, Houdini, Nuke, and other DCC applications.

## Architecture

<p align="center">
  <img src="assets/images/architecture.png" alt="AuroraView Architecture" width="800">
</p>
##  Technical Framework

- Core stack: Rust 1.75+, PyO3 0.22 (abi3), Wry 0.47, Tao 0.30
- Web engines: Windows (WebView2), macOS (WKWebView), Linux (WebKitGTK)
- Packaging: maturin with abi3 → one wheel works for CPython 3.7-3.13
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


## Features

### Core Features
- [OK] **Native WebView Integration**: Uses system WebView (WebView2/WKWebView/WebKitGTK) for minimal footprint
- [OK] **Bidirectional Communication**: Python ↔ JavaScript IPC with async/await support
- [OK] **Custom Protocol Handler**: Load resources from DCC projects (`auroraview://`, custom protocols)
- [OK] **Event System**: Node.js-style EventEmitter with `on()`, `once()`, `off()`, `emit()`
- [OK] **Multi-Window Support**: WindowManager for creating/managing multiple windows with cross-window events
- [OK] **Thread-Safe**: Rust-guaranteed memory safety and concurrent operations

### Storage & Data
- [OK] **localStorage/sessionStorage**: Full CRUD operations for web storage
- [OK] **Cookie Management**: set/get/delete/clear cookies
- [OK] **Browsing Data**: Clear cache, cookies, history with `clear_browsing_data()`

### Window & Navigation
- [OK] **File Dialogs**: open_file, save_file, select_folder, select_folders
- [OK] **Message Dialogs**: confirm, alert, error, ok_cancel dialogs
- [OK] **Navigation Control**: go_back, go_forward, reload, stop, can_go_back/forward
- [OK] **Window Events**: on_window_show/hide/focus/blur/resize, on_fullscreen_changed

### DCC Integration
- [OK] **Lifecycle Management**: Automatic cleanup when parent DCC application closes
- [OK] **Qt Backend**: QtWebView for seamless Qt-based DCC integration
- [OK] **WebView2 Warmup**: Pre-initialize WebView2 for faster DCC startup
- [OK] **Performance Monitoring**: get_performance_metrics(), get_ipc_stats()

### Security
- [OK] **CSP Configuration**: Content Security Policy support
- [OK] **CORS Control**: Cross-Origin Resource Sharing management
- [OK] **Permission System**: Fine-grained permission controls

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

> **Note for DCC Integration**: Qt-based DCC applications (Maya, Houdini, Nuke, 3ds Max) require QtPy as a middleware layer to handle different Qt versions across DCC applications. The `[qt]` extra installs QtPy automatically.

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

AuroraView provides three main integration modes for different use cases:

| Mode | Class | Best For | Docking Support |
|------|-------|----------|-----------------|
| **Qt Native** | `QtWebView` | Maya, Houdini, Nuke, 3ds Max | ✅ QDockWidget |
| **HWND** | `AuroraView` | Unreal Engine, non-Qt apps | ✅ via HWND API |
| **Standalone** | `run_standalone` | Desktop applications | N/A |

### WebView vs AuroraView: Choosing the Right Class

Understanding the difference between `WebView` and `AuroraView` helps you choose the right class for your use case:

| Feature | `WebView` | `AuroraView` |
|---------|-----------|--------------|
| **Purpose** | Core class, direct Rust binding | High-level wrapper for HWND integration |
| **API Style** | Mixin-based, full control | Simplified, delegate pattern |
| **Best For** | Advanced customization, DCC embedding | Unreal Engine, quick prototyping |
| **Keep-alive** | Manual management | Automatic (class-level registry) |
| **API Binding** | Manual `bind_api()` call | Auto-bind via `api` parameter |
| **Lifecycle Hooks** | `on_loaded`, `on_shown`, etc. | `on_show`, `on_hide`, `on_close`, `on_ready` |

**When to use `WebView`:**
- You need full control over the WebView lifecycle
- You're building Qt-based DCC integrations (use with `QtWebView`)
- You need advanced features like custom protocols, asset roots, etc.

**When to use `AuroraView`:**
- You're integrating with Unreal Engine or other HWND-based applications
- You want a simpler API with automatic keep-alive management
- You're migrating from pywebview and want a familiar API

**Example: WebView (Core)**
```python
from auroraview import WebView

# Full control mode
webview = WebView.create(
    title="My Tool",
    url="http://localhost:3000",
    auto_show=True,
    auto_timer=True
)
webview.bind_api(my_api_object)  # Manual API binding
webview.show()
```

**Example: AuroraView (HWND wrapper)**
```python
from auroraview import AuroraView

# Simplified mode with auto API binding
webview = AuroraView(
    url="http://localhost:3000",
    api=my_api_object  # Auto-bound at construction
)
webview.show()
```

#### 1. Qt Native Mode (QtWebView)

**Best for Qt-based DCC applications** - Maya, Houdini, Nuke, 3ds Max.

This mode creates a true Qt widget that can be docked, embedded in layouts, and managed by Qt's parent-child system.

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDialog, QVBoxLayout

# Create a dockable dialog
dialog = QDialog(maya_main_window())
layout = QVBoxLayout(dialog)

# Create embedded WebView as Qt widget
webview = QtWebView(
    parent=dialog,
    width=800,
    height=600
)
layout.addWidget(webview)

# Load content
webview.load_url("http://localhost:3000")

# Show dialog - WebView closes automatically with parent
dialog.show()
webview.show()
```

**Key features:**
- ✅ Works with `QDockWidget` for dockable panels
- ✅ Automatic lifecycle management (closes with parent)
- ✅ Native Qt event integration
- ✅ Supports all Qt layout managers

#### 2. HWND Mode (AuroraView)

**Best for Unreal Engine and non-Qt applications** that need direct window handle access.

```python
from auroraview import AuroraView

# Create standalone WebView
webview = AuroraView(url="http://localhost:3000")
webview.show()

# Get HWND for external integration
hwnd = webview.get_hwnd()
if hwnd:
    # Unreal Engine integration
    import unreal
    unreal.parent_external_window_to_slate(hwnd)
```

**Key features:**
- ✅ Direct HWND access via `get_hwnd()`
- ✅ Works with any application that accepts HWND
- ✅ No Qt dependency required
- ✅ Full control over window positioning

#### 3. Standalone Mode

**Best for desktop applications** - quick one-liner for standalone apps.

```python
from auroraview import run_standalone

# Launch standalone app (blocks until closed)
run_standalone(
    title="My App",
    url="https://example.com",
    width=1024,
    height=768
)
```

**Key features:**
- ✅ Simplest API - one function call
- ✅ Automatic event loop management
- ✅ No parent window required

### Quick Start (v0.2.0 New API)

**Standalone window (2 lines!):**
```python
from auroraview import WebView

# Create and show - that's it!
webview = WebView.create("My App", url="http://localhost:3000")
webview.show()  # Auto-blocks until closed
```

**Maya integration:**
```python
from auroraview import WebView


maya_hwnd = int(omui.MQtUtil.mainWindow())
webview = WebView.create("Maya Tool", url="http://localhost:3000", parent=maya_hwnd)
webview.show()  # Embedded mode: non-blocking, auto timer
```

**Houdini integration:**
```python
from auroraview import WebView
import hou

hwnd = int(hou.qt.mainWindow().winId())
webview = WebView.create("Houdini Tool", url="http://localhost:3000", parent=hwnd)
webview.show()  # Embedded mode: non-blocking, auto timer
```

### Command-Line Interface

AuroraView includes a CLI for quickly launching WebView windows:

```bash
# Load a URL
auroraview --url https://example.com

# Load a local HTML file
auroraview --html /path/to/file.html

# Custom window configuration
auroraview --url https://example.com --title "My App" --width 1024 --height 768

# Using with uvx (no installation required)
uvx auroraview --url https://example.com
```

**[See CLI Documentation](./docs/CLI.md)** for more details.

### Custom Window Icon

AuroraView displays the default AuroraView icon by default. You can customize it with your own icon:

```python
from auroraview import WebView

# Use custom icon
webview = WebView.create(
    "My App",
    url="http://localhost:3000",
    icon="path/to/my-icon.png"  # Custom icon path
)
webview.show()
```

**Icon Requirements:**
| Property | Recommendation |
|----------|----------------|
| **Format** | PNG (recommended), ICO, JPEG, BMP, GIF |
| **Size** | 32×32 (taskbar), 64×64 (alt-tab), 256×256 (high-DPI) |
| **Color Depth** | 32-bit RGBA for transparency support |
| **Best Practice** | Use a square image; non-square images will be stretched |

> **Tip**: For best results across all Windows UI elements, provide a 32×32 PNG with transparency.

**Nuke integration:**
```python
from auroraview import WebView
from qtpy import QtWidgets

main = QtWidgets.QApplication.activeWindow()
hwnd = int(main.winId())
webview = WebView.create("Nuke Tool", url="http://localhost:3000", parent=hwnd)
webview.show()  # Embedded mode: non-blocking, auto timer
```

**Blender integration:**
```python
from auroraview import WebView

# Blender runs standalone (no parent window)
webview = WebView.create("Blender Tool", url="http://localhost:3000")
webview.show()  # Standalone: blocks until closed (use show(wait=False) for async)
```

## Usage Patterns

AuroraView supports multiple API styles to fit your development workflow. Choose the pattern that best matches your project's complexity and team's preferences.

### Pattern 1: Decorator Style (Simplest)

Best for: **Quick prototypes, simple tools, one-off scripts**

```python
from auroraview import WebView

view = WebView(title="My Tool", url="http://localhost:3000")

# Register API methods using bind_call decorator
@view.bind_call("api.get_data")
def get_data() -> dict:
    """Called from JS: await auroraview.api.get_data()"""
    return {"items": [1, 2, 3], "count": 3}

@view.bind_call("api.save_file")
def save_file(path: str = "", content: str = "") -> dict:
    """Called from JS: await auroraview.api.save_file({path: "/tmp/a.txt", content: "hello"})"""
    with open(path, "w") as f:
        f.write(content)
    return {"ok": True, "path": path}

# Register event handlers (no return value)
@view.on("button_clicked")
def handle_click(data: dict):
    """Called when JS sends: auroraview.send_event("button_clicked", {...})"""
    print(f"Button clicked: {data['id']}")

view.show()
```

**JavaScript side:**
```javascript
// Call API methods (with return value)
const data = await auroraview.api.get_data();
console.log(data.items);  // [1, 2, 3]

const result = await auroraview.api.save_file({ path: "/tmp/test.txt", content: "Hello" });
console.log(result.ok);  // true

// Send events to Python (fire-and-forget)
auroraview.send_event("button_clicked", { id: "save_btn" });

// Listen for Python events
auroraview.on("data_updated", (data) => {
    console.log("Data updated:", data);
});
```

### Pattern 2: Class Inheritance (Recommended, Qt-like)

Best for: **Production tools, team collaboration, complex applications**

```python
from auroraview import WebView, Signal

class OutlinerTool(WebView):
    """A Maya-style outliner tool demonstrating Qt-like patterns."""

    # ─── Signal Definitions (Python → JS notifications) ───
    selection_changed = Signal(list)
    progress_updated = Signal(int, str)
    scene_loaded = Signal(str)

    def __init__(self):
        super().__init__(
            title="Outliner Tool",
            url="http://localhost:3000",
            width=400,
            height=600
        )
        self._setup_api()
        self._setup_connections()

    def _setup_api(self):
        """Bind API methods for JavaScript access."""
        # Bind all public methods of self under "api" namespace
        self.bind_api(self, namespace="api")

    # ─── API Methods (JS → Python) ───
    def get_hierarchy(self, root: str = None) -> dict:
        """Get scene hierarchy. JS: await auroraview.api.get_hierarchy()"""
        return {
            "children": ["group1", "mesh_cube", "camera1"],
            "count": 3
        }

    def rename_object(self, old_name: str = "", new_name: str = "") -> dict:
        """Rename scene object. JS: await auroraview.api.rename_object({old_name: "a", new_name: "b"})"""
        return {"ok": True, "old": old_name, "new": new_name}

    def delete_objects(self, names: list = None) -> dict:
        """Delete objects. JS: await auroraview.api.delete_objects({names: ["obj1", "obj2"]})"""
        names = names or []
        return {"ok": True, "deleted": len(names)}

    # ─── Event Handlers ───
    def _setup_connections(self):
        """Setup event handlers and signal connections."""
        @self.on("item_selected")
        def handle_selection(data: dict):
            items = data.get("items", [])
            self.selection_changed.emit(items)

        @self.on("viewport_orbit")
        def handle_orbit(data: dict):
            dx, dy = data.get("dx", 0), data.get("dy", 0)
            print(f"Orbiting: dx={dx}, dy={dy}")

        # Connect signals to handlers
        self.selection_changed.connect(self._log_selection)

    def _log_selection(self, items: list):
        """Internal handler for selection changes."""
        print(f"Selection changed: {items}")

# Usage
tool = OutlinerTool()
tool.show()
```

**JavaScript side:**
```javascript
// Call API methods
const hierarchy = await auroraview.api.get_hierarchy();
const result = await auroraview.api.rename_object({ old_name: "cube1", new_name: "hero_cube" });

// Send events to Python
auroraview.send_event("item_selected", { items: ["mesh1", "mesh2"] });
auroraview.send_event("viewport_orbit", { dx: 10, dy: 5 });

// Listen for Python signals
auroraview.on("selection_changed", (items) => {
    highlightItems(items);
});

auroraview.on("progress_updated", (percent, message) => {
    updateProgressBar(percent, message);
});
```

### Pattern 3: Explicit Binding (Advanced)

Best for: **Dynamic configurations, plugin systems, runtime customization**

```python
from auroraview import WebView

view = WebView(title="Plugin Host", url="http://localhost:3000")

# Define functions separately
def get_plugins() -> dict:
    return {"plugins": ["plugin_a", "plugin_b"]}

def load_plugin(name: str) -> dict:
    print(f"Loading plugin: {name}")
    return {"ok": True, "plugin": name}

def on_plugin_event(data: dict):
    print(f"Plugin event: {data}")

# Explicitly bind at runtime
view.bind_call("get_plugins", get_plugins)
view.bind_call("load_plugin", load_plugin)

# Connect to built-in signals
view.on_ready.connect(lambda: print("WebView is ready!"))
view.on_navigate.connect(lambda url: print(f"Navigated to: {url}"))

# Register event handlers
view.register_callback("plugin_event", on_plugin_event)

# Dynamic binding based on configuration
config = {"features": ["export", "import"]}
if "export" in config["features"]:
    view.bind_call("export_data", lambda fmt: {"data": "...", "format": fmt})
if "import" in config["features"]:
    view.bind_call("import_data", lambda data: {"ok": True})

view.show()
```

**JavaScript side:**
```javascript
// Call dynamically bound methods
const plugins = await auroraview.api.get_plugins();
const result = await auroraview.api.load_plugin({ name: "plugin_a" });

// Call feature-specific methods (if enabled)
if (await auroraview.api.export_data) {
    const exported = await auroraview.api.export_data({ fmt: "json" });
}

// Emit events
auroraview.emit("plugin_event", { type: "activated", plugin: "plugin_a" });
```

### Pattern Comparison

| Aspect | Decorator | Class Inheritance | Explicit Binding |
|--------|-----------|------------------|------------------|
| **Complexity** | ⭐ Simple | ⭐⭐ Medium | ⭐⭐⭐ Advanced |
| **Best For** | Prototypes | Production | Plugins |
| **Signal Support** | ❌ | ✅ Full | ⚠️ Limited |
| **Auto-binding** | ❌ Manual | ✅ via bind_api | ❌ Manual |
| **Type Hints** | ✅ | ✅ | ✅ |
| **Qt Familiarity** | Low | High | Medium |
| **Testability** | Good | Excellent | Good |

> **Recommendation**: Start with **Pattern 1** for prototypes, graduate to **Pattern 2** for production tools. Use **Pattern 3** when building extensible systems.

See the [examples/](./examples/) directory for complete, runnable examples of each pattern.

### Advanced Usage

**Load HTML content:**
```python
from auroraview import WebView

html = """
<!DOCTYPE html>
<html>
<body>
    <h1>Hello from AuroraView!</h1>
    <button onclick="alert('Hello!')">Click Me</button>
</body>
</html>
"""

webview = WebView.create("My App", html=html)
webview.show()
```

**Custom configuration:**
```python
from auroraview import WebView

webview = WebView.create(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768,
    resizable=True,
    frame=True,  # Show window frame
    debug=True,  # Enable dev tools
    context_menu=False,  # Disable native context menu for custom menus
)
webview.show()
```

**Embedded mode helper (2025):**
```python
from auroraview import WebView

# Convenience helper = create(..., auto_show=True, auto_timer=True)
webview = WebView.run_embedded(
    "My Tool", url="http://localhost:3000", parent=parent_hwnd, mode="owner"
)
```

**Window Events System:**

AuroraView provides a comprehensive window event system for tracking window lifecycle:

```python
from auroraview import WebView
from auroraview.core.events import WindowEvent, WindowEventData

webview = WebView(title="My App", width=800, height=600)

# Register window event handlers using decorators
@webview.on_shown
def on_shown(data: WindowEventData):
    print("Window is now visible")

@webview.on_focused
def on_focused(data: WindowEventData):
    print("Window gained focus")

@webview.on_blurred
def on_blurred(data: WindowEventData):
    print("Window lost focus")

@webview.on_resized
def on_resized(data: WindowEventData):
    print(f"Window resized to {data.width}x{data.height}")

@webview.on_moved
def on_moved(data: WindowEventData):
    print(f"Window moved to ({data.x}, {data.y})")

@webview.on_closing
def on_closing(data: WindowEventData):
    print("Window is closing...")
    return True  # Return True to allow close, False to cancel

# Window control methods
webview.resize(1024, 768)
webview.move(100, 100)
webview.minimize()
webview.maximize()
webview.restore()
webview.toggle_fullscreen()
webview.focus()
webview.hide()

# Read-only window properties
print(f"Size: {webview.width}x{webview.height}")
print(f"Position: ({webview.x}, {webview.y})")
```

**Callback deregistration (EventTimer):**
```python
from auroraview import EventTimer

timer = EventTimer(webview, interval_ms=16)

def _on_close(): ...

timer.on_close(_on_close)
# Later, to remove the handler:
timer.off_close(_on_close)  # also available: off_tick(handler)
```

**Shared State (PyWebView-inspired):**

AuroraView provides automatic bidirectional state synchronization between Python and JavaScript:

```python
from auroraview import WebView

webview = WebView.create("My App", width=800, height=600)

# Access shared state (dict-like interface)
webview.state["user"] = "Alice"
webview.state["theme"] = "dark"
webview.state["count"] = 0

# Track state changes
@webview.state.on_change
def on_state_change(key: str, value, old_value):
    print(f"State changed: {key} = {value} (was {old_value})")

# In JavaScript:
# window.auroraview.state.user = "Bob";  // Syncs to Python
# console.log(window.auroraview.state.theme);  // "dark"
```

**Command System (Tauri-inspired):**

Register Python functions as RPC-style commands callable from JavaScript:

```python
from auroraview import WebView

webview = WebView.create("My App", width=800, height=600)

# Register commands using decorator
@webview.command
def greet(name: str) -> str:
    return f"Hello, {name}!"

@webview.command("add_numbers")
def add(x: int, y: int) -> int:
    return x + y

# In JavaScript:
# const msg = await auroraview.invoke("greet", {name: "World"});
# const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
```

**Channel Streaming:**

Stream large data from Python to JavaScript using channels:

```python
from auroraview import WebView

webview = WebView.create("My App", width=800, height=600)

# Create a channel for streaming data
with webview.create_channel() as channel:
    for i in range(100):
        channel.send({"progress": i, "data": f"chunk_{i}"})

# In JavaScript:
# const channel = auroraview.channel("channel_id");
# channel.onMessage((data) => console.log("Received:", data));
# channel.onClose(() => console.log("Stream complete"));
```

**Custom Protocol Handlers (Solve CORS Issues):**

AuroraView provides custom protocol handlers to load local resources without CORS restrictions:

```python
from auroraview import WebView

# 1. Built-in auroraview:// protocol for static assets
webview = WebView.create(
    title="My App",
    asset_root="C:/projects/my_app/assets"  # Enable auroraview:// protocol
)

# Now you can use auroraview:// in HTML
html = """
<html>
    <head>
        <link rel="stylesheet" href="auroraview://css/style.css">
    </head>
    <body>
        <img src="auroraview://icons/logo.png">
        <script src="auroraview://js/app.js"></script>
    </body>
</html>
"""
webview.load_html(html)

# 2. Register custom protocols for DCC-specific resources
def handle_fbx_protocol(uri: str) -> dict:
    """Load FBX files from Maya project"""
    path = uri.replace("fbx://", "")
    full_path = f"C:/maya_projects/current/{path}"

    try:
        with open(full_path, "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "application/octet-stream",
                "status": 200
            }
    except FileNotFoundError:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

webview.register_protocol("fbx", handle_fbx_protocol)

# Now you can use fbx:// in JavaScript
# fetch('fbx://models/character.fbx').then(r => r.arrayBuffer())
```

**Benefits:**
- ✅ No CORS restrictions (unlike `file://` URLs)
- ✅ Clean URLs (`auroraview://logo.png` vs `file:///C:/long/path/logo.png`)
- ✅ Security (limited to configured directories)
- ✅ Cross-platform path handling

### Custom Protocol Best Practices

#### Platform-Specific URL Format

The `auroraview://` protocol uses different URL formats on each platform:

| Platform | URL Format | Example |
|----------|------------|---------|
| **Windows** | `https://auroraview.localhost/path` | `https://auroraview.localhost/index.html` |
| **macOS** | `auroraview://path` | `auroraview://index.html` |
| **Linux** | `auroraview://path` | `auroraview://index.html` |

> **Note**: On Windows, wry (the underlying WebView library) maps custom protocols to HTTP/HTTPS format.
> We use `.localhost` as the host for security reasons.

#### Why `.localhost` is Secure

The `.localhost` TLD provides strong security guarantees:

1. **IANA Reserved** - `.localhost` is a reserved TLD (RFC 6761) that cannot be registered by anyone
2. **Local Only** - Browsers treat `.localhost` as a local address (127.0.0.1)
3. **Pre-DNS Interception** - Our protocol handler intercepts requests BEFORE DNS resolution
4. **No Network Traffic** - Requests never leave the local machine

#### Comparing Local Resource Loading Methods

| Method | Security | Recommendation |
|--------|----------|----------------|
| `auroraview://` with `asset_root` | ✅ **High** - Access restricted to specified directory | **Recommended** |
| `allow_file_protocol=True` | ⚠️ Low - Access to ANY file on system | Use with caution |
| HTTP server | ✅ High - Controlled access | Good for development |

**Recommended approach (using `asset_root` with relative paths):**

<table>
<tr><th>WebView.create()</th><th>run_standalone()</th></tr>
<tr>
<td>

```python
from auroraview import WebView

# Secure: Only files under assets/ are accessible
webview = WebView.create(
    title="My App",
    asset_root="./assets",
)

# Use relative paths in HTML - they resolve to asset_root
html = """
<html>
<body>
    <img src="./images/logo.png">
    <img src="./images/animation.gif">
</body>
</html>
"""
webview.load_html(html)
```

</td>
<td>

```python
from auroraview import run_standalone

# Secure: Only files under assets/ are accessible
# Use relative paths - they resolve to asset_root
html = """
<html>
<body>
    <img src="./images/logo.png">
    <img src="./images/animation.gif">
</body>
</html>
"""

run_standalone(
    title="My App",
    html=html,
    asset_root="./assets",
)
```

</td>
</tr>
</table>

**Less secure approach (using `file://` protocol):**

<table>
<tr><th>WebView.create()</th><th>run_standalone()</th></tr>
<tr>
<td>

```python
from auroraview import WebView
from auroraview import path_to_file_url

# ⚠️ Warning: Allows access to ANY file
gif_url = path_to_file_url("C:/path/to/animation.gif")

webview = WebView.create(
    title="My App",
    allow_file_protocol=True,
)

html = f'<img src="{gif_url}">'
webview.load_html(html)
```

</td>
<td>

```python
from auroraview import run_standalone
from auroraview import path_to_file_url

# ⚠️ Warning: Allows access to ANY file
gif_url = path_to_file_url("C:/path/to/animation.gif")

html = f'<img src="{gif_url}">'

run_standalone(
    title="My App",
    html=html,
    allow_file_protocol=True,
)
```

</td>
</tr>
</table>

> **Note**: The `path_to_file_url()` helper converts local paths to proper `file:///` URLs.
> Example: `C:\images\logo.gif` → `file:///C:/images/logo.gif`

See [examples/custom_protocol_example.py](./examples/custom_protocol_example.py) and [examples/local_assets_example.py](./examples/local_assets_example.py) for complete examples.


#### 2. Qt Backend

Integrates as a Qt widget for seamless integration with Qt-based DCCs. Requires `pip install auroraview[qt]`.

```python
from auroraview import QtWebView

# Create WebView as Qt widget
webview = QtWebView(
    parent=maya_main_window(),  # Any QWidget (optional)
    title="My Tool",
    width=800,
    height=600
)

# Load content
webview.load_url("http://localhost:3000")
# Or load HTML
webview.load_html("<html><body><h1>Hello from Qt!</h1></body></html>")

# Show the widget
webview.show()

# ✨ Event processing is automatic - no need to call process_events()!
# The Qt backend automatically handles all JavaScript execution and events
```

#### WebView2 Pre-warming (Automatic)

`QtWebView` automatically pre-warms WebView2 on first instantiation, reducing subsequent creation time by ~50%. No manual setup required:

```python
from auroraview.integration.qt import QtWebView

# First QtWebView automatically triggers pre-warming
webview = QtWebView(parent=maya_main_window())
webview.load_url("http://localhost:3000")
webview.show()
```

**For advanced users** who want explicit control over pre-warming timing:

```python
from auroraview.integration.qt import WebViewPool, QtWebView

# Explicit pre-warm at DCC startup (e.g., in userSetup.py)
WebViewPool.prewarm()

# Check pre-warm status
if WebViewPool.has_prewarmed():
    print(f"Pre-warm took {WebViewPool.get_prewarm_time():.2f}s")

# Disable auto-prewarm if using explicit control
webview = QtWebView(parent=maya_main_window(), auto_prewarm=False)

# Cleanup when done (optional, called automatically on exit)
WebViewPool.cleanup()
```

**Benefits:**
- ✅ Automatic pre-warming on first `QtWebView` creation
- ✅ Reduces WebView creation time by ~50%
- ✅ Thread-safe and idempotent (safe to call multiple times)
- ✅ Automatic cleanup on application exit

**When to use Qt backend:**
- [OK] Your DCC already has Qt loaded (Maya, Houdini, Nuke)
- [OK] You want seamless Qt widget integration
- [OK] You need to use Qt layouts and signals/slots
- [OK] You want automatic event processing (no manual `process_events()` calls)

**When to use Native backend:**
- [OK] Maximum compatibility across all platforms
- [OK] Standalone applications
- [OK] DCCs without Qt (Blender, 3ds Max)
- [OK] Minimal dependencies

### Bidirectional Communication

AuroraView provides a complete bidirectional communication system between Python and JavaScript.

#### Communication API Overview

| Direction | JavaScript API | Python API | Use Case |
|-----------|---------------|------------|----------|
| JS → Python | `auroraview.call(method, params)` | `@webview.bind_call` | RPC with return value |
| JS → Python | `auroraview.send_event(event, data)` | `@webview.on(event)` | Fire-and-forget events |
| Python → JS | - | `webview.emit(event, data)` | Push notifications |
| JS only | `auroraview.on(event, handler)` | - | Receive Python events |
| JS only | `auroraview.trigger(event, data)` | - | Local JS events (not sent to Python) |

> **Important**: `auroraview.trigger()` is for JavaScript-side local events only. To send events to Python, use `auroraview.send_event()`.

#### Python → JavaScript (Push Events)

```python
# Python side: emit events to JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})
webview.emit("selection_changed", {"items": ["mesh1", "mesh2"]})
```

```javascript
// JavaScript side: listen for Python events
window.auroraview.on('update_data', (data) => {
    console.log('Frame:', data.frame);
    console.log('Objects:', data.objects);
});

window.auroraview.on('selection_changed', (data) => {
    highlightItems(data.items);
});
```

#### JavaScript → Python (Events)

```javascript
// JavaScript side: send events to Python
window.auroraview.send_event('export_scene', {
    path: '/path/to/export.fbx',
    format: 'fbx'
});

window.auroraview.send_event('viewport_orbit', { dx: 10, dy: 5 });
```

```python
# Python side: register event handlers
@webview.on("export_scene")
def handle_export(data):
    print(f"Exporting to: {data['path']}")
    # Your DCC export logic here

@webview.on("viewport_orbit")
def handle_orbit(data):
    rotate_camera(data['dx'], data['dy'])
```

#### JavaScript → Python (RPC with Return Value)

For request-response patterns, use `auroraview.call()` with `bind_call`:

```javascript
// JavaScript side: call Python method and get result
const hierarchy = await auroraview.call('api.get_hierarchy', { root: 'scene' });
console.log('Scene hierarchy:', hierarchy);

const result = await auroraview.call('api.rename_object', {
    old_name: 'cube1',
    new_name: 'hero_cube'
});
if (result.ok) {
    console.log('Renamed successfully');
}
```

```python
# Python side: bind callable methods
@webview.bind_call("api.get_hierarchy")
def get_hierarchy(root=None):
    # Return value is sent back to JavaScript
    return {"children": ["group1", "mesh_cube"], "count": 2}

@webview.bind_call("api.rename_object")
def rename_object(old_name, new_name):
    # Perform rename in DCC
    cmds.rename(old_name, new_name)
    return {"ok": True, "old": old_name, "new": new_name}
```

#### Common Mistakes

```javascript
// WRONG: trigger() is JS-local only, won't reach Python
auroraview.trigger('my_event', data);  // Python won't receive this!

// WRONG: dispatchEvent is browser API, won't reach Python
window.dispatchEvent(new CustomEvent('my_event', {detail: data}));  // Python won't receive!

// CORRECT: use send_event() for fire-and-forget
auroraview.send_event('my_event', data);  // Python receives via @webview.on()

// CORRECT: use call() for request-response
const result = await auroraview.call('api.my_method', data);  // Python receives via @webview.bind_call()
```

### Advanced Features

#### Lifecycle Management

Automatically close WebView when parent DCC application closes:

```python
from auroraview import WebView

# Get parent window handle (HWND on Windows)
parent_hwnd = get_maya_main_window_hwnd()  # Your DCC-specific function

webview = WebView(
    title="My Tool",
    width=800,
    height=600,
    parent_hwnd=parent_hwnd,  # Monitor this parent window
    parent_mode="owner"  # Use owner mode for cross-thread safety
)

webview.show()
# WebView will automatically close when parent window is destroyed
```

#### Third-Party Website Integration

Inject JavaScript into third-party websites and establish bidirectional communication:

```python
from auroraview import WebView

webview = WebView(title="AI Chat", width=1200, height=800, dev_tools=True)

# Register event handlers
@webview.on("get_scene_info")
def handle_get_scene_info(data):
    # Get DCC scene data
    selection = maya.cmds.ls(selection=True)
    webview.emit("scene_info_response", {"selection": selection})

@webview.on("execute_code")
def handle_execute_code(data):
    # Execute AI-generated code in DCC
    code = data.get("code", "")
    exec(code)
    webview.emit("execution_result", {"status": "success"})

# Load third-party website
webview.load_url("https://ai-chat-website.com")

# Inject custom JavaScript
injection_script = """
(function() {
    // Add custom button to the page
    const btn = document.createElement('button');
    btn.textContent = 'Get DCC Selection';
    btn.onclick = () => {
        window.dispatchEvent(new CustomEvent('get_scene_info', {
            detail: { timestamp: Date.now() }
        }));
    };
    document.body.appendChild(btn);

    // Listen for responses
    window.addEventListener('scene_info_response', (e) => {
        console.log('DCC Selection:', e.detail);
    });
})();
"""

import time
time.sleep(1)  # Wait for page to load
webview.eval_js(injection_script)

webview.show()
```

For detailed guide, see [Third-Party Integration Guide](./docs/THIRD_PARTY_INTEGRATION.md).

## Documentation

-  [Architecture](./docs/ARCHITECTURE.md) - Modular backend architecture
-  [Technical Design](./docs/TECHNICAL_DESIGN.md) - Technical implementation details
-  [DCC Integration Guide](./docs/DCC_INTEGRATION_GUIDE.md) - Integration with DCC applications
-  [Third-Party Integration Guide](./docs/THIRD_PARTY_INTEGRATION.md) - JavaScript injection and AI chat integration
-  [Project Roadmap](./docs/ROADMAP.md) - Future plans and development

## DCC Software Support

| DCC Software | Status | Python Version | Example |
|--------------|--------|----------------|---------|
| Maya | [OK] Supported | 3.7+ | [Maya Outliner Example](https://github.com/loonghao/auroraview-maya-outliner) |
| 3ds Max | [OK] Supported | 3.7+ | - |
| Houdini | [OK] Supported | 3.7+ | - |
| Blender | [OK] Supported | 3.7+ | - |
| Photoshop | [CONSTRUCTION] Planned | 3.7+ | - |
| Unreal Engine | [CONSTRUCTION] Planned | 3.7+ | - |

> **📚 Examples**: For a complete working example, check out the [Maya Outliner Example](https://github.com/loonghao/auroraview-maya-outliner) - a modern, web-based Maya Outliner built with AuroraView, Vue 3, and TypeScript.

## Development

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

## Project Structure

```
auroraview/
├── src/                    # Rust core library
├── python/                 # Python bindings
├── tests/                  # Test suites
├── docs/                   # Documentation
└── benches/                # Performance benchmarks
```

## Testing

AuroraView provides a comprehensive testing framework with multiple backends for different testing scenarios.

### HeadlessWebView - Unified Testing Framework

AuroraView includes a unified headless testing framework that supports multiple backends:

```python
from auroraview.testing import HeadlessWebView

# Auto-detect best backend (Playwright recommended)
with HeadlessWebView.auto() as webview:
    webview.goto("https://example.com")
    webview.click("#button")
    assert webview.text("#result") == "Success"

# Or explicitly use Playwright backend
with HeadlessWebView.playwright() as webview:
    webview.load_html("<h1>Test</h1>")
    assert webview.text("h1") == "Test"
    webview.screenshot("test.png")
```

**Available Backends:**

| Backend | Method | Platform | Use Case |
|---------|--------|----------|----------|
| Playwright | `HeadlessWebView.playwright()` | All | Recommended for CI/CD |
| Xvfb | `HeadlessWebView.virtual_display()` | Linux | Real WebView testing |
| WebView2 CDP | `HeadlessWebView.webview2_cdp(url)` | Windows | Real WebView2 testing |

**Features:**
- Unified API across all backends
- Automatic backend selection with `HeadlessWebView.auto()`
- Full Playwright API access (locators, screenshots, network interception)
- Pytest fixtures included
- CI/CD ready

**Requirements:** `pip install playwright && playwright install chromium`

### Pytest Integration

```python
import pytest
from auroraview.testing import HeadlessWebView

# Using context manager
def test_basic_navigation():
    with HeadlessWebView.playwright() as webview:
        webview.goto("https://example.com")
        assert "Example" in webview.title()

# Using pytest fixture
def test_with_fixture(headless_webview):
    headless_webview.load_html("<button id='btn'>Click</button>")
    headless_webview.click("#btn")
```

### Running Tests

```bash
# Using nox (recommended)
uvx nox -s pytest          # Test without Qt
uvx nox -s pytest-qt       # Test with Qt
uvx nox -s pytest-all      # Run all tests

# Or using pytest directly
uv run pytest tests/python/ -v

# Run headless WebView tests
uv run pytest tests/python/integration/test_headless_webview.py -v
```

### CI/CD Configuration

```yaml
# GitHub Actions example
- name: Install dependencies
  run: |
    pip install playwright
    playwright install chromium

- name: Run tests
  run: pytest tests/ -v
```

### Available Nox Sessions

```bash
uvx nox -l                 # List all sessions
uvx nox -s pytest          # Test without Qt
uvx nox -s pytest-qt       # Test with Qt
uvx nox -s pytest-all      # Run all tests
uvx nox -s lint            # Run linting
uvx nox -s coverage        # Generate coverage report
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Acknowledgments

- [Wry](https://github.com/tauri-apps/wry) - Cross-platform WebView library
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [Tauri](https://tauri.app/) - Inspiration and ecosystem

## Contact

- Author: Hal Long
- Email: hal.long@outlook.com
- GitHub: [@loonghao](https://github.com/loonghao)

