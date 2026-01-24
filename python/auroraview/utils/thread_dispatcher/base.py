# -*- coding: utf-8 -*-
"""Thread dispatcher base classes and exceptions.

This module contains the abstract base class for thread dispatcher backends
and related exceptions.
"""

from __future__ import annotations

import threading
from abc import ABC, abstractmethod
from typing import Any, Callable, TypeVar

T = TypeVar("T")


# =============================================================================
# Thread Safety Exceptions
# =============================================================================


class ThreadSafetyError(Exception):
    """Base exception for thread safety errors."""

    pass


class ThreadDispatchTimeoutError(ThreadSafetyError):
    """Raised when main thread dispatch times out.

    This typically indicates a potential deadlock or an operation that
    is taking too long to complete on the main thread.
    """

    pass


class DeadlockDetectedError(ThreadSafetyError):
    """Raised when a potential deadlock is detected.

    This can occur when:
    - Lock order violations are detected
    - Circular wait conditions are identified
    - Cross-thread synchronous calls create a deadlock
    """

    pass


class ShutdownInProgressError(ThreadSafetyError):
    """Raised when an operation is attempted during shutdown.

    This occurs when trying to dispatch work to a thread or queue
    that is in the process of shutting down.
    """

    pass


# =============================================================================
# Abstract Base Class
# =============================================================================


class ThreadDispatcherBackend(ABC):
    """Abstract base class for thread dispatcher backends.

    Subclass this to implement custom thread dispatchers for different DCC environments.
    Each backend must implement three methods: is_available(), run_deferred(), and run_sync().
    """

    @abstractmethod
    def is_available(self) -> bool:
        """Check if this backend is available in the current environment.

        Returns:
            True if the backend can be used, False otherwise.
        """
        pass

    @abstractmethod
    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute a function on the main thread without waiting for result.

        This is a fire-and-forget operation. The function will be queued
        for execution on the main thread and this method returns immediately.

        Args:
            func: Function to execute on the main thread
            *args: Positional arguments to pass to the function
            **kwargs: Keyword arguments to pass to the function
        """
        pass

    @abstractmethod
    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute a function on the main thread and wait for the result.

        This is a blocking operation. The function will be executed on the
        main thread and this method blocks until it completes.

        Args:
            func: Function to execute on the main thread
            *args: Positional arguments to pass to the function
            **kwargs: Keyword arguments to pass to the function

        Returns:
            The return value of the function

        Raises:
            Exception: Re-raises any exception that occurred in the function
        """
        pass

    def is_main_thread(self) -> bool:
        """Check if the current thread is the main thread.

        Returns:
            True if running on the main thread, False otherwise.

        Note:
            Default implementation uses threading.main_thread().
            Override this for DCC-specific main thread detection.
        """
        return threading.current_thread() is threading.main_thread()

    def get_name(self) -> str:
        """Get the backend name for logging.

        Returns:
            Backend name (defaults to class name without 'Backend' suffix)
        """
        name = self.__class__.__name__
        if name.endswith("Backend"):
            name = name[:-7]
        if name.endswith("Dispatcher"):
            name = name[:-10]
        return name
