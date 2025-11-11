# DCC Integration Guide

This guide explains how to integrate AuroraView into DCC (Digital Content Creation) applications like Maya, Houdini, Nuke, and 3ds Max.

## Requirements

For Qt-based DCC applications, you need to install QtPy as a middleware layer:

```bash
pip install auroraview[qt]
```

QtPy provides compatibility across different Qt bindings (PySide2, PySide6, PyQt5, PyQt6) used by different DCC applications.

## Why QtPy?

Different DCC applications use different Qt versions:
- **Maya 2022-2024**: PySide2 (Qt 5.15)
- **Maya 2025+**: PySide6 (Qt 6.x)
- **Houdini 19.5+**: PySide2 or PySide6
- **Nuke 13+**: PySide2
- **3ds Max 2023+**: PySide6

QtPy automatically detects and uses the correct Qt binding available in your DCC environment.

## Integration Pattern

### 1. Basic Setup

```python
from auroraview import WebView
from qtpy.QtCore import QTimer  # QtPy handles version compatibility

# Get DCC main window HWND
import hou  # or maya.OpenMayaUI, nuke, etc.
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
webview.load_html("<h1>Hello from DCC!</h1>")

# Setup Qt timer to process messages (REQUIRED!)
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)  # 60 FPS

# Keep references to prevent garbage collection
_webview = webview
_timer = timer
```

### 2. Maya Example

```python
from auroraview import WebView
from qtpy.QtCore import QTimer
import maya.OpenMayaUI as omui

# Get Maya main window
try:
    from shiboken2 import wrapInstance
except ImportError:
    from shiboken6 import wrapInstance

maya_main_window_ptr = omui.MQtUtil.mainWindow()
maya_main_window = wrapInstance(int(maya_main_window_ptr), QtWidgets.QWidget)
hwnd = int(maya_main_window.winId())

# Create WebView
webview = WebView.for_dcc(
    parent_hwnd=hwnd,
    title="Maya Tool",
    width=800,
    height=600
)

webview.load_url("http://localhost:3000")

# Setup timer
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)

# Store references
_maya_webview = webview
_maya_timer = timer
```

### 3. Houdini Example

```python
from auroraview import WebView
from qtpy.QtCore import QTimer
import hou

# Get Houdini main window
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create WebView
webview = WebView.for_dcc(
    parent_hwnd=hwnd,
    title="Houdini Tool",
    width=650,
    height=500
)

webview.load_html("<h1>Hello from Houdini!</h1>")

# Setup timer
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)

# Store references
_houdini_webview = webview
_houdini_timer = timer
```

### 4. Nuke Example

```python
from auroraview import WebView
from qtpy.QtCore import QTimer
import nuke

# Get Nuke main window
nuke_main_window = nuke.activeViewer().node()
hwnd = int(nuke_main_window.winId())

# Create WebView
webview = WebView.for_dcc(
    parent_hwnd=hwnd,
    title="Nuke Tool",
    width=800,
    height=600
)

webview.load_url("http://localhost:3000")

# Setup timer
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)

# Store references
_nuke_webview = webview
_nuke_timer = timer
```

## Important Notes

1. **QtPy Installation**: Always install `auroraview[qt]` to get QtPy
2. **Timer Required**: The Qt timer is REQUIRED to process WebView messages
3. **Keep References**: Store webview and timer in module-level variables to prevent garbage collection
4. **Non-Blocking**: DCC UI remains fully responsive while WebView is open
5. **Independent Window**: WebView creates an independent window (not embedded)

## Troubleshooting

### QtPy Import Error

```
ImportError: No module named 'qtpy'
```

**Solution**: Install QtPy
```bash
pip install auroraview[qt]
```

### Window Disappears Immediately

**Cause**: WebView or timer object was garbage collected

**Solution**: Store references in module-level variables:
```python
_webview = webview
_timer = timer
```

### DCC UI Freezes

**Cause**: Forgot to setup Qt timer

**Solution**: Always setup the timer:
```python
timer = QTimer()
timer.timeout.connect(webview.process_messages)
timer.start(16)
```

## See Also

- [Architecture Documentation](./ARCHITECTURE.md)
- [Houdini Example](../examples/houdini_examples/dcc_integration.py)

