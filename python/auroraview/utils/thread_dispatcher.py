# -*- coding: utf-8 -*-
"""Thread dispatcher for DCC applications.

This module provides a unified API for executing code on the main/UI thread
across different DCC applications. Many DCC applications (Maya, Houdini, Blender,
Unreal Engine, etc.) require certain operations to be performed on the main thread.

The module uses a Strategy pattern with **lazy loading** and **dynamic registration**,
allowing:
- Automatic detection of the current DCC environment
- Registration of custom backends for specific applications
- Priority-based backend selection
- **Lazy loading**: DCC-specific backends are only loaded when needed
- **String-based registration**: Register backends by module path to avoid import errors
- **Entry point support**: External packages can register backends via entry points
- **Environment variable override**: Force a specific backend via AURORAVIEW_DISPATCHER

Design Philosophy:
    Unlike hardcoded backend classes, this module uses a **lazy discovery** approach.
    DCC backends are registered as string paths (e.g., "auroraview.utils.thread_dispatcher:MayaDispatcherBackend")
    and only imported when `get_dispatcher_backend()` is called. This prevents import
    errors when DCC modules are not available.

Supported DCC Applications:
    - Maya: Uses maya.utils.executeDeferred() and executeInMainThreadWithResult()
    - Houdini: Uses hdefereval.executeDeferred() and executeInMainThread()
    - Blender: Uses bpy.app.timers.register() for deferred execution
    - Nuke: Uses nuke.executeInMainThread() and executeInMainThreadWithResult()
    - 3ds Max: Uses pymxs.runtime.execute() with MaxPlus.Core.EvalOnMainThread()
    - Unreal Engine: Uses unreal.execute_on_game_thread() (UE5+)
    - Qt Applications: Uses QTimer.singleShot() for main thread execution

Example - Basic usage:
    >>> from auroraview.utils.thread_dispatcher import run_on_main_thread
    >>>
    >>> # Fire-and-forget execution on main thread
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
    >>> # This blocks until the function completes and returns the result
    >>> selected = run_on_main_thread_sync(get_selection)
    >>> print(selected)

Example - Custom backend registration (class):
    >>> from auroraview.utils.thread_dispatcher import (
    ...     ThreadDispatcherBackend,
    ...     register_dispatcher_backend
    ... )
    >>>
    >>> class MyDCCBackend(ThreadDispatcherBackend):
    ...     def is_available(self) -> bool:
    ...         try:
    ...             import my_dcc
    ...             return True
    ...         except ImportError:
    ...             return False
    ...
    ...     def run_deferred(self, func, *args, **kwargs):
    ...         import my_dcc
    ...         my_dcc.execute_deferred(lambda: func(*args, **kwargs))
    ...
    ...     def run_sync(self, func, *args, **kwargs):
    ...         import my_dcc
    ...         return my_dcc.execute_in_main_thread(lambda: func(*args, **kwargs))
    >>>
    >>> register_dispatcher_backend(MyDCCBackend, priority=300)

Example - String-based registration (lazy loading):
    >>> from auroraview.utils.thread_dispatcher import register_dispatcher_backend
    >>>
    >>> # Register by module path - only loaded when needed
    >>> register_dispatcher_backend(
    ...     "my_package.dispatchers:MyDCCBackend",
    ...     priority=300
    ... )

Example - Environment variable override:
    >>> # Set AURORAVIEW_DISPATCHER=qt to force Qt backend
    >>> # Set AURORAVIEW_DISPATCHER=fallback to use fallback backend
    >>> import os
    >>> os.environ["AURORAVIEW_DISPATCHER"] = "qt"
"""

from __future__ import annotations

import importlib
import logging
import os
import queue
import threading
from abc import ABC, abstractmethod
from typing import Any, Callable, List, Optional, Tuple, Type, TypeVar, Union

logger = logging.getLogger(__name__)

T = TypeVar("T")

# Environment variable to force a specific backend
ENV_DISPATCHER_BACKEND = "AURORAVIEW_DISPATCHER"


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
        return name


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

            # Maya provides this function to check main thread
            return maya.utils.isMainThread()
        except (ImportError, AttributeError):
            return super().is_main_thread()


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


class BlenderDispatcherBackend(ThreadDispatcherBackend):
    """Blender thread dispatcher backend.

    Uses bpy.app.timers.register() for deferred execution.
    For blocking execution, uses a queue-based approach with timers.

    Reference:
        https://docs.blender.org/api/current/bpy.app.timers.html
    """

    def is_available(self) -> bool:
        """Check if Blender is available."""
        try:
            import bpy  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using bpy.app.timers.register()."""
        import bpy

        def timer_callback():
            func(*args, **kwargs)
            return None  # Don't repeat

        bpy.app.timers.register(timer_callback, first_interval=0.0)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on main thread and wait for result.

        Uses a queue-based approach since Blender doesn't have a built-in
        executeInMainThreadWithResult equivalent.
        """
        import bpy

        if self.is_main_thread():
            return func(*args, **kwargs)

        result_queue: queue.Queue[Tuple[bool, Any]] = queue.Queue()

        def timer_callback():
            try:
                result = func(*args, **kwargs)
                result_queue.put((True, result))
            except Exception as e:
                result_queue.put((False, e))
            return None

        bpy.app.timers.register(timer_callback, first_interval=0.0)

        # Wait for result
        success, value = result_queue.get()
        if success:
            return value
        else:
            raise value

    def is_main_thread(self) -> bool:
        """Check if running on Blender's main thread."""
        # Blender's main thread is the Python main thread
        return threading.current_thread() is threading.main_thread()


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


class MaxDispatcherBackend(ThreadDispatcherBackend):
    """3ds Max thread dispatcher backend.

    Uses Qt's event loop for main thread execution since 3ds Max uses Qt internally.
    MaxPlus is deprecated since 3ds Max 2020, so we use Qt-based scheduling.

    For 3ds Max 2020+, the recommended approach is to use Qt's QTimer.singleShot()
    since 3ds Max's main thread runs a Qt event loop.

    Reference:
        https://help.autodesk.com/view/MAXDEV/2024/ENU/?guid=MAXDEV_Python_python_pymxs_html
    """

    def is_available(self) -> bool:
        """Check if 3ds Max is available."""
        try:
            import pymxs  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function on main thread (fire-and-forget).

        Uses Qt's QTimer.singleShot() since 3ds Max runs a Qt event loop.
        """
        if self.is_main_thread():
            func(*args, **kwargs)
            return

        try:
            # 3ds Max uses Qt internally, so we can use QTimer for deferred execution
            from qtpy.QtCore import QTimer

            QTimer.singleShot(0, lambda: func(*args, **kwargs))
        except ImportError:
            # If Qt is not available, log warning and execute directly
            logger.warning(
                "Qt not available in 3ds Max - executing function directly. "
                "This may cause thread safety issues."
            )
            func(*args, **kwargs)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on main thread and wait for result.

        Uses Qt's event loop with a blocking wait mechanism.
        """
        if self.is_main_thread():
            return func(*args, **kwargs)

        try:
            from qtpy.QtCore import QEventLoop, QTimer

            result_holder: list = [None]

            exception_holder: list = [None]
            event_loop = QEventLoop()

            def wrapper():
                try:
                    result_holder[0] = func(*args, **kwargs)
                except Exception as e:
                    exception_holder[0] = e
                finally:
                    event_loop.quit()

            QTimer.singleShot(0, wrapper)

            # Process events until wrapper completes
            event_loop.exec_()

            if exception_holder[0] is not None:
                raise exception_holder[0]
            return result_holder[0]
        except ImportError:
            # If Qt is not available, execute directly with warning
            logger.warning(
                "Qt not available in 3ds Max - executing function directly. "
                "This may cause thread safety issues."
            )
            return func(*args, **kwargs)

    def is_main_thread(self) -> bool:
        """Check if current thread is the main thread."""
        try:
            from qtpy.QtCore import QCoreApplication, QThread

            app = QCoreApplication.instance()
            if app is not None:
                return QThread.currentThread() == app.thread()
        except ImportError:
            pass

        return threading.current_thread() is threading.main_thread()


class UnrealDispatcherBackend(ThreadDispatcherBackend):
    """Unreal Engine thread dispatcher backend.

    Uses unreal.execute_on_game_thread() for game thread execution.
    This is critical for UE5 where many operations must run on the game thread.

    Reference:
        https://docs.unrealengine.com/5.0/en-US/PythonAPI/
        UE5 C++: AsyncTask(ENamedThreads::GameThread, [](){ ... });
    """

    def is_available(self) -> bool:
        """Check if Unreal Engine is available."""
        try:
            import unreal  # noqa: F401

            return True
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function on game thread using slate tick."""
        import unreal

        # UE5 Python API doesn't have direct execute_on_game_thread
        # Use slate application tick callback instead
        def tick_callback(delta_time):
            func(*args, **kwargs)
            return False  # Unregister after first call

        unreal.register_slate_post_tick_callback(tick_callback)

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on game thread and wait for result.

        Uses an event-based approach since UE Python doesn't have
        a built-in blocking main thread execution.
        """
        import unreal

        if self.is_main_thread():
            return func(*args, **kwargs)

        result_holder = [None]
        exception_holder = [None]
        done_event = threading.Event()

        def tick_callback(delta_time):
            try:
                result_holder[0] = func(*args, **kwargs)
            except Exception as e:
                exception_holder[0] = e
            finally:
                done_event.set()
            return False

        unreal.register_slate_post_tick_callback(tick_callback)

        # Wait for completion
        done_event.wait()

        if exception_holder[0] is not None:
            raise exception_holder[0]
        return result_holder[0]

    def is_main_thread(self) -> bool:
        """Check if running on Unreal's game thread."""
        try:
            import unreal

            return unreal.is_game_thread()
        except (ImportError, AttributeError):
            return super().is_main_thread()


class QtDispatcherBackend(ThreadDispatcherBackend):
    """Qt thread dispatcher backend.

    Uses QTimer.singleShot() for deferred execution and
    QMetaObject.invokeMethod() for blocking execution.

    This backend works with any Qt-based application including
    Maya, Houdini, Nuke, and standalone Qt applications.
    """

    def is_available(self) -> bool:
        """Check if Qt is available."""
        try:
            from qtpy.QtCore import QCoreApplication

            return QCoreApplication.instance() is not None
        except ImportError:
            return False

    def run_deferred(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
        """Execute function using QTimer.singleShot()."""
        from qtpy.QtCore import QTimer

        QTimer.singleShot(0, lambda: func(*args, **kwargs))

    def run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Execute function on Qt main thread and wait for result."""
        from qtpy.QtCore import QCoreApplication, QThread

        app = QCoreApplication.instance()
        if app is None:
            # No Qt app, just run directly
            return func(*args, **kwargs)

        # Check if already on main thread
        if QThread.currentThread() == app.thread():
            return func(*args, **kwargs)

        # Use event-based approach for cross-thread execution
        result_holder = [None]
        exception_holder = [None]
        done_event = threading.Event()

        def wrapper():
            try:
                result_holder[0] = func(*args, **kwargs)
            except Exception as e:
                exception_holder[0] = e
            finally:
                done_event.set()

        from qtpy.QtCore import QTimer

        QTimer.singleShot(0, wrapper)

        # Wait for completion
        done_event.wait()

        if exception_holder[0] is not None:
            raise exception_holder[0]
        return result_holder[0]

    def is_main_thread(self) -> bool:
        """Check if running on Qt's main thread."""
        try:
            from qtpy.QtCore import QCoreApplication, QThread

            app = QCoreApplication.instance()
            if app is None:
                return super().is_main_thread()
            return QThread.currentThread() == app.thread()
        except ImportError:
            return super().is_main_thread()


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
            # Queue for later execution - this is best effort
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


# Type alias for backend specification: either a class or a string path
BackendSpec = Union[Type[ThreadDispatcherBackend], str]

# Global registry of dispatcher backends
# Format: (priority, backend_spec, name_hint)
# backend_spec can be a class or a string path like "module:ClassName"
_DISPATCHER_BACKENDS: List[Tuple[int, BackendSpec, str]] = []

# Cached backend instance
_cached_backend: Optional[ThreadDispatcherBackend] = None

# Flag to track if built-in backends have been registered
_builtins_registered: bool = False


def _load_backend_class(spec: BackendSpec) -> Optional[Type[ThreadDispatcherBackend]]:
    """Load a backend class from a specification.

    Args:
        spec: Either a class or a string path like "module:ClassName"

    Returns:
        The backend class, or None if loading failed
    """
    if isinstance(spec, type):
        return spec

    if not isinstance(spec, str):
        logger.warning(f"Invalid backend spec type: {type(spec)}")
        return None

    # Parse "module:ClassName" format
    if ":" not in spec:
        logger.warning(f"Invalid backend spec format (expected 'module:ClassName'): {spec}")
        return None

    module_path, class_name = spec.rsplit(":", 1)

    try:
        module = importlib.import_module(module_path)
        cls = getattr(module, class_name)
        if not isinstance(cls, type) or not issubclass(cls, ThreadDispatcherBackend):
            logger.warning(f"{spec} is not a ThreadDispatcherBackend subclass")
            return None
        return cls
    except ImportError as e:
        logger.debug(f"Could not import {module_path}: {e}")
        return None
    except AttributeError as e:
        logger.debug(f"Could not find {class_name} in {module_path}: {e}")
        return None


def _get_spec_name(spec: BackendSpec, name_hint: str) -> str:
    """Get a display name for a backend specification."""
    if name_hint:
        return name_hint
    if isinstance(spec, type):
        name = spec.__name__
        if name.endswith("Backend"):
            name = name[:-7]
        return name
    if isinstance(spec, str) and ":" in spec:
        class_name = spec.rsplit(":", 1)[1]
        if class_name.endswith("Backend"):
            class_name = class_name[:-7]
        return class_name
    return str(spec)


def register_dispatcher_backend(
    backend: BackendSpec,
    priority: int = 0,
    *,
    name: str = "",
) -> None:
    """Register a thread dispatcher backend.

    Backends are tried in order of priority (highest first).
    The first available backend is used.

    This function supports two registration modes:
    1. **Class registration**: Pass a ThreadDispatcherBackend subclass directly
    2. **String registration**: Pass a module path like "module:ClassName"
       for lazy loading (only imported when needed)

    Args:
        backend: Either a ThreadDispatcherBackend subclass or a string path
                 in "module:ClassName" format
        priority: Priority value (higher = tried first). Default: 0
                 Built-in backends use:
                 - Maya: 200
                 - Houdini: 190
                 - Nuke: 180
                 - Blender: 170
                 - Max: 160
                 - Unreal: 150
                 - Qt: 100
                 - Fallback: 0
        name: Optional display name for the backend (used in logging)

    Example - Class registration:
        >>> class MyDCCBackend(ThreadDispatcherBackend):
        ...     # ... implementation ...
        >>>
        >>> register_dispatcher_backend(MyDCCBackend, priority=250)

    Example - String registration (lazy loading):
        >>> # Only loaded when get_dispatcher_backend() is called
        >>> register_dispatcher_backend(
        ...     "my_package.dispatchers:MyDCCBackend",
        ...     priority=250,
        ...     name="MyDCC"
        ... )
    """
    global _DISPATCHER_BACKENDS, _cached_backend

    # Invalidate cache when registering new backend
    _cached_backend = None

    # Get identifier for comparison
    if isinstance(backend, type):
        identifier = backend
    else:
        identifier = backend  # String path

    # Check if already registered (by class or by string path)
    for i, (_, existing_spec, _) in enumerate(_DISPATCHER_BACKENDS):
        if existing_spec is identifier or existing_spec == identifier:
            _DISPATCHER_BACKENDS[i] = (priority, backend, name)
            _DISPATCHER_BACKENDS.sort(key=lambda x: x[0], reverse=True)
            display_name = _get_spec_name(backend, name)
            logger.debug(f"Updated dispatcher backend {display_name} with priority {priority}")
            return

    _DISPATCHER_BACKENDS.append((priority, backend, name))
    _DISPATCHER_BACKENDS.sort(key=lambda x: x[0], reverse=True)
    display_name = _get_spec_name(backend, name)
    logger.debug(f"Registered dispatcher backend {display_name} with priority {priority}")


def unregister_dispatcher_backend(backend: BackendSpec) -> bool:
    """Unregister a previously registered backend.

    Args:
        backend: The backend class or string path to unregister

    Returns:
        True if the backend was found and removed, False otherwise
    """
    global _DISPATCHER_BACKENDS, _cached_backend

    for i, (_, existing_spec, _) in enumerate(_DISPATCHER_BACKENDS):
        if existing_spec is backend or existing_spec == backend:
            _DISPATCHER_BACKENDS.pop(i)
            _cached_backend = None
            return True
    return False


def clear_dispatcher_backends() -> None:
    """Clear all registered backends and reset to initial state.

    This is mainly useful for testing or when you want to completely
    reconfigure the backend system.
    """
    global _DISPATCHER_BACKENDS, _cached_backend, _builtins_registered
    _DISPATCHER_BACKENDS.clear()
    _cached_backend = None
    _builtins_registered = False


def _register_builtin_backends() -> None:
    """Register built-in backends lazily.

    This is called automatically on first use. Built-in backends are
    registered as classes (not string paths) since they're defined in
    this module.
    """
    global _builtins_registered

    if _builtins_registered:
        return

    _builtins_registered = True

    # Register built-in backends with appropriate priorities
    # Higher priority = tried first
    # DCC-specific backends have higher priority than generic Qt backend
    #
    # Note: These are registered as classes since they're defined in this module.
    # External packages should use string registration for lazy loading.
    register_dispatcher_backend(MayaDispatcherBackend, priority=200, name="Maya")
    register_dispatcher_backend(HoudiniDispatcherBackend, priority=190, name="Houdini")
    register_dispatcher_backend(NukeDispatcherBackend, priority=180, name="Nuke")
    register_dispatcher_backend(BlenderDispatcherBackend, priority=170, name="Blender")
    register_dispatcher_backend(MaxDispatcherBackend, priority=160, name="Max")
    register_dispatcher_backend(UnrealDispatcherBackend, priority=150, name="Unreal")
    register_dispatcher_backend(QtDispatcherBackend, priority=100, name="Qt")
    register_dispatcher_backend(FallbackDispatcherBackend, priority=0, name="Fallback")


def get_dispatcher_backend() -> ThreadDispatcherBackend:
    """Get the first available thread dispatcher backend.

    Tries backends in order of priority (highest first).
    Supports environment variable override via AURORAVIEW_DISPATCHER.

    Environment Variable:
        AURORAVIEW_DISPATCHER: Force a specific backend by name (case-insensitive).
        Valid values: "maya", "houdini", "nuke", "blender", "max", "unreal", "qt", "fallback"

    Returns:
        First available ThreadDispatcherBackend instance.

    Raises:
        RuntimeError: If no backend is available (should never happen
                     since FallbackDispatcherBackend is always available).
    """
    global _cached_backend

    if _cached_backend is not None:
        return _cached_backend

    # Ensure built-in backends are registered
    _register_builtin_backends()

    # Check for environment variable override
    env_backend = os.environ.get(ENV_DISPATCHER_BACKEND, "").strip().lower()
    if env_backend:
        for priority, spec, name_hint in _DISPATCHER_BACKENDS:
            display_name = _get_spec_name(spec, name_hint).lower()
            if display_name == env_backend:
                backend_class = _load_backend_class(spec)
                if backend_class is not None:
                    try:
                        backend = backend_class()
                        if backend.is_available():
                            logger.info(
                                f"Using dispatcher backend from environment: "
                                f"{backend.get_name()} (priority={priority})"
                            )
                            _cached_backend = backend
                            return backend
                        else:
                            logger.warning(
                                f"Environment-specified backend '{env_backend}' is not available"
                            )
                    except Exception as e:
                        logger.warning(
                            f"Failed to initialize environment-specified backend "
                            f"'{env_backend}': {e}"
                        )
                break

    # Try backends in priority order
    for priority, spec, name_hint in _DISPATCHER_BACKENDS:
        backend_class = _load_backend_class(spec)
        if backend_class is None:
            continue

        try:
            backend = backend_class()
            if backend.is_available():
                logger.debug(
                    f"Selected dispatcher backend: {backend.get_name()} (priority={priority})"
                )
                _cached_backend = backend
                return backend
        except Exception as e:
            display_name = _get_spec_name(spec, name_hint)
            logger.warning(f"Failed to initialize {display_name}: {e}", exc_info=True)
            continue

    raise RuntimeError("No thread dispatcher backend available!")


def list_dispatcher_backends() -> List[Tuple[int, str, bool]]:
    """List all registered backends with their availability.

    Returns:
        List of (priority, name, is_available) tuples.

    Example:
        >>> for priority, name, available in list_dispatcher_backends():
        ...     status = "+" if available else "-"
        ...     print(f"{status} {name} (priority={priority})")
    """
    # Ensure built-in backends are registered
    _register_builtin_backends()

    result = []
    for priority, spec, name_hint in _DISPATCHER_BACKENDS:
        display_name = _get_spec_name(spec, name_hint)

        backend_class = _load_backend_class(spec)
        if backend_class is None:
            result.append((priority, display_name, False))
            continue

        try:
            backend = backend_class()
            available = backend.is_available()
        except Exception:
            available = False

        result.append((priority, display_name, available))

    return result


# DCC detection priority threshold - backends with priority >= this are DCC-specific
_DCC_PRIORITY_THRESHOLD = 150  # Unreal is 150, Qt is 100, Fallback is 0


def is_dcc_environment() -> bool:
    """Check if we're running inside a DCC application.

    This function detects whether AuroraView is running inside a DCC
    (Digital Content Creation) application like Maya, Houdini, Blender, etc.

    Returns:
        True if a DCC-specific backend is available (priority >= 150),
        False if only generic backends (Qt, Fallback) are available.

    Example:
        >>> from auroraview.utils.thread_dispatcher import is_dcc_environment
        >>>
        >>> if is_dcc_environment():
        ...     print("Running inside a DCC application")
        ... else:
        ...     print("Running standalone")

    Note:
        This checks for DCC-specific backends with priority >= 150:
        - Maya (200), Houdini (190), Nuke (180), Blender (170),
          3ds Max (160), Unreal (150)

        Generic backends are excluded:
        - Qt (100), Fallback (0)
    """
    _register_builtin_backends()

    for priority, spec, _ in _DISPATCHER_BACKENDS:
        # Skip non-DCC backends (Qt, Fallback)
        if priority < _DCC_PRIORITY_THRESHOLD:
            continue

        backend_class = _load_backend_class(spec)
        if backend_class is None:
            continue

        try:
            backend = backend_class()
            if backend.is_available():
                logger.debug(f"DCC environment detected: {backend.get_name()}")
                return True
        except Exception:
            continue

    return False


def get_current_dcc_name() -> Optional[str]:
    """Get the name of the current DCC application.

    Returns:
        Name of the DCC application (e.g., "Maya", "Blender", "Houdini"),
        or None if not running inside a DCC.

    Example:
        >>> from auroraview.utils.thread_dispatcher import get_current_dcc_name
        >>>
        >>> dcc = get_current_dcc_name()
        >>> if dcc:
        ...     print(f"Running in {dcc}")
    """
    _register_builtin_backends()

    for priority, spec, _ in _DISPATCHER_BACKENDS:
        if priority < _DCC_PRIORITY_THRESHOLD:
            continue

        backend_class = _load_backend_class(spec)
        if backend_class is None:
            continue

        try:
            backend = backend_class()
            if backend.is_available():
                return backend.get_name()
        except Exception:
            continue

    return None


def run_on_main_thread(func: Callable[..., T], *args: Any, **kwargs: Any) -> None:
    """Execute a function on the main thread (fire-and-forget).

    This is a convenience function that uses the best available backend
    to execute the given function on the main/UI thread.

    Args:
        func: Function to execute on the main thread
        *args: Positional arguments to pass to the function
        **kwargs: Keyword arguments to pass to the function

    Example:
        >>> def create_sphere():
        ...     import maya.cmds as cmds
        ...     cmds.polySphere()
        >>>
        >>> run_on_main_thread(create_sphere)
    """
    backend = get_dispatcher_backend()
    backend.run_deferred(func, *args, **kwargs)


def run_on_main_thread_sync(func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
    """Execute a function on the main thread and wait for the result.

    This is a convenience function that uses the best available backend
    to execute the given function on the main/UI thread and blocks until
    the function completes.

    Args:
        func: Function to execute on the main thread
        *args: Positional arguments to pass to the function
        **kwargs: Keyword arguments to pass to the function

    Returns:
        The return value of the function

    Raises:
        Exception: Re-raises any exception that occurred in the function

    Example:
        >>> def get_selected_objects():
        ...     import maya.cmds as cmds
        ...     return cmds.ls(selection=True)
        >>>
        >>> selected = run_on_main_thread_sync(get_selected_objects)
        >>> print(selected)
    """
    backend = get_dispatcher_backend()
    return backend.run_sync(func, *args, **kwargs)


def run_on_main_thread_sync_with_timeout(
    func: Callable[..., T],
    *args: Any,
    timeout: float = 30.0,
    **kwargs: Any,
) -> T:
    """Execute a function on the main thread with timeout protection.

    This is similar to run_on_main_thread_sync but adds timeout protection
    to prevent indefinite blocking in case of deadlocks or hung operations.

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

    Example:
        >>> def slow_operation():
        ...     import maya.cmds as cmds
        ...     return cmds.ls(selection=True)
        >>>
        >>> try:
        ...     result = run_on_main_thread_sync_with_timeout(
        ...         slow_operation, timeout=5.0
        ...     )
        ... except ThreadDispatchTimeoutError:
        ...     print("Operation timed out!")
    """
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


def is_main_thread() -> bool:
    """Check if the current thread is the main/UI thread.

    Uses the best available backend to determine if the current
    thread is the main thread.

    Returns:
        True if running on the main thread, False otherwise.

    Example:
        >>> if not is_main_thread():
        ...     run_on_main_thread(my_ui_function)
        ... else:
        ...     my_ui_function()
    """
    backend = get_dispatcher_backend()
    return backend.is_main_thread()


def ensure_main_thread(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to ensure a function runs on the main thread.

    If called from a background thread, the function will be
    dispatched to the main thread. If already on the main thread,
    the function runs directly.

    Args:
        func: Function to wrap

    Returns:
        Wrapped function that always runs on main thread

    Example:
        >>> @ensure_main_thread
        ... def update_ui():
        ...     import maya.cmds as cmds
        ...     cmds.refresh()
        >>>
        >>> # Safe to call from any thread
        >>> update_ui()
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

    Example:
        >>> @defer_to_main_thread
        ... def log_to_ui(message):
        ...     print(f"[UI] {message}")
        >>>
        >>> # Returns immediately, executes later on main thread
        >>> log_to_ui("Hello from background thread!")
    """
    import functools

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> None:
        run_on_main_thread(func, *args, **kwargs)

    return wrapper


# =============================================================================
# DCC Thread Safety Utilities for WebView Integration
# =============================================================================
#
# The following utilities provide thread-safe integration between WebView
# and DCC applications. They are aliases/wrappers around the core dispatcher
# functions with DCC-specific naming and documentation.
# =============================================================================


def dcc_thread_safe(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to ensure function runs on DCC main thread.

    When called from a background thread (e.g., WebView event handler),
    the function execution is marshaled to the DCC main thread and the
    call blocks until completion.

    When called from the main thread, the function executes directly
    without any overhead.

    This is an alias for `ensure_main_thread` with DCC-specific naming
    and documentation for WebView integration.

    Args:
        func: Function to wrap

    Returns:
        Wrapped function that always runs on main thread

    Example:
        >>> from auroraview import WebView
        >>> from auroraview.utils import dcc_thread_safe
        >>>
        >>> webview = WebView(parent=dcc_hwnd)
        >>>
        >>> @webview.on("export")
        >>> @dcc_thread_safe
        >>> def handle_export(data):
        ...     import maya.cmds as cmds
        ...     cmds.file(exportSelected=True, type='mayaAscii')
        ...     return {"exported": True}

    Note:
        This decorator is blocking - it waits for the function to complete.
        Use @dcc_thread_safe_async for fire-and-forget execution.
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
    and returns immediately without waiting. Use this when you don't need
    the return value.

    This is an alias for `defer_to_main_thread` with DCC-specific naming
    and documentation for WebView integration.

    Args:
        func: Function to wrap (should return None)

    Returns:
        Wrapped function that queues execution on main thread

    Example:
        >>> from auroraview.utils import dcc_thread_safe_async
        >>>
        >>> @dcc_thread_safe_async
        >>> def update_viewport():
        ...     import maya.cmds as cmds
        ...     cmds.refresh()
        >>>
        >>> update_viewport()  # Returns immediately

    Warning:
        - The return value of the function is discarded
        - Exceptions are logged but not raised to the caller
        - Order of execution is not guaranteed for multiple calls
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


class DCCThreadSafeWrapper:
    """Thread-safe wrapper for WebView operations in DCC environments.

    This wrapper provides thread-safe versions of common WebView methods
    that can be safely called from any thread. It uses the internal
    WebViewProxy and message queue for cross-thread communication.

    All methods are designed to be non-blocking where possible, using
    the message queue to deliver operations to the WebView thread.

    Example:
        >>> webview = WebView(parent=dcc_hwnd)
        >>> safe = webview.thread_safe()
        >>>
        >>> # These can be called from any thread:
        >>> safe.eval_js("updateUI()")
        >>> safe.emit("status", {"ready": True})
        >>> safe.load_url("https://example.com")

    Attributes:
        _webview: Reference to the original WebView instance
        _proxy: Thread-safe WebViewProxy for cross-thread operations
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

        The script is queued for execution on the WebView thread.
        This method returns immediately without waiting for the result.

        Args:
            script: JavaScript code to execute

        Example:
            >>> safe.eval_js("console.log('Hello from Python!')")
            >>> safe.eval_js("document.title = 'New Title'")
        """
        self._proxy.eval_js(script)

    def eval_js_sync(
        self,
        script: str,
        timeout_ms: int = 5000,
    ) -> Any:
        """Execute JavaScript and wait for result (blocking, thread-safe).

        This method blocks until the JavaScript execution completes or
        times out. Use with caution as it can block the calling thread.

        Args:
            script: JavaScript code to execute
            timeout_ms: Timeout in milliseconds (default: 5000)

        Returns:
            The result of the JavaScript execution (JSON-compatible value)

        Raises:
            RuntimeError: If JavaScript execution fails or times out
            TimeoutError: If the operation times out

        Example:
            >>> result = safe.eval_js_sync("document.title")
            >>> print(f"Title: {result}")
            >>>
            >>> data = safe.eval_js_sync("JSON.stringify(appState)")
        """

        result_holder: list = [None]
        error_holder: list = [None]
        event = threading.Event()

        def callback(res: Any, err: Optional[str]) -> None:
            result_holder[0] = res
            error_holder[0] = err
            event.set()

        self._proxy.eval_js_async(script, callback, timeout_ms)

        # Wait with a bit of extra time for the callback
        timeout_sec = timeout_ms / 1000.0 + 1.0
        if not event.wait(timeout=timeout_sec):
            raise TimeoutError(f"JavaScript execution timed out after {timeout_ms}ms")

        if error_holder[0]:
            raise RuntimeError(f"JavaScript error: {error_holder[0]}")

        return result_holder[0]

    def emit(self, event_name: str, data: Optional[dict] = None) -> None:
        """Emit an event to JavaScript (thread-safe).

        Events are delivered asynchronously to the JavaScript side.
        The event will be received by handlers registered via
        `window.auroraview.on(event_name, handler)`.

        Args:
            event_name: Name of the event to emit
            data: Optional dictionary of data to send with the event

        Example:
            >>> safe.emit("dataLoaded", {"count": 100})
            >>> safe.emit("error", {"message": "Something went wrong"})
        """
        self._proxy.emit(event_name, data or {})

    def load_url(self, url: str) -> None:
        """Load a URL in the WebView (thread-safe).

        Args:
            url: URL to load (http://, https://, or file://)

        Example:
            >>> safe.load_url("https://example.com")
            >>> safe.load_url("file:///C:/path/to/index.html")
        """
        self._proxy.load_url(url)

    def load_html(self, html: str) -> None:
        """Load HTML content in the WebView (thread-safe).

        Args:
            html: HTML content to load

        Example:
            >>> safe.load_html("<h1>Hello World</h1>")
        """
        self._proxy.load_html(html)

    def reload(self) -> None:
        """Reload the current page (thread-safe).

        Example:
            >>> safe.reload()
        """
        self._proxy.reload()

    def close(self) -> None:
        """Close the WebView window (thread-safe).

        This sends a close request to the WebView. The actual closing
        may happen asynchronously.

        Example:
            >>> safe.close()
        """
        self._proxy.close()

    def __repr__(self) -> str:
        """String representation of the wrapper."""
        return f"DCCThreadSafeWrapper(webview={self._webview})"


def wrap_callback_for_dcc(
    callback: Callable[..., T],
    async_mode: bool = False,
) -> Callable[..., T]:
    """Wrap a callback function for safe execution in DCC environments.

    This is a utility function used internally to wrap event handlers
    when dcc_mode is enabled on a WebView.

    Args:
        callback: The callback function to wrap
        async_mode: If True, use fire-and-forget mode (default: False)

    Returns:
        Wrapped callback that runs on DCC main thread

    Example:
        >>> def my_handler(data):
        ...     import maya.cmds as cmds
        ...     return cmds.ls(selection=True)
        >>>
        >>> safe_handler = wrap_callback_for_dcc(my_handler)
    """
    if async_mode:
        return dcc_thread_safe_async(callback)  # type: ignore
    return dcc_thread_safe(callback)
