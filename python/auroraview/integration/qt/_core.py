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
import time
from pathlib import Path
from typing import Any, Callable, Optional

try:
    from qtpy.QtCore import QCoreApplication, QEvent, Qt, QTimer, Signal
    from qtpy.QtGui import QWindow
    from qtpy.QtWidgets import QLabel, QStackedWidget, QVBoxLayout, QWidget
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from auroraview.core.webview import WebView

# Import Qt compatibility layer for version-specific handling
from auroraview.integration.qt._compat import (
    apply_clip_styles_to_parent,
    create_container_widget,
    embed_window_directly,
    get_qt_info,
    hide_window_for_init,
    is_qt6,
    post_container_setup,
    prepare_hwnd_for_container,
    show_window_after_init,
    supports_direct_embedding,
    update_embedded_window_geometry,
)
from auroraview.integration.qt.dialogs import FileDialogMixin

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
# In DCC environments, excessive logging causes severe UI performance issues
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)


class QtEventProcessor:
    """Event processor for Qt integration (strategy pattern).

    This class handles event processing for Qt-integrated WebViews by:
    1. Processing Qt events (QCoreApplication.processEvents())
    2. Processing WebView message queue (webview._core.process_events())

    This ensures both Qt and WebView events are handled correctly.

    Architecture:
        WebView (base class)
            ↓ uses
        QtEventProcessor (strategy)
            ↓ processes
        Qt events + WebView events

    Example:
        >>> webview = WebView()
        >>> processor = QtEventProcessor(webview)
        >>> webview.set_event_processor(processor)
        >>>
        >>> # Now emit() and eval_js() automatically process Qt + WebView events
        >>> webview.emit("my_event", {"data": 123})
    """

    def __init__(self, webview: WebView):
        """Initialize Qt event processor.

        Args:
            webview: WebView instance to process events for
        """
        self._webview = webview
        self._process_count = 0

    def process(self) -> None:
        """Process Qt events and WebView message queue.

        This is the core method called by WebView._auto_process_events().

        Following main branch design:
        1. Process Qt events first (QCoreApplication.processEvents())
        2. Process AuroraView message queue (WebView.process_events())

        Without step 2, JavaScript code sent via eval_js() or emit() will
        remain in the message queue and never execute, causing Promises to hang.
        """
        self._process_count += 1

        try:
            # Step 1: Process Qt events first
            QCoreApplication.processEvents()

            # Step 2: Process AuroraView message queue
            # This is CRITICAL - without this, eval_js/emit messages stay in queue
            self._webview.process_events()
        except Exception as e:  # pragma: no cover - best-effort only
            if _VERBOSE_LOGGING:
                logger.debug(f"QtEventProcessor: Event processing failed: {e}")


class QtWebView(FileDialogMixin, QWidget):
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

    Example (Houdini panel)::

        from auroraview import QtWebView
        import hou

        panel = hou.qt.mainWindow()
        webview = QtWebView(parent=panel, url="http://localhost:3000")
        webview.show()

    For deferred creation in DCC tools (to show loading feedback), use
    :meth:`create_deferred`::

        >>> def on_ready(webview):
        ...     webview.load_url("http://localhost:3000")
        ...     webview.show()
        >>>
        >>> QtWebView.create_deferred(
        ...     parent=maya_main_window(),
        ...     on_ready=on_ready,
        ... )

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
        allow_file_protocol: bool = False,
        frameless: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        embed_mode: str = "child",
        ipc_batch_size: int = 0,
        auto_prewarm: bool = True,
    ) -> None:
        """Initialize QtWebView.

        Args:
            parent: Parent Qt widget
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional). If provided, the URL will be loaded
                after the WebView is shown.
            html: HTML content to load (optional). If provided, the HTML will
                be loaded after the WebView is shown. Ignored if ``url`` is set.
            dev_tools: Enable developer tools (F12 or right-click > Inspect)
            context_menu: Enable native context menu
            asset_root: Root directory for auroraview:// protocol.
                When set, enables the auroraview:// custom protocol for secure
                local resource loading. Files under this directory can be accessed
                using URLs like ``auroraview://path/to/file``.

                **Platform-specific URL format**:

                - Windows: ``https://auroraview.localhost/path``
                - macOS/Linux: ``auroraview://path``

                **Security**: Uses ``.localhost`` TLD (IANA reserved, RFC 6761)
                which cannot be registered and is treated as a local address.
                Requests are intercepted before DNS resolution.

                **Recommended** over ``allow_file_protocol=True`` because access
                is restricted to the specified directory only.

            allow_file_protocol: Enable file:// protocol support (default: False).
                **WARNING**: Enabling this allows access to ANY file on the system
                that the process can read. Only use with trusted content.
                Prefer using ``asset_root`` for secure local resource loading.

            frameless: Enable frameless window mode (default: False).
                When True, the window will have no title bar or borders.
                You'll need to implement custom window controls (close, minimize,
                maximize) and drag behavior in your frontend.

                **Frontend drag support**: Use CSS ``-webkit-app-region: drag``
                on elements that should be draggable, and ``no-drag`` on
                interactive elements within the drag region.

            transparent: Enable transparent window background (default: False).
                When True, the window background will be transparent, allowing
                the WebView content to define its own background.

                **Requirements**:
                - Your HTML/CSS must also have transparent background
                - WebView2 Runtime must support transparency
                - May have performance implications on some systems

            background_color: Custom background color for the WebView (optional).
                Format: CSS color string (e.g., "rgba(0,0,0,0)" for transparent,
                "#1a1a2e" for dark blue). If not specified, uses the default
                dark theme color.

            embed_mode: WebView embedding mode (default: "child").
                Controls how the WebView window relates to the parent Qt widget:

                - "child": WS_CHILD mode - WebView is a real child window embedded
                  inside this QWidget. Best for tight integration but may cause
                  UI freezes in some DCC environments due to message pump conflicts.

                - "owner": GWLP_HWNDPARENT mode - WebView is an owned independent
                  window. Cross-thread safe, recommended for DCC tools that
                  experience freezing with "child" mode. The window follows the
                  parent's minimize/restore behavior.

                - "none": No parent relationship at WebView level. The window is
                  completely independent. Not recommended for DCC integration as
                  the window won't follow parent's lifecycle.

                **Recommendation**: Use "owner" mode if you experience UI freezes
                with the default "child" mode in Maya, Houdini, or other DCCs.

            ipc_batch_size: Maximum number of IPC messages to process per tick
                (default: 0 = unlimited). Use this to limit message processing
                in DCC environments where the main thread is busy (e.g., Houdini
                during cooking). Recommended values:

                - 0: Unlimited (Maya, Unreal)
                - 5: Houdini (slow main thread)
                - 10: Nuke, 3ds Max

            auto_prewarm: Automatically trigger WebView2 pre-warming on first
                instantiation (default: True). This provides ~50% faster WebView
                creation by initializing the WebView2 Runtime in advance.

                The pre-warming is:
                - Idempotent: Safe to call multiple times
                - Non-blocking: Runs synchronously but fast
                - One-time: Only triggers on first QtWebView creation

                Set to False if you want explicit control via WebViewPool.prewarm().
        """
        # Auto-prewarm on first instantiation (if enabled and not already done)
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

        self._title = title
        self._width = width
        self._height = height
        self._dev_tools = dev_tools
        self._context_menu = context_menu
        self._asset_root = asset_root  # Store for load_file to use auroraview:// protocol
        self._frameless = frameless
        self._transparent = transparent
        self._embed_mode = embed_mode
        self._initial_url = url
        self._initial_html = html

        self.setWindowTitle(title)
        self.resize(width, height)

        # Apply frameless window flags if requested
        if frameless:
            # Remove window decorations but keep it as a proper window
            self.setWindowFlags(Qt.Window | Qt.FramelessWindowHint)
            if _VERBOSE_LOGGING:
                logger.info("QtWebView: Frameless mode enabled")

        # Apply transparent background if requested
        if transparent:
            self.setAttribute(Qt.WA_TranslucentBackground, True)
            self.setAttribute(Qt.WA_NoSystemBackground, True)
            self.setStyleSheet("background: transparent;")
            if _VERBOSE_LOGGING:
                logger.info("QtWebView: Transparent background enabled")
        else:
            # Ensure no borders or margins on the QtWebView widget itself
            self.setStyleSheet("background: #0d0d0d; border: none; margin: 0; padding: 0;")
            self.setContentsMargins(0, 0, 0, 0)

        # We host a native window inside this QWidget using createWindowContainer.
        # This allows the WebView to participate in Qt's layout system automatically!
        self.setAttribute(Qt.WA_NativeWindow, True)
        self.setAttribute(Qt.WA_DeleteOnClose, True)

        # Get the Qt widget's HWND - needed to tell WebView.create() this is embedded mode
        # even though we'll use createWindowContainer for layout instead of Win32 parenting
        qt_hwnd = int(self.winId())

        # Resize throttling state - balanced for 60 FPS (avoid UI blocking)
        self._last_resize_time = 0
        self._resize_throttle_ms = 16  # ~60fps (16.67ms per frame)
        self._pending_resize = None
        self._last_emitted_size = (0, 0)  # Track last emitted size to avoid duplicates

        # Store embed mode for geometry synchronization
        # Supported modes:
        # - "child": WS_CHILD mode - WebView is a real child window, cannot be dragged independently
        # - "owner": GWLP_HWNDPARENT mode - owned window, can be dragged (not recommended)
        # - "container": Qt createWindowContainer mode - standalone but non-blocking
        self._embed_mode = embed_mode

        # For all embedded modes, ALWAYS disable window decorations (title bar, borders)
        # The WebView should appear seamlessly embedded in the Qt widget
        # This is critical for proper Qt integration - the WebView must be frameless

        # Create the core WebView with the specified embed mode
        # - parent=qt_hwnd tells WebView.create() this is embedded mode (enables auto_timer, etc.)
        # - mode controls how the WebView window relates to the parent:
        #   - "child": WS_CHILD window, fully embedded, cannot be dragged independently
        #   - "owner": Owned window, cross-thread safe but can be dragged
        #   - "container": Standalone window for Qt createWindowContainer
        # - frame=False: ALWAYS frameless for embedded mode (critical!)
        self._webview = WebView.create(
            title=title,
            width=width,
            height=height,
            parent=qt_hwnd,  # Tell WebView.create() this is embedded mode
            mode=embed_mode,  # Use the specified embed mode
            frame=False,  # ALWAYS frameless for embedded WebView
            debug=dev_tools,
            context_menu=context_menu,
            asset_root=asset_root,
            allow_file_protocol=allow_file_protocol,
            auto_show=False,  # Don't auto-show, we control visibility
            auto_timer=True,
            transparent=transparent,
            background_color=background_color,
            ipc_batch_size=ipc_batch_size,  # Max messages per tick (0=unlimited)
        )

        # Track cleanup state so we can make close idempotent.
        self._is_closing = False

        # Set up Qt event processor (strategy pattern)
        # This ensures Qt events are processed along with WebView events
        self._event_processor = QtEventProcessor(self._webview)
        self._webview.set_event_processor(self._event_processor)

        # Create Qt layout with QStackedWidget for page management
        # Pages: 0=Loading, 1=WebView, 2=Error (future)
        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0, 0, 0, 0)
        self._layout.setSpacing(0)

        # QStackedWidget manages page switching with automatic size sync
        self._stack = QStackedWidget()
        self._layout.addWidget(self._stack)

        # Page 0: Loading page (shown during initialization)
        self._loading_page = self._create_loading_page()
        self._stack.addWidget(self._loading_page)

        # Page 1: WebView page (will contain the container)
        self._webview_page = QWidget()
        self._webview_page_layout = QVBoxLayout(self._webview_page)
        self._webview_page_layout.setContentsMargins(0, 0, 0, 0)
        self._webview_page_layout.setSpacing(0)
        self._stack.addWidget(self._webview_page)

        # Start with loading page visible
        self._stack.setCurrentIndex(0)

        # Container will be created in show() after WebView HWND is available
        self._webview_container = None
        self._webview_qwindow = None

        # Install event filter on parent window to track:
        # 1. Window moves (for owner mode positioning)
        # 2. Window close (to close WebView when parent closes)
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
                "QtWebView created: %s (%sx%s, mode=%s, frameless=%s, transparent=%s, container=%s)",
                title,
                width,
                height,
                embed_mode,
                frameless,
                transparent,
                "createWindowContainer" if self._webview_container else "manual",
            )

    def _create_loading_page(self) -> QWidget:
        """Create the loading page widget for QStackedWidget.

        This page is shown during WebView initialization and provides
        a dark background matching the WebView theme with a loading message.

        Returns:
            QWidget: The loading page widget.
        """
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
        """Set up event handlers to bridge WebView events to Qt signals.

        This connects internal WebView events to Qt signal emissions,
        enabling standard Qt signal/slot connections.
        """

        # Navigation started
        @self._webview.on("navigation_started")
        def on_nav_started(data):
            url = data.get("url", "") if data else ""
            self._qt_signal_state["is_loading"] = True
            self._qt_signal_state["load_progress"] = 0
            self.loadStarted.emit()
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        # Navigation finished / Load finished
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

        # Load progress
        @self._webview.on("load_progress")
        def on_load_progress(data):
            progress = data.get("progress", 0) if data else 0
            progress = max(0, min(100, int(progress)))
            if progress != self._qt_signal_state["load_progress"]:
                self._qt_signal_state["load_progress"] = progress
                self.loadProgress.emit(progress)

        # Title changed
        @self._webview.on("title_changed")
        def on_title_changed(data):
            title = data.get("title", "") if data else ""
            if title and title != self._qt_signal_state["current_title"]:
                self._qt_signal_state["current_title"] = title
                self.titleChanged.emit(title)

        # URL changed (explicit)
        @self._webview.on("url_changed")
        def on_url_changed(data):
            url = data.get("url", "") if data else ""
            if url and url != self._qt_signal_state["current_url"]:
                self._qt_signal_state["current_url"] = url
                self.urlChanged.emit(url)

        # ========== Error handling signals ==========

        # JavaScript errors
        @self._webview.on("js_error")
        def on_js_error(data):
            if data:
                message = data.get("message", "Unknown error")
                line = data.get("line", 0)
                source = data.get("source", "")
                self.jsError.emit(message, line, source)

        # Console messages
        @self._webview.on("console_message")
        def on_console_message(data):
            if data:
                level = data.get("level", 0)  # 0=log, 1=warning, 2=error
                message = data.get("message", "")
                line = data.get("line", 0)
                source = data.get("source", "")
                self.consoleMessage.emit(level, message, line, source)

        # Render process terminated
        @self._webview.on("render_process_terminated")
        def on_render_terminated(data):
            if data:
                status = data.get("status", 0)
                exit_code = data.get("exit_code", 0)
                self.renderProcessTerminated.emit(status, exit_code)

        # ========== IPC signals ==========

        # Note: IPC message forwarding is handled via _forward_ipc_to_signal()
        # which wraps user-registered callbacks to also emit Qt signals.
        # This allows users to use both callback style and Qt signal/slot style.

        # ========== Selection signals ==========

        @self._webview.on("selection_changed")
        def on_selection_changed(data):
            self.selectionChanged.emit()

        # ========== Icon signals ==========

        @self._webview.on("icon_changed")
        def on_icon_changed(data):
            self.iconChanged.emit()
            if data:
                url = data.get("url", "")
                if url:
                    self.iconUrlChanged.emit(url)

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView: Signal bridge initialized (with extended signals)")

    @classmethod
    def create_deferred(
        cls,
        parent=None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        dev_tools: bool = True,
        context_menu: bool = True,
        asset_root: Optional[str] = None,
        allow_file_protocol: bool = False,
        frameless: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        embed_mode: str = "child",
        on_ready: Optional[Callable[["QtWebView"], None]] = None,
        on_error: Optional[Callable[[str], None]] = None,
        ipc_batch_size: int = 0,
        auto_prewarm: bool = True,
    ) -> QWidget:
        """Create QtWebView with deferred initialization for DCC environments.

        This method immediately returns a placeholder widget with a loading
        indicator, then schedules the actual WebView creation using QTimer.
        This allows the DCC UI to remain responsive while showing user feedback.

        **Important**: WebView2 initialization still blocks briefly (~200-500ms)
        because it must run on the main thread. However, this method:

        1. Shows loading feedback immediately
        2. Lets the event loop process pending events before initialization
        3. Provides callbacks for completion and error handling

        For truly non-blocking initialization, consider pre-warming WebView2
        during application startup.

        Args:
            parent: Parent Qt widget
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            dev_tools: Enable developer tools
            context_menu: Enable native context menu
            asset_root: Root directory for auroraview:// protocol
            allow_file_protocol: Enable file:// protocol support
            frameless: Enable frameless window mode
            transparent: Enable transparent window background
            background_color: Custom background color
            embed_mode: WebView embedding mode (default: "child" for proper embedding).
                Use "child" to prevent WebView from being dragged independently.
                Use "owner" if you experience UI freezes with "child" mode.
            on_ready: Callback invoked with QtWebView instance when ready
            on_error: Callback invoked with error message if creation fails
            ipc_batch_size: Max IPC messages per tick (0=unlimited, 5 for Houdini)
            auto_prewarm: Automatically trigger WebView2 pre-warming (default: True)

        Returns:
            A placeholder QWidget that shows loading indicator initially.

        Example:
            >>> def on_ready(webview):
            ...     webview.load_url("http://localhost:3000")
            ...     webview.show()
            ...
            >>> QtWebView.create_deferred(
            ...     parent=maya_main_window(),
            ...     on_ready=on_ready,
            ... )
        """
        # Create placeholder widget immediately (non-blocking)
        placeholder = QWidget(parent)
        placeholder.setWindowTitle(title)
        placeholder.resize(width, height)
        placeholder.setAttribute(Qt.WA_NativeWindow, True)

        # Add loading indicator
        layout = QVBoxLayout(placeholder)
        layout.setContentsMargins(0, 0, 0, 0)
        loading_label = QLabel("Loading WebView...")
        loading_label.setAlignment(Qt.AlignCenter)
        loading_label.setStyleSheet("QLabel { color: #888; font-size: 14px; background: #1a1a2e; }")
        layout.addWidget(loading_label)

        if _VERBOSE_LOGGING:
            logger.debug(
                "QtWebView.create_deferred: Created placeholder, scheduling WebView creation"
            )

        def do_create():
            """Create WebView on main thread (deferred via QTimer)."""
            try:
                # Process any pending Qt events first
                QCoreApplication.processEvents()

                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView.create_deferred: Creating WebView")
                start_time = time.time()

                # Create the actual QtWebView
                webview_widget = cls(
                    parent=parent,
                    title=title,
                    width=width,
                    height=height,
                    dev_tools=dev_tools,
                    context_menu=context_menu,
                    asset_root=asset_root,
                    allow_file_protocol=allow_file_protocol,
                    frameless=frameless,
                    transparent=transparent,
                    background_color=background_color,
                    embed_mode=embed_mode,
                    ipc_batch_size=ipc_batch_size,
                    auto_prewarm=auto_prewarm,
                )

                elapsed = time.time() - start_time
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView.create_deferred: WebView created in %.3fs", elapsed)

                # Hide placeholder
                placeholder.hide()

                # Invoke callback
                if on_ready:
                    on_ready(webview_widget)

            except Exception as e:
                logger.error("QtWebView.create_deferred: Failed - %s", e)
                loading_label.setText(f"Error: {e}")
                if on_error:
                    on_error(str(e))

        # Schedule creation after a short delay to let UI update
        # Use 0ms to run as soon as event loop is free
        QTimer.singleShot(0, do_create)

        return placeholder

    # ------------------------------------------------------------------
    # High-level AuroraView-compatible API (delegated to WebView)
    # ------------------------------------------------------------------

    def load_url(self, url: str) -> None:
        """Load a URL into the embedded WebView."""
        self._webview.load_url(url)
        logger.debug("QtWebView loading URL: %s", url)

    def load_html(self, html: str) -> None:
        """Load HTML content into the embedded WebView.

        This is a thin pass-through to :meth:`WebView.load_html`, which
        accepts only the HTML string. If you need to load a static HTML
        file together with its local assets (images/CSS/JS), prefer
        :meth:`load_url` with a ``file:///`` URL instead of relying on a
        ``base_url`` argument.
        """
        self._webview.load_html(html)
        logger.debug("QtWebView loading HTML (%s bytes)", len(html))

    def load_file(self, path: Any) -> None:
        """Load a local HTML file in embedded Qt/DCC mode.

        When ``asset_root`` is set during construction, this method uses the
        ``auroraview://`` custom protocol to load the HTML file. This allows
        relative asset paths (CSS, JS, images) to work correctly because
        the page origin is ``auroraview://`` rather than ``null``.

        When ``asset_root`` is NOT set, the method falls back to reading the
        file content and loading via :meth:`load_html`, which works for
        single-file HTML (no external assets).

        Args:
            path: Path to the HTML file to load.

        Example::

            # Best practice: use asset_root for multi-file frontends
            webview = QtWebView(parent, asset_root="/path/to/dist")
            webview.load_file("/path/to/dist/index.html")
            # Loads via auroraview://index.html, assets resolve correctly

            # Alternative: single-file HTML (all assets inlined)
            webview = QtWebView(parent)
            webview.load_file("/path/to/bundled.html")
            # Loads via load_html(), works for self-contained HTML
        """
        html_path = Path(path).expanduser().resolve()

        # If asset_root is configured, use auroraview:// protocol for proper origin
        if self._asset_root:
            asset_root_path = Path(self._asset_root).expanduser().resolve()
            try:
                # Get relative path from asset_root to the HTML file
                relative_path = html_path.relative_to(asset_root_path)
                # Use forward slashes for URL
                url_path = str(relative_path).replace("\\", "/")
                # Windows: wry maps custom protocols to https://<scheme>.<host>/<path>
                #          We use "localhost" as host to keep path structure correct
                #          So https://auroraview.localhost/index.html -> relative ./assets/x
                #          resolves to https://auroraview.localhost/assets/x
                # macOS/Linux: wry uses <scheme>://<path>
                if sys.platform == "win32":
                    # On Windows, custom protocol "auroraview" becomes https://auroraview.xxx
                    # We use "localhost" as host, then path follows
                    # This ensures relative paths resolve correctly
                    auroraview_url = f"https://auroraview.localhost/{url_path}"
                else:
                    auroraview_url = f"auroraview://{url_path}"
                if _VERBOSE_LOGGING:
                    logger.debug(
                        "QtWebView loading via auroraview protocol: %s (asset_root: %s)",
                        auroraview_url,
                        asset_root_path,
                    )
                self.load_url(auroraview_url)
                return
            except ValueError:
                # HTML file is not under asset_root, log warning and continue
                logger.warning(
                    "HTML file %s is not under asset_root %s, falling back to load_html",
                    html_path,
                    asset_root_path,
                )

        # Fallback: read file and load via load_html (for single-file HTML)
        try:
            html = html_path.read_text(encoding="utf-8")
            self.load_html(html)
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView loaded HTML from file via load_html(): %s", html_path)
        except Exception:
            # Last resort: use the underlying WebView.load_file implementation
            load_file = getattr(self._webview, "load_file", None)
            if callable(load_file):
                load_file(path)
            else:  # pragma: no cover - defensive, for older backends
                self.load_url(html_path.as_uri())

    def eval_js(self, script: str) -> None:
        """Execute JavaScript in the embedded WebView.

        Note: Event processing is automatic via _post_eval_js_hook.
        You don't need to manually call process_events().
        """
        self._webview.eval_js(script)

    def emit(self, event_name: str, data: Any = None, auto_process: bool = True) -> None:
        """Emit an AuroraView event to the embedded WebView.

        Note: Event processing is automatic via _auto_process_events override.

        Args:
            event_name: Name of the event
            data: Data to send with the event
            auto_process: Automatically process events (default: True)
        """
        # Call parent implementation (which will call _auto_process_events)
        self._webview.emit(event_name, data, auto_process=auto_process)

    def on(self, event_name: str) -> Callable:
        """Decorator to register event handler with Qt signal emission.

        This wraps the callback to also emit ipcMessageReceived signal,
        allowing both callback-style and Qt signal/slot-style event handling.

        Example:
            >>> @webview.on("my_event")
            >>> def handle(data):
            ...     print(data)  # Callback style
            >>>
            >>> # Or use Qt signals
            >>> webview.ipcMessageReceived.connect(lambda e, d: print(e, d))
        """

        def decorator(func: Callable) -> Callable:
            # Create wrapper that also emits Qt signal
            def wrapper(data):
                # Emit Qt signal first
                self.ipcMessageReceived.emit(event_name, data)
                # Then call user callback
                return func(data)

            # Register wrapped callback
            self._webview.register_callback(event_name, wrapper)
            return func

        return decorator

    def register_callback(self, event_name: str, callback: Callable) -> None:
        """Register a callback for an event with Qt signal emission.

        This wraps the callback to also emit ipcMessageReceived signal.
        """

        # Create wrapper that also emits Qt signal
        def wrapper(data):
            # Emit Qt signal first
            self.ipcMessageReceived.emit(event_name, data)
            # Then call user callback
            return callback(data)

        self._webview.register_callback(event_name, wrapper)

    # Window event convenience methods (delegate to underlying WebView)
    def on_shown(self, callback: Callable) -> Callable:
        """Register a callback for when the window becomes visible."""
        return self._webview.on_shown(callback)

    def on_closing(self, callback: Callable) -> Callable:
        """Register a callback for before the window closes."""
        return self._webview.on_closing(callback)

    def on_closed(self, callback: Callable) -> Callable:
        """Register a callback for after the window has closed."""
        return self._webview.on_closed(callback)

    def on_resized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is resized."""
        return self._webview.on_resized(callback)

    def on_moved(self, callback: Callable) -> Callable:
        """Register a callback for when the window is moved."""
        return self._webview.on_moved(callback)

    def on_focused(self, callback: Callable) -> Callable:
        """Register a callback for when the window gains focus."""
        return self._webview.on_focused(callback)

    def on_blurred(self, callback: Callable) -> Callable:
        """Register a callback for when the window loses focus."""
        return self._webview.on_blurred(callback)

    def on_minimized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is minimized."""
        return self._webview.on_minimized(callback)

    def on_maximized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is maximized."""
        return self._webview.on_maximized(callback)

    def on_restored(self, callback: Callable) -> Callable:
        """Register a callback for when the window is restored from min/max."""
        return self._webview.on_restored(callback)

    # ------------------------------------------------------------------
    # State and Command API (delegate to underlying WebView)
    # ------------------------------------------------------------------

    @property
    def state(self):
        """Get the shared state container for Python ↔ JavaScript sync.

        Returns:
            State container with dict-like interface

        Example:
            >>> webview.state["user"] = {"name": "Alice"}
            >>> # JavaScript: auroraview.state.user.name === "Alice"
        """
        return self._webview.state

    @property
    def commands(self):
        """Get the command registry for Python ↔ JavaScript RPC.

        Returns:
            CommandRegistry instance

        Example:
            >>> webview.commands.register("greet", lambda name: f"Hello, {name}!")
        """
        return self._webview.commands

    def command(self, func_or_name=None):
        """Decorator to register a command callable from JavaScript.

        This is a convenience shortcut for `webview.commands.register`.

        Args:
            func_or_name: Either a function to register, or a string name.

        Example:
            >>> @webview.command
            ... def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
            >>>
            >>> @webview.command("custom_name")
            ... def my_func():
            ...     return "result"
        """
        return self._webview.command(func_or_name)

    @property
    def channels(self):
        """Get the channel manager for streaming data to JavaScript.

        Returns:
            ChannelManager instance

        Example:
            >>> with webview.create_channel("progress") as channel:
            ...     for i in range(100):
            ...         channel.send({"percent": i})
        """
        return self._webview.channels

    def create_channel(self, name: str):
        """Create a new channel for streaming data to JavaScript.

        Args:
            name: Channel name (used as event prefix)

        Returns:
            Channel instance (context manager)

        Example:
            >>> with webview.create_channel("download") as ch:
            ...     ch.send({"progress": 50})
        """
        return self._webview.create_channel(name)

    def bind_call(self, method: str, func: Optional[Callable[..., Any]] = None):
        """Bind a Python callable for ``auroraview.call`` (delegates to WebView)."""
        return self._webview.bind_call(method, func)

    def bind_api(self, api: Any, namespace: str = "api") -> None:
        """Bind an object's public methods as ``auroraview.api.*`` (delegates)."""
        self._webview.bind_api(api, namespace)

    @property
    def title(self) -> str:
        """Get window title."""
        return self.windowTitle()

    @title.setter
    def title(self, value: str) -> None:
        """Set window title (and keep underlying WebView title in sync)."""
        self._title = value
        self.setWindowTitle(value)
        try:
            # Best-effort sync; the WebView exposes title via logs/diagnostics.
            self._webview._title = value  # type: ignore[attr-defined]
        except Exception:
            pass

    # ------------------------------------------------------------------
    # Qt signal state properties
    # ------------------------------------------------------------------

    @property
    def current_url(self) -> str:
        """Get the current URL (from Qt signal state).

        This returns the last URL that triggered urlChanged signal.
        """
        return self._qt_signal_state.get("current_url", "")

    @property
    def current_title(self) -> str:
        """Get the current page title (from Qt signal state).

        This returns the last title that triggered titleChanged signal.
        """
        return self._qt_signal_state.get("current_title", "")

    @property
    def is_loading(self) -> bool:
        """Check if page is currently loading.

        Returns True between loadStarted and loadFinished signals.
        """
        return self._qt_signal_state.get("is_loading", False)

    @property
    def load_progress_value(self) -> int:
        """Get current load progress (0-100).

        This returns the last progress value from loadProgress signal.
        """
        return self._qt_signal_state.get("load_progress", 0)

    # ------------------------------------------------------------------
    # Qt integration helpers
    # ------------------------------------------------------------------

    def get_diagnostics(self) -> dict:
        """Get diagnostic information about event processing.

        Returns:
            Dictionary containing:
            - event_process_count: Number of times events have been processed
            - last_event_process_time: Timestamp of last event processing
            - has_post_eval_hook: Whether the automatic event processing hook is installed

        Example:
            >>> webview = QtWebView(title="My Tool")
            >>> # ... use the webview ...
            >>> diag = webview.get_diagnostics()
            >>> print(f"Events processed: {diag['event_process_count']}")
        """
        return {
            "event_processor_type": type(self._event_processor).__name__,
            "event_process_count": self._event_processor._process_count,
            "has_event_processor": self._webview._event_processor is not None,
            "processor_is_correct": isinstance(self._event_processor, QtEventProcessor),
        }

    def _sync_embedded_geometry(self) -> None:
        """Resize the embedded native WebView window to match this QWidget.

        When using createWindowContainer, Qt handles geometry automatically,
        so this method only needs to remove window borders on first call.

        For legacy mode (manual sync), this handles the full geometry update.
        """
        try:
            if sys.platform != "win32":
                return

            # When using createWindowContainer, Qt handles geometry automatically!
            # Border removal is done by prepare_hwnd_for_container() in
            # _create_webview_container(), so nothing else to do here.
            # Note: Legacy manual geometry sync mode is no longer supported.
            pass
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView: failed to sync embedded geometry: %s", e)

    def _create_webview_container(self, core, hwnd=None) -> None:
        """Create Qt container for WebView after WebView is initialized.

        This method supports two embedding modes:
        1. Direct embedding (Qt6 preferred): Uses SetParent() directly, bypassing
           createWindowContainer which has known issues on Qt6.
        2. createWindowContainer (Qt5 fallback): Uses Qt's native container mechanism.

        The mode is automatically selected based on:
        - Qt version (Qt6 prefers direct embedding)
        - Platform support (Windows required for direct embedding)
        - Environment variable override: AURORAVIEW_USE_DIRECT_EMBED=1/0

        Uses the Qt compatibility layer to handle differences between
        Qt5 (PySide2) and Qt6 (PySide6).
        """
        try:
            if hwnd is not None:
                webview_hwnd = hwnd
            else:
                get_hwnd = getattr(core, "get_hwnd", None)
                webview_hwnd = get_hwnd() if callable(get_hwnd) else None

            if not webview_hwnd:
                logger.warning("[QtWebView] No HWND available for container")
                return

            # Log Qt version info for debugging
            qt_binding, qt_version = get_qt_info()
            if _VERBOSE_LOGGING:
                logger.debug(
                    f"[QtWebView] Creating container for HWND=0x{webview_hwnd:X} "
                    f"(Qt binding: {qt_binding}, version: {qt_version})"
                )

            # Determine embedding mode:
            # - Environment variable override
            # - Default: Use direct embedding on Qt6 if supported
            env_direct = os.environ.get("AURORAVIEW_USE_DIRECT_EMBED", "").lower()
            if env_direct in ("1", "true", "yes", "on"):
                use_direct_embed = True
            elif env_direct in ("0", "false", "no", "off"):
                use_direct_embed = False
            else:
                # Auto-detect: Use direct embedding on Qt6 if platform supports it
                use_direct_embed = is_qt6() and supports_direct_embedding()

            if use_direct_embed:
                self._create_container_direct(webview_hwnd)
            else:
                self._create_container_qt(webview_hwnd)

        except Exception as e:
            logger.exception(f"[QtWebView] Failed to create container: {e}")
            self._webview_container = None

    def _create_container_direct(self, webview_hwnd: int) -> None:
        """Create container using direct SetParent embedding (bypasses createWindowContainer).

        This mode is preferred for Qt6 where createWindowContainer has known issues
        with WebView2. It uses Win32 SetParent() directly to establish the parent-child
        relationship.

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        logger.info(f"[QtWebView] Using DIRECT embedding mode for HWND=0x{webview_hwnd:X}")

        # Get our widget's HWND
        parent_hwnd = int(self.winId())
        if not parent_hwnd:
            logger.error("[QtWebView] Failed to get parent widget HWND")
            self._create_container_qt(webview_hwnd)  # Fallback
            return

        # Get initial size
        size = self.size()
        width = size.width() if size.width() > 0 else 800
        height = size.height() if size.height() > 0 else 600

        # Use direct embedding via platform backend
        success = embed_window_directly(webview_hwnd, parent_hwnd, width, height)
        if not success:
            logger.warning(
                "[QtWebView] Direct embedding failed, falling back to createWindowContainer"
            )
            self._create_container_qt(webview_hwnd)
            return

        # Mark that we're using direct embedding mode
        self._using_direct_embed = True
        self._direct_embed_hwnd = webview_hwnd

        # Create a placeholder widget to participate in Qt layout
        # This widget doesn't actually contain the WebView, but helps with layout
        self._webview_container = QWidget(self)
        self._webview_container.setStyleSheet(
            "border: none; margin: 0; padding: 0; background-color: transparent;"
        )
        self._webview_page.setStyleSheet("background-color: #0d0d0d;")

        # Add placeholder to layout
        self._webview_page_layout.addWidget(self._webview_container, 1)

        # Apply clip styles to parent
        apply_clip_styles_to_parent(parent_hwnd)

        # Finalize anti-flicker optimizations
        core = getattr(self._webview, "_core", None)
        if core is not None:
            finalize_fn = getattr(core, "finalize_container_embedding", None)
            if callable(finalize_fn):
                try:
                    finalize_fn()
                except Exception:
                    pass

        # Fix WebView2 child windows - IMMEDIATELY and with DELAYED retries
        # WebView2 creates child windows asynchronously, so we need multiple attempts
        self._schedule_child_window_fixes(webview_hwnd)

        logger.info(f"[QtWebView] Direct embedding successful: HWND=0x{webview_hwnd:X}")

    def _schedule_child_window_fixes(self, webview_hwnd: int) -> None:
        """Schedule multiple attempts to fix WebView2 child windows.

        WebView2 creates child windows (Chrome_WidgetWin_0, etc.) asynchronously
        after the main window is created. We need to fix them multiple times
        to catch all of them as they're created.

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        from auroraview.integration.qt.platforms import get_backend

        def fix_children():
            """Fix all child windows."""
            try:
                backend = get_backend()
                if hasattr(backend, "_fix_all_child_windows_recursive"):
                    count = backend._fix_all_child_windows_recursive(webview_hwnd)
                    if count > 0:
                        logger.info(f"[QtWebView] Fixed {count} WebView2 child windows")
            except Exception as e:
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] fix_children failed: {e}")

        # Fix immediately
        fix_children()

        # Schedule delayed fixes to catch asynchronously created child windows
        # WebView2 creates windows at various times during initialization
        delays = [50, 100, 200, 500, 1000, 2000]
        for delay in delays:
            QTimer.singleShot(delay, fix_children)

    def _create_container_qt(self, webview_hwnd: int) -> None:
        """Create container using Qt's createWindowContainer.

        This is the traditional embedding mode that works well on Qt5.
        On Qt6, it may have issues with WebView2 (white frames, dragging problems).

        Args:
            webview_hwnd: The WebView's native window handle.
        """
        logger.info(f"[QtWebView] Using createWindowContainer mode for HWND=0x{webview_hwnd:X}")

        self._using_direct_embed = False

        # Step 1: Prepare HWND using compat layer (handles Qt5/Qt6 differences)
        prepare_hwnd_for_container(webview_hwnd)

        # Step 2: Wrap the native HWND as a QWindow
        self._webview_qwindow = QWindow.fromWinId(webview_hwnd)
        if self._webview_qwindow is None:
            logger.error("[QtWebView] QWindow.fromWinId returned None")
            return

        if _VERBOSE_LOGGING:
            logger.debug("[QtWebView] QWindow created from HWND")

        # Step 3: Create container using compat layer (handles Qt5/Qt6 differences)
        self._webview_container = create_container_widget(
            self._webview_qwindow,
            self,
            focus_policy="strong",
        )
        if self._webview_container is None:
            logger.error("[QtWebView] create_container_widget returned None")
            return

        # Ensure container has minimal styling - no borders or margins
        self._webview_container.setStyleSheet(
            "border: none; margin: 0; padding: 0; background-color: #0d0d0d;"
        )
        self._webview_page.setStyleSheet("background-color: #0d0d0d;")

        # Step 4: Add container to webview page layout
        self._webview_page_layout.addWidget(self._webview_container, 1)

        # Step 5: Apply clip styles to QtWebView widget
        if sys.platform == "win32":
            self_hwnd = int(self.winId())
            if self_hwnd:
                apply_clip_styles_to_parent(self_hwnd)

        # Step 6: Finalize anti-flicker optimizations
        core = getattr(self._webview, "_core", None)
        if core is not None:
            finalize_fn = getattr(core, "finalize_container_embedding", None)
            if callable(finalize_fn):
                try:
                    finalize_fn()
                    if _VERBOSE_LOGGING:
                        logger.debug("[QtWebView] Anti-flicker optimizations removed")
                except Exception as e:
                    if _VERBOSE_LOGGING:
                        logger.debug(f"[QtWebView] finalize_container_embedding failed: {e}")

        # Step 7: Post-container setup (handles Qt version quirks)
        post_container_setup(self._webview_container, webview_hwnd)

        # Step 8: Force container to fill parent layout immediately
        self._force_container_geometry()

        # Step 9: Fix WebView2 child windows for Qt6 compatibility
        if sys.platform == "win32":
            try:
                import auroraview

                fix_fn = getattr(auroraview, "fix_webview2_child_windows", None)
                if callable(fix_fn):
                    fix_fn(webview_hwnd)
                    if _VERBOSE_LOGGING:
                        logger.debug(
                            f"[QtWebView] Fixed WebView2 child windows for HWND=0x{webview_hwnd:X}"
                        )
            except Exception as e:
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] fix_webview2_child_windows failed: {e}")

        if _VERBOSE_LOGGING:
            logger.debug("[QtWebView] Container created successfully for HWND=0x%X", webview_hwnd)

    def resizeEvent(self, event) -> None:  # type: ignore[override]
        """Handle Qt widget resize.

        For direct embedding mode, we need to manually update the WebView geometry.
        For createWindowContainer mode, Qt handles geometry automatically but
        WebView2's internal controller bounds may need explicit sync.
        """
        super().resizeEvent(event)

        new_size = event.size()
        width = new_size.width()
        height = new_size.height()

        # Handle direct embedding mode - manually update WebView geometry
        if getattr(self, "_using_direct_embed", False):
            direct_hwnd = getattr(self, "_direct_embed_hwnd", None)
            if direct_hwnd:
                update_embedded_window_geometry(direct_hwnd, 0, 0, width, height)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Direct embed resize: {width}x{height}")

        # Force container to fill parent and sync WebView2 controller bounds
        # Use getattr for safety - _webview_container may not exist if __init__ failed early
        if getattr(self, "_webview_container", None) is not None:
            # Force container geometry to match our size
            container = self._webview_container
            container.setGeometry(0, 0, width, height)
            # Sync WebView2 controller bounds
            self._sync_webview2_controller_bounds()

    def moveEvent(self, event) -> None:  # type: ignore[override]
        """Handle Qt widget move.

        When using createWindowContainer, Qt handles position automatically.
        """
        super().moveEvent(event)
        # createWindowContainer handles all positioning - nothing to do here!

    def _sync_webview2_controller_bounds(self, force_width: int = 0, force_height: int = 0) -> None:
        """Sync WebView2 controller bounds with container size.

        This is needed because createWindowContainer only handles the native
        window position/size, but WebView2's controller may need explicit
        bounds update to render content correctly.

        This is especially important for:
        - Qt6 where controller bounds sometimes don't auto-sync
        - High DPI scenarios
        - Rapid resize events

        Args:
            force_width: If > 0, use this width instead of container size.
            force_height: If > 0, use this height instead of container size.
        """
        try:
            # Use getattr for safety - _webview_container may not exist if __init__ failed
            container = getattr(self, "_webview_container", None)
            if container is None:
                logger.debug("[QtWebView] _sync_webview2_controller_bounds: container is None")
                return

            # Get container size or use forced size
            if force_width > 0 and force_height > 0:
                width = force_width
                height = force_height
            else:
                container_size = container.size()
                width = container_size.width()
                height = container_size.height()

            if width <= 0 or height <= 0:
                logger.debug(
                    f"[QtWebView] _sync_webview2_controller_bounds: invalid size {width}x{height}"
                )
                return

            logger.info(
                f"[QtWebView] _sync_webview2_controller_bounds: syncing to {width}x{height}"
            )

            # Try to sync WebView2 controller bounds via Rust API
            core = getattr(self._webview, "_core", None)
            if core is not None:
                # Prefer sync_bounds for Qt6 compatibility (directly syncs wry WebView bounds)
                sync_bounds = getattr(core, "sync_bounds", None)
                if callable(sync_bounds):
                    try:
                        sync_bounds(width, height)
                        logger.info(
                            f"[QtWebView] sync_bounds({width}, {height}) called successfully"
                        )
                        return
                    except Exception as e:
                        logger.warning(f"[QtWebView] sync_bounds failed: {e}")
                else:
                    logger.warning("[QtWebView] sync_bounds not available on core")

                # Fallback to set_size (also calls sync_webview_bounds internally now)
                set_size = getattr(core, "set_size", None)
                if callable(set_size):
                    try:
                        set_size(width, height)
                        logger.info(
                            f"[QtWebView] Synced WebView2 bounds via set_size: {width}x{height}"
                        )
                    except Exception as e:
                        logger.warning(f"[QtWebView] set_size failed: {e}")
            else:
                logger.warning("[QtWebView] _sync_webview2_controller_bounds: core is None")

        except Exception as e:
            logger.warning(f"[QtWebView] _sync_webview2_controller_bounds failed: {e}")

    def _force_container_geometry(self) -> None:
        """Force container to fill parent layout immediately.

        Qt5-style minimal implementation.
        """
        try:
            from qtpy.QtWidgets import QApplication

            container = getattr(self, "_webview_container", None)
            if container is None:
                return

            # Get our size (the QtWebView widget size)
            our_size = self.size()
            width = our_size.width()
            height = our_size.height()

            if width <= 0 or height <= 0:
                return

            # Force container to fill our size
            container.setGeometry(0, 0, width, height)
            container.resize(width, height)

            # Also resize the QWindow if available
            qwindow = getattr(self, "_webview_qwindow", None)
            if qwindow is not None:
                try:
                    qwindow.resize(width, height)
                except Exception:
                    pass

            # Qt5-style: single processEvents
            QApplication.processEvents()

            # Sync WebView2 controller bounds
            self._sync_webview2_controller_bounds(width, height)

            if _VERBOSE_LOGGING:
                logger.debug(f"[QtWebView] Forced container geometry: {width}x{height}")

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[QtWebView] _force_container_geometry failed: {e}")

    def eventFilter(self, watched, event) -> bool:
        """Filter events from parent window.

        Only need to handle parent window close to cleanup properly.
        Position/resize are handled automatically by createWindowContainer.
        """
        # Use getattr for safety - _parent_window may not exist if __init__ failed early
        parent_window = getattr(self, "_parent_window", None)
        if watched == parent_window and parent_window is not None:
            # Close WebView when parent window closes
            if event.type() == QEvent.Close:
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView: Parent window closing")
                try:
                    if not getattr(self, "_is_closing", False):
                        self._is_closing = True
                        webview = getattr(self, "_webview", None)
                        if webview is not None:
                            webview.close()
                except Exception as e:  # pragma: no cover
                    if _VERBOSE_LOGGING:
                        logger.debug("QtWebView: error closing on parent close: %s", e)

        return super().eventFilter(watched, event)

    def show(self) -> None:  # type: ignore[override]
        """Show the Qt host widget and start the embedded WebView.

        Due to Rust WebView limitations (!Send), we must create and run
        the WebView on the main thread. We use progressive initialization
        with QApplication.processEvents() to keep the UI responsive.

        Anti-flicker strategy:
        1. Hide the Qt widget using WS_EX_LAYERED with zero alpha
        2. Show the widget (now invisible)
        3. Initialize WebView and create container
        4. Switch QStackedWidget from loading page to webview page
        5. Restore visibility after everything is ready

        This prevents the white flash and visible embedding process.
        """
        from qtpy.QtWidgets import QApplication

        self._show_start_time = time.time()
        if _VERBOSE_LOGGING:
            logger.debug("[QtWebView] show() started with anti-flicker")

        # Step 1: Hide the window before showing (anti-flicker)
        # This makes the window completely invisible during initialization
        self._pre_show_hidden = False
        if sys.platform == "win32":
            # Ensure native window handle exists
            self.setAttribute(Qt.WA_NativeWindow, True)
            qt_hwnd = int(self.winId())
            if qt_hwnd:
                self._pre_show_hidden = hide_window_for_init(qt_hwnd)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Pre-show hidden applied: HWND=0x{qt_hwnd:X}")

        # Ensure QStackedWidget shows loading page
        self._stack.setCurrentIndex(0)

        # Step 2: Show the Qt widget (invisible due to zero alpha)
        super().show()

        # Process events to ensure the widget geometry is established
        QApplication.processEvents()

        # Step 3: Initialize WebView with progressive event processing
        self._init_webview_progressive()

    def _init_webview_progressive(self) -> None:
        """Initialize WebView on main thread with progressive event processing.

        This keeps the DCC UI responsive by processing Qt events between
        initialization steps. WebView2 creation still blocks briefly, but
        the UI doesn't appear frozen.
        """
        from qtpy.QtWidgets import QApplication

        start_time = getattr(self, "_show_start_time", time.time())

        # Step 1: Get the core WebView object
        core = getattr(self._webview, "_core", None)
        if core is None:
            logger.warning("[QtWebView] No core WebView available, using fallback")
            self._webview.show()
            return

        # Process events to keep UI responsive
        QApplication.processEvents()

        # Step 2: Create and show the embedded WebView in a non-blocking way.
        #
        # IMPORTANT:
        #   * We must NOT call core.show() on the Qt main thread when embedding
        #     into a DCC (Maya/PT). The Rust core's show() runs its own event
        #     loop and can block the host's Qt event loop for the entire
        #     lifetime of the window (Maya/PT UI完全卡死，心跳 QTimer 也不再触发).
        #   * Instead we use core.show_embedded(), which:
        #       - Creates the WebView / native window
        #       - Does NOT start its own event loop
        #       - Delegates message pumping to process_events()/process_ipc_only()
        #         (driven by our Qt EventTimer backend)
        #
        # This keeps the DCC UI responsive while still creating the underlying
        # WebView window so that get_hwnd() returns a valid handle for
        # _create_webview_container().
        embed_mode = getattr(self, "_embed_mode", None)
        show_embedded = getattr(core, "show_embedded", None)

        # Setup callback for event-driven initialization
        # This avoids RefCell borrow errors and race conditions by initializing
        # the container immediately when the HWND is created in Rust.
        setup_via_callback = False
        if hasattr(core, "set_on_hwnd_created"):

            def on_hwnd_created(hwnd):
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Rust callback: HWND created 0x{hwnd:X}")
                # Initialize container immediately (safe on main thread)
                self._create_webview_container(core, hwnd=hwnd)

            try:
                core.set_on_hwnd_created(on_hwnd_created)
                setup_via_callback = True
                if _VERBOSE_LOGGING:
                    logger.debug("[QtWebView] set_on_hwnd_created callback registered")
            except Exception as e:
                logger.warning(f"[QtWebView] Failed to set on_hwnd_created callback: {e}")

        try:
            if callable(show_embedded):
                core_show_start = time.time()
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[QtWebView] Calling core.show_embedded() for embed_mode={embed_mode!r}"
                    )
                show_embedded()
                core_show_time = (time.time() - core_show_start) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[QtWebView] core.show_embedded() returned in {core_show_time:.1f}ms"
                    )
            else:
                # Extremely unlikely with current Rust core, but keep a guarded
                # fallback for older versions.
                core_show_start = time.time()
                logger.warning(
                    "[QtWebView] core.show_embedded() not available; "
                    "falling back to core.show() (may block DCC UI!)"
                )
                core.show()
                core_show_time = (time.time() - core_show_start) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[QtWebView] core.show() fallback returned in {core_show_time:.1f}ms"
                    )
        except Exception as exc:
            # If show_embedded()/show() fails for some reason, fall back to the
            # high-level Python WebView.show() which will use the background
            # thread code-path. This is non-blocking for the DCC UI, even though
            # it does not use the new container embedding.
            logger.warning(
                f"[QtWebView] core.show_embedded()/core.show() failed ({exc}); "
                "falling back to WebView.show() (non-blocking background thread)"
            )
            self._webview.show()
            return

        # Process events after blocking operation
        QApplication.processEvents()

        # Step 3: Create Qt container for WebView
        # Now that WebView is created, we can get its HWND and wrap it with Qt's layout
        # If setup_via_callback is True, it was already called inside show_embedded()
        if not setup_via_callback:
            self._create_webview_container(core)

        QApplication.processEvents()

        # Step 4: Ensure WebView is visible after container creation
        # createWindowContainer may affect visibility, so we explicitly set it
        try:
            core.set_visible(True)
            # Process the visibility message immediately
            core.process_events()
            if _VERBOSE_LOGGING:
                logger.debug("[QtWebView] WebView visibility ensured after container creation")
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[QtWebView] Failed to set visibility: {e}")

        QApplication.processEvents()

        # Step 5: Switch from loading page to webview page
        # QStackedWidget handles the page transition cleanly
        self._stack.setCurrentIndex(1)

        # Step 6: Restore window visibility (anti-flicker completion)
        # Now that the container is ready and WebView is embedded, show the window
        if getattr(self, "_pre_show_hidden", False) and sys.platform == "win32":
            qt_hwnd = int(self.winId())
            if qt_hwnd:
                show_window_after_init(qt_hwnd)
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Restored visibility: HWND=0x{qt_hwnd:X}")
            self._pre_show_hidden = False

        QApplication.processEvents()

        # Step 7: Schedule delayed geometry sync for DCC apps
        # Some DCCs (especially Maya) need additional time for layout to stabilize
        # Qt6 requires more aggressive syncing due to delayed layout updates
        from qtpy.QtCore import QTimer

        def delayed_geometry_sync() -> None:
            """Sync geometry after layout has stabilized."""
            try:
                self._force_container_geometry()
                # Additional explicit bounds sync for Qt6
                self._sync_webview2_controller_bounds()
                if _VERBOSE_LOGGING:
                    logger.debug("[QtWebView] Delayed geometry sync completed")
            except Exception:
                pass

        # Schedule multiple syncs at different intervals for robustness
        # Qt6 needs more time for layout to stabilize
        QTimer.singleShot(50, delayed_geometry_sync)
        QTimer.singleShot(100, delayed_geometry_sync)
        QTimer.singleShot(250, delayed_geometry_sync)
        QTimer.singleShot(500, delayed_geometry_sync)
        QTimer.singleShot(1000, delayed_geometry_sync)  # Final sync for slow DCCs

        # Step 8: Load initial content (url or html)
        # This must happen after WebView is created and visible
        if self._initial_url:
            if _VERBOSE_LOGGING:
                logger.debug(f"[QtWebView] Loading initial URL: {self._initial_url}")
            self._webview.load_url(self._initial_url)
        elif self._initial_html:
            if _VERBOSE_LOGGING:
                logger.debug(f"[QtWebView] Loading initial HTML ({len(self._initial_html)} bytes)")
            self._webview.load_html(self._initial_html)

        # Step 9: Start EventTimer for message processing
        timer = getattr(self._webview, "_auto_timer", None)
        if timer is not None:
            try:
                timer.start()
                total_time = (time.time() - start_time) * 1000
                if _VERBOSE_LOGGING:
                    logger.debug(f"[QtWebView] Ready in {total_time:.1f}ms")
                return
            except Exception as exc:
                logger.warning(f"[QtWebView] EventTimer failed ({exc}), using fallback")

        # Fallback
        self._webview.show()

    def closeEvent(self, event) -> None:  # type: ignore[override]
        """Handle Qt close event and cleanup embedded WebView."""
        if self._is_closing:
            event.accept()
            return

        if _VERBOSE_LOGGING:
            logger.debug("QtWebView closeEvent")
        self._is_closing = True

        try:
            # Close the WebView
            try:
                self._webview.close()
            except Exception as e:  # pragma: no cover - best-effort cleanup
                if _VERBOSE_LOGGING:
                    logger.debug("QtWebView: error closing embedded WebView: %s", e)
        finally:
            event.accept()
            super().closeEvent(event)

    def __del__(self) -> None:
        """Destructor – ensure cleanup if the widget is GC'ed unexpectedly."""
        try:
            if not getattr(self, "_is_closing", False) and hasattr(self, "_webview"):
                self._webview.close()
        except Exception as e:  # pragma: no cover - best-effort cleanup
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView __del__ error: %s", e)

    def __repr__(self) -> str:
        """String representation."""
        try:
            return f"QtWebView(title='{self.windowTitle()}', size={self.width()}x{self.height()})"
        except RuntimeError:  # pragma: no cover - widget already deleted
            return "QtWebView(<deleted>)"

    def get_hwnd(self) -> Optional[int]:
        """Get the native window handle (HWND) of the embedded WebView.

        This is useful for integrating with external applications that need
        the native window handle, such as:
        - Unreal Engine: `unreal.parent_external_window_to_slate(hwnd)`
        - Windows API: Direct window manipulation
        - Other DCC tools with HWND-based integration

        Returns:
            int: The native window handle (HWND), or None if not available.

        Example:
            >>> qt_webview = QtWebView(...)
            >>> qt_webview.show()
            >>> hwnd = qt_webview.get_hwnd()
            >>> if hwnd:
            ...     print(f"WebView HWND: 0x{hwnd:x}")
            ...     # Use with Unreal Engine
            ...     # unreal.parent_external_window_to_slate(hwnd)
        """
        try:
            return self._webview.get_hwnd()
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug("QtWebView.get_hwnd() error: %s", e)
            return None


__all__ = ["QtWebView", "QtEventProcessor"]
