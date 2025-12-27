# Getting Started

AuroraView is a lightweight WebView framework for DCC (Digital Content Creation) software, built with Rust and Python bindings.

## Prerequisites

- Python 3.7+
- Windows, macOS, or Linux

## Installation

### Windows and macOS

**Basic installation** (Native backend only):

```bash
pip install auroraview
```

**With Qt support** (for Qt-based DCCs like Maya, Houdini, Nuke):

```bash
pip install auroraview[qt]
```

### Linux

Linux requires webkit2gtk system dependencies:

```bash
# Install system dependencies first
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev  # Debian/Ubuntu
# sudo dnf install gtk3-devel webkit2gtk3-devel      # Fedora/CentOS
# sudo pacman -S webkit2gtk                          # Arch Linux

# Install from PyPI
pip install auroraview
```

## Quick Start

### Desktop Application

The simplest way to create a desktop application:

```python
from auroraview import run_desktop

run_desktop(
    title="My App",
    url="http://localhost:3000",
    width=1024,
    height=768
)
```

### Load HTML Content

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

### DCC Integration

For Qt-based DCC applications (Maya, Houdini, Nuke):

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=dcc_main_window(),  # Your DCC's main window
    url="http://localhost:3000",
    width=800,
    height=600
)
webview.show()
```

## Integration Modes

AuroraView provides three integration modes:

| Mode | Class | Description | Best For |
|------|-------|-------------|----------|
| **Desktop** | `WebView` + `show()` | Independent window with own event loop | Standalone tools, desktop apps |
| **Native (HWND)** | `WebView` + `parent=hwnd` | Embedded via HWND, no Qt dependency | Blender, Unreal Engine, non-Qt apps |
| **Qt** | `QtWebView` | Embedded as Qt widget child | Maya, Houdini, Nuke, 3ds Max |

## Next Steps

- [Installation Guide](/guide/installation) - Detailed installation instructions
- [Architecture](/guide/architecture) - Understanding AuroraView's architecture
- [WebView Basics](/guide/webview-basics) - Core WebView concepts
- [DCC Integration](/dcc/) - Integrate with specific DCC software
