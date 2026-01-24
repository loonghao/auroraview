# -*- coding: utf-8 -*-
"""Houdini thread dispatcher backend."""

from __future__ import annotations

from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class HoudiniDispatcherBackend(ThreadDispatcherBackend):
    """Houdini thread dispatcher backend.

    Uses hdefereval.executeDeferred() for fire-and-forget execution
    and hdefereval.executeInMainThread() for blocking execution.

    Reference:
        https://www.sidefx.com/docs/houdini/hom/hou/hdefereval.html
    """

    def is_available(self) -> bool:
        """Check if Houdini is available."""
        try:
            import hdefereval  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using hdefereval.executeDeferred()."""
        import hdefereval

        hdefereval.executeDeferred(lambda: func(*args, **kwargs))

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function using hdefereval.executeInMainThread()."""
        import hdefereval

        return hdefereval.executeInMainThread(lambda: func(*args, **kwargs))
