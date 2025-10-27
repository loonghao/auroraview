"""AuroraView - Rust-powered WebView for Python & DCC embedding.

This package provides a modern web-based UI solution for professional DCC applications
like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine.

Example:
    Basic usage::

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

    Bidirectional communication::

        # Python → JavaScript
        webview.emit("update_data", {"frame": 120})

        # JavaScript → Python
        @webview.on("export_scene")
        def handle_export(data):
            print(f"Exporting to: {data['path']}")
"""

from typing import Any, Callable, Dict, Optional

try:
    from ._core import WebView as _CoreWebView
    from ._core import __author__, __version__
except ImportError:
    # Fallback for development without compiled extension
    _CoreWebView = None
    __version__ = "0.1.0"
    __author__ = "Hal Long <hal.long@outlook.com>"

from .decorators import on_event
from .webview import WebView

__all__ = [
    "WebView",
    "on_event",
    "__version__",
    "__author__",
]
