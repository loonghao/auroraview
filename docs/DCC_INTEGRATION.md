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
import hou  # or maya.OpenMayaUI, nuke, etc.

# Get DCC main window HWND
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create embedded WebView (auto-timer; no manual Qt timer needed)
webview = WebView.create(
    title="My Tool",
    width=650,
    height=500,
    parent=hwnd,
    mode="owner",
)
webview.load_html("<h1>Hello from DCC!</h1>")
webview.show()
```

### 2. Maya Example

```python
from auroraview import WebView
import maya.OpenMayaUI as omui

# Get Maya main window HWND
hwnd = int(omui.MQtUtil.mainWindow())

# Create embedded WebView (auto-timer)
webview = WebView.create(
    title="Maya Tool",
    width=800,
    height=600,
    parent=hwnd,
    mode="owner",
)
webview.load_url("http://localhost:3000")
webview.show()
```

### 3. Houdini Example

```python
from auroraview import WebView
import hou

# Get Houdini main window
main_window = hou.qt.mainWindow()
hwnd = int(main_window.winId())

# Create embedded WebView (auto-timer)
webview = WebView.create(
    title="Houdini Tool",
    width=650,
    height=500,
    parent=hwnd,
    mode="owner",
)
webview.load_html("<h1>Hello from Houdini!</h1>")
webview.show()
```

### 4. Nuke Example

```python
from auroraview import WebView
from qtpy import QtWidgets

# Get Nuke main window
main = QtWidgets.QApplication.activeWindow()
hwnd = int(main.winId())

# Create embedded WebView (auto-timer)
webview = WebView.create(
    title="Nuke Tool",
    width=800,
    height=600,
    parent=hwnd,
    mode="owner",
)
webview.load_url("http://localhost:3000")
webview.show()
```

## Important Notes

1. **QtPy**: Recommended for a unified Qt API in DCCs, but not strictly required
2. **Auto Timer**: Embedded mode auto-starts an event timer; no manual `process_messages` is needed
3. **Keep References**: Keep `webview` alive if your script ends immediately to avoid GC
4. **Responsive UI**: DCC UI remains responsive; the window is embedded under the DCC main window

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

