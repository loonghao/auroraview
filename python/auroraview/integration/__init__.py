# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AuroraView Integration Module.

This module contains DCC and Qt integration functionality:
- AuroraView: High-level framework class for DCC integration
- Bridge: WebSocket bridge for DCC communication
- BrowserExtensionBridge: Bridge for Chrome/Firefox extension integration
- Qt: Qt/PySide integration utilities

Example:
    >>> from auroraview.integration import AuroraView, Bridge
    >>> app = AuroraView(title="DCC Tool", url="http://localhost:3000")
    >>> app.show()

    >>> # Browser extension integration
    >>> from auroraview.integration import BrowserExtensionBridge
    >>> bridge = BrowserExtensionBridge(ws_port=9001, http_port=9002)
    >>> @bridge.on("get_data")
    ... async def handle(data, client):
    ...     return {"result": "ok"}
    >>> bridge.start_background()
"""

from __future__ import annotations

from .framework import AuroraView

# Bridge for DCC integration (optional - requires websockets)
_BRIDGE_IMPORT_ERROR = None
try:
    from .bridge import Bridge
except ImportError as e:
    _BRIDGE_IMPORT_ERROR = str(e)

    class Bridge:  # type: ignore
        """Bridge placeholder - websockets not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "Bridge requires websockets library. "
                "Install with: pip install websockets\n"
                f"Original error: {_BRIDGE_IMPORT_ERROR}"
            )


# Browser Extension Bridge (optional - requires websockets)
_BROWSER_BRIDGE_IMPORT_ERROR = None
try:
    from .browser_extension import BrowserExtensionBridge, create_browser_bridge
except ImportError as e:
    _BROWSER_BRIDGE_IMPORT_ERROR = str(e)

    class BrowserExtensionBridge:  # type: ignore
        """BrowserExtensionBridge placeholder - websockets not available."""

        def __init__(self, *_args, **_kwargs):
            raise ImportError(
                "BrowserExtensionBridge requires websockets library. "
                "Install with: pip install websockets\n"
                f"Original error: {_BROWSER_BRIDGE_IMPORT_ERROR}"
            )

    def create_browser_bridge(*_args, **_kwargs):
        raise ImportError(
            "create_browser_bridge requires websockets library. "
            "Install with: pip install websockets"
        )


# Qt backend placeholder class (for testing and when Qt is not available)
class _QtWebViewPlaceholder:
    """Qt backend placeholder - not available."""

    def __init__(self, *_args, **_kwargs):
        raise ImportError(
            "Qt backend is not available. "
            "Install with: pip install auroraview[qt]\n"
            "Original error: Qt/PySide not installed"
        )


# Qt backend is optional
_QT_IMPORT_ERROR = None
try:
    from .qt import QtWebView
except ImportError as e:
    _QT_IMPORT_ERROR = str(e)
    QtWebView = _QtWebViewPlaceholder  # type: ignore


# Import submodules for attribute access
from . import bridge as bridge
from . import framework as framework

# Browser extension module (optional)
try:
    from . import browser_extension as browser_extension
except ImportError:
    browser_extension = None  # type: ignore

# Qt module is optional
try:
    from . import qt as qt
except ImportError:
    qt = None  # type: ignore

__all__ = [
    # Framework
    "AuroraView",
    # Bridge
    "Bridge",
    # Browser Extension Bridge
    "BrowserExtensionBridge",
    "create_browser_bridge",
    # Qt Integration (optional)
    "QtWebView",
    # Qt placeholder (for testing)
    "_QtWebViewPlaceholder",
    # Submodules
    "bridge",
    "browser_extension",
    "framework",
    "qt",
]
