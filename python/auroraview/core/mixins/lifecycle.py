# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Lifecycle Mixin.

This module provides lifecycle methods for the WebView class.
"""

from __future__ import annotations

import logging
import threading
from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from typing import Callable

logger = logging.getLogger(__name__)


class WebViewLifecycleMixin:
    """Mixin providing lifecycle methods.

    Provides methods for controlling the WebView lifecycle:
    - show: Show the WebView window (smart mode)
    - show_async: Show window in non-blocking mode
    - show_blocking: Show window and block until closed
    - wait: Wait for window to close
    - close: Close the WebView
    """

    # Type hints for attributes from main class
    _core: Any
    _ready_events: Any
    _show_thread: Optional[threading.Thread]
    _is_running: bool
    _title: str
    _stored_url: Optional[str]
    _stored_html: Optional[str]
    _auto_timer: Any
    _bridge: Any
    _async_core: Any
    _async_core_lock: threading.Lock
    _cached_hwnd: Optional[int]
    _cached_hwnd_lock: threading.Lock
    _close_requested: bool
    _event_handlers: Any
    _event_handlers_lock: threading.Lock
    _in_blocking_event_loop: bool
    _singleton_registry: Any
    _window_id: Optional[str]

    def show(self, *, wait: Optional[bool] = None) -> None:
        """Show the WebView window (smart mode).

        Automatically detects standalone/embedded/packed mode and chooses the best behavior:
        - Packed mode: Runs as headless API server (no window, JSON-RPC via stdin/stdout)
        - Standalone window: Blocks until closed (unless wait=False)
        - Embedded window: Non-blocking, auto-starts timer if available

        Args:
            wait: Whether to wait for window to close
                - None: Auto-detect (standalone=True, embedded=False)
                - True: Block until window closes
                - False: Return immediately (background thread)

        Examples:
            >>> # Standalone window - auto-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show()  # Blocks until closed

            >>> # Standalone window - force non-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show(wait=False)  # Returns immediately
            >>> input("Press Enter to exit...")

            >>> # Embedded window - auto non-blocking
            >>> webview = WebView(title="Tool", parent=maya_hwnd)
            >>> webview.show()  # Returns immediately, timer auto-runs

            >>> # Packed mode - automatic API server (no code changes needed)
            >>> # When running in a packed .exe, show() automatically switches
            >>> # to API server mode. All bind_call() handlers work seamlessly.
        """
        # Check for packed mode first - transparent to developers
        from .packed import is_packed_mode, run_api_server

        if is_packed_mode():
            logger.info("Packed mode detected: running as API server")
            run_api_server(self)
            return

        # Detect mode
        is_embedded = self._core is not None and hasattr(self._core, "_is_embedded") and self._core._is_embedded

        if wait is None:
            wait = not is_embedded  # Standalone=blocking, Embedded=non-blocking

        if wait:
            self.show_blocking()
        else:
            self.show_async()

    def show_async(self) -> None:
        """Show the WebView window in non-blocking mode (compatibility helper).

        Equivalent to calling show(wait=False). Safe to call multiple times; if the
        WebView is already running, the call is ignored.
        """
        self._show_non_blocking()

    def _show_non_blocking(self) -> None:
        """Internal method: non-blocking show (background thread)."""
        if self._is_running:
            logger.warning("WebView is already running")
            return

        logger.info(f"Showing WebView in background thread: {self._title}")
        self._is_running = True

        def _run_webview():
            """Run the WebView in a background thread.

            Note: We create a new WebView instance in the background thread
            because the Rust core requires the WebView to be created and shown
            in the same thread due to GUI event loop requirements.
            """
            try:
                logger.info("Background thread: Creating WebView instance")
                # Create a new WebView instance in this thread
                # This is necessary because the Rust core is not Send/Sync
                from auroraview._core import WebView as _CoreWebView

                core = _CoreWebView(
                    title=self._title,
                    width=self._width,
                    height=self._height,
                    dev_tools=self._debug,  # Use new parameter name
                    resizable=self._resizable,
                    decorations=self._frame,  # Use new parameter name
                    parent_hwnd=self._parent,  # Use new parameter name
                    parent_mode=self._mode,  # Use new parameter name
                    always_on_top=self._always_on_top,  # Keep window always on top
                    transparent=self._transparent,  # Enable transparent window
                    background_color=self._background_color,  # Window background color
                    tool_window=self._tool_window,  # Tool window style
                    undecorated_shadow=self._undecorated_shadow,  # Shadow for frameless
                    allow_new_window=self._allow_new_window,  # Allow window.open()
                    remote_debugging_port=self._remote_debugging_port,  # CDP port
                )

                # Set up HWND callback to cache HWND for cross-thread access
                def on_hwnd_created(hwnd: int) -> None:
                    with self._cached_hwnd_lock:
                        self._cached_hwnd = hwnd
                    logger.info(f"Background thread: Cached HWND 0x{hwnd:X}")

                if hasattr(core, "set_on_hwnd_created"):
                    core.set_on_hwnd_created(on_hwnd_created)

                # Store the core instance for use by emit() and other methods
                with self._async_core_lock:
                    self._async_core = core

                # If close was requested before the background core became ready,
                # exit early without entering the event loop.
                if getattr(self, "_close_requested", False):
                    logger.info(
                        "Background thread: close already requested; skipping show() and exiting"
                    )
                    return

                # Re-register all event handlers in the background thread
                # Snapshot the handlers under lock to avoid race conditions
                # with the main thread adding handlers concurrently.
                with self._event_handlers_lock:
                    handlers_snapshot = {k: list(v) for k, v in self._event_handlers.items()}

                logger.info(
                    f"Background thread: Re-registering {len(handlers_snapshot)} event handlers"
                )
                for event_name, handlers in handlers_snapshot.items():
                    for handler in handlers:
                        logger.debug(f"Background thread: Registering handler for '{event_name}'")
                        core.on(event_name, handler)

                # Load the same content that was loaded in the main thread
                if self._stored_html:
                    logger.info("Background thread: Loading stored HTML")
                    core.load_html(self._stored_html)
                elif self._stored_url:
                    logger.info("Background thread: Loading stored URL")
                    core.load_url(self._stored_url)
                else:
                    logger.warning("Background thread: No content loaded")

                logger.info("Background thread: Starting WebView event loop")
                core.show()
                # Note: show() is blocking - the HWND callback is invoked before
                # entering the event loop, so _cached_hwnd is already set
                logger.info("Background thread: WebView event loop exited")
            except Exception as e:
                logger.error(f"Error in background WebView: {e}", exc_info=True)
            finally:
                # Clear the async core reference
                with self._async_core_lock:
                    self._async_core = None
                self._is_running = False
                logger.info("Background thread: WebView thread finished")

        # Create and start the background thread as daemon
        # CRITICAL: daemon=True allows Maya to exit cleanly when user closes Maya
        # The event loop now uses run_return() instead of run(), which prevents
        # the WebView from calling std::process::exit() and terminating Maya
        self._show_thread = threading.Thread(target=_run_webview, daemon=True)
        self._show_thread.start()
        logger.info("WebView background thread started (daemon=True)")

    def show_blocking(self) -> None:
        """Show the WebView window (blocking - for standalone scripts).

        This method blocks until the window is closed. Use this in standalone scripts
        where you want the script to wait for the user to close the window.

        NOT recommended for DCC integration (Maya, Houdini, etc.) as it will freeze
        the main application.

        Example:
            >>> webview = WebView(title="My App", width=800, height=600)
            >>> webview.load_html("<h1>Hello</h1>")
            >>> webview.show_blocking()  # Blocks until window closes
            >>> print("Window was closed")
        """
        logger.info(f"Showing WebView (blocking): {self._title}")
        logger.info("Calling _core.show()...")

        # Check if we're in embedded mode
        is_embedded = self._parent is not None  # Use new parameter name

        # Mark that we're entering blocking event loop
        # This tells eval_js to skip _auto_process_events since the event loop
        # will handle message queue processing automatically
        self._in_blocking_event_loop = True

        try:
            self._core.show()
            logger.info("_core.show() returned successfully")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise
        finally:
            # Clear the flag when event loop exits
            self._in_blocking_event_loop = False

        # IMPORTANT: Only cleanup in standalone mode
        # In embedded mode, the window should stay open until explicitly closed
        if not is_embedded:
            logger.info("Standalone mode: WebView show_blocking() completed, cleaning up...")
            try:
                self.close()
            except Exception as cleanup_error:
                logger.warning(f"Error during cleanup: {cleanup_error}")
        else:
            logger.info("Embedded mode: WebView window is now open (non-blocking)")
            logger.info("IMPORTANT: Keep this Python object alive to prevent window from closing")
            logger.info("Example: __main__.webview = webview")

    def wait(self, timeout: Optional[float] = None) -> bool:
        """Wait for the WebView to close.

        Args:
            timeout: Maximum time to wait in seconds (None = indefinitely)

        Returns:
            True if the WebView closed, False if timeout expired

        Example:
            >>> webview.show_async()
            >>> if webview.wait(timeout=60):
            ...     print("WebView closed by user")
            ... else:
            ...     print("Timeout waiting for WebView")
        """
        if self._show_thread is None:
            logger.warning("WebView is not running")
            return True

        logger.info(f"Waiting for WebView to close (timeout={timeout})")
        self._show_thread.join(timeout=timeout)

        if self._show_thread.is_alive():
            logger.warning("Timeout waiting for WebView to close")
            return False

        logger.info("WebView closed")
        return True

    def close(self) -> None:
        """Close the WebView window and remove from registries."""
        logger.info("Closing WebView")

        # Teardown telemetry before closing
        self._teardown_telemetry()

        # Mark close intent early so background thread can bail out if it hasn't
        # entered the event loop yet.
        self._close_requested = True

        # Prefer closing the background-thread core (if present). Fall back to
        # the main-thread core as a best-effort.
        cores = []
        try:
            with self._async_core_lock:
                if self._async_core is not None:
                    cores.append(self._async_core)
        except Exception:
            # Lock acquisition should never fail, but keep close best-effort.
            pass

        cores.append(getattr(self, "_core", None))

        seen = set()
        for core in cores:
            if core is None:
                continue
            core_id = id(core)
            if core_id in seen:
                continue
            seen.add(core_id)

            try:
                core.close()
                logger.info("Core WebView close requested")
            except Exception as e:
                logger.warning(f"Error requesting core close: {e}")

        # Wait for background thread if running
        if self._show_thread is not None and self._show_thread.is_alive():
            logger.info("Waiting for background thread to finish...")
            self._show_thread.join(timeout=5.0)
            if self._show_thread.is_alive():
                logger.warning("Background thread did not finish within timeout")
            else:
                logger.info("Background thread finished successfully")

        # Remove from singleton registry
        for key, instance in list(self._singleton_registry.items()):
            if instance is self:
                del self._singleton_registry[key]
                logger.info(f"Removed from singleton registry: '{key}'")
                break

        # Remove from WindowManager
        if self._window_id:
            from .window_manager import get_window_manager

            wm = get_window_manager()
            wm.unregister(self._window_id)
            logger.debug(f"WebView unregistered from WindowManager: {self._window_id}")

        logger.info("WebView closed successfully")
