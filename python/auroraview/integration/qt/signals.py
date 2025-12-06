"""Qt signal support for AuroraView QtWebView.

This module provides Qt signal/slot integration for QtWebView, enabling
standard Qt-style event handling with signals like:

Navigation signals:
- urlChanged(str) - Emitted when the URL changes
- loadFinished(bool) - Emitted when page loading completes
- titleChanged(str) - Emitted when the page title changes
- loadProgress(int) - Emitted during page load with progress (0-100)
- loadStarted() - Emitted when page loading starts

Error handling signals:
- jsError(str, int, str) - Emitted on JavaScript errors (message, line, source)
- consoleMessage(int, str, int, str) - Console messages (level, message, line, source)
- renderProcessTerminated(int, int) - Render process crash (status, exitCode)

IPC signals:
- ipcMessageReceived(str, object) - IPC message from JavaScript (event, data)

Selection signals:
- selectionChanged() - Emitted when text selection changes

Example:
    >>> from auroraview import QtWebView
    >>>
    >>> webview = QtWebView(parent=parent_widget)
    >>>
    >>> # Connect to Qt signals
    >>> webview.urlChanged.connect(lambda url: print(f"URL: {url}"))
    >>> webview.loadFinished.connect(lambda ok: print(f"Loaded: {ok}"))
    >>> webview.titleChanged.connect(lambda title: print(f"Title: {title}"))
    >>> webview.loadProgress.connect(lambda p: print(f"Progress: {p}%"))
    >>>
    >>> # Connect to IPC messages via Qt signals
    >>> webview.ipcMessageReceived.connect(lambda event, data: print(f"{event}: {data}"))
    >>>
    >>> # Connect to error signals
    >>> webview.jsError.connect(lambda msg, line, src: print(f"JS Error: {msg}"))
    >>>
    >>> webview.load_url("https://example.com")
    >>> webview.show()
"""

import logging
import os
from typing import Optional

# Performance optimization: Check verbose logging once at import time
# In DCC environments, excessive logging causes severe UI performance issues
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)

try:
    from qtpy.QtCore import QObject, Signal
except ImportError as e:
    raise ImportError("Qt signals require qtpy. Install with: pip install auroraview[qt]") from e

logger = logging.getLogger(__name__)


class QtWebViewSignals(QObject):
    """Qt signals for WebView events.

    This class provides Qt-compatible signals that are emitted when
    WebView events occur. It bridges the Rust/Python event system
    to Qt's signal/slot mechanism.

    Signals:
        Navigation:
            urlChanged(str): Emitted when the current URL changes
            loadFinished(bool): Emitted when page loading finishes (True=success)
            titleChanged(str): Emitted when the page title changes
            loadProgress(int): Emitted during loading with progress (0-100)
            loadStarted(): Emitted when navigation begins
            iconChanged(): Emitted when the favicon changes
            iconUrlChanged(str): Emitted when the favicon URL changes

        Error handling:
            jsError(str, int, str): JavaScript error (message, lineNumber, sourceId)
            consoleMessage(int, str, int, str): Console message (level, msg, line, source)
            renderProcessTerminated(int, int): Render crash (terminationStatus, exitCode)

        IPC:
            ipcMessageReceived(str, object): IPC message from JS (eventName, data)

        Selection:
            selectionChanged(): Emitted when text selection changes

        Window:
            windowCloseRequested(): Emitted when window close is requested
            fullScreenRequested(bool): Emitted when fullscreen is requested
    """

    # Navigation signals
    urlChanged = Signal(str)
    loadStarted = Signal()
    loadFinished = Signal(bool)
    loadProgress = Signal(int)

    # Page signals
    titleChanged = Signal(str)
    iconChanged = Signal()
    iconUrlChanged = Signal(str)

    # Error handling signals
    jsError = Signal(str, int, str)  # message, lineNumber, sourceId
    consoleMessage = Signal(int, str, int, str)  # level, message, lineNumber, sourceId
    renderProcessTerminated = Signal(int, int)  # terminationStatus, exitCode

    # IPC signals - enables Qt signal/slot style IPC handling
    ipcMessageReceived = Signal(str, object)  # eventName, data

    # Selection signals
    selectionChanged = Signal()

    # Window signals
    windowCloseRequested = Signal()
    fullScreenRequested = Signal(bool)

    def __init__(self, parent: Optional[QObject] = None):
        """Initialize Qt signals.

        Args:
            parent: Parent QObject for proper Qt object hierarchy
        """
        super().__init__(parent)
        self._current_url = ""
        self._current_title = ""
        self._current_progress = 0
        self._is_loading = False

    def emit_url_changed(self, url: str) -> None:
        """Emit urlChanged signal if URL actually changed.

        Args:
            url: New URL
        """
        if url != self._current_url:
            self._current_url = url
            logger.debug(f"QtWebViewSignals: urlChanged -> {url}")
            self.urlChanged.emit(url)

    def emit_load_started(self) -> None:
        """Emit loadStarted signal."""
        if not self._is_loading:
            self._is_loading = True
            self._current_progress = 0
            logger.debug("QtWebViewSignals: loadStarted")
            self.loadStarted.emit()

    def emit_load_finished(self, success: bool) -> None:
        """Emit loadFinished signal.

        Args:
            success: True if loading completed successfully
        """
        self._is_loading = False
        self._current_progress = 100 if success else 0
        logger.debug(f"QtWebViewSignals: loadFinished -> {success}")
        self.loadFinished.emit(success)

    def emit_load_progress(self, progress: int) -> None:
        """Emit loadProgress signal.

        Args:
            progress: Loading progress (0-100)
        """
        progress = max(0, min(100, progress))
        if progress != self._current_progress:
            self._current_progress = progress
            logger.debug(f"QtWebViewSignals: loadProgress -> {progress}")
            self.loadProgress.emit(progress)

    def emit_title_changed(self, title: str) -> None:
        """Emit titleChanged signal if title actually changed.

        Args:
            title: New page title
        """
        if title != self._current_title:
            self._current_title = title
            logger.debug(f"QtWebViewSignals: titleChanged -> {title}")
            self.titleChanged.emit(title)

    def emit_close_requested(self) -> None:
        """Emit windowCloseRequested signal."""
        logger.debug("QtWebViewSignals: windowCloseRequested")
        self.windowCloseRequested.emit()

    def emit_fullscreen_requested(self, fullscreen: bool) -> None:
        """Emit fullScreenRequested signal.

        Args:
            fullscreen: True to enter fullscreen, False to exit
        """
        logger.debug(f"QtWebViewSignals: fullScreenRequested -> {fullscreen}")
        self.fullScreenRequested.emit(fullscreen)

    # ========== Error handling signal emitters ==========

    def emit_js_error(self, message: str, line_number: int, source_id: str) -> None:
        """Emit jsError signal for JavaScript errors.

        Args:
            message: Error message
            line_number: Line number where error occurred
            source_id: Source file/URL where error occurred
        """
        if _VERBOSE_LOGGING:
            logger.debug(f"QtWebViewSignals: jsError -> {message} at {source_id}:{line_number}")
        self.jsError.emit(message, line_number, source_id)

    def emit_console_message(
        self, level: int, message: str, line_number: int, source_id: str
    ) -> None:
        """Emit consoleMessage signal for console output.

        Args:
            level: Log level (0=log, 1=warning, 2=error)
            message: Console message
            line_number: Line number
            source_id: Source file/URL
        """
        if _VERBOSE_LOGGING:
            logger.debug(f"QtWebViewSignals: consoleMessage[{level}] -> {message}")
        self.consoleMessage.emit(level, message, line_number, source_id)

    def emit_render_process_terminated(self, termination_status: int, exit_code: int) -> None:
        """Emit renderProcessTerminated signal when render process crashes.

        Args:
            termination_status: Termination status code
            exit_code: Process exit code
        """
        logger.warning(
            f"QtWebViewSignals: renderProcessTerminated -> status={termination_status}, "
            f"exitCode={exit_code}"
        )
        self.renderProcessTerminated.emit(termination_status, exit_code)

    # ========== IPC signal emitters ==========

    def emit_ipc_message(self, event_name: str, data: object) -> None:
        """Emit ipcMessageReceived signal for IPC messages from JavaScript.

        This enables Qt signal/slot style handling of JavaScript messages:

        Example:
            webview.ipcMessageReceived.connect(self.handle_ipc)

            def handle_ipc(self, event_name, data):
                if event_name == "save_file":
                    self.save_file(data["path"])

        Args:
            event_name: Name of the IPC event
            data: Data payload from JavaScript
        """
        if _VERBOSE_LOGGING:
            logger.debug(f"QtWebViewSignals: ipcMessageReceived -> {event_name}")
        self.ipcMessageReceived.emit(event_name, data)

    # ========== Selection signal emitters ==========

    def emit_selection_changed(self) -> None:
        """Emit selectionChanged signal when text selection changes."""
        if _VERBOSE_LOGGING:
            logger.debug("QtWebViewSignals: selectionChanged")
        self.selectionChanged.emit()

    def emit_icon_url_changed(self, url: str) -> None:
        """Emit iconUrlChanged signal when favicon URL changes.

        Args:
            url: New favicon URL
        """
        logger.debug(f"QtWebViewSignals: iconUrlChanged -> {url}")
        self.iconUrlChanged.emit(url)

    # ========== Properties ==========

    @property
    def current_url(self) -> str:
        """Get the current URL."""
        return self._current_url

    @property
    def current_title(self) -> str:
        """Get the current page title."""
        return self._current_title

    @property
    def is_loading(self) -> bool:
        """Check if page is currently loading."""
        return self._is_loading

    @property
    def load_progress_value(self) -> int:
        """Get current load progress (0-100)."""
        return self._current_progress
