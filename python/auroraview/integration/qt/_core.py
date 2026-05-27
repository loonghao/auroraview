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

import functools
import logging
import os
import sys
from pathlib import Path
from typing import Any, Callable, Dict, Optional, Tuple

try:
    from qtpy.QtCore import QCoreApplication, QEvent, Qt, QTimer, Signal
    from qtpy.QtWidgets import QFrame, QLabel, QStackedWidget, QVBoxLayout, QWidget
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


# Event names registered by _setup_signal_bridge(). Single source of truth
# used by both setup (documentation/assertions) and teardown (cleanup).
_SIGNAL_BRIDGE_EVENTS: tuple = (
    "navigation_started",
    "navigation_finished",
    "load_progress",
    "title_changed",
    "url_changed",
    "js_error",
    "console_message",
    "render_process_terminated",
    "selection_changed",
    "icon_changed",
)


def _guard_alive(method):
    """Decorator that guards public API methods against zombie widget access.

    If the widget is closing or the C++ object has been destroyed,
    the method becomes a silent no-op returning None. This prevents
    RuntimeError from propagating to user code in async callback scenarios.

    Return value semantics:
        When the widget is not alive, the decorated method returns None
        regardless of its normal return type. Callers of guarded methods
        that normally return values (if any are added in future) MUST
        handle None as a possible return value indicating the widget is
        dead. Currently all guarded methods are void (load_url, load_html,
        eval_js, emit, load_file), so this is transparent.
    """

    @functools.wraps(method)
    def wrapper(self, *args, **kwargs):
        if not self.is_alive:
            if _VERBOSE_LOGGING:
                logger.debug(
                    "QtWebView.%s() skipped: widget is not alive",
                    method.__name__,
                )
            return None
        try:
            return method(self, *args, **kwargs)
        except RuntimeError as e:
            # Only swallow Qt C++ deletion errors (TOCTOU window between
            # is_alive check and actual Qt call). Re-raise anything else
            # (e.g., path errors in load_file, Rust core panics) so that
            # genuine bugs are not silently hidden.
            msg = str(e).lower()
            if "deleted" in msg or "c++ object" in msg or "wrapped c++" in msg:
                if _VERBOSE_LOGGING:
                    logger.debug(
                        "QtWebView.%s() caught RuntimeError (C++ object deleted)",
                        method.__name__,
                    )
                return None
            raise

    return wrapper


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

    # Lifecycle signals
    aboutToClose = Signal()  # Emitted before close sequence, widget still alive

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
        capture_file_drop: Optional[bool] = None,
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
            self.setStyleSheet("background: #0d0d0d; border: 0px; margin: 0px; padding: 0px;")
            self.setContentsMargins(0, 0, 0, 0)

        # Native window attributes
        self.setAttribute(Qt.WA_NativeWindow, True)
        # NOTE: WA_DeleteOnClose intentionally NOT set.
        # In DCC environments (Maya, Houdini, Nuke), Qt may fire spurious
        # closeEvents during DPI changes or native window rebuilds.
        # WA_DeleteOnClose would permanently destroy the C++ object, making
        # the Python reference a "zombie" that crashes on any attribute access.
        # Instead, we rely on explicit destroy()/deleteLater() or Python GC.
        # The _reset_state_for_reuse() mechanism allows the widget to be
        # shown again after a close.

        qt_hwnd = int(self.winId())

        # Resize throttling state
        self._last_resize_time = 0
        self._resize_throttle_ms = 16  # ~60fps
        self._pending_resize = None
        self._last_emitted_size = (0, 0)

        # Create the core WebView
        self._webview: Optional[WebView] = WebView.create(
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
            # RFC 0017: pass tri-state Optional[bool] through unchanged. The
            # DCC/Qt path does not provide a default; the value flows to
            # Rust unwrap_or(false) just like every other entry point.
            capture_file_drop=capture_file_drop,
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
        # Thread safety note: _is_closing is a plain bool read/written without
        # explicit locks. This is safe because:
        # 1. CPython GIL guarantees atomic LOAD_ATTR/STORE_ATTR bytecode ops
        # 2. All WebView event callbacks are dispatched on the main thread via
        #    QtEventProcessor (auto_timer=True mode), so reads in signal bridge
        #    callbacks happen on the same thread as writes in closeEvent/showEvent.
        # If migrating to free-threaded Python (PEP 703) or non-CPython runtime,
        # consider using threading.Event or an atomic flag.
        self._is_closing: bool = False
        # Latching flag: once the C++ QWidget object is confirmed destroyed
        # (objectName() raised RuntimeError), this is set True permanently.
        # Avoids repeated try/except overhead in is_alive on high-frequency
        # callback paths (e.g., load_progress events during large page loads).
        self._cpp_dead: bool = False

        # Bridge handler tracking: maps event_name → guarded callback reference.
        # Used by _teardown_signal_bridge() to surgically remove only bridge
        # callbacks from the core WebView, preserving user-registered handlers.
        self._bridge_handlers: Dict[str, Callable] = {}
        # ConnectionIds returned by core.register_callback() for bridge handlers.
        # Enables targeted signal system disconnect without clearing user signals.
        self._bridge_conn_ids: Dict[str, Any] = {}

        # Cross-task mutex flags + last-synced bounds memo shared by
        # EmbeddingMixin (child-window fixer) and LifecycleMixin
        # (delayed geometry sync).  See auroraview.integration.qt._locks
        # for the rationale; _last_synced_bounds backs the idempotency
        # guard inside _sync_webview2_controller_bounds.
        self._geometry_sync_in_progress = False
        self._child_window_fix_in_progress = False
        self._last_synced_bounds: Optional[Tuple[int, int]] = None

        # Set up Qt event processor
        self._event_processor = QtEventProcessor(self._webview)
        self._webview.set_event_processor(self._event_processor)

        # Create Qt layout with QStackedWidget
        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0, 0, 0, 0)
        self._layout.setSpacing(0)

        self._stack = QStackedWidget()
        self._stack.setFrameShape(QFrame.NoFrame)
        self._stack.setContentsMargins(0, 0, 0, 0)
        self._stack.setStyleSheet(
            "QStackedWidget { border: none; margin: 0; padding: 0; background-color: #0d0d0d; }"
        )
        self._layout.addWidget(self._stack)

        # Page 0: Loading page
        self._loading_page = self._create_loading_page()
        self._stack.addWidget(self._loading_page)

        # Page 1: WebView page
        self._webview_page = QWidget()
        self._webview_page.setContentsMargins(0, 0, 0, 0)
        self._webview_page.setStyleSheet("background-color: #0d0d0d; border: none;")
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

    def _guarded_bridge_callback(self, fn):
        """Wrap a signal bridge callback with is_alive guard + RuntimeError catch.

        All signal bridge callbacks share the same guard pattern: skip if the
        widget is dead, catch RuntimeError from C++ object deletion during the
        TOCTOU window. This helper centralizes that pattern.
        """

        @functools.wraps(fn)
        def wrapper(data):
            if not self.is_alive:
                return
            try:
                return fn(data)
            except RuntimeError as e:
                # Consistent with _guard_alive and _make_guarded_ipc_wrapper:
                # only swallow Qt C++ deletion errors, re-raise anything else.
                msg = str(e).lower()
                if "deleted" in msg or "c++ object" in msg or "wrapped c++" in msg:
                    return None
                raise

        return wrapper

    def _setup_signal_bridge(self) -> None:
        """Set up event handlers to bridge WebView events to Qt signals.

        Each callback is guarded by is_alive to prevent RuntimeError when
        async WebView events arrive after the widget has been closed or
        its C++ object has been destroyed.

        This method is idempotent: it calls _teardown_signal_bridge() first
        to clear any existing registrations, preventing duplicate handlers
        on re-show scenarios.

        Bridge callbacks are tracked in self._bridge_handlers so that
        _teardown_signal_bridge() can surgically remove them without
        affecting user-registered handlers on the same event names.
        """
        # Defensive teardown: prevent duplicate registration on re-show.
        # Since register_callback() appends (not replaces), calling setup
        # twice without teardown would cause events to be handled twice.
        self._teardown_signal_bridge()
        self._bridge_handlers = {}
        self._bridge_conn_ids = {}

        # -- Define raw handler functions (business logic only) --

        def on_nav_started(data):
            url = data.get("url", "") if data else ""
            self._qt_signal_state["is_loading"] = True
            self._qt_signal_state["load_progress"] = 0
            self.loadStarted.emit()
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        def on_nav_finished(data):
            success = data.get("success", True) if data else True
            url = data.get("url", "") if data else ""
            self._qt_signal_state["is_loading"] = False
            self._qt_signal_state["load_progress"] = 100 if success else 0
            self.loadFinished.emit(success)
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        def on_load_progress(data):
            progress = data.get("progress", 0) if data else 0
            progress = max(0, min(100, int(progress)))
            if progress != self._qt_signal_state["load_progress"]:
                self._qt_signal_state["load_progress"] = progress
                self.loadProgress.emit(progress)

        def on_title_changed(data):
            title = data.get("title", "") if data else ""
            if title and title != self._qt_signal_state["current_title"]:
                self._qt_signal_state["current_title"] = title
                self.titleChanged.emit(title)

        def on_url_changed(data):
            url = data.get("url", "") if data else ""
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        def on_js_error(data):
            if data:
                self.jsError.emit(
                    data.get("message", "Unknown error"),
                    data.get("line", 0),
                    data.get("source", ""),
                )

        def on_console_message(data):
            if data:
                self.consoleMessage.emit(
                    data.get("level", 0),
                    data.get("message", ""),
                    data.get("line", 0),
                    data.get("source", ""),
                )

        def on_render_terminated(data):
            if data:
                self.renderProcessTerminated.emit(
                    data.get("status", 0),
                    data.get("exit_code", 0),
                )

        def on_selection_changed(data):
            self.selectionChanged.emit()

        def on_icon_changed(data):
            self.iconChanged.emit()
            if data:
                url = data.get("url", "")
                if url:
                    self.iconUrlChanged.emit(url)

        # -- Register each handler with guard wrapper, track references --

        _handlers_map = {
            "navigation_started": on_nav_started,
            "navigation_finished": on_nav_finished,
            "load_progress": on_load_progress,
            "title_changed": on_title_changed,
            "url_changed": on_url_changed,
            "js_error": on_js_error,
            "console_message": on_console_message,
            "render_process_terminated": on_render_terminated,
            "selection_changed": on_selection_changed,
            "icon_changed": on_icon_changed,
        }

        for event_name, raw_handler in _handlers_map.items():
            guarded = self._guarded_bridge_callback(raw_handler)
            conn_id = self._webview.register_callback(event_name, guarded)
            self._bridge_handlers[event_name] = guarded
            self._bridge_conn_ids[event_name] = conn_id

        # Dev-time assertion: ensure we registered exactly the events
        # declared in _SIGNAL_BRIDGE_EVENTS. Catches drift when adding
        # new events to one place but not the other.
        assert set(self._bridge_handlers.keys()) == set(_SIGNAL_BRIDGE_EVENTS), (
            f"Signal bridge event mismatch: registered={set(self._bridge_handlers.keys())}, "
            f"expected={set(_SIGNAL_BRIDGE_EVENTS)}"
        )

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView: Signal bridge initialized (%d callbacks)", len(_SIGNAL_BRIDGE_EVENTS))

    def _teardown_signal_bridge(self) -> None:
        """Remove only bridge-registered callbacks from the core WebView.

        This is the symmetric counterpart of _setup_signal_bridge(). It
        surgically removes bridge callbacks tracked in self._bridge_handlers
        without affecting user-registered handlers on the same event names.

        Strategy:
        - All bridge callbacks are already guarded by is_alive, so they are
          effectively no-ops once _is_closing is True.
        - This teardown is primarily for breaking reference cycles to enable
          proper garbage collection in long-running DCC sessions.
        - User handlers registered via QtWebView.on() or register_callback()
          on the same event names (e.g., "navigation_started") are preserved.

        Called during:
        - _handle_close_event(): release callbacks before WebView.close()
        - showEvent() re-show: clear stale callbacks before re-initialization
        - eventFilter parent close: clean up on parent destruction
        - _setup_signal_bridge(): idempotent cleanup before (re-)registration
        """
        webview = getattr(self, "_webview", None)
        if webview is None:
            return

        bridge_handlers = getattr(self, "_bridge_handlers", {})
        bridge_conn_ids = getattr(self, "_bridge_conn_ids", {})

        # Layer 1: Remove bridge callbacks from legacy event handler registry.
        # Only removes the specific callable we registered, preserving any
        # user-registered handlers on the same event name.
        try:
            with webview._event_handlers_lock:
                for ev, handler in bridge_handlers.items():
                    try:
                        handlers_list = webview._event_handlers.get(ev)
                        if handlers_list:
                            try:
                                handlers_list.remove(handler)
                            except ValueError:
                                pass  # Already removed or not found
                            # Clean up empty lists to prevent unbounded growth
                            if not handlers_list:
                                del webview._event_handlers[ev]
                    except Exception:
                        pass
        except (AttributeError, RuntimeError):
            pass  # Core structure may differ in test mocks or C++ deleted

        # Layer 2: Disconnect from signal system using stored ConnectionIds.
        # This only disconnects bridge connections, not user connections.
        try:
            if hasattr(webview, "signals") and webview.signals is not None:
                for event_name, conn_id in bridge_conn_ids.items():
                    try:
                        webview.signals.custom.disconnect(event_name, conn_id)
                    except (AttributeError, RuntimeError, Exception):
                        pass
        except (AttributeError, RuntimeError):
            pass

        # Clear tracked references
        self._bridge_handlers = {}
        self._bridge_conn_ids = {}

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView: Signal bridge torn down (%d events)", len(bridge_handlers))

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

    @_guard_alive
    def load_url(self, url: str) -> None:
        """Load a URL into the embedded WebView.

        Safe to call at any point in the widget lifecycle. If the widget
        is closing or destroyed, this is a silent no-op.
        """
        self._webview.load_url(url)
        logger.debug("QtWebView loading URL: %s", url)

    @_guard_alive
    def load_html(self, html: str) -> None:
        """Load HTML content into the embedded WebView.

        Safe to call at any point in the widget lifecycle. If the widget
        is closing or destroyed, this is a silent no-op.
        """
        self._webview.load_html(html)
        logger.debug("QtWebView loading HTML (%s bytes)", len(html))

    @_guard_alive
    def load_file(self, path: Any) -> None:
        """Load a local HTML file.

        Safe to call at any point in the widget lifecycle. If the widget
        is closing or destroyed, this is a silent no-op.
        """
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

    @_guard_alive
    def eval_js(self, script: str) -> None:
        """Execute JavaScript in the embedded WebView.

        Safe to call at any point in the widget lifecycle. If the widget
        is closing or destroyed, this is a silent no-op.
        """
        self._webview.eval_js(script)

    @_guard_alive
    def emit(self, event_name: str, data: Any = None, auto_process: bool = True) -> None:
        """Emit an AuroraView event to the embedded WebView.

        Safe to call at any point in the widget lifecycle. If the widget
        is closing or destroyed, this is a silent no-op.
        """
        self._webview.emit(event_name, data, auto_process=auto_process)

    def _make_guarded_ipc_wrapper(self, event_name: str, callback: Callable) -> Callable:
        """Create a guarded IPC wrapper for user-registered event callbacks.

        Combines is_alive guard + ipcMessageReceived signal emission +
        RuntimeError protection. Used by both on() and register_callback().
        """

        def wrapper(data):
            if not self.is_alive:
                return None
            try:
                self.ipcMessageReceived.emit(event_name, data)
                return callback(data)
            except RuntimeError as e:
                # Only swallow Qt C++ deletion errors. Re-raise user
                # callback RuntimeErrors (e.g., argument validation) so
                # they surface properly instead of being silently eaten.
                msg = str(e).lower()
                if "deleted" in msg or "c++ object" in msg or "wrapped c++" in msg:
                    return None
                raise

        return wrapper

    def on(self, event_name: str) -> Callable:
        """Decorator to register event handler with Qt signal emission.

        The registered wrapper is guarded: if the widget is closing or
        the C++ object has been destroyed when an IPC message arrives,
        the callback is silently skipped.
        """

        def decorator(func: Callable) -> Callable:
            self._webview.register_callback(
                event_name, self._make_guarded_ipc_wrapper(event_name, func)
            )
            return func

        return decorator

    def register_callback(self, event_name: str, callback: Callable) -> None:
        """Register a callback for an event with Qt signal emission.

        The registered wrapper is guarded: if the widget is closing or
        the C++ object has been destroyed when an IPC message arrives,
        the callback is silently skipped.
        """
        self._webview.register_callback(
            event_name, self._make_guarded_ipc_wrapper(event_name, callback)
        )

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

    @property
    def is_alive(self) -> bool:
        """Check if the widget and its underlying WebView are still usable.

        Returns False if:
        - The widget is in the process of closing (_is_closing)
        - The C++ QWidget was previously confirmed destroyed (_cpp_dead)
        - The internal WebView reference is None (after destroy())
        - The underlying C++ QWidget has been destroyed (deleteLater)

        This is the single source of truth for all validity guards.
        External code and internal callbacks should check this before
        accessing the widget to avoid RuntimeError from dead C++ objects.

        Performance note:
            Once objectName() raises RuntimeError (confirming C++ death),
            the _cpp_dead flag is latched True. Subsequent calls short-
            circuit without entering try/except, avoiding overhead on
            high-frequency callback paths (e.g., load_progress during
            large page loads may fire dozens of times per second).
            The flag is reset in showEvent on DCC widget reuse.

        Thread-safety / timing note:
            After destroy() calls deleteLater(), the C++ object is not
            immediately deleted — it waits for event loop dispatch. However,
            _is_closing is always set True BEFORE deleteLater() is called
            (both in destroy() and _handle_close_event()), so the fast-path
            ``if self._is_closing: return False`` catches this case before
            objectName() is ever reached. If _is_closing were somehow
            bypassed, objectName() would still return True until the event
            loop processes the deferred deletion — this is acceptable since
            it means the C++ object is technically still alive at that point.
        """
        if self._is_closing:
            return False
        if self._cpp_dead:
            return False
        if getattr(self, "_webview", None) is None:
            return False
        try:
            # objectName() is a O(1) Qt property access that will raise
            # RuntimeError if the C++ object has been destroyed.
            # Unlike isVisible() which traverses the parent chain, this
            # is a direct member access with negligible overhead.
            self.objectName()
            return True
        except RuntimeError:
            # C++ object confirmed dead; latch the flag so subsequent
            # calls skip the try/except entirely (avoids overhead on
            # high-frequency event paths like load_progress).
            self._cpp_dead = True
            return False

    # ------------------------------------------------------------------
    # Qt integration
    # ------------------------------------------------------------------

    def get_diagnostics(self) -> dict:
        """Get diagnostic information."""
        if self._webview is None:
            return {"error": "widget destroyed", "is_alive": False}
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
            try:
                container.setGeometry(0, 0, width, height)
                self._sync_webview2_controller_bounds()
            except RuntimeError:
                # C++ container object was destroyed (likely DPI change or
                # parent deleteLater). Log and clear reference so we don't
                # attempt further operations on the dead object.
                logger.warning(
                    "QtWebView.resizeEvent: container C++ object already deleted "
                    "(DPI change or close sequence). Clearing container reference."
                )
                self._webview_container = None

    def moveEvent(self, event) -> None:
        """Handle Qt widget move."""
        super().moveEvent(event)

    def eventFilter(self, watched, event) -> bool:
        """Filter events from parent window.

        Guarded against RuntimeError from Qt C++ object deletion only.
        Non-deletion RuntimeErrors are re-raised to surface genuine bugs.
        """
        try:
            parent_window = getattr(self, "_parent_window", None)
            if watched == parent_window and parent_window is not None:
                if event.type() == QEvent.Close:
                    if _VERBOSE_LOGGING:
                        logger.debug("QtWebView: Parent window closing")
                    # Reuse standard close logic for consistency (emits
                    # aboutToClose, sets _is_closing, tears down bridge).
                    self._handle_close_event()

            return super().eventFilter(watched, event)
        except RuntimeError as e:
            # Only swallow Qt C++ deletion errors. Re-raise anything else
            # so that genuine bugs (e.g., logic errors in super()) propagate.
            msg = str(e).lower()
            if "deleted" in msg or "c++ object" in msg or "wrapped c++" in msg:
                return False
            raise

    def showEvent(self, event) -> None:
        """Handle Qt show event.

        On re-show after a previous close (DCC reuse pattern), resets the
        closing flag so that the widget is fully functional again.

        Ordering is important: _is_closing is reset first (so is_alive
        returns True), then WebView is (re-)initialized, and finally the
        signal bridge is re-established on the ready core.
        """
        super().showEvent(event)

        # Track whether we're coming from a closed state (for bridge setup)
        was_closing = self._is_closing

        # Reset closing flag on re-show (DCC reuse: close -> show again)
        if self._is_closing:
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView: Resetting _is_closing on re-show (reuse)")
            self._is_closing = False
            self._cpp_dead = False  # C++ object is alive if showEvent fires
            # Reset signal state to prevent stale deduplication
            # (e.g., navigating to the same URL won't emit urlChanged if
            # the old value is still cached).
            self._qt_signal_state = {
                "current_url": "",
                "current_title": "",
                "is_loading": False,
                "load_progress": 0,
            }

        if not self._webview_initialized:
            self._webview_initialized = True
            self._initialize_webview()

        # Re-establish signal bridge AFTER core is (re-)initialized.
        # This ensures handlers are registered on a ready WebView core,
        # not one that's about to be reset by _initialize_webview().
        if was_closing:
            self._setup_signal_bridge()

    def show(self) -> None:
        """Show the Qt widget."""
        super().show()

    def closeEvent(self, event) -> None:
        """Handle Qt close event.

        Accepts the close and cleans up internal WebView state. Since
        WA_DeleteOnClose is NOT set, the C++ QWidget survives the close
        and can be shown again (DCC reuse pattern). The internal WebView
        is released and will be re-created on next showEvent.
        """
        if self._handle_close_event():
            event.accept()
            return

        event.accept()
        super().closeEvent(event)

    def destroy(self) -> None:
        """Explicitly destroy the widget and release all resources.

        This is the recommended way to permanently dispose of a QtWebView
        instance. After calling destroy(), the widget cannot be reused.

        Performs:
        1. Runs standard close logic via _handle_close_event()
           (emits aboutToClose, sets _is_closing, tears down signal bridge,
            closes WebView, resets state)
        2. Removes event filter from parent
        3. Clears WebView reference (breaks all remaining cycles)
        4. Schedules C++ object deletion via deleteLater()

        Use this instead of relying on Python GC for deterministic cleanup.
        """
        # Reuse standard close logic (idempotent: if _is_closing is already
        # True, _handle_close_event() returns True immediately -- harmless).
        self._handle_close_event()

        # Additional permanent cleanup beyond close-for-reuse:

        # Remove event filter from parent window
        parent_window = getattr(self, "_parent_window", None)
        if parent_window is not None:
            try:
                parent_window.removeEventFilter(self)
            except (RuntimeError, Exception):
                pass
            self._parent_window = None

        # Release WebView reference to break all remaining cycles
        self._webview = None

        # Schedule C++ object deletion
        try:
            self.deleteLater()
        except RuntimeError:
            pass  # Already deleted

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
