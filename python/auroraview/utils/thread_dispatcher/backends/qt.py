# -*- coding: utf-8 -*-
"""Qt thread dispatcher backend."""

from __future__ import annotations

import threading
from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class QtDispatcherBackend(ThreadDispatcherBackend):
    """Qt thread dispatcher backend.

    Uses QTimer.singleShot() for deferred execution.
    Works with any Qt-based application including Maya, Houdini, Nuke,
    and standalone Qt applications.
    """

    def is_available(self) -> bool:
        """Check if Qt is available."""
        try:
            from qtpy.QtCore import QCoreApplication

            return QCoreApplication.instance() is not None
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using QTimer.singleShot()."""
        from qtpy.QtCore import QTimer

        QTimer.singleShot(0, lambda: func(*args, **kwargs))

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on Qt main thread and wait for result."""
        from qtpy.QtCore import QCoreApplication, QThread

        app = QCoreApplication.instance()
        if app is None:
            return func(*args, **kwargs)

        if QThread.currentThread() == app.thread():
            return func(*args, **kwargs)

        result_holder = [None]
        exception_holder = [None]
        done_event = threading.Event()

        def wrapper():
            try:
                result_holder[0] = func(*args, **kwargs)
            except Exception as e:
                exception_holder[0] = e
            finally:
                done_event.set()

        from qtpy.QtCore import QTimer

        QTimer.singleShot(0, wrapper)
        done_event.wait()

        if exception_holder[0] is not None:
            raise exception_holder[0]
        return result_holder[0]

    def is_main_thread(self) -> bool:
        """Check if running on Qt's main thread."""
        try:
            from qtpy.QtCore import QCoreApplication, QThread

            app = QCoreApplication.instance()
            if app is None:
                return super().is_main_thread()
            return QThread.currentThread() == app.thread()
        except ImportError:
            return super().is_main_thread()
