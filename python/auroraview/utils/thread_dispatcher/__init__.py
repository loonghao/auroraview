# -*- coding: utf-8 -*-
"""Thread dispatcher for DCC applications.

This module provides a unified API for executing code on the main/UI thread
across different DCC applications. Many DCC applications (Maya, Houdini, Blender,
Unreal Engine, etc.) require certain operations to be performed on the main thread.

Example - Basic usage:
    >>> from auroraview.utils.thread_dispatcher import run_on_main_thread
    >>>
    >>> def create_cube():
    ...     import maya.cmds as cmds
    ...     cmds.polyCube()
    >>>
    >>> run_on_main_thread(create_cube)

Example - Blocking execution with return value:
    >>> from auroraview.utils.thread_dispatcher import run_on_main_thread_sync
    >>>
    >>> def get_selection():
    ...     import maya.cmds as cmds
    ...     return cmds.ls(selection=True)
    >>>
    >>> selected = run_on_main_thread_sync(get_selection)
    >>> print(selected)
"""

from __future__ import annotations

# Base classes and exceptions
from .base import (
    DeadlockDetectedError,
    ShutdownInProgressError,
    ThreadDispatcherBackend,
    ThreadDispatchTimeoutError,
    ThreadSafetyError,
)

# Registry functions
from .registry import (
    DispatcherPriority,
    clear_dispatcher_backends,
    get_current_dcc_name,
    get_dispatcher_backend,
    is_dcc_environment,
    list_dispatcher_backends,
    register_dispatcher_backend,
    unregister_dispatcher_backend,
)

# Wrapper and utility functions
from .wrapper import (
    DCCThreadSafeWrapper,
    dcc_thread_safe,
    dcc_thread_safe_async,
    defer_to_main_thread,
    ensure_main_thread,
    is_main_thread,
    run_on_main_thread,
    run_on_main_thread_sync,
    run_on_main_thread_sync_with_timeout,
    wrap_callback_for_dcc,
)

__all__ = [
    # Exceptions
    "ThreadSafetyError",
    "ThreadDispatchTimeoutError",
    "DeadlockDetectedError",
    "ShutdownInProgressError",
    # Base class
    "ThreadDispatcherBackend",
    # Priority constants
    "DispatcherPriority",
    # Registry functions
    "register_dispatcher_backend",
    "unregister_dispatcher_backend",
    "clear_dispatcher_backends",
    "get_dispatcher_backend",
    "list_dispatcher_backends",
    "is_dcc_environment",
    "get_current_dcc_name",
    # Convenience functions
    "run_on_main_thread",
    "run_on_main_thread_sync",
    "run_on_main_thread_sync_with_timeout",
    "is_main_thread",
    # Decorators
    "ensure_main_thread",
    "defer_to_main_thread",
    "dcc_thread_safe",
    "dcc_thread_safe_async",
    "wrap_callback_for_dcc",
    # Wrapper class
    "DCCThreadSafeWrapper",
]
