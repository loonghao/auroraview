"""EventEmitter base class for AuroraView.

This module provides a Node.js-inspired EventEmitter pattern for managing
event subscriptions and emissions.

Example:
    >>> from auroraview.core.event_emitter import EventEmitter
    >>>
    >>> class MyWebView(EventEmitter):
    ...     def load_page(self, url):
    ...         self.emit("navigation", {"url": url, "type": "start"})
    ...         # ... load page ...
    ...         self.emit("navigation", {"url": url, "type": "end", "success": True})
    >>>
    >>> view = MyWebView()
    >>> unsub = view.on("navigation", lambda e: print(f"Nav: {e}"))
    >>> view.load_page("https://example.com")
    >>> unsub()  # Unsubscribe
"""

from __future__ import annotations

import logging
import threading
import warnings
from dataclasses import dataclass
from typing import Any, Callable, Dict, List, Optional, Union

try:
    from typing import Literal  # py38+
except ImportError:  # pragma: no cover - only for py37
    from typing_extensions import Literal  # type: ignore

logger = logging.getLogger(__name__)

# Type aliases
EventHandler = Callable[[Any], Any]
UnsubscribeFunc = Callable[[], None]


@dataclass
class NavigationEvent:
    """Unified navigation event data.

    Combines start, progress, and end navigation events into a single type.

    Attributes:
        url: The URL being navigated to
        event_type: Type of navigation event ("start", "progress", "end")
        success: Whether navigation succeeded (only for "end" events)
        error: Error message if navigation failed
        progress: Loading progress 0-100 (only for "progress" events)
        status_code: HTTP status code if available
    """

    url: str
    event_type: Literal["start", "progress", "end"] = "start"
    success: bool = True
    error: Optional[str] = None
    progress: int = 0
    status_code: Optional[int] = None

    def __repr__(self) -> str:
        if self.event_type == "start":
            return f"NavigationEvent(url={self.url!r}, type='start')"
        elif self.event_type == "progress":
            return f"NavigationEvent(url={self.url!r}, type='progress', progress={self.progress})"
        else:
            if self.success:
                return f"NavigationEvent(url={self.url!r}, type='end', success=True)"
            else:
                return f"NavigationEvent(url={self.url!r}, type='end', error={self.error!r})"


@dataclass
class WindowEvent:
    """Unified window event data.

    Attributes:
        event_type: Type of window event
        width: Window width (for resize events)
        height: Window height (for resize events)
        x: Window X position (for move events)
        y: Window Y position (for move events)
        fullscreen: Fullscreen state (for fullscreen events)
    """

    event_type: Literal[
        "show",
        "hide",
        "focus",
        "blur",
        "resize",
        "move",
        "minimize",
        "maximize",
        "restore",
        "fullscreen",
    ]
    width: Optional[int] = None
    height: Optional[int] = None
    x: Optional[int] = None
    y: Optional[int] = None
    fullscreen: Optional[bool] = None


@dataclass
class LoadEvent:
    """Page load event data.

    Attributes:
        url: Current URL
        title: Page title
        ready: Whether DOM is ready
    """

    url: str
    title: Optional[str] = None
    ready: bool = False


@dataclass
class _EventListener:
    """Internal event listener representation."""

    handler: EventHandler
    once: bool = False


class EventEmitter:
    """Node.js-inspired event emitter base class.

    Provides a clean API for event subscription and emission with support for:
    - Multiple handlers per event
    - One-time handlers (once)
    - Unsubscribe functions
    - Thread-safe operations

    Example:
        >>> emitter = EventEmitter()
        >>>
        >>> # Subscribe to event
        >>> unsub = emitter.on("data", lambda d: print(f"Got: {d}"))
        >>>
        >>> # One-time subscription
        >>> emitter.once("ready", lambda: print("Ready!"))
        >>>
        >>> # Emit events
        >>> emitter.emit("data", {"value": 42})
        >>> emitter.emit("ready")
        >>>
        >>> # Unsubscribe
        >>> unsub()
    """

    def __init__(self) -> None:
        """Initialize the event emitter."""
        self._listeners: Dict[str, List[_EventListener]] = {}
        self._lock = threading.RLock()

    def on(
        self,
        event: str,
        handler: Optional[EventHandler] = None,
    ) -> Union[UnsubscribeFunc, Callable[[EventHandler], UnsubscribeFunc]]:
        """Register an event handler.

        Can be used as a method or decorator:

            # As method
            unsub = emitter.on("event", handler)

            # As decorator
            @emitter.on("event")
            def handler(data):
                print(data)

        Args:
            event: Event name to listen for
            handler: Callback function (optional for decorator usage)

        Returns:
            Unsubscribe function, or decorator if handler not provided
        """
        if handler is None:
            # Decorator usage
            def decorator(fn: EventHandler) -> UnsubscribeFunc:
                return self._add_listener(event, fn, once=False)

            return decorator

        return self._add_listener(event, handler, once=False)

    def once(
        self,
        event: str,
        handler: Optional[EventHandler] = None,
    ) -> Union[UnsubscribeFunc, Callable[[EventHandler], UnsubscribeFunc]]:
        """Register a one-time event handler.

        The handler will be automatically removed after it fires once.

        Args:
            event: Event name to listen for
            handler: Callback function (optional for decorator usage)

        Returns:
            Unsubscribe function, or decorator if handler not provided
        """
        if handler is None:
            # Decorator usage
            def decorator(fn: EventHandler) -> UnsubscribeFunc:
                return self._add_listener(event, fn, once=True)

            return decorator

        return self._add_listener(event, handler, once=True)

    def off(self, event: str, handler: Optional[EventHandler] = None) -> None:
        """Remove an event handler.

        Args:
            event: Event name
            handler: Specific handler to remove, or None to remove all
        """
        with self._lock:
            if event not in self._listeners:
                return

            if handler is None:
                # Remove all listeners for this event
                del self._listeners[event]
            else:
                # Remove specific handler
                self._listeners[event] = [
                    listener for listener in self._listeners[event] if listener.handler != handler
                ]
                if not self._listeners[event]:
                    del self._listeners[event]

    def emit(self, event: str, data: Any = None) -> bool:
        """Emit an event to all registered handlers.

        Args:
            event: Event name
            data: Event data to pass to handlers

        Returns:
            True if any handlers were called
        """
        return self._emit_internal(event, data, cancellable=False)

    def emit_cancellable(self, event: str, data: Any = None) -> bool:
        """Emit a cancellable event to all registered handlers.

        If any handler returns False, the event is considered cancelled.
        Useful for events like 'closing' where handlers can prevent the action.

        Args:
            event: Event name
            data: Event data to pass to handlers

        Returns:
            True if event was NOT cancelled (all handlers returned True/None)
            False if event was cancelled (any handler returned False)
        """
        return self._emit_internal(event, data, cancellable=True)

    def _emit_internal(self, event: str, data: Any, cancellable: bool) -> bool:
        """Internal emit implementation with cancellation support.

        Args:
            event: Event name
            data: Event data to pass to handlers
            cancellable: If True, check handler return values for cancellation

        Returns:
            For cancellable events: True if not cancelled, False if cancelled
            For non-cancellable events: True if any handlers were called, False otherwise
        """
        with self._lock:
            listeners = self._listeners.get(event, []).copy()

        if not listeners:
            # No handlers:
            # - For cancellable events: return False (not cancellable = False)
            # - For non-cancellable events: return False (no handlers were called)
            return False

        # Process listeners outside lock
        to_remove: List[_EventListener] = []
        cancelled = False

        for listener in listeners:
            try:
                if data is None:
                    result = listener.handler()
                else:
                    result = listener.handler(data)

                # Check for cancellation
                if cancellable and result is False:
                    cancelled = True
                    # Continue calling other handlers but mark as cancelled

            except Exception as e:
                logger.exception(f"Error in event handler for '{event}': {e}")

            if listener.once:
                to_remove.append(listener)

        # Remove once listeners
        if to_remove:
            with self._lock:
                for listener in to_remove:
                    if event in self._listeners and listener in self._listeners[event]:
                        self._listeners[event].remove(listener)

        if cancellable:
            return not cancelled  # Return True if NOT cancelled
        return True  # Return True if handlers were called

    def remove_all_listeners(self, event: Optional[str] = None) -> None:
        """Remove all listeners, or all listeners for a specific event.

        Args:
            event: Event name, or None to remove all listeners
        """
        with self._lock:
            if event is None:
                self._listeners.clear()
            elif event in self._listeners:
                del self._listeners[event]

    def listener_count(self, event: str) -> int:
        """Get the number of listeners for an event.

        Args:
            event: Event name

        Returns:
            Number of registered listeners
        """
        with self._lock:
            return len(self._listeners.get(event, []))

    def event_names(self) -> List[str]:
        """Get all event names with registered listeners.

        Returns:
            List of event names
        """
        with self._lock:
            return list(self._listeners.keys())

    def _add_listener(self, event: str, handler: EventHandler, once: bool) -> UnsubscribeFunc:
        """Internal method to add a listener.

        Args:
            event: Event name
            handler: Event handler
            once: Whether this is a one-time listener

        Returns:
            Unsubscribe function
        """
        listener = _EventListener(handler=handler, once=once)

        with self._lock:
            if event not in self._listeners:
                self._listeners[event] = []
            self._listeners[event].append(listener)

        def unsubscribe() -> None:
            with self._lock:
                if event in self._listeners and listener in self._listeners[event]:
                    self._listeners[event].remove(listener)
                    if not self._listeners[event]:
                        del self._listeners[event]

        return unsubscribe


# Deprecated function wrappers for migration
def deprecated(message: str) -> Callable:
    """Decorator to mark functions as deprecated."""

    def decorator(func: Callable) -> Callable:
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            warnings.warn(
                f"{func.__name__} is deprecated: {message}",
                DeprecationWarning,
                stacklevel=2,
            )
            return func(*args, **kwargs)

        wrapper.__doc__ = f"DEPRECATED: {message}\n\n{func.__doc__ or ''}"
        return wrapper

    return decorator
