# -*- coding: utf-8 -*-
"""Unreal Engine thread dispatcher backend."""

from __future__ import annotations

import logging
import threading
from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend, ThreadDispatchTimeoutError

T = TypeVar("T")
logger = logging.getLogger(__name__)

# Default timeout for synchronous game thread dispatch (seconds)
DEFAULT_SYNC_TIMEOUT = 30.0


class UnrealDispatcherBackend(ThreadDispatcherBackend):
    """Unreal Engine thread dispatcher backend.

    Uses unreal.register_slate_post_tick_callback() for game thread execution.
    This is critical for UE5 where many operations must run on the game thread.

    Reference:
        https://docs.unrealengine.com/5.0/en-US/PythonAPI/
    """

    def __init__(self, sync_timeout: float = DEFAULT_SYNC_TIMEOUT) -> None:
        """Initialize the Unreal dispatcher backend.

        Args:
            sync_timeout: Timeout in seconds for synchronous game thread
                dispatch. Set to 0 or negative to wait indefinitely
                (not recommended). Default: 30.0 seconds.
        """
        self._sync_timeout = sync_timeout

    def is_available(self) -> bool:
        """Check if Unreal Engine is available."""
        try:
            import unreal  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function on game thread using slate tick.

        Exceptions are caught and logged to prevent crashing the
        Slate tick loop.
        """
        import unreal

        def tick_callback(delta_time):
            try:
                func(*args, **kwargs)
            except Exception:
                logger.error(
                    "Error in deferred game thread call: %s",
                    func,
                    exc_info=True,
                )
            return False  # Unregister after first call

        unreal.register_slate_post_tick_callback(tick_callback)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on game thread and wait for result.

        Uses an event-based approach since UE Python doesn't have
        a built-in blocking main thread execution.

        Raises:
            ThreadDispatchTimeoutError: If the game thread doesn't complete
                within the configured timeout.
        """
        import unreal

        if self.is_main_thread():
            return func(*args, **kwargs)

        result_holder: list = [None]
        exception_holder: list = [None]
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

        # Wait for completion with timeout to prevent deadlocks
        timeout = self._sync_timeout if self._sync_timeout > 0 else None
        if not done_event.wait(timeout=timeout):
            raise ThreadDispatchTimeoutError(
                f"Game thread dispatch timed out after {self._sync_timeout}s. "
                f"The game thread may be blocked or Slate ticks are paused. "
                f"Function: {getattr(func, '__name__', repr(func))}"
            )

        if exception_holder[0] is not None:
            raise exception_holder[0]
        return result_holder[0]

    def is_main_thread(self) -> bool:
        """Check if running on Unreal's game thread."""
        try:
            import unreal

            # Try both API names for compatibility across UE versions
            if hasattr(unreal, "is_game_thread"):
                return unreal.is_game_thread()
            if hasattr(unreal, "is_in_game_thread"):
                return unreal.is_in_game_thread()
            return super().is_main_thread()
        except (ImportError, AttributeError):
            return super().is_main_thread()
