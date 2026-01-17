# -*- coding: utf-8 -*-
"""Lifecycle event system for WebView.

This module provides event waiting mechanisms and decorators
to ensure WebView operations are executed at the right time.
"""

from __future__ import annotations

import logging
import time
from functools import wraps
from threading import Event as ThreadEvent
from typing import TYPE_CHECKING, Any, Callable, TypeVar

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)

F = TypeVar("F", bound=Callable[..., Any])


class ReadyEvents:
    """Event container for WebView lifecycle states.

    Provides thread-safe waiting mechanisms for various WebView states:
    - created: WebView instance created
    - shown: Window is visible
    - loaded: Page content loaded
    - bridge_ready: JS bridge is ready for communication

    Example:
        >>> events = ReadyEvents(webview)
        >>> events.wait_loaded(timeout=10)
        True
        >>> events.wait_bridge_ready()
        True
    """

    def __init__(self, window: "WebView"):
        """Initialize ReadyEvents.

        Args:
            window: The WebView instance to track
        """
        self._window = window
        self.created = ThreadEvent()
        self.shown = ThreadEvent()
        self.loaded = ThreadEvent()
        self.bridge_ready = ThreadEvent()

    def wait_created(self, timeout: float = 20.0) -> bool:
        """Wait for WebView to be created.

        Args:
            timeout: Maximum time to wait in seconds

        Returns:
            True if event was set, False if timeout occurred
        """
        result = self.created.wait(timeout)
        if not result:
            logger.warning(f"Timeout waiting for WebView creation ({timeout}s)")
        return result

    def wait_shown(self, timeout: float = 20.0) -> bool:
        """Wait for window to be shown.

        Args:
            timeout: Maximum time to wait in seconds

        Returns:
            True if event was set, False if timeout occurred
        """
        result = self.shown.wait(timeout)
        if not result:
            logger.warning(f"Timeout waiting for window to show ({timeout}s)")
        return result

    def wait_loaded(self, timeout: float = 20.0) -> bool:
        """Wait for page to be loaded.

        Args:
            timeout: Maximum time to wait in seconds

        Returns:
            True if event was set, False if timeout occurred
        """
        result = self.loaded.wait(timeout)
        if not result:
            logger.warning(f"Timeout waiting for page to load ({timeout}s)")
        return result

    def wait_bridge_ready(self, timeout: float = 20.0) -> bool:
        """Wait for JS bridge to be ready.

        Args:
            timeout: Maximum time to wait in seconds

        Returns:
            True if event was set, False if timeout occurred
        """
        result = self.bridge_ready.wait(timeout)
        if not result:
            logger.warning(f"Timeout waiting for JS bridge ({timeout}s)")
        return result

    def wait_all(self, timeout: float = 30.0) -> bool:
        """Wait for all events (created, shown, loaded, bridge_ready).

        Args:
            timeout: Maximum total time to wait in seconds

        Returns:
            True if all events were set, False if timeout occurred
        """
        start = time.monotonic()
        remaining = timeout

        events = [
            ("created", self.created),
            ("shown", self.shown),
            ("loaded", self.loaded),
            ("bridge_ready", self.bridge_ready),
        ]

        for name, event in events:
            if not event.wait(remaining):
                logger.warning(f"Timeout waiting for '{name}' event")
                return False
            remaining = timeout - (time.monotonic() - start)
            if remaining <= 0:
                logger.warning("Timeout waiting for all events")
                return False

        return True

    def set_created(self) -> None:
        """Mark WebView as created."""
        self.created.set()
        logger.debug("Event: created")

    def set_shown(self) -> None:
        """Mark window as shown."""
        self.shown.set()
        logger.debug("Event: shown")

    def set_loaded(self) -> None:
        """Mark page as loaded."""
        self.loaded.set()
        logger.debug("Event: loaded")

    def set_bridge_ready(self) -> None:
        """Mark JS bridge as ready."""
        self.bridge_ready.set()
        logger.debug("Event: bridge_ready")

    def reset(self) -> None:
        """Reset all events to unset state."""
        self.created.clear()
        self.shown.clear()
        self.loaded.clear()
        self.bridge_ready.clear()
        logger.debug("All events reset")

    def is_created(self) -> bool:
        """Check if WebView is created."""
        return self.created.is_set()

    def is_shown(self) -> bool:
        """Check if window is shown."""
        return self.shown.is_set()

    def is_loaded(self) -> bool:
        """Check if page is loaded."""
        return self.loaded.is_set()

    def is_bridge_ready(self) -> bool:
        """Check if JS bridge is ready."""
        return self.bridge_ready.is_set()

    def is_ready(self) -> bool:
        """Check if all events are set."""
        return all(
            [
                self.created.is_set(),
                self.shown.is_set(),
                self.loaded.is_set(),
                self.bridge_ready.is_set(),
            ]
        )

    def status(self) -> dict:
        """Get status of all events.

        Returns:
            Dict with event names and their status
        """
        return {
            "created": self.created.is_set(),
            "shown": self.shown.is_set(),
            "loaded": self.loaded.is_set(),
            "bridge_ready": self.bridge_ready.is_set(),
        }


# Decorators for automatic waiting


def require_created(func: F) -> F:
    """Decorator to ensure WebView is created before executing.

    Example:
        >>> class MyWebView(WebView):
        ...     @require_created
        ...     def custom_method(self):
        ...         pass
    """

    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if hasattr(self, "_ready_events") and self._ready_events is not None:
            if not self._ready_events.wait_created(timeout=20):
                raise RuntimeError("WebView failed to create within timeout")
        return func(self, *args, **kwargs)

    return wrapper  # type: ignore


def require_shown(func: F) -> F:
    """Decorator to ensure window is shown before executing.

    Example:
        >>> class MyWebView(WebView):
        ...     @require_shown
        ...     def capture_screenshot(self):
        ...         pass
    """

    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if hasattr(self, "_ready_events") and self._ready_events is not None:
            if not self._ready_events.wait_shown(timeout=20):
                raise RuntimeError("WebView failed to show within timeout")
        return func(self, *args, **kwargs)

    return wrapper  # type: ignore


def require_loaded(func: F) -> F:
    """Decorator to ensure page is loaded before executing.

    Example:
        >>> class MyWebView(WebView):
        ...     @require_loaded
        ...     def evaluate_js(self, script):
        ...         pass
    """

    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if hasattr(self, "_ready_events") and self._ready_events is not None:
            if not self._ready_events.wait_loaded(timeout=20):
                raise RuntimeError("WebView failed to load within timeout")
        return func(self, *args, **kwargs)

    return wrapper  # type: ignore


def require_bridge_ready(func: F) -> F:
    """Decorator to ensure JS bridge is ready before executing.

    Example:
        >>> class MyWebView(WebView):
        ...     @require_bridge_ready
        ...     def call_js_api(self, method, params):
        ...         pass
    """

    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if hasattr(self, "_ready_events") and self._ready_events is not None:
            if not self._ready_events.wait_bridge_ready(timeout=20):
                raise RuntimeError("JS bridge failed to initialize within timeout")
        return func(self, *args, **kwargs)

    return wrapper  # type: ignore


def require_ready(func: F) -> F:
    """Decorator to ensure WebView is fully ready before executing.

    This waits for all events: created, shown, loaded, and bridge_ready.

    Example:
        >>> class MyWebView(WebView):
        ...     @require_ready
        ...     def full_interaction(self):
        ...         pass
    """

    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if hasattr(self, "_ready_events") and self._ready_events is not None:
            if not self._ready_events.wait_all(timeout=30):
                raise RuntimeError("WebView failed to become ready within timeout")
        return func(self, *args, **kwargs)

    return wrapper  # type: ignore
