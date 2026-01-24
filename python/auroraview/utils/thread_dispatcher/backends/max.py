# -*- coding: utf-8 -*-
"""3ds Max thread dispatcher backend."""

from __future__ import annotations

import logging
import threading
from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")
logger = logging.getLogger(__name__)


class MaxDispatcherBackend(ThreadDispatcherBackend):
    """3ds Max thread dispatcher backend.

    Uses Qt's event loop for main thread execution since 3ds Max uses Qt internally.
    MaxPlus is deprecated since 3ds Max 2020, so we use Qt-based scheduling.

    For 3ds Max 2020+, the recommended approach is to use Qt's QTimer.singleShot()
    since 3ds Max's main thread runs a Qt event loop.

    Reference:
        https://help.autodesk.com/view/MAXDEV/2024/ENU/?guid=MAXDEV_Python_python_pymxs_html
    """

    def is_available(self) -> bool:
        """Check if 3ds Max is available."""
        try:
            import pymxs  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function on main thread (fire-and-forget).

        Uses Qt's QTimer.singleShot() since 3ds Max runs a Qt event loop.
        """
        if self.is_main_thread():
            func(*args, **kwargs)
            return

        try:
            from qtpy.QtCore import QTimer

            QTimer.singleShot(0, lambda: func(*args, **kwargs))
        except ImportError:
            logger.warning(
                "Qt not available in 3ds Max - executing function directly. "
                "This may cause thread safety issues."
            )
            func(*args, **kwargs)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on main thread and wait for result.

        Uses Qt's event loop with a blocking wait mechanism.
        """
        if self.is_main_thread():
            return func(*args, **kwargs)

        try:
            from qtpy.QtCore import QEventLoop, QTimer

            result_holder: list = [None]
            exception_holder: list = [None]
            event_loop = QEventLoop()

            def wrapper():
                try:
                    result_holder[0] = func(*args, **kwargs)
                except Exception as e:
                    exception_holder[0] = e
                finally:
                    event_loop.quit()

            QTimer.singleShot(0, wrapper)
            event_loop.exec_()

            if exception_holder[0] is not None:
                raise exception_holder[0]
            return result_holder[0]
        except ImportError:
            logger.warning(
                "Qt not available in 3ds Max - executing function directly. "
                "This may cause thread safety issues."
            )
            return func(*args, **kwargs)

    def is_main_thread(self) -> bool:
        """Check if current thread is the main thread."""
        try:
            from qtpy.QtCore import QCoreApplication, QThread

            app = QCoreApplication.instance()
            if app is not None:
                return QThread.currentThread() == app.thread()
        except ImportError:
            pass

        return threading.current_thread() is threading.main_thread()
