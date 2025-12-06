"""Qt integration module for AuroraView.

This module provides Qt-native WebView integration for DCC applications
like Maya, Houdini, Nuke, and 3ds Max. It includes:

- QtWebView: Qt widget embedding the core AuroraView WebView
- QtEventProcessor: Event processor for Qt/WebView integration
- QtWebViewSignals: Qt signals for WebView events
- FileDialog: File dialog type enum (OPEN, SAVE, FOLDER)
- WebViewPool: Pre-warming pool for faster WebView initialization

**Requirements**:
    Install with Qt support: `pip install auroraview[qt]`

Example:
    >>> from auroraview.integration.qt import QtWebView, FileDialog, WebViewPool
    >>>
    >>> # Pre-warm during DCC startup (optional but recommended)
    >>> WebViewPool.prewarm()
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
    >>>
    >>> # Use file dialogs
    >>> files = webview.create_file_dialog(
    ...     FileDialog.OPEN,
    ...     allow_multiple=True,
    ...     file_types=('Images (*.png *.jpg)', 'All files (*.*)')
    ... )
"""

from auroraview.integration.qt._core import QtEventProcessor, QtWebView
from auroraview.integration.qt.dialogs import FileDialog, create_file_dialog
from auroraview.integration.qt.pool import WebViewPool
from auroraview.integration.qt.signals import QtWebViewSignals

__all__ = [
    "QtWebView",
    "QtEventProcessor",
    "QtWebViewSignals",
    "FileDialog",
    "create_file_dialog",
    "WebViewPool",
]
