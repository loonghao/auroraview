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

# The Qt-backed re-exports below require ``qtpy`` and a real Qt binding
# (PySide2/PySide6/PyQt5/PyQt6). On environments without Qt available
# (e.g. Python 3.7 CI runs that intentionally skip Qt tests), we still
# want pure-Python submodules of this package — such as ``_locks`` — to
# be importable so they can be unit-tested in isolation.
#
# We therefore wrap the Qt-dependent imports in a guard: if Qt bindings
# are missing, we leave the public names undefined here. Code that tries
# to use ``QtWebView`` / etc. will then receive a clear ImportError on
# direct ``from auroraview.integration.qt._core import QtWebView``, which
# is the canonical failure path documented in ``_core.py``.

__all__ = []

try:
    from auroraview.integration.qt._core import QtWebView
    from auroraview.integration.qt.dialogs import FileDialog, create_file_dialog
    from auroraview.integration.qt.embedding import EmbeddingMixin
    from auroraview.integration.qt.event_processor import QtEventProcessor
    from auroraview.integration.qt.lifecycle import LifecycleMixin
    from auroraview.integration.qt.pool import WebViewPool
    from auroraview.integration.qt.signals import QtWebViewSignals
except ImportError:
    # Qt bindings unavailable; pure-Python submodules (e.g. ``_locks``)
    # remain importable via their fully-qualified module paths.
    pass
else:
    __all__ = [
        "QtWebView",
        "QtEventProcessor",
        "QtWebViewSignals",
        "FileDialog",
        "create_file_dialog",
        "WebViewPool",
        # Mixins (for advanced customization)
        "EmbeddingMixin",
        "LifecycleMixin",
    ]
