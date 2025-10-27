"""Decorators for WebView event handling."""

from functools import wraps
from typing import Any, Callable, Optional


def on_event(event_name: str, webview: Optional[Any] = None) -> Callable:
    """Decorator to register a function as an event handler.

    This decorator can be used in two ways:

    1. With a WebView instance::

        webview = WebView()

        @on_event("my_event", webview)
        def handle_event(data):
            print(data)

    2. As a standalone decorator (requires manual registration)::

        @on_event("my_event")
        def handle_event(data):
            print(data)

        # Later, register with a WebView
        webview.register_callback("my_event", handle_event)

    Args:
        event_name: Name of the event to handle
        webview: Optional WebView instance to register with

    Returns:
        Decorated function
    """

    def decorator(func: Callable) -> Callable:
        # Store event name as function attribute
        func._event_name = event_name  # type: ignore

        # If webview is provided, register immediately
        if webview is not None:
            webview.register_callback(event_name, func)

        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)

        return wrapper

    return decorator


def throttle(seconds: float) -> Callable:
    """Decorator to throttle function calls.

    Ensures the decorated function is called at most once per `seconds` interval.

    Args:
        seconds: Minimum time between calls in seconds

    Example:
        >>> @throttle(1.0)
        >>> def on_mouse_move(data):
        >>>     print("Mouse moved")
    """
    import time

    def decorator(func: Callable) -> Callable:
        last_called = [0.0]

        @wraps(func)
        def wrapper(*args, **kwargs):
            now = time.time()
            if now - last_called[0] >= seconds:
                last_called[0] = now
                return func(*args, **kwargs)

        return wrapper

    return decorator


def debounce(seconds: float) -> Callable:
    """Decorator to debounce function calls.

    Delays function execution until `seconds` have passed since the last call.

    Args:
        seconds: Delay time in seconds

    Example:
        >>> @debounce(0.5)
        >>> def on_text_change(data):
        >>>     print("Text changed:", data)
    """
    import threading

    def decorator(func: Callable) -> Callable:
        timer = [None]

        @wraps(func)
        def wrapper(*args, **kwargs):
            # Cancel previous timer
            if timer[0] is not None:
                timer[0].cancel()

            # Create new timer
            timer[0] = threading.Timer(seconds, lambda: func(*args, **kwargs))
            timer[0].start()

        return wrapper

    return decorator
