# -*- coding: utf-8 -*-
"""Fallback thread dispatcher backend."""

from __future__ import annotations

import logging
from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")
logger = logging.getLogger(__name__)


class FallbackDispatcherBackend(ThreadDispatcherBackend):
    """Fallback thread dispatcher backend.

    Uses a simple threading approach when no DCC-specific backend is available.
    This backend assumes the main thread is the Python main thread.

    Warning:
        This backend may not work correctly in all DCC environments.
        It's provided as a last resort fallback.
    """

    def is_available(self) -> bool:
        """Fallback backend is always available."""
        return True

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function (may not be on main thread)."""
        if self.is_main_thread():
            func(*args, **kwargs)
        else:
            logger.warning(
                "FallbackDispatcherBackend: Cannot guarantee main thread execution. "
                "Consider using a DCC-specific backend."
            )
            func(*args, **kwargs)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function and return result."""
        if not self.is_main_thread():
            logger.warning(
                "FallbackDispatcherBackend: Cannot guarantee main thread execution. "
                "Consider using a DCC-specific backend."
            )
        return func(*args, **kwargs)
