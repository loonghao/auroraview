# -*- coding: utf-8 -*-
"""Nuke thread dispatcher backend."""

from __future__ import annotations

from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class NukeDispatcherBackend(ThreadDispatcherBackend):
    """Nuke thread dispatcher backend.

    Uses nuke.executeInMainThread() for deferred execution
    and nuke.executeInMainThreadWithResult() for blocking execution.

    Reference:
        https://learn.foundry.com/nuke/developers/latest/pythondevguide/threading.html
    """

    def is_available(self) -> bool:
        """Check if Nuke is available."""
        try:
            import nuke

            return hasattr(nuke, "executeInMainThread")
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using nuke.executeInMainThread()."""
        import nuke

        nuke.executeInMainThread(lambda: func(*args, **kwargs))

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function using nuke.executeInMainThreadWithResult()."""
        import nuke

        return nuke.executeInMainThreadWithResult(lambda: func(*args, **kwargs))
