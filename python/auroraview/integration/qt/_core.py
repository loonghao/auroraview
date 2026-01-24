# -*- coding: utf-8 -*-
"""Qt backend - Qt host widget embedding the core AuroraView WebView.

This module provides a Qt ``QWidget`` subclass (:class:`QtWebView`) that
embeds the core AuroraView :class:`auroraview.webview.WebView` using the
native parent window handle (HWND on Windows). It is designed for DCC
applications that already have Qt loaded (e.g., Maya, Houdini, Nuke),
where Qt continues to own the main event loop and window hierarchy.

Compared to the old QWebEngine/QWebChannel-based backend, this design:

- Uses the same Rust/WebView2 core as the standalone backend
- Removes the duplicated JavaScript bridge and WebChannel wiring
- Keeps a single, unified JS API (``window.auroraview``) across all modes

**Requirements**:
    Install with Qt support: `pip install auroraview[qt]`

    This will install qtpy and compatible Qt bindings (PySide2, PySide6, PyQt5, or PyQt6).

Example:
    >>> from auroraview import QtWebView
    >>>
    >>> # Create WebView as Qt widget
    >>> webview = QtWebView(
    ...     parent=maya_main_window(),
    ...     title="My Tool",
    ...     width=800,
    ...     height=600
    ... )
    >>>
    >>> # Register event handler
    >>> @webview.on('export_scene')
    >>> def handle_export(data):
    ...     print(f"Exporting to: {data['path']}")
    >>>
    >>> # Load HTML
    >>> webview.load_html("<html><body>Hello!</body></html>")
    >>>
    >>> # Show window
    >>> webview.show()
"""

import logging
import os
import sys
from pathlib import Path
from typing import Any, Callable, Optional

try:
    from qtpy.QtCore import QCoreApplication, QEvent, Qt, QTimer, Signal
    from qtpy.QtWidgets import QLabel, QStackedWidget, QVBoxLayout, QWidget
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from auroraview.core.webview import WebView
from auroraview.integration.qt._compat import (
    update_embedded_window_geometry,
)
from auroraview.integration.qt.dialogs import FileDialogMixin
from auroraview.integration.qt.embedding import EmbeddingMixin
from auroraview.integration.qt.event_processor import QtEventProcessor
from auroraview.integration.qt.lifecycle import LifecycleMixin

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)


class QtWebView(LifecycleMixin, EmbeddingMixin, FileDialogMixin, QWidget):
    """Qt-native WebView widget for DCC applications.

    This is the recommended class for integrating WebView into Qt-based DCC
    applications like Maya, Houdini, Nuke, and 3ds Max. It provides:

    * Native Qt widget integration (works with QDockWidget, QMdiArea, etc.)
    * Automatic lifecycle management (closes with parent window)
    * Compatible high-level API (``load_url``, ``load_html``, ``eval_js``,
      ``emit``, ``on``, ``bind_call``, ``bind_api``)

    For non-Qt applications (e.g., Unreal Engine), use :class:`AuroraView`
    instead, which provides HWND-based integration.

    Example (Maya dockable tool)::

        from auroraview import QtWebView
        import maya.cmds as cmds

        # Get Maya main window
        main_window = maya_main_window()

        # Create dockable dialog
        dialog = QDialog(main_window)
        layout = QVBoxLayout(dialog)

        # Create embedded WebView
        webview = QtWebView(
            parent=dialog,
            url="http://localhost:3000",
            width=800,
            height=600
        )
        layout.addWidget(webview)

        # Show dialog
        dialog.show()
        webview.show()

    Qt Signals:
        urlChanged(str): Emitted when the current URL changes
        loadStarted(): Emitted when navigation begins
        loadFinished(bool): Emitted when page loading finishes (True=success)
        loadProgress(int): Emitted during loading with progress (0-100)
        titleChanged(str): Emitted when the page title changes

    Error Signals:
        jsError(str, int, str): JavaScript error (message, lineNumber, sourceId)
        consoleMessage(int, str, int, str): Console message (level, msg, line, source)
        renderProcessTerminated(int, int): Render crash (terminationStatus, exitCode)

    IPC Signals:
        ipcMessageReceived(str, object): IPC message from JS (eventName, data)

    Selection Signals:
        selectionChanged(): Emitted when text selection changes
    """

    # Navigation signals (Qt5/6 compatible)
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

    # Class-level flag to track if auto-prewarm has been triggered
    _auto_prewarm_triggered: bool = False

    def __init__(
        self,
        parent=None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        dev_tools: bool = True,
        context_menu: bool = True,
        asset_root: Optional[str] = None,
        data_directory: Optional[str] = None,
        allow_file_protocol: bool = False,
        always_on_top: bool = False,
        frameless: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        embed_mode: str = "child",
        ipc_batch_size: int = 0,
        icon: Optional[str] = None,
        tool_window: bool = False,
        auto_prewarm: bool = True,
        allow_new_window: bool = False,
        new_window_mode: Optional[str] = None,
        remote_debugging_port: Optional[int] = None,
    ) -> None:
        """Initialize QtWebView.

        Args:
            parent: Parent Qt widget
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional, ignored if url is set)
            dev_tools: Enable developer tools (F12)
            context_menu: Enable native context menu
            asset_root: Root directory for auroraview:// protocol
            data_directory: User data directory for WebView
            allow_file_protocol: Enable file:// protocol support
            always_on_top: Keep window always on top
            frameless: Enable frameless window mode
            transparent: Enable transparent window background
            background_color: Custom background color
            embed_mode: WebView embedding mode ("child", "owner", "none")
            ipc_batch_size: Max IPC messages per tick (0=unlimited)
            icon: Window icon path
            tool_window: Hide from taskbar/Alt+Tab (Windows)
            auto_prewarm: Auto-trigger WebView2 pre-warming
            allow_new_window: Allow opening new windows
            new_window_mode: How to handle new window requests
            remote_debugging_port: CDP remote debugging port
        """
        # Auto-prewarm on first instantiation
        if auto_prewarm and not QtWebView._auto_prewarm_triggered:
            QtWebView._auto_prewarm_triggered = True
            try:
                from auroraview.integration.qt.pool import WebViewPool

                if not WebViewPool.has_prewarmed():
                    logger.debug("[QtWebView] Auto-triggering WebViewPool.prewarm()")
                    WebViewPool.prewarm()
            except Exception as e:
                logger.debug(f"[QtWebView] Auto-prewarm failed (non-critical): {e}")

        super().__init__(parent)

        # Store configuration
        self._title = title
        self._width = width
        self._height = height
        self._dev_tools = dev_tools
        self._context_menu = context_menu
        self._asset_root = asset_root
        self._frameless = frameless
        self._transparent = transparent
        self._embed_mode = embed_mode
        self._initial_url = url
        self._initial_html = html

        self.setWindowTitle(title)
        self.resize(width, height)

        # Apply window flags
        if frameless:
            self.setWindowFlags(Qt.Window | Qt.FramelessWindowHint)
            if _VERBOSE_LOGGING:
                logger.info("QtWebView: Frameless mode enabled")

        if transparent:
            self.setAttribute(Qt.WA_TranslucentBackground, True)
            self.setAttribute(Qt.WA_NoSystemBackground, True)
            self.setStyleSheet("background: transparent;")
            if _VERBOSE_LOGGING:
                logger.info("QtWebView: Transparent background enabled")
        else:
            self.setStyleSheet("background: #0d0d0d; border: none; margin: 0; padding: 0;")
            self.setContentsMargins(0, 0, 0, 0)

        # Native window attributes
        self.setAttribute(Qt.WA_NativeWindow, True)
        self.setAttribute(Qt.WA_DeleteOnClose, True)

        qt_hwnd = int(self.winId())

        # Resize throttling state
        self._last_resize_time = 0
        self._resize_throttle_ms = 16  # ~60fps
        self._pending_resize = None
        self._last_emitted_size = (0, 0)

        # Create the core WebView
        self._webview = WebView.create(
            title=title,
            width=width,
            height=height,
            parent=qt_hwnd,
            mode=embed_mode,
            frame=False,  # Always frameless for embedded WebView
            debug=dev_tools,
            context_menu=context_menu,
            asset_root=asset_root,
            data_directory=data_directory,
            allow_file_protocol=allow_file_protocol,
            always_on_top=always_on_top,
            auto_show=False,
            auto_timer=True,
            transparent=transparent,
            background_color=background_color,
            ipc_batch_size=ipc_batch_size,
            icon=icon,
            tool_window=tool_window,
            allow_new_window=allow_new_window,
            new_window_mode=new_window_mode,
            remote_debugging_port=remote_debugging_port,
        )

        # Track cleanup state
        self._is_closing = False

        # Set up Qt event processor
        self._event_processor = QtEventProcessor(self._webview)
        self._webview.set_event_processor(self._event_processor)

        # Create Qt layout with QStackedWidget
        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0, 0, 0, 0)
        self._layout.setSpacing(0)

        self._stack = QStackedWidget()
        self._layout.addWidget(self._stack)

        # Page 0: Loading page
        self._loading_page = self._create_loading_page()
        self._stack.addWidget(self._loading_page)

        # Page 1: WebView page
        self._webview_page = QWidget()
        self._webview_page_layout = QVBoxLayout(self._webview_page)
        self._webview_page_layout.setContentsMargins(0, 0, 0, 0)
        self._webview_page_layout.setSpacing(0)
        self._stack.addWidget(self._webview_page)

        # Start with loading page
        self._stack.setCurrentIndex(0)

        # Container references (created in show())
        self._webview_container = None
        self._webview_qwindow = None
        self._using_direct_embed = False
        self._direct_embed_hwnd = None

        # Track initialization state
        self._webview_initialized = False

        # Install event filter on parent window
        if parent is not None:
            self._parent_window = parent.window() if hasattr(parent, "window") else parent
            if self._parent_window is not None:
                self._parent_window.installEventFilter(self)
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView: Installed event filter on parent window")
        else:
            self._parent_window = None

        # Initialize Qt signal state tracking
        self._qt_signal_state = {
            "current_url": "",
            "current_title": "",
            "is_loading": False,
            "load_progress": 0,
        }

        # Bridge WebView events to Qt signals
        self._setup_signal_bridge()

        if _VERBOSE_LOGGING:
            logger.info(
                "QtWebView created: %s (%sx%s, mode=%s)",
                title,
                width,
                height,
                embed_mode,
            )

    def _create_loading_page(self) -> QWidget:
        """Create the loading page widget."""
        page = QWidget()
        page.setStyleSheet("background-color: #0d0d0d; border: none;")
        layout = QVBoxLayout(page)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        label = QLabel("Loading...")
        label.setAlignment(Qt.AlignCenter)
        label.setStyleSheet("QLabel { color: #555; font-size: 12px; background: transparent; }")
        layout.addWidget(label)

        return page

    def _setup_signal_bridge(self) -> None:
        """Set up event handlers to bridge WebView events to Qt signals."""

        @self._webview.on("navigation_started")
        def on_nav_started(data):
            url = data.get("url", "") if data else ""
            self._qt_signal_state["is_loading"] = True
            self._qt_signal_state["load_progress"] = 0
            self.loadStarted.emit()
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        @self._webview.on("navigation_finished")
        def on_nav_finished(data):
            success = data.get("success", True) if data else True
            url = data.get("url", "") if data else ""
            self._qt_signal_state["is_loading"] = False
            self._qt_signal_state["load_progress"] = 100 if success else 0
            self.loadFinished.emit(success)
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        @self._webview.on("load_progress")
        def on_load_progress(data):
            progress = data.get("progress", 0) if data else 0
            progress = max(0, min(100, int(progress)))
            if progress != self._qt_signal_state["load_progress"]:
                self._qt_signal_state["load_progress"] = progress
                self.loadProgress.emit(progress)

        @self._webview.on("title_changed")
        def on_title_changed(data):
            title = data.get("title", "") if data else ""
            if title and title != self._qt_signal_state["current_title"]:
                self._qt_signal_state["current_title"] = title
                self.titleChanged.emit(title)

        @self._webview.on("url_changed")
        def on_url_changed(data):
            url = data.get("url", "") if data else ""
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        @self._webview.on("js_error")
        def on_js_error(data):
            if data:
                self.jsError.emit(
                    data.get("message", "Unknown error"),
                    data.get("line", 0),
                    data.get("source", ""),
                )

        @self._webview.on("console_message")
        def on_console_message(data):
            if data:
                self.consoleMessage.emit(
                    data.get("level", 0),
                    data.get("message", ""),
                    data.get("line", 0),
                    data.get("source", ""),
                )

        @self._webview.on("render_process_terminated")
        def on_render_terminated(data):
            if data:
                self.renderProcessTerminated.emit(
                    data.get("status", 0),
                    data.get("exit_code", 0),
                )

        @self._webview.on("selection_changed")
        def on_selection_changed(data):
            self.selectionChanged.emit()

        @self._webview.on("icon_changed")
        def on_icon_changed(data):
            self.iconChanged.emit()
            if data:
                url = data.get("url", "")
                if url:
                    self.iconUrlChanged.emit(url)

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView: Signal bridge initialized")

    @classmethod
    def create_deferred(
        cls,
        parent=None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        on_ready: Optional[Callable[["QtWebView"], None]] = None,
        on_error: Optional[Callable[[str], None]] = None,
        **kwargs,
    ) -> QWidget:
        """Create QtWebView with deferred initialization.

        Returns a placeholder widget immediately, then schedules actual
        WebView creation. This keeps the UI responsive.

        Args:
            parent: Parent Qt widget
            title: Window title
            width: Window width
            height: Window height
            on_ready: Callback when WebView is ready
            on_error: Callback on creation failure
            **kwargs: Additional arguments passed to QtWebView

        Returns:
            A placeholder QWidget that shows loading indicator.
        """
        placeholder = QWidget(parent)
        placeholder.setWindowTitle(title)
        placeholder.resize(width, height)
        placeholder.setAttribute(Qt.WA_NativeWindow, True)

        layout = QVBoxLayout(placeholder)
        layout.setContentsMargins(0, 0, 0, 0)
        loading_label = QLabel("Loading WebView...")
        loading_label.setAlignment(Qt.AlignCenter)
        loading_label.setStyleSheet("QLabel { color: #888; font-size: 14px; background: #1a1a2e; }")
        layout.addWidget(loading_label)

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView.create_deferred: Created placeholder")

        def do_create():
            try:
                QCoreApplication.processEvents()
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView.create_deferred: Creating WebView")

                webview_widget = cls(
                    parent=parent,
                    title=title,
                    width=width,
                    height=height,
                    **kwargs,
                )

                placeholder.hide()
                if on_ready:
                    on_ready(webview_widget)

            except Exception as e:
                logger.error("QtWebView.create_deferred: Failed - %s", e)
                loading_label.setText(f"Error: {e}")
                if on_error:
                    on_error(str(e))

        QTimer.singleShot(0, do_create)
        return placeholder

    # ------------------------------------------------------------------
    # High-level API (delegated to WebView)
    # ------------------------------------------------------------------

    def load_url(self, url: str) -> None:
        """Load a URL into the embedded WebView."""
        self._webview.load_url(url)
        logger.debug("QtWebView loading URL: %s", url)

    def load_html(self, html: str) -> None:
        """Load HTML content into the embedded WebView."""
        self._webview.load_html(html)
        logger.debug("QtWebView loading HTML (%s bytes)", len(html))

    def load_file(self, path: Any) -> None:
        """Load a local HTML file."""
        html_path = Path(path).expanduser().resolve()

        if self._asset_root:
            asset_root_path = Path(self._asset_root).expanduser().resolve()
            try:
                relative_path = html_path.relative_to(asset_root_path)
                url_path = str(relative_path).replace("\\", "/")
                if sys.platform == "win32":
                    auroraview_url = f"https://auroraview.localhost/{url_path}"
                else:
                    auroraview_url = f"auroraview://{url_path}"
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView loading via auroraview protocol: %s", auroraview_url)
                self.load_url(auroraview_url)
                return
            except ValueError:
                logger.warning(
                    "HTML file %s is not under asset_root %s, falling back to load_html",
                    html_path,
                    asset_root_path,
                )

        try:
            html = html_path.read_text(encoding="utf-8")
            self.load_html(html)
        except Exception:
            load_file = getattr(self._webview, "load_file", None)
            if callable(load_file):
                load_file(path)
            else:
                self.load_url(html_path.as_uri())

    def eval_js(self, script: str) -> None:
        """Execute JavaScript in the embedded WebView."""
        self._webview.eval_js(script)

    def emit(self, event_name: str, data: Any = None, auto_process: bool = True) -> None:
        """Emit an AuroraView event to the embedded WebView."""
        self._webview.emit(event_name, data, auto_process=auto_process)

    def on(self, event_name: str) -> Callable:
        """Decorator to register event handler with Qt signal emission."""

        def decorator(func: Callable) -> Callable:
            def wrapper(data):
                self.ipcMessageReceived.emit(event_name, data)
                return func(data)

            self._webview.register_callback(event_name, wrapper)
            return func

        return decorator

    def register_callback(self, event_name: str, callback: Callable) -> None:
        """Register a callback for an event with Qt signal emission."""

        def wrapper(data):
            self.ipcMessageReceived.emit(event_name, data)
            return callback(data)

        self._webview.register_callback(event_name, wrapper)

    # Window event callbacks (delegate to WebView)
    def on_shown(self, callback: Callable) -> Callable:
        """Register callback for window shown event."""
        return self._webview.on_shown(callback)

    def on_closing(self, callback: Callable) -> Callable:
        """Register callback for window closing event."""
        return self._webview.on_closing(callback)

    def on_closed(self, callback: Callable) -> Callable:
        """Register callback for window closed event."""
        return self._webview.on_closed(callback)

    def on_resized(self, callback: Callable) -> Callable:
        """Register callback for window resized event."""
        return self._webview.on_resized(callback)

    def on_moved(self, callback: Callable) -> Callable:
        """Register callback for window moved event."""
        return self._webview.on_moved(callback)

    def on_focused(self, callback: Callable) -> Callable:
        """Register callback for window focused event."""
        return self._webview.on_focused(callback)

    def on_blurred(self, callback: Callable) -> Callable:
        """Register callback for window blurred event."""
        return self._webview.on_blurred(callback)

    def on_minimized(self, callback: Callable) -> Callable:
        """Register callback for window minimized event."""
        return self._webview.on_minimized(callback)

    def on_maximized(self, callback: Callable) -> Callable:
        """Register callback for window maximized event."""
        return self._webview.on_maximized(callback)

    def on_restored(self, callback: Callable) -> Callable:
        """Register callback for window restored event."""
        return self._webview.on_restored(callback)

    # ------------------------------------------------------------------
    # State and Command API
    # ------------------------------------------------------------------

    @property
    def state(self):
        """Get the shared state container."""
        return self._webview.state

    @property
    def commands(self):
        """Get the command registry."""
        return self._webview.commands

    def command(self, func_or_name=None):
        """Decorator to register a command."""
        return self._webview.command(func_or_name)

    @property
    def channels(self):
        """Get the channel manager."""
        return self._webview.channels

    def create_channel(self, name: str):
        """Create a new channel for streaming data."""
        return self._webview.create_channel(name)

    def bind_call(self, method: str, func: Optional[Callable[..., Any]] = None):
        """Bind a Python callable for auroraview.call."""
        return self._webview.bind_call(method, func)

    def bind_api(self, api: Any, namespace: str = "api") -> None:
        """Bind an object's public methods as auroraview.api.*."""
        self._webview.bind_api(api, namespace)

    @property
    def title(self) -> str:
        """Get window title."""
        return self.windowTitle()

    @title.setter
    def title(self, value: str) -> None:
        """Set window title."""
        self._title = value
        self.setWindowTitle(value)
        try:
            self._webview._title = value
        except Exception:
            pass

    # ------------------------------------------------------------------
    # Qt signal state properties
    # ------------------------------------------------------------------

    @property
    def current_url(self) -> str:
        """Get the current URL."""
        return self._qt_signal_state.get("current_url", "")

    @property
    def current_title(self) -> str:
        """Get the current page title."""
        return self._qt_signal_state.get("current_title", "")

    @property
    def is_loading(self) -> bool:
        """Check if page is loading."""
        return self._qt_signal_state.get("is_loading", False)

    @property
    def load_progress_value(self) -> int:
        """Get current load progress (0-100)."""
        return self._qt_signal_state.get("load_progress", 0)

    # ------------------------------------------------------------------
    # Qt integration
    # ------------------------------------------------------------------

    def get_diagnostics(self) -> dict:
        """Get diagnostic information."""
        return {
            "event_processor_type": type(self._event_processor).__name__,
            "event_process_count": self._event_processor._process_count,
            "has_event_processor": self._webview._event_processor is not None,
            "processor_is_correct": isinstance(self._event_processor, QtEventProcessor),
        }

    def resizeEvent(self, event) -> None:
        """Handle Qt widget resize."""
        super().resizeEvent(event)

        new_size = event.size()
        width = new_size.width()
        height = new_size.height()

        # Handle direct embedding mode
        if getattr(self, "_using_direct_embed", False):
            direct_hwnd = getattr(self, "_direct_embed_hwnd", None)
            if direct_hwnd:
                update_embedded_window_geometry(direct_hwnd, 0, 0, width, height)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Direct embed resize: {width}x{height}")

        # Sync container and WebView2 bounds
        container = getattr(self, "_webview_container", None)
        if container is not None:
            container.setGeometry(0, 0, width, height)
            self._sync_webview2_controller_bounds()

    def moveEvent(self, event) -> None:
        """Handle Qt widget move."""
        super().moveEvent(event)

    def eventFilter(self, watched, event) -> bool:
        """Filter events from parent window."""
        parent_window = getattr(self, "_parent_window", None)
        if watched == parent_window and parent_window is not None:
            if event.type() == QEvent.Close:
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView: Parent window closing")
                try:
                    if not getattr(self, "_is_closing", False):
                        self._is_closing = True
                        webview = getattr(self, "_webview", None)
                        if webview is not None:
                            webview.close()
                except Exception as e:
                    if _VERBOSE_LOGGING:
                        logger.debug("QtWebView: error closing on parent close: %s", e)

        return super().eventFilter(watched, event)

    def showEvent(self, event) -> None:
        """Handle Qt show event."""
        super().showEvent(event)

        if not self._webview_initialized:
            self._webview_initialized = True
            self._initialize_webview()

    def show(self) -> None:
        """Show the Qt widget."""
        super().show()

    def closeEvent(self, event) -> None:
        """Handle Qt close event."""
        if self._handle_close_event():
            event.accept()
            return

        event.accept()
        super().closeEvent(event)

    def __del__(self) -> None:
        """Destructor."""
        self._handle_destructor()

    def __repr__(self) -> str:
        """String representation."""
        try:
            return f"QtWebView(title='{self.windowTitle()}', size={self.width()}x{self.height()})"
        except RuntimeError:
            return "QtWebView(<deleted>)"

    def get_hwnd(self) -> Optional[int]:
        """Get the native window handle (HWND) of the embedded WebView."""
        try:
            return self._webview.get_hwnd()
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView.get_hwnd() error: %s", e)
            return None


__all__ = ["QtWebView", "QtEventProcessor"]
