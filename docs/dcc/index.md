# DCC Integration

AuroraView is designed specifically for integration with Digital Content Creation (DCC) software.

## Supported Software

| DCC Software | Status | Python Version | Integration Mode |
|--------------|--------|----------------|------------------|
| Maya | ✅ Supported | 3.7+ | Qt Mode |
| Houdini | ✅ Supported | 3.7+ | Qt Mode |
| 3ds Max | ✅ Supported | 3.7+ | Qt Mode |
| Blender | ✅ Supported | 3.7+ | Desktop / Native Mode |
| Nuke | ✅ Supported | 3.7+ | Qt Mode |
| Photoshop | 🚧 Planned | 3.7+ | - |
| Unreal Engine | 🚧 Planned | 3.7+ | Native Mode (HWND) |

## Integration Modes

AuroraView provides three integration modes for different scenarios:

| Mode | Class | Description | Best For |
|------|-------|-------------|----------|
| **Desktop** | `WebView` + `show()` | Independent window with own event loop | Standalone tools, desktop apps |
| **Native (HWND)** | `WebView` + `parent=hwnd` | Embedded via HWND, no Qt dependency | Blender, Unreal Engine, non-Qt apps |
| **Qt** | `QtWebView` | Embedded as Qt widget child | Maya, Houdini, Nuke, 3ds Max |

### Desktop Mode

**Best for:** Standalone tools, development, Blender (floating windows)

Creates an independent window with its own event loop.

```python
from auroraview import run_desktop

run_desktop(
    title="My Tool",
    url="http://localhost:3000"
)
```

**Key features:**
- ✅ Full window effects support (click-through, blur, mica)
- ✅ No DCC dependency
- ✅ Owns event loop

### Native Mode (HWND)

**Best for:** Blender, Unreal Engine, non-Qt applications

Embeds WebView via HWND without Qt dependency.

```python
from auroraview import WebView

# Get parent HWND from non-Qt app
parent_hwnd = get_app_window_handle()

webview = WebView.create(
    title="My Tool",
    parent=parent_hwnd,
    mode="owner",
)
webview.show()

# Get HWND for external integration
hwnd = webview.get_hwnd()
```

**Key features:**
- ✅ Direct HWND access
- ✅ Full window effects support
- ✅ Works with any HWND-accepting application
- ✅ No Qt dependency required

### Qt Mode

**Best for:** Maya, Houdini, Nuke, 3ds Max

Creates a true Qt widget that can be docked and managed by Qt's parent-child system.

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=dcc_main_window(),
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

**Key features:**
- ✅ Works with `QDockWidget` for dockable panels
- ✅ Automatic lifecycle management
- ✅ Native Qt event integration
- ✅ Supports all Qt layout managers
- ⚠️ Limited window effects support

## Installation

### Basic Installation

```bash
pip install auroraview
```

### With Qt Support

For Qt-based DCCs (Maya, Houdini, Nuke):

```bash
pip install auroraview[qt]
```

This installs QtPy as middleware to handle different Qt versions.

## Common Patterns

### Getting Main Window

Each DCC has its own way to get the main window:

::: code-group

```python [Maya]
import maya.OpenMayaUI as omui
from qtpy import QtWidgets
import shiboken2

def maya_main_window():
    ptr = omui.MQtUtil.mainWindow()
    return shiboken2.wrapInstance(int(ptr), QtWidgets.QWidget)
```

```python [Houdini]
import hou

def houdini_main_window():
    return hou.qt.mainWindow()
```

```python [Nuke]
from qtpy import QtWidgets

def nuke_main_window():
    return QtWidgets.QApplication.activeWindow()
```

:::

### Dockable Panel

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QDockWidget

# Create dock widget
dock = QDockWidget("My Tool", main_window)

# Create WebView
webview = QtWebView(parent=dock)
webview.load_url("http://localhost:3000")

# Set as dock widget content
dock.setWidget(webview)
main_window.addDockWidget(Qt.RightDockWidgetArea, dock)

webview.show()
```

## Lifecycle Management

AuroraView automatically handles cleanup when the parent DCC closes:

```python
webview = QtWebView(
    parent=dcc_main_window(),  # Monitor this parent
    url="http://localhost:3000"
)
# WebView closes automatically when parent is destroyed
```

For manual control:

```python
webview = AuroraView(
    url="http://localhost:3000",
    parent_hwnd=get_dcc_hwnd(),
    parent_mode="owner"
)
# WebView follows parent minimize/restore/close
```
