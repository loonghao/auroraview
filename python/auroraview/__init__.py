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

Basic usage (recommended)::

    from auroraview import WebView

    # Create and show a WebView (2 lines!)
    webview = WebView.create("My App", url="http://localhost:3000")
    webview.show()  # Auto-blocks until closed

DCC integration - Maya::

    from auroraview import WebView
    import maya.OpenMayaUI as omui

    # Create embedded WebView under Maya main window
    maya_hwnd = int(omui.MQtUtil.mainWindow())
    webview = WebView.create("Maya Tool", url="http://localhost:3000", parent=maya_hwnd, mode="owner")
    webview.show()  # Embedded mode: non-blocking

DCC integration - Houdini::

    from auroraview import WebView
    import hou

    # Create embedded WebView under Houdini main window
    hwnd = int(hou.qt.mainWindow().winId())
    webview = WebView.create("Houdini Tool", url="http://localhost:3000", parent=hwnd, mode="owner")
    webview.show()  # Embedded mode: non-blocking

DCC integration - Blender::

    from auroraview import WebView

    # Standalone window (no parent window in Blender)
    webview = WebView.create("Blender Tool", url="http://localhost:3000")
    webview.show()  # Blocks until closed (use show(wait=False) for async)

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
    from ._core import (
        # Window utilities
        WindowInfo,
        __author__,
        __version__,
        close_window_by_hwnd,
        destroy_window_by_hwnd,
        find_window_by_exact_title,
        find_windows_by_title,
        get_all_windows,
        get_foreground_window,
    )
except ImportError:
    # Fallback for development without compiled extension
    __version__ = "0.1.0"
    __author__ = "Hal Long <hal.long@outlook.com>"

    # Placeholder for window utilities
    WindowInfo = None  # type: ignore
    get_foreground_window = None  # type: ignore
    find_windows_by_title = None  # type: ignore
    find_window_by_exact_title = None  # type: ignore
    get_all_windows = None  # type: ignore
    close_window_by_hwnd = None  # type: ignore
    destroy_window_by_hwnd = None  # type: ignore

from .event_timer import EventTimer
from .webview import WebView

# Bridge for DCC integration (optional - requires websockets)
_BRIDGE_IMPORT_ERROR = None
try:
    from .bridge import Bridge
except ImportError as e:
    _BRIDGE_IMPORT_ERROR = str(e)

    # Create placeholder class that raises helpful error
    class Bridge:  # type: ignore
        """Bridge placeholder - websockets not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "Bridge requires websockets library. "
                "Install with: pip install websockets\n"
                f"Original error: {_BRIDGE_IMPORT_ERROR}"
            )


# Service Discovery (optional - requires Rust core)
_SERVICE_DISCOVERY_IMPORT_ERROR = None
try:
    from ._core import ServiceDiscovery, ServiceInfo
except ImportError as e:
    _SERVICE_DISCOVERY_IMPORT_ERROR = str(e)

    # Create placeholder classes
    class ServiceDiscovery:  # type: ignore
        """ServiceDiscovery placeholder - Rust core not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "ServiceDiscovery requires Rust core module. "
                "Rebuild the package with: pip install -e .\n"
                f"Original error: {_SERVICE_DISCOVERY_IMPORT_ERROR}"
            )

    class ServiceInfo:  # type: ignore
        """ServiceInfo placeholder - Rust core not available."""

        pass


# Qt backend is optional
_QT_IMPORT_ERROR = None
try:
    from .qt_integration import QtWebView
except ImportError as e:
    _QT_IMPORT_ERROR = str(e)

    # Create placeholder class that raises helpful error
    class QtWebView:  # type: ignore
        """Qt backend placeholder - not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "Qt backend is not available. "
                "Install with: pip install auroraview[qt]\n"
                f"Original error: {_QT_IMPORT_ERROR}"
            )


# Public flags for test/diagnostics
_HAS_QT = _QT_IMPORT_ERROR is None

# Backward-compatibility alias
AuroraViewQt = QtWebView

# Simple top-level event decorator (for tests/backward-compat)
_EVENT_HANDLERS = {}


def on_event(event_name: str):
    """Top-level event decorator used in basic examples/tests.

    Note: This is a lightweight registry; core event routing is per-WebView via
    webview.on(). This helper exists for compatibility with older code/tests.
    """

    def decorator(func):
        _EVENT_HANDLERS.setdefault(event_name, []).append(func)
        return func

    return decorator


__all__ = [
    # Base classes
    "WebView",
    # Qt backend (may raise ImportError if not installed)
    "QtWebView",
    "AuroraViewQt",
    # Bridge for DCC integration (may raise ImportError if websockets not installed)
    "Bridge",
    # Service Discovery (may raise ImportError if Rust core not available)
    "ServiceDiscovery",
    "ServiceInfo",
    # Utilities
    "EventTimer",
    "on_event",
    # Window utilities
    "WindowInfo",
    "get_foreground_window",
    "find_windows_by_title",
    "find_window_by_exact_title",
    "get_all_windows",
    "close_window_by_hwnd",
    "destroy_window_by_hwnd",
    # Metadata
    "__version__",
    "__author__",
]
