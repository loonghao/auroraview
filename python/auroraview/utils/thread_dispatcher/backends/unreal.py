# -*- coding: utf-8 -*-
"""Unreal Engine thread dispatcher backend."""

from __future__ import annotations

import threading
from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class UnrealDispatcherBackend(ThreadDispatcherBackend):
    """Unreal Engine thread dispatcher backend.

    Uses unreal.register_slate_post_tick_callback() for game thread execution.
    This is critical for UE5 where many operations must run on the game thread.

    Reference:
        https://docs.unrealengine.com/5.0/en-US/PythonAPI/
    """

    def is_available(self) -> bool:
        """Check if Unreal Engine is available."""
        try:
            import unreal  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function on game thread using slate tick."""
        import unreal

        def tick_callback(delta_time):
            func(*args, **kwargs)
            return False  # Unregister after first call

        unreal.register_slate_post_tick_callback(tick_callback)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on game thread and wait for result.

        Uses an event-based approach since UE Python doesn't have
        a built-in blocking main thread execution.
        """
        import unreal

        if self.is_main_thread():
            return func(*args, **kwargs)

        result_holder = [None]
        exception_holder = [None]
        done_event = threading.Event()

        def tick_callback(delta_time):
            try:
                result_holder[0] = func(*args, **kwargs)
            except Exception as e:
                exception_holder[0] = e
            finally:
                done_event.set()
            return False

        unreal.register_slate_post_tick_callback(tick_callback)

        # Wait for completion
        done_event.wait()

        if exception_holder[0] is not None:
            raise exception_holder[0]
        return result_holder[0]

    def is_main_thread(self) -> bool:
        """Check if running on Unreal's game thread."""
        try:
            import unreal

            return unreal.is_game_thread()
        except (ImportError, AttributeError):
            return super().is_main_thread()
