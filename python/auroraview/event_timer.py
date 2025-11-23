"""Event timer for WebView event processing.

This module provides a timer-based event loop for processing WebView events
in embedded mode. It's designed to work with applications that have their
own event loops.

The timer periodically checks for:
- Window messages (WM_CLOSE, WM_DESTROY, etc.)
- Window validity (IsWindow check)
- User-defined callbacks

Supported Timer Backends (in priority order):
1. Qt QTimer - Most precise, works in Qt-based applications
2. Thread-based timer - Fallback for all platforms

Note: For DCC-specific timer implementations (Maya, Blender, Houdini, etc.),
use the integration modules in the `integrations/` package.

IMPORTANT: Qt backend runs in the main thread to avoid thread-safety issues.
The thread-based fallback uses a background daemon thread.

Example:
    >>> from auroraview import WebView
    >>> from auroraview.event_timer import EventTimer
    >>>
    >>> webview = WebView(parent=parent_hwnd, mode="owner")
    >>>
    >>> # Create timer with 16ms interval (60 FPS)
    >>> timer = EventTimer(webview, interval_ms=16)
    >>>
    >>> # Register close callback
    >>> @timer.on_close
    >>> def handle_close():
    ...     print("WebView closed")
    ...     timer.stop()
    >>>
    >>> # Start timer (auto-detects best backend)
    >>> timer.start()
"""

import logging
import time
from typing import Any, Callable, Literal, Optional

logger = logging.getLogger(__name__)


# Type alias for timer backend kinds (for type hints and IDE support)
TimerType = Literal["qt", "thread"]


class EventTimer:
    """Timer-based event processor for WebView.

    This class provides a timer that periodically processes WebView events
    and checks window validity. It's designed for embedded mode where the
    WebView is integrated into a DCC application's event loop.

    IMPORTANT: This timer uses the host application's event loop (Maya scriptJob,
    Qt QTimer, etc.) instead of background threads to avoid thread-safety issues
    with Rust's PyO3 bindings.

    Args:
        webview: WebView instance to monitor
        interval_ms: Timer interval in milliseconds (default: 16ms = ~60 FPS)
        check_window_validity: Whether to check if window is still valid (default: True)

    Example:
        >>> timer = EventTimer(webview, interval_ms=16)
        >>> timer.on_close(lambda: print("Closed"))
        >>> timer.start()
        >>> # ... later ...
        >>> timer.stop()
    """

    def __init__(
        self,
        webview,
        interval_ms: int = 16,
        check_window_validity: bool = True,
    ):
        """Initialize event timer.

        Args:
            webview: WebView instance to monitor
            interval_ms: Timer interval in milliseconds (default: 16ms = ~60 FPS)
            check_window_validity: Whether to check if window is still valid
        """
        self._webview = webview
        self._interval_ms = interval_ms
        self._check_validity = check_window_validity
        self._running = False
        self._timer_impl: Optional[Any] = None  # Qt QTimer or thread
        self._timer_type: Optional["TimerType"] = None  # "qt" or "thread"
        self._close_callbacks: list[Callable[[], None]] = []
        self._tick_callbacks: list[Callable[[], None]] = []
        self._last_valid = True
        self._tick_count = 0

        logger.debug(
            f"EventTimer created: interval={interval_ms}ms, check_validity={check_window_validity}"
        )

    def start(self) -> None:
        """Start the timer.

        This attempts to use the best available timer backend in priority order:
        1. Qt QTimer (most precise, works in Qt-based applications)
        2. Thread-based timer (fallback)

        Qt backend runs in the main thread to avoid thread-safety issues.
        Thread backend uses a daemon thread.

        Raises:
            RuntimeError: If timer is already running or no timer backend available
        """
        if self._running:
            raise RuntimeError("Timer is already running")

        self._running = True

        # Try Qt QTimer first (most precise)
        if self._try_start_qt_timer():
            logger.info(f"EventTimer started with Qt QTimer (interval={self._interval_ms}ms)")
            return

        # Fallback to thread-based timer
        if self._try_start_thread_timer():
            logger.info(
                f"EventTimer started with thread-based timer (interval={self._interval_ms}ms)"
            )
            return

        # No timer backend available
        self._running = False
        raise RuntimeError(
            "No timer backend available. EventTimer requires Qt or threading support."
        )

    def stop(self) -> None:
        """Stop the timer and cleanup resources.

        This stops the Maya scriptJob or Qt QTimer and clears the webview reference
        to prevent circular references.
        """
        if not self._running:
            return

        self._running = False
        self._stop_timer_impl()

        # Clear webview reference to prevent circular references
        # This is important for proper cleanup in DCC environments
        self._webview = None

        logger.info("EventTimer stopped and cleaned up")

    def cleanup(self) -> None:
        """Cleanup all resources and references.

        This method should be called when the EventTimer is no longer needed.
        It stops the timer and clears all references to prevent memory leaks.
        """
        self.stop()

        # Clear all callbacks
        self._close_callbacks.clear()
        self._tick_callbacks.clear()

        logger.info("EventTimer cleanup complete")

    def _stop_timer_impl(self) -> None:
        """Internal method to stop the timer implementation."""
        if not self._timer_impl:
            return

        # Stop based on timer type
        if self._timer_type == "qt":
            try:
                self._timer_impl.stop()
                logger.info("Qt QTimer stopped")
            except Exception as e:
                logger.error(f"Failed to stop Qt QTimer: {e}")

        elif self._timer_type == "thread":
            # Thread will stop automatically when self._running becomes False
            logger.info("Thread-based timer stopped")

        self._timer_impl = None
        self._timer_type = None

    def on_close(self, callback: Callable[[], None]) -> Callable[[], None]:
        """Register callback for window close event.

        The callback will be called when the window is closed or becomes invalid.

        Args:
            callback: Function to call when window closes

        Returns:
            The callback function (for decorator usage)

        Example:
            >>> @timer.on_close
            >>> def handle_close():
            ...     print("Window closed")
        """
        self._close_callbacks.append(callback)
        logger.debug(f"Close callback registered: {callback.__name__}")
        return callback

    def on_tick(self, callback: Callable[[], None]) -> Callable[[], None]:
        """Register callback for timer tick.

        The callback will be called on every timer tick, before processing events.

        Args:
            callback: Function to call on each tick

        Returns:
            The callback function (for decorator usage)

        Example:
            >>> @timer.on_tick
            >>> def handle_tick():
            ...     print("Tick")
        """
        self._tick_callbacks.append(callback)
        logger.debug(f"Tick callback registered: {callback.__name__}")
        return callback

    def off_close(self, callback: Callable[[], None]) -> bool:
        """Unregister a previously registered close callback.

        Returns:
            True if the callback was removed, False if it was not found.
        """
        try:
            self._close_callbacks.remove(callback)
            logger.debug(
                f"Close callback unregistered: {getattr(callback, '__name__', repr(callback))}"
            )
            return True
        except ValueError:
            logger.debug("Close callback not found during unregistration")
            return False

    def off_tick(self, callback: Callable[[], None]) -> bool:
        """Unregister a previously registered tick callback.

        Returns:
            True if the callback was removed, False if it was not found.
        """
        try:
            self._tick_callbacks.remove(callback)
            logger.debug(
                f"Tick callback unregistered: {getattr(callback, '__name__', repr(callback))}"
            )
            return True
        except ValueError:
            logger.debug("Tick callback not found during unregistration")
            return False

    def _try_start_qt_timer(self) -> bool:
        """Try to start Qt QTimer.

        Returns:
            True if Qt timer started successfully, False otherwise
        """
        try:
            from qtpy.QtCore import QTimer

            timer = QTimer()
            timer.setInterval(self._interval_ms)
            timer.timeout.connect(self._tick)
            timer.start()
            self._timer_impl = timer
            self._timer_type = "qt"
            return True
        except Exception as e:
            logger.debug(f"Qt QTimer not available: {e}")
            return False

    def _try_start_thread_timer(self) -> bool:
        """Try to start thread-based timer (fallback).

        Returns:
            True if thread timer started successfully, False otherwise
        """
        try:
            import threading

            def timer_thread():
                while self._running:
                    time.sleep(self._interval_ms / 1000.0)
                    if self._running:
                        self._tick()

            thread = threading.Thread(target=timer_thread, daemon=True)
            thread.start()
            self._timer_impl = thread
            self._timer_type = "thread"
            return True
        except Exception as e:
            logger.debug(f"Thread-based timer not available: {e}")
            return False

    def _tick(self) -> None:
        """Timer tick callback (runs in main thread for Qt, background thread for thread backend)."""
        if not self._running:
            return

        self._tick_count += 1

        try:
            # Call tick callbacks
            for callback in self._tick_callbacks:
                try:
                    callback()
                except Exception as e:
                    logger.error(f"Error in tick callback: {e}", exc_info=True)

            # Process WebView events (only if WebView is initialized)
            should_close = False
            try:
                # Check if WebView is initialized (for non-blocking mode)
                if hasattr(self._webview, "_async_core"):
                    # Non-blocking mode: check if core is ready
                    with self._webview._async_core_lock:
                        if self._webview._async_core is None:
                            # WebView not yet initialized, skip this tick
                            return

                # Choose event-processing strategy based on timer backend.
                if self._timer_type == "qt" and hasattr(self._webview, "process_events_ipc_only"):
                    # Qt hosts own the native event loop.
                    # In this mode we only drain AuroraView's internal IPC queue
                    # and rely on Qt to drive the Win32/WebView2 message pump.
                    should_close = self._webview.process_events_ipc_only()
                else:
                    # Thread backend uses the full process_events() path,
                    # which drives the native message pump directly.
                    should_close = self._webview.process_events()
            except RuntimeError as e:
                if "not initialized" in str(e):
                    # WebView not yet initialized, skip this tick silently
                    return
                logger.error(f"Error processing events: {e}", exc_info=True)
            except Exception as e:
                logger.error(f"Error processing events: {e}", exc_info=True)

            # Check window validity (Windows only, and only if WebView is initialized)
            if self._check_validity and hasattr(self._webview, "_core"):
                try:
                    # Check if WebView is initialized (for non-blocking mode)
                    if hasattr(self._webview, "_async_core"):
                        with self._webview._async_core_lock:
                            if self._webview._async_core is None:
                                # WebView not yet initialized, skip validity check
                                return

                    is_valid = self._check_window_valid()
                    if self._last_valid and not is_valid:
                        logger.info("Window became invalid")
                        should_close = True
                    self._last_valid = is_valid
                except RuntimeError as e:
                    if "not initialized" in str(e):
                        # WebView not yet initialized, skip validity check silently
                        return
                    logger.error(f"Error checking window validity: {e}", exc_info=True)
                except Exception as e:
                    logger.error(f"Error checking window validity: {e}", exc_info=True)

            # Handle close event
            if should_close:
                logger.info("Close event detected")
                # Stop timer FIRST to prevent further ticks
                self._running = False
                self._stop_timer_impl()

                # Then call close callbacks
                for callback in self._close_callbacks:
                    try:
                        callback()
                    except Exception as e:
                        logger.error(f"Error in close callback: {e}", exc_info=True)

        except Exception as e:
            logger.error(f"Unexpected error in timer tick: {e}", exc_info=True)

    def _check_window_valid(self) -> bool:
        """Check if window is still valid (Windows only).

        Returns:
            True if window is valid, False otherwise
        """
        try:
            # Call Rust function to check window validity
            if hasattr(self._webview, "_core"):
                return self._webview._core.is_window_valid()
            return True
        except Exception as e:
            logger.error(f"Error checking window validity: {e}")
            return False

    @property
    def is_running(self) -> bool:
        """Check if timer is running."""
        return self._running

    @property
    def interval_ms(self) -> int:
        """Get timer interval in milliseconds."""
        return self._interval_ms

    @interval_ms.setter
    def interval_ms(self, value: int) -> None:
        """Set timer interval in milliseconds.

        Note: This only takes effect after restarting the timer.
        """
        if value <= 0:
            raise ValueError("Interval must be positive")
        self._interval_ms = value
        logger.debug(f"Timer interval set to {value}ms")

    def __enter__(self):
        """Context manager entry."""
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.stop()
        return False

    def __repr__(self) -> str:
        """String representation."""
        status = "running" if self._running else "stopped"
        return f"EventTimer(interval={self._interval_ms}ms, status={status})"


__all__ = ["EventTimer"]
