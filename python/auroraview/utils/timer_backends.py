# -*- coding: utf-8 -*-
"""Timer backend abstraction for EventTimer.

This module provides an extensible timer backend system using the Strategy pattern.
Users can implement custom timer backends for different environments (Maya, Blender, etc.)
and register them for automatic discovery.

Example - Creating a custom backend:
    >>> from auroraview.timer_backends import TimerBackend, register_timer_backend
    >>>
    >>> class MayaTimerBackend(TimerBackend):
    ...     '''Maya scriptJob-based timer backend.'''
    ...
    ...     def is_available(self) -> bool:
    ...         try:
    ...             import maya.cmds
    ...             return True
    ...         except ImportError:
    ...             return False
    ...
    ...     def start(self, interval_ms: int, callback: Callable) -> Any:
    ...         import maya.cmds as cmds
    ...         # Maya uses idle events, so we ignore interval_ms
    ...         job_id = cmds.scriptJob(event=["idle", callback])
    ...         return job_id
    ...
    ...     def stop(self, handle: Any) -> None:
    ...         import maya.cmds as cmds
    ...         if cmds.scriptJob(exists=handle):
    ...             cmds.scriptJob(kill=handle, force=True)
    >>>
    >>> # Register with high priority so it's preferred in Maya
    >>> register_timer_backend(MayaTimerBackend, priority=200)

Example - Using a specific backend:
    >>> from auroraview import WebView
    >>> from auroraview.event_timer import EventTimer
    >>> from auroraview.timer_backends import QtTimerBackend
    >>>
    >>> webview = WebView()
    >>> backend = QtTimerBackend()
    >>> timer = EventTimer(webview, backend=backend)
    >>> timer.start()
"""

import logging
import threading
import time
from abc import ABC, abstractmethod
from typing import Any, Callable, List, Optional, Tuple, Type

logger = logging.getLogger(__name__)

# Try to import Qt via qtpy (supports PySide6, PyQt6, PySide2, PyQt5)
_QT_AVAILABLE = False
_QTimer = None

try:
    from qtpy.QtCore import QTimer as _QTimer

    _QT_AVAILABLE = True
except ImportError:
    pass


class TimerBackend(ABC):
    """Abstract base class for timer backends.

    Subclass this to implement custom timer backends for different environments.
    Each backend must implement three methods: is_available(), start(), and stop().
    """

    @abstractmethod
    def is_available(self) -> bool:
        """Check if this backend is available in the current environment.

        Returns:
            True if the backend can be used, False otherwise.

        Example:
            >>> def is_available(self) -> bool:
            ...     try:
            ...         import maya.cmds
            ...         return True
            ...     except ImportError:
            ...         return False
        """
        pass

    @abstractmethod
    def start(self, interval_ms: int, callback: Callable[[], None]) -> Any:
        """Start the timer with the given interval and callback.

        Args:
            interval_ms: Timer interval in milliseconds
            callback: Function to call on each timer tick

        Returns:
            Timer handle (can be any type) that will be passed to stop()

        Example:
            >>> def start(self, interval_ms: int, callback: Callable) -> Any:
            ...     from PySide6.QtCore import QTimer
            ...     timer = QTimer()
            ...     timer.timeout.connect(callback)
            ...     timer.start(interval_ms)
            ...     return timer
        """
        pass

    @abstractmethod
    def stop(self, handle: Any) -> None:
        """Stop the timer using the handle returned by start().

        Args:
            handle: Timer handle returned by start()

        Example:
            >>> def stop(self, handle: Any) -> None:
            ...     handle.stop()  # For Qt QTimer
        """
        pass

    def get_name(self) -> str:
        """Get the backend name for logging.

        Returns:
            Backend name (defaults to class name without 'Backend' suffix)
        """
        name = self.__class__.__name__
        if name.endswith("Backend"):
            name = name[:-7]  # Remove 'Backend' suffix
        return name


class QtTimerBackend(TimerBackend):
    """Qt QTimer-based backend.

    Uses Qt's QTimer for precise timing in Qt-based applications.
    Runs in the main thread to avoid thread-safety issues.

    Uses qtpy for Qt compatibility (supports PySide6, PyQt6, PySide2, PyQt5).
    """

    def is_available(self) -> bool:
        """Check if Qt is available via qtpy."""
        return _QT_AVAILABLE

    def start(self, interval_ms: int, callback: Callable[[], None]) -> Any:
        """Start Qt timer.

        Args:
            interval_ms: Timer interval in milliseconds
            callback: Function to call on each timer tick

        Returns:
            QTimer instance

        Raises:
            RuntimeError: If Qt is not available
        """
        if not _QT_AVAILABLE or _QTimer is None:
            raise RuntimeError(
                "Qt is not available. Install qtpy and a Qt binding: pip install qtpy PySide6"
            )

        timer = _QTimer()
        timer.timeout.connect(callback)
        timer.start(interval_ms)
        return timer

    def stop(self, handle: Any) -> None:
        """Stop Qt timer.

        Args:
            handle: QTimer instance returned by start()
        """
        if handle is not None:
            handle.stop()


class ThreadTimerBackend(TimerBackend):
    """Thread-based timer backend.

    Uses a daemon thread with time.sleep() for timing.
    This is the universal fallback that works everywhere.
    """

    def is_available(self) -> bool:
        """Thread backend is always available."""
        return True

    def start(self, interval_ms: int, callback: Callable[[], None]) -> Any:
        """Start thread-based timer."""
        stop_event = threading.Event()
        interval_sec = interval_ms / 1000.0

        def timer_loop():
            while not stop_event.is_set():
                callback()
                time.sleep(interval_sec)

        thread = threading.Thread(target=timer_loop, daemon=True)
        thread.start()

        # Return both thread and stop_event so we can stop it later
        return (thread, stop_event)

    def stop(self, handle: Any) -> None:
        """Stop thread-based timer."""
        if handle is not None:
            thread, stop_event = handle
            stop_event.set()
            # Don't join() - it's a daemon thread and will exit on its own


# Global registry of timer backends (priority, backend_class)
_TIMER_BACKENDS: List[Tuple[int, Type[TimerBackend]]] = []


def register_timer_backend(backend_class: Type[TimerBackend], priority: int = 0) -> None:
    """Register a custom timer backend.

    Backends are tried in order of priority (highest first).
    The first available backend is used.

    Args:
        backend_class: TimerBackend subclass to register
        priority: Priority value (higher = tried first). Default: 0
                 Built-in backends use: Qt=100, Thread=0

    Example:
        >>> class MayaTimerBackend(TimerBackend):
        ...     # ... implementation ...
        >>>
        >>> # Register with high priority for Maya environments
        >>> register_timer_backend(MayaTimerBackend, priority=200)
    """
    global _TIMER_BACKENDS

    # Check if already registered
    for i, (_, existing_class) in enumerate(_TIMER_BACKENDS):
        if existing_class is backend_class:
            # Update priority if already registered
            _TIMER_BACKENDS[i] = (priority, backend_class)
            _TIMER_BACKENDS.sort(key=lambda x: x[0], reverse=True)
            logger.debug(f"Updated timer backend {backend_class.__name__} with priority {priority}")
            return

    # Add new backend
    _TIMER_BACKENDS.append((priority, backend_class))
    _TIMER_BACKENDS.sort(key=lambda x: x[0], reverse=True)
    logger.debug(f"Registered timer backend {backend_class.__name__} with priority {priority}")


def get_available_backend() -> Optional[TimerBackend]:
    """Get the first available timer backend.

    Tries backends in order of priority (highest first).

    Returns:
        First available TimerBackend instance, or None if none available.
    """
    for priority, backend_class in _TIMER_BACKENDS:
        try:
            backend = backend_class()
            if backend.is_available():
                logger.debug(f"Selected timer backend: {backend.get_name()} (priority={priority})")
                return backend
        except Exception as e:
            logger.warning(f"Failed to initialize {backend_class.__name__}: {e}", exc_info=True)
            continue

    logger.warning("No timer backend available!")
    return None


def list_registered_backends() -> List[Tuple[int, str, bool]]:
    """List all registered backends with their availability.

    Returns:
        List of (priority, name, is_available) tuples.

    Example:
        >>> for priority, name, available in list_registered_backends():
        ...     status = "✓" if available else "✗"
        ...     print(f"{status} {name} (priority={priority})")
    """
    result = []
    for priority, backend_class in _TIMER_BACKENDS:
        try:
            backend = backend_class()
            name = backend.get_name()
            available = backend.is_available()
        except Exception:
            name = backend_class.__name__
            available = False

        result.append((priority, name, available))

    return result


# Register built-in backends
register_timer_backend(QtTimerBackend, priority=100)
register_timer_backend(ThreadTimerBackend, priority=0)
