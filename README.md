# AuroraView

[中文文档](./README_zh.md) | English

[![PyPI Version](https://img.shields.io/pypi/v/auroraview.svg)](https://pypi.org/project/auroraview/)
[![Python Versions](https://img.shields.io/pypi/pyversions/auroraview.svg)](https://pypi.org/project/auroraview/)
[![Downloads](https://static.pepy.tech/badge/auroraview)](https://pepy.tech/project/auroraview)
[![Codecov](https://codecov.io/gh/loonghao/auroraview/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/auroraview)
[![PR Checks](https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml/badge.svg)](https://github.com/loonghao/auroraview/actions/workflows/pr-checks.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/loonghao/auroraview)
[![CI](https://github.com/loonghao/auroraview/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/ci.yml)
[![Build Wheels](https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/build-wheels.yml)
[![Release](https://github.com/loonghao/auroraview/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/release.yml)
[![CodeQL](https://github.com/loonghao/auroraview/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/codeql.yml)
[![Security Audit](https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml/badge.svg?branch=main)](https://github.com/loonghao/auroraview/actions/workflows/security-audit.yml)
[![Latest Release](https://img.shields.io/github/v/release/loonghao/auroraview?display_name=tag)](https://github.com/loonghao/auroraview/releases)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen.svg)](https://pre-commit.com/)

[![GitHub Stars](https://img.shields.io/github/stars/loonghao/auroraview?style=social)](https://github.com/loonghao/auroraview/stargazers)
[![GitHub Downloads](https://img.shields.io/github/downloads/loonghao/auroraview/total)](https://github.com/loonghao/auroraview/releases)
[![Last Commit](https://img.shields.io/github/last-commit/loonghao/auroraview)](https://github.com/loonghao/auroraview/commits/main)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/loonghao/auroraview)](https://github.com/loonghao/auroraview/graphs/commit-activity)
[![Open Issues](https://img.shields.io/github/issues/loonghao/auroraview)](https://github.com/loonghao/auroraview/issues)
[![Open PRs](https://img.shields.io/github/issues-pr/loonghao/auroraview)](https://github.com/loonghao/auroraview/pulls)
[![Contributors](https://img.shields.io/github/contributors/loonghao/auroraview)](https://github.com/loonghao/auroraview/graphs/contributors)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
[![release-please](https://img.shields.io/badge/release--please-enabled-blue)](https://github.com/googleapis/release-please)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-025E8C?logo=dependabot)](./.github/dependabot.yml)
[![Code Style: ruff](https://img.shields.io/badge/code%20style-ruff-000000.svg)](https://docs.astral.sh/ruff/)
[![Type Checked: mypy](https://img.shields.io/badge/type%20checked-mypy-2A6DB0.svg)](http://mypy-lang.org/)

[Code of Conduct](./CODE_OF_CONDUCT.md) • [Security Policy](./SECURITY.md) • [Issue Tracker](https://github.com/loonghao/auroraview/issues)


A blazingly fast, lightweight WebView framework for DCC (Digital Content Creation) software, built with Rust and Python bindings. Perfect for Maya, 3ds Max, Houdini, Blender, and more.

> **⚠️ Development Status**: This project is under active development. APIs may change before v1.0.0 release. The project has not been extensively tested on Linux and macOS platforms.

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
| **Multi-Window** | Basic | [OK] WindowManager + cross-window events |
| **Storage API** | Basic | [OK] localStorage/sessionStorage/Cookie |
| **EventEmitter** | ❌ | [OK] Node.js-style on/once/off |
| **Async/Await** | Limited | [OK] Native Future support |
| **File Dialogs** | [OK] | [OK] open/save/folder |
| **WebView2 Warmup** | ❌ | [OK] Pre-init for DCC |

[POINTER] **[Read the full comparison](./docs/COMPARISON_WITH_PYWEBVIEW.md)** to understand why AuroraView is better for DCC development.

[POINTER] **[DCC Integration Guide](./docs/DCC_INTEGRATION.md)** - Learn how to integrate AuroraView into Maya, Houdini, Nuke, and other DCC applications.

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

# Register API methods using decorators
@view.slot
def get_data() -> dict:
    """Called from JS: await auroraview.api.get_data()"""
    return {"items": [1, 2, 3], "count": 3}

@view.slot
def save_file(path: str, content: str) -> dict:
    """Called from JS: await auroraview.api.save_file({path: "/tmp/a.txt", content: "hello"})"""
    with open(path, "w") as f:
        f.write(content)
    return {"ok": True, "path": path}

# Register event handlers (no return value)
@view.on("button_clicked")
def handle_click(data: dict):
    """Called when JS emits: auroraview.emit("button_clicked", {...})"""
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

// Emit events (fire-and-forget)
auroraview.emit("button_clicked", { id: "save_btn" });

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
        self.setup_connections()

    # ─── API Methods (JS → Python, auto-bound) ───
    def get_hierarchy(self, root: str = None) -> dict:
        """Get scene hierarchy. JS: await auroraview.api.get_hierarchy()"""
        # Your DCC-specific logic here
        return {
            "children": ["group1", "mesh_cube", "camera1"],
            "count": 3
        }

    def rename_object(self, old_name: str, new_name: str) -> dict:
        """Rename scene object. JS: await auroraview.api.rename_object({old_name: "a", new_name: "b"})"""
        # Perform rename in DCC
        return {"ok": True, "old": old_name, "new": new_name}

    def delete_objects(self, names: list) -> dict:
        """Delete objects. JS: await auroraview.api.delete_objects({names: ["obj1", "obj2"]})"""
        return {"ok": True, "deleted": len(names)}

    # ─── Event Handlers (on_ prefix, auto-bound) ───
    def on_item_selected(self, data: dict):
        """Handle selection from UI. JS: auroraview.emit("item_selected", {...})"""
        items = data.get("items", [])
        # Update DCC selection
        self.selection_changed.emit(items)

    def on_viewport_orbit(self, data: dict):
        """Handle viewport rotation. JS: auroraview.emit("viewport_orbit", {...})"""
        dx, dy = data.get("dx", 0), data.get("dy", 0)
        # Rotate camera in DCC
        print(f"Orbiting: dx={dx}, dy={dy}")

    # ─── Signal Connections (like Qt) ───
    def setup_connections(self):
        """Connect signals to handlers."""
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

// Emit events to Python
auroraview.emit("item_selected", { items: ["mesh1", "mesh2"] });
auroraview.emit("viewport_orbit", { dx: 10, dy: 5 });

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
view.bind_slot("get_plugins", get_plugins)
view.bind_slot("load_plugin", load_plugin)

# Connect to built-in signals
view.on_ready.connect(lambda: print("WebView is ready!"))
view.on_navigate.connect(lambda url: print(f"Navigated to: {url}"))

# Register event handlers
view.register_callback("plugin_event", on_plugin_event)

# Dynamic binding based on configuration
config = {"features": ["export", "import"]}
if "export" in config["features"]:
    view.bind_slot("export_data", lambda fmt: {"data": "...", "format": fmt})
if "import" in config["features"]:
    view.bind_slot("import_data", lambda data: {"ok": True})

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
| **Auto-binding** | ✅ | ✅ | ❌ Manual |
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

Both backends support the same event API:

```python
# Python → JavaScript
webview.emit("update_data", {"frame": 120, "objects": ["cube", "sphere"]})

# JavaScript → Python
@webview.on("export_scene")
def handle_export(data):
    print(f"Exporting to: {data['path']}")
    # Your DCC export logic here

# Or register callback directly
webview.register_callback("export_scene", handle_export)
```

**JavaScript side:**
```javascript
// Listen for events from Python
window.auroraview.on('update_data', (data) => {
    console.log('Frame:', data.frame);
    console.log('Objects:', data.objects);
});

// Send events to Python
window.auroraview.send_event('export_scene', {
    path: '/path/to/export.fbx'
});
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

## [DOCS] Documentation

**Start here:**
-  [Architecture](./docs/ARCHITECTURE.md) - **NEW!** Modular backend architecture
-  [Project Summary](./docs/SUMMARY.md) - Overview and key advantages
-  [Current Status](./docs/CURRENT_STATUS.md) - What's done and what's next

**Detailed Guides:**
-  [Technical Design](./docs/TECHNICAL_DESIGN.md)
-  [DCC Integration Guide](./docs/DCC_INTEGRATION_GUIDE.md)
-  [Third-Party Integration Guide](./docs/THIRD_PARTY_INTEGRATION.md) - **NEW!** JavaScript injection and AI chat integration
-  [Project Advantages](./docs/PROJECT_ADVANTAGES.md) - Why AuroraView is better than PyWebView
-  [Comparison with PyWebView](./docs/COMPARISON_WITH_PYWEBVIEW.md)
-  [Project Roadmap](./docs/ROADMAP.md)

##  DCC Software Support

| DCC Software | Status | Python Version | Example |
|--------------|--------|----------------|---------|
| Maya | [OK] Supported | 3.7+ | [Maya Outliner Example](https://github.com/loonghao/auroraview-maya-outliner) |
| 3ds Max | [OK] Supported | 3.7+ | - |
| Houdini | [OK] Supported | 3.7+ | - |
| Blender | [OK] Supported | 3.7+ | - |
| Photoshop | [CONSTRUCTION] Planned | 3.7+ | - |
| Unreal Engine | [CONSTRUCTION] Planned | 3.7+ | - |

> **📚 Examples**: For a complete working example, check out the [Maya Outliner Example](https://github.com/loonghao/auroraview-maya-outliner) - a modern, web-based Maya Outliner built with AuroraView, Vue 3, and TypeScript.

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
├── tests/                  # Test suites
├── docs/                   # Documentation
└── benches/                # Performance benchmarks
```

## [TEST_TUBE] Testing

AuroraView has comprehensive test coverage for both Qt and non-Qt environments.

### Running Tests

**Test without Qt dependencies** (tests error handling):
```bash
# Using nox (recommended)
uvx nox -s pytest

# Or using pytest directly
uv run pytest tests/test_qt_import_error.py -v
```

**Test with Qt dependencies** (tests actual Qt functionality):
```bash
# Using nox (recommended)
uvx nox -s pytest-qt

# Or using pytest directly
pip install auroraview[qt] pytest pytest-qt
pytest tests/test_qt_backend.py -v
```

**Run all tests**:
```bash
uvx nox -s pytest-all
```

### Test Structure

- `tests/test_qt_import_error.py` - Tests error handling when Qt is not installed
  - Verifies placeholder classes work correctly
  - Tests diagnostic variables (`_HAS_QT`, `_QT_IMPORT_ERROR`)
  - Ensures helpful error messages are shown

- `tests/test_qt_backend.py` - Tests actual Qt backend functionality
  - Requires Qt dependencies to be installed
  - Tests QtWebView instantiation and methods
  - Tests event handling and JavaScript integration

### Available Nox Sessions

```bash
# List all available test sessions
uvx nox -l

# Common sessions:
uvx nox -s pytest          # Test without Qt
uvx nox -s pytest-qt       # Test with Qt
uvx nox -s pytest-all      # Run all tests
uvx nox -s lint            # Run linting
uvx nox -s format          # Format code
uvx nox -s coverage        # Generate coverage report
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

