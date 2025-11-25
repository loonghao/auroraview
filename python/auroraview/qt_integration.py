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
import sys
import time
from pathlib import Path
from typing import Any, Callable, Optional

# Windows-specific imports for HWND manipulation
# Only used in _sync_embedded_geometry() which has platform check
if sys.platform == "win32":
    import ctypes
    from ctypes import wintypes

try:
    from qtpy.QtCore import QCoreApplication, Qt, QTimer
    from qtpy.QtWidgets import QWidget
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

from .webview import WebView

logger = logging.getLogger(__name__)


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
        """
        self._process_count += 1

        try:
            # Step 1: Process Qt events
            QCoreApplication.processEvents()

            # Step 2: Process WebView message queue
            # This is CRITICAL - without this, eval_js/emit messages stay in queue
            self._webview._core.process_events()
        except Exception as e:  # pragma: no cover - best-effort only
            logger.debug(f"QtEventProcessor: Event processing failed: {e}")


class QtWebView(QWidget):
    """Qt host widget that embeds the core AuroraView :class:`WebView`.

    This replaces the previous QWebEngine-based implementation. From the
    outside it still behaves like a regular ``QWidget`` with a compatible
    high-level API (``load_url``, ``load_html``, ``eval_js``, ``emit``,
    ``on``, ``bind_call``, ``bind_api``, ``title``), but all real browser
    work is handled by the Rust/WebView2 backend.

    The goal of this design is:

    * Keep the public Python API stable for existing tools
    * Remove the duplicated JavaScript bridge and QWebChannel wiring
    * Let the core :class:`WebView` own the window and IPC lifecycle
    """

    def __init__(
        self,
        parent=None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        dev_tools: bool = True,
        context_menu: bool = True,
    ) -> None:
        super().__init__(parent)

        self._title = title
        self._width = width
        self._height = height
        self._dev_tools = dev_tools
        self._context_menu = context_menu

        self.setWindowTitle(title)
        self.resize(width, height)

        # We host a native child window (HWND on Windows) inside this QWidget.
        # This lets the Rust/WebView2 backend render directly into the Qt
        # widget without relying on QWebEngine.
        self.setAttribute(Qt.WA_NativeWindow, True)
        self.setAttribute(Qt.WA_DeleteOnClose, True)

        # Native handle used by the WebView backend for embedding.
        hwnd = int(self.winId())
        logger.debug("QtWebView host widget created, hwnd=%s", hwnd)

        # Resize throttling state - balanced for 60 FPS (avoid UI blocking)
        self._last_resize_time = 0
        self._resize_throttle_ms = 16  # ~60fps (16.67ms per frame)
        self._pending_resize = None
        self._last_emitted_size = (0, 0)  # Track last emitted size to avoid duplicates

        # Create the core WebView in embedded/child mode.
        # In Qt/DCC environments (e.g., Maya) we run everything on the Qt
        # main thread and let the DCC's own event loop drive painting/input.
        # We therefore:
        #   * use EmbedMode::Child (real WS_CHILD window inside this QWidget)
        #   * disable native window decorations (frame=False) so that the Qt
        #     dialog controls the outer chrome/title bar
        #   * enable auto_timer so EventTimer (Qt QTimer backend) can pump
        #     Win32/WebView events without spawning extra threads.
        self._webview = WebView.create(
            title=title,
            width=width,
            height=height,
            parent=hwnd,
            mode="child",  # Real child window, lives inside this QWidget
            frame=False,
            debug=dev_tools,
            context_menu=context_menu,
            auto_show=False,
            auto_timer=True,
        )

        # Track cleanup state so we can make close idempotent.
        self._is_closing = False

        # Set up Qt event processor (strategy pattern)
        # This ensures Qt events are processed along with WebView events
        self._event_processor = QtEventProcessor(self._webview)
        self._webview.set_event_processor(self._event_processor)

        logger.info("QtWebView created with QtEventProcessor: %s (%sx%s)", title, width, height)

    # ------------------------------------------------------------------
    # High-level AuroraView-compatible API (delegated to WebView)
    # ------------------------------------------------------------------

    def load_url(self, url: str) -> None:
        """Load a URL into the embedded WebView."""
        self._webview.load_url(url)
        logger.info("QtWebView loading URL: %s", url)

    def load_html(self, html: str) -> None:
        """Load HTML content into the embedded WebView.

        This is a thin pass-through to :meth:`WebView.load_html`, which
        accepts only the HTML string. If you need to load a static HTML
        file together with its local assets (images/CSS/JS), prefer
        :meth:`load_url` with a ``file:///`` URL instead of relying on a
        ``base_url`` argument.
        """
        self._webview.load_html(html)
        logger.info("QtWebView loading HTML (%s bytes)", len(html))

    def load_file(self, path: Any) -> None:
        """Load a local HTML file in embedded Qt/DCC mode.

        In embedded WebView2 inside DCC hosts (Maya, Houdini, etc.) direct
        ``file://`` navigation is often restricted. To keep
        ``QtWebView.load_file(...)`` convenient for simple demos and tools,
        we first try to read the file contents and feed it through
        :meth:`load_html`. This works well for single-file HTML frontends.

        If reading the file fails for any reason, we fall back to the
        original behavior and delegate to the core :class:`WebView.load_file`
        helper, which uses a ``file:///`` URL.
        """
        try:
            html_path = Path(path).expanduser().resolve()
            html = html_path.read_text(encoding="utf-8")
        except Exception:
            # Fallback: use the underlying WebView.load_file implementation,
            # which resolves the path and dispatches to load_url.
            load_file = getattr(self._webview, "load_file", None)
            if callable(load_file):
                load_file(path)
            else:  # pragma: no cover - defensive, for older backends
                self.load_url(Path(path).expanduser().resolve().as_uri())
        else:
            self.load_html(html)
            logger.info("QtWebView loaded HTML from file via load_html(): %s", html_path)

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
        """Decorator to register event handler (AuroraView API compatibility)."""
        return self._webview.on(event_name)

    def register_callback(self, event_name: str, callback: Callable) -> None:
        """Register a callback for an event (compatibility helper)."""
        self._webview.register_callback(event_name, callback)

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

        This is currently implemented for Win32 child windows only; on other
        platforms the helper is a no-op.

        IMPORTANT: We add a small buffer zone (EDGE_BUFFER pixels) around the edges
        to allow Qt to handle window resize operations. This prevents the WebView
        from capturing mouse events at the window edges, making it easier to resize
        the Qt window.

        We also remove the WebView window border by modifying its window style.
        """
        try:
            if sys.platform != "win32":
                return

            core = getattr(self._webview, "_core", None)
            get_hwnd = getattr(core, "get_hwnd", None) if core is not None else None
            hwnd = get_hwnd() if callable(get_hwnd) else None
            if not hwnd:
                # HWND not ready yet - this is normal during initialization or window creation
                # Use debug level to avoid spamming logs during resize operations
                logger.debug("[QtWebView] _sync_embedded_geometry: No HWND available yet")
                return

            logger.debug(f"[QtWebView] _sync_embedded_geometry: HWND={hwnd}")

            user32 = ctypes.windll.user32

            # Remove window border by modifying window style
            GWL_STYLE = -16
            GWL_EXSTYLE = -20
            WS_BORDER = 0x00800000
            WS_THICKFRAME = 0x00040000
            WS_DLGFRAME = 0x00400000
            WS_EX_CLIENTEDGE = 0x00000200
            WS_EX_WINDOWEDGE = 0x00000100
            WS_EX_STATICEDGE = 0x00020000

            # Get current styles
            style = user32.GetWindowLongW(wintypes.HWND(int(hwnd)), GWL_STYLE)
            ex_style = user32.GetWindowLongW(wintypes.HWND(int(hwnd)), GWL_EXSTYLE)

            # Remove all border-related styles
            style &= ~(WS_BORDER | WS_THICKFRAME | WS_DLGFRAME)
            ex_style &= ~(WS_EX_CLIENTEDGE | WS_EX_WINDOWEDGE | WS_EX_STATICEDGE)

            # Apply new styles
            user32.SetWindowLongW(wintypes.HWND(int(hwnd)), GWL_STYLE, style)
            user32.SetWindowLongW(wintypes.HWND(int(hwnd)), GWL_EXSTYLE, ex_style)

            logger.debug(
                f"[QtWebView] Removed WebView window border (style={hex(style)}, ex_style={hex(ex_style)})"
            )

            # Use contentsRect() to get the actual content area excluding margins
            rect = self.contentsRect()

            # Zero buffer for perfect edge alignment - eliminate white edges
            # Frontend CSS handles edge buffer zones for resize operations
            EDGE_BUFFER = 0  # pixels - no buffer, perfect alignment

            x = rect.x() + EDGE_BUFFER
            y = rect.y() + EDGE_BUFFER
            width = max(0, rect.width() - 2 * EDGE_BUFFER)
            height = max(0, rect.height() - 2 * EDGE_BUFFER)

            logger.debug(
                f"[QtWebView] _sync_embedded_geometry: pos=({x},{y}) size={width}x{height} (buffer={EDGE_BUFFER}px)"
            )

            SWP_NOZORDER = 0x0004
            SWP_NOACTIVATE = 0x0010

            # Set both position and size to ensure webview stays aligned with Qt widget
            # Use minimal flags to avoid performance issues
            user32.SetWindowPos(
                wintypes.HWND(int(hwnd)),
                0,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        except Exception as e:  # pragma: no cover - best-effort only
            logger.debug("QtWebView: failed to sync embedded geometry: %s", e)

    def resizeEvent(self, event) -> None:  # type: ignore[override]
        """Resize the embedded WebView when the Qt widget is resized.

        Uses aggressive throttling to maintain 120 FPS during rapid resize operations.
        Optimized for ultra-smooth visual updates with minimal latency.
        """
        super().resizeEvent(event)
        try:
            current_time = time.time() * 1000  # Convert to milliseconds
            time_since_last = current_time - self._last_resize_time

            # Get new size
            rect = self.contentsRect()
            new_width = rect.width()
            new_height = rect.height()
            new_size = (new_width, new_height)

            # Skip if size hasn't changed (avoid duplicate events)
            if new_size == self._last_emitted_size:
                return

            # Aggressive throttling for 120 FPS
            # Always update immediately for first event or if enough time passed
            if self._last_resize_time == 0 or time_since_last >= self._resize_throttle_ms:
                # Update immediately for ultra-smooth experience
                self._sync_embedded_geometry()
                self._webview.emit("window_resized", {"width": new_width, "height": new_height})
                self._last_resize_time = current_time
                self._last_emitted_size = new_size
                logger.debug(
                    f"[QtWebView] resizeEvent: {new_width}x{new_height} (immediate, Δ{time_since_last:.1f}ms)"
                )
            else:
                # For rapid events, only schedule if not already scheduled
                # This ensures we capture the final size without flooding
                if self._pending_resize is None:

                    def delayed_resize():
                        self._pending_resize = None
                        current = time.time() * 1000

                        # Re-check size in case it changed again
                        rect = self.contentsRect()
                        width = rect.width()
                        height = rect.height()
                        size = (width, height)

                        if size != self._last_emitted_size:
                            self._sync_embedded_geometry()
                            self._webview.emit("window_resized", {"width": width, "height": height})
                            self._last_resize_time = current
                            self._last_emitted_size = size
                            logger.debug(f"[QtWebView] resizeEvent: {width}x{height} (delayed)")

                    # Schedule for next frame (8ms for 120fps)
                    self._pending_resize = QTimer.singleShot(
                        self._resize_throttle_ms, delayed_resize
                    )
                    logger.debug(
                        f"[QtWebView] resizeEvent: scheduled (throttled, Δ{time_since_last:.1f}ms)"
                    )

        except Exception as e:  # pragma: no cover - best-effort only
            logger.debug("QtWebView: resizeEvent sync failed: %s", e)

    def show(self) -> None:  # type: ignore[override]
        """Show the Qt host widget and start the embedded WebView.

        For Qt/DCC hosts (Maya, Houdini, etc.) we prefer to drive the
        embedded WebView via the main-thread :class:`EventTimer` (Qt
        ``QTimer`` backend) instead of spawning an additional background
        thread. This keeps all GUI work on the host's main thread and
        avoids subtle deadlocks.
        """
        start_time = time.time()
        logger.info("[QtWebView] show() started")

        super().show()

        # First, ensure the underlying core WebView is created in embedded mode.
        core = getattr(self._webview, "_core", None)
        if core is not None:
            try:
                # In embedded (owner/child) mode, _core.show() is non-blocking and
                # just creates the embedded window without its own event loop.
                core_show_start = time.time()
                core.show()
                core_show_time = (time.time() - core_show_start) * 1000
                logger.info(f"QtWebView: core.show() succeeded in {core_show_time:.1f}ms")
            except Exception as exc:  # pragma: no cover - best-effort fallback
                logger.warning(
                    "QtWebView: core.show() failed (%s), falling back to WebView.show()",
                    exc,
                )
                # Fall back to the generic WebView.show() behavior and let it
                # manage threading/timers on its own.
                self._webview.show()
                return

        # Sync geometry once after the embedded window has been created.
        try:
            sync_start = time.time()
            self._sync_embedded_geometry()
            sync_time = (time.time() - sync_start) * 1000
            logger.info(f"QtWebView: initial geometry sync completed in {sync_time:.1f}ms")
        except Exception as e:  # pragma: no cover - best-effort only
            logger.debug("QtWebView: initial geometry sync failed: %s", e)

        # Prefer using the auto-created EventTimer if available.
        timer = getattr(self._webview, "_auto_timer", None)
        if timer is not None:
            try:
                timer.start()
                total_time = (time.time() - start_time) * 1000
                logger.info(
                    f"QtWebView: started embedded WebView via EventTimer in {total_time:.1f}ms total"
                )
                return
            except Exception as exc:  # pragma: no cover - best-effort fallback
                logger.warning(
                    "QtWebView: failed to start EventTimer (%s), falling back to WebView.show()",
                    exc,
                )

        # Fallback: use the generic WebView.show() behavior.
        self._webview.show()

    def closeEvent(self, event) -> None:  # type: ignore[override]
        """Handle Qt close event and cleanup embedded WebView."""
        if self._is_closing:
            event.accept()
            return

        logger.info("QtWebView closeEvent triggered")
        self._is_closing = True

        try:
            try:
                self._webview.close()
            except Exception as e:  # pragma: no cover - best-effort cleanup
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
            logger.debug("QtWebView __del__ error: %s", e)

    def __repr__(self) -> str:
        """String representation."""
        try:
            return f"QtWebView(title='{self.windowTitle()}', size={self.width()}x{self.height()})"
        except RuntimeError:  # pragma: no cover - widget already deleted
            return "QtWebView(<deleted>)"


# Backward-compatibility alias
AuroraViewQt = QtWebView

__all__ = ["QtWebView", "AuroraViewQt"]
