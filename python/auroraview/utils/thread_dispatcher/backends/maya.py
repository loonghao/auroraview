# -*- coding: utf-8 -*-
"""Maya thread dispatcher backend."""

from __future__ import annotations

from typing import Any, Callable, TypeVar

from ..base import ThreadDispatcherBackend

T = TypeVar("T")


class MayaDispatcherBackend(ThreadDispatcherBackend):
    """Maya thread dispatcher backend.

    Uses maya.utils.executeDeferred() for fire-and-forget execution
    and maya.utils.executeInMainThreadWithResult() for blocking execution.

    Reference:
        https://help.autodesk.com/cloudhelp/2024/ENU/Maya-Tech-Docs/PyMel/generated/pymel.utils.html
    """

    def is_available(self) -> bool:
        """Check if Maya is available."""
        try:
            import maya.utils  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using maya.utils.executeDeferred()."""
        import maya.utils

        maya.utils.executeDeferred(lambda: func(*args, **kwargs))

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function using maya.utils.executeInMainThreadWithResult()."""
        import maya.utils

        return maya.utils.executeInMainThreadWithResult(lambda: func(*args, **kwargs))

    def is_main_thread(self) -> bool:
        """Check if running on Maya's main thread."""
        try:
            import maya.utils

            return maya.utils.isMainThread()
        except (ImportError, AttributeError):
            return super().is_main_thread()
