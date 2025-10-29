"""AuroraView - Rust-powered WebView for Python & DCC embedding.

This package provides a modern web-based UI solution for professional DCC applications
like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine.

## Backends

AuroraView supports two integration modes:

1. **Native Backend** (default): Uses platform-specific APIs (HWND on Windows)
   - Best for standalone applications
   - Works in any Python environment
   - No additional dependencies

2. **Qt Backend**: Integrates with Qt framework
   - Best for Qt-based DCC applications (Maya, Houdini, Nuke)
   - Requires Qt bindings (install with: `pip install auroraview[qt]`)
   - Seamless integration with existing Qt widgets

## Examples

Basic usage (Native backend)::

    from auroraview import WebView

    # Create a WebView instance
    webview = WebView(
        title="My App",
        width=800,
        height=600,
        url="http://localhost:3000"
    )

    # Show the window
    webview.show()

DCC integration (Native backend)::

    from auroraview import NativeWebView
    import maya.OpenMayaUI as omui

    # Get Maya main window handle
    maya_hwnd = int(omui.MQtUtil.mainWindow())

    # Create embedded WebView
    webview = NativeWebView(
        title="Maya Tool",
        parent_hwnd=maya_hwnd,
        parent_mode="owner"  # Safer for cross-thread usage
    )
    webview.show_async()

Qt integration::

    from auroraview import QtWebView

    # Create WebView as Qt widget
    webview = QtWebView(
        parent=maya_main_window(),
        title="My Tool",
        width=800,
        height=600
    )
    webview.show()

Bidirectional communication::

    # Python → JavaScript
    webview.emit("update_data", {"frame": 120})

    # JavaScript → Python
    @webview.on("export_scene")
    def handle_export(data):
        print(f"Exporting to: {data['path']}")
"""

try:
    from ._core import __author__, __version__
except ImportError:
    # Fallback for development without compiled extension
    __version__ = "0.1.0"
    __author__ = "Hal Long <hal.long@outlook.com>"

from .decorators import on_event
from .native import AuroraView, NativeWebView
from .webview import WebView

# Qt backend is optional
try:
    from .qt_integration import AuroraViewQt, QtWebView

    __all__ = [
        # Base class
        "WebView",
        # Native backend
        "NativeWebView",
        "AuroraView",  # Backward compatibility
        # Qt backend
        "QtWebView",
        "AuroraViewQt",  # Backward compatibility
        # Utilities
        "on_event",
        # Metadata
        "__version__",
        "__author__",
    ]
except ImportError:
    # Qt backend not available
    __all__ = [
        # Base class
        "WebView",
        # Native backend
        "NativeWebView",
        "AuroraView",  # Backward compatibility
        # Utilities
        "on_event",
        # Metadata
        "__version__",
        "__author__",
    ]
