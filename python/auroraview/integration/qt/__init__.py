"""Qt integration module for AuroraView.

This module provides Qt-native WebView integration for DCC applications
like Maya, Houdini, Nuke, and 3ds Max. It includes:

- QtWebView: Qt widget embedding the core AuroraView WebView
- QtEventProcessor: Event processor for Qt/WebView integration
- QtWebViewSignals: Qt signals for WebView events

**Requirements**:
    Install with Qt support: `pip install auroraview[qt]`

Example:
    >>> from auroraview.integration.qt import QtWebView
    >>>
    >>> # Create WebView as Qt widget
    >>> webview = QtWebView(
    ...     parent=maya_main_window(),
    ...     title="My Tool",
    ...     width=800,
    ...     height=600
    ... )
    >>>
    >>> # Connect to Qt signals
    >>> webview.urlChanged.connect(lambda url: print(f"URL: {url}"))
    >>> webview.loadFinished.connect(lambda ok: print(f"Loaded: {ok}"))
    >>>
    >>> # Load content
    >>> webview.load_url("https://example.com")
    >>> webview.show()
"""

from auroraview.integration.qt._core import QtEventProcessor, QtWebView
from auroraview.integration.qt.signals import QtWebViewSignals

__all__ = [
    "QtWebView",
    "QtEventProcessor",
    "QtWebViewSignals",
]

