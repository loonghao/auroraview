# -*- coding: utf-8 -*-
"""Thread-safe wrapper for WebView operations in DCC environments."""

from __future__ import annotations

import logging
import threading
from typing import Any, Callable, Optional, TypeVar

from .registry import get_dispatcher_backend

T = TypeVar("T")
logger = logging.getLogger(__name__)


# =============================================================================
# Convenience Functions
# =============================================================================


def run_on_main_thread(func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
    """Execute a function on the main thread (fire-and-forget).

    Args:
        func: Function to execute on the main thread
        *args: Positional arguments to pass to the function
        **kwargs: Keyword arguments to pass to the function
    """
    backend = get_dispatcher_backend()
    backend.run_deferred(func, *args, **kwargs)


def run_on_main_thread_sync(func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
    """Execute a function on the main thread and wait for the result.

    Args:
        func: Function to execute on the main thread
        *args: Positional arguments to pass to the function
        **kwargs: Keyword arguments to pass to the function

    Returns:
        The return value of the function

    Raises:
        Exception: Re-raises any exception that occurred in the function
    """
    backend = get_dispatcher_backend()
    return backend.run_sync(func, *args, **kwargs)


def is_main_thread() -> bool:
    """Check if the current thread is the main/UI thread.

    Returns:
        True if running on the main thread, False otherwise.
    """
    backend = get_dispatcher_backend()
    return backend.is_main_thread()


def run_on_main_thread_sync_with_timeout(
    func: Callable[..., T],
    *args: Any,
    timeout: float = 30.0,
    **kwargs: Any,
) -> T:
    """Execute a function on the main thread with timeout protection.

    Args:
        func: Function to execute on the main thread
        *args: Positional arguments to pass to the function
        timeout: Maximum wait time in seconds (default: 30.0)
        **kwargs: Keyword arguments to pass to the function

    Returns:
        The return value of the function

    Raises:
        ThreadDispatchTimeoutError: If execution doesn't complete within timeout
        Exception: Re-raises any exception that occurred in the function
    """
    from .base import ThreadDispatchTimeoutError

    if is_main_thread():
        return func(*args, **kwargs)

    result_holder: list = [None]
    error_holder: list = [None]
    event = threading.Event()

    def wrapper():
        try:
            result_holder[0] = func(*args, **kwargs)
        except Exception as e:
            error_holder[0] = e
        finally:
            event.set()

    run_on_main_thread(wrapper)

    if not event.wait(timeout=timeout):
        raise ThreadDispatchTimeoutError(
            f"Main thread execution timed out after {timeout}s. Function: {func.__name__}"
        )

    if error_holder[0]:
        raise error_holder[0]

    return result_holder[0]


# =============================================================================
# Decorators
# =============================================================================


def ensure_main_thread(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to ensure a function runs on the main thread.

    If called from a background thread, the function will be
    dispatched to the main thread. If already on the main thread,
    the function runs directly.

    Args:
        func: Function to wrap

    Returns:
        Wrapped function that always runs on main thread
    """
    import functools

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> T:
        if is_main_thread():
            return func(*args, **kwargs)
        else:
            return run_on_main_thread_sync(func, *args, **kwargs)

    return wrapper


def defer_to_main_thread(func: Callable[..., T]) -> Callable[..., None]:
    """Decorator to defer a function to the main thread (fire-and-forget).

    The decorated function will always be queued for execution on the
    main thread and returns immediately without waiting.

    Args:
        func: Function to wrap

    Returns:
        Wrapped function that defers to main thread
    """
    import functools

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> None:
        run_on_main_thread(func, *args, **kwargs)

    return wrapper


def dcc_thread_safe(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to ensure function runs on DCC main thread.

    When called from a background thread, the function execution is
    marshaled to the DCC main thread and the call blocks until completion.

    Args:
        func: Function to wrap

    Returns:
        Wrapped function that always runs on main thread
    """
    import functools

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> T:
        if is_main_thread():
            return func(*args, **kwargs)
        logger.debug(f"[dcc_thread_safe] Marshaling {func.__name__} to main thread")
        return run_on_main_thread_sync(func, *args, **kwargs)

    return wrapper


def dcc_thread_safe_async(func: Callable[..., None]) -> Callable[..., None]:
    """Decorator for fire-and-forget execution on DCC main thread.

    The decorated function will be queued for execution on the main thread
    and returns immediately without waiting.

    Args:
        func: Function to wrap (should return None)

    Returns:
        Wrapped function that queues execution on main thread
    """
    import functools

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> None:
        if is_main_thread():
            func(*args, **kwargs)
        else:
            logger.debug(f"[dcc_thread_safe_async] Queueing {func.__name__} for main thread")
            run_on_main_thread(func, *args, **kwargs)

    return wrapper


def wrap_callback_for_dcc(
    callback: Callable[..., T],
    async_mode: bool = False,
) -> Callable[..., T]:
    """Wrap a callback function for safe execution in DCC environments.

    Args:
        callback: The callback function to wrap
        async_mode: If True, use fire-and-forget mode (default: False)

    Returns:
        Wrapped callback that runs on DCC main thread
    """
    if async_mode:
        return dcc_thread_safe_async(callback)  # type: ignore
    return dcc_thread_safe(callback)


# =============================================================================
# DCCThreadSafeWrapper
# =============================================================================


class DCCThreadSafeWrapper:
    """Thread-safe wrapper for WebView operations in DCC environments.

    This wrapper provides thread-safe versions of common WebView methods
    that can be safely called from any thread.
    """

    def __init__(self, webview: Any) -> None:
        """Initialize the thread-safe wrapper.

        Args:
            webview: The WebView instance to wrap
        """
        self._webview = webview
        self._proxy = webview.get_proxy()
        logger.debug(f"[DCCThreadSafeWrapper] Created wrapper for {webview}")

    def eval_js(self, script: str) -> None:
        """Execute JavaScript code (thread-safe, fire-and-forget).

        Args:
            script: JavaScript code to execute
        """
        self._proxy.eval_js(script)

    def eval_js_sync(
        self,
        script: str,
        timeout_ms: int = 5000,
    ) -> Any:
        """Execute JavaScript and wait for result (blocking, thread-safe).

        Args:
            script: JavaScript code to execute
            timeout_ms: Timeout in milliseconds (default: 5000)

        Returns:
            The result of the JavaScript execution

        Raises:
            RuntimeError: If JavaScript execution fails
            TimeoutError: If the operation times out
        """
        result_holder: list = [None]
        error_holder: list = [None]
        event = threading.Event()

        def callback(res: Any, err: Optional[str]) -> None:
            result_holder[0] = res
            error_holder[0] = err
            event.set()

        self._proxy.eval_js_async(script, callback, timeout_ms)

        timeout_sec = timeout_ms / 1000.0 + 1.0
        if not event.wait(timeout=timeout_sec):
            raise TimeoutError(f"JavaScript execution timed out after {timeout_ms}ms")

        if error_holder[0]:
            raise RuntimeError(f"JavaScript error: {error_holder[0]}")

        return result_holder[0]

    def emit(self, event_name: str, data: Optional[dict] = None) -> None:
        """Emit an event to JavaScript (thread-safe).

        Args:
            event_name: Name of the event to emit
            data: Optional dictionary of data to send with the event
        """
        self._proxy.emit(event_name, data or {})

    def load_url(self, url: str) -> None:
        """Load a URL in the WebView (thread-safe).

        Args:
            url: URL to load
        """
        self._proxy.load_url(url)

    def load_html(self, html: str) -> None:
        """Load HTML content in the WebView (thread-safe).

        Args:
            html: HTML content to load
        """
        self._proxy.load_html(html)

    def reload(self) -> None:
        """Reload the current page (thread-safe)."""
        self._proxy.reload()

    def close(self) -> None:
        """Close the WebView window (thread-safe)."""
        self._proxy.close()

    def __repr__(self) -> str:
        """String representation of the wrapper."""
        return f"DCCThreadSafeWrapper(webview={self._webview})"
