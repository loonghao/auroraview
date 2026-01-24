# -*- coding: utf-8 -*-
"""Blender thread dispatcher backend."""

from __future__ import annotations

import queue
import threading
from typing import Any, Callable, Tuple, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class BlenderDispatcherBackend(ThreadDispatcherBackend):
    """Blender thread dispatcher backend.

    Uses bpy.app.timers.register() for deferred execution.
    For blocking execution, uses a queue-based approach with timers.

    Reference:
        https://docs.blender.org/api/current/bpy.app.timers.html
    """

    def is_available(self) -> bool:
        """Check if Blender is available."""
        try:
            import bpy  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using bpy.app.timers.register()."""
        import bpy

        def timer_callback():
            func(*args, **kwargs)
            return None  # Don't repeat

        bpy.app.timers.register(timer_callback, first_interval=0.0)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on main thread and wait for result.

        Uses a queue-based approach since Blender doesn't have a built-in
        executeInMainThreadWithResult equivalent.
        """
        import bpy

        if self.is_main_thread():
            return func(*args, **kwargs)

        result_queue: queue.Queue[Tuple[bool, Any]] = queue.Queue()

        def timer_callback():
            try:
                result = func(*args, **kwargs)
                result_queue.put((True, result))
            except Exception as e:
                result_queue.put((False, e))
            return None

        bpy.app.timers.register(timer_callback, first_interval=0.0)

        # Wait for result
        success, value = result_queue.get()
        if success:
            return value
        else:
            raise value

    def is_main_thread(self) -> bool:
        """Check if running on Blender's main thread."""
        return threading.current_thread() is threading.main_thread()
