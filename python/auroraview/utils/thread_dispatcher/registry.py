# -*- coding: utf-8 -*-
"""Thread dispatcher backend registry.

This module manages the registration and discovery of thread dispatcher backends.
"""

from __future__ import annotations

import importlib
import logging
import os
from typing import TYPE_CHECKING, List, Optional, Tuple, Type, Union

if TYPE_CHECKING:
    from .base import ThreadDispatcherBackend

logger = logging.getLogger(__name__)

# Environment variable to force a specific backend
ENV_DISPATCHER_BACKEND = "AURORAVIEW_DISPATCHER"

# Type alias for backend specification: either a class or a string path
BackendSpec = Union[Type["ThreadDispatcherBackend"], str]

# Global registry of dispatcher backends
# Format: (priority, backend_spec, name_hint)
_DISPATCHER_BACKENDS: List[Tuple[int, BackendSpec, str]] = []

# Cached backend instance
_cached_backend: Optional["ThreadDispatcherBackend"] = None

# Flag to track if built-in backends have been registered
_builtins_registered: bool = False


class DispatcherPriority:
    """Priority constants for dispatcher backends.

    Higher priority backends are tried first. DCC-specific backends
    have higher priority than generic backends.
    """

    MAYA = 200
    HOUDINI = 190
    NUKE = 180
    BLENDER = 170
    MAX = 160
    UNREAL = 150
    DCC_THRESHOLD = 150  # Backends >= this are considered DCC
    QT = 100
    FALLBACK = 0


def _load_backend_class(spec: BackendSpec) -> Optional[Type["ThreadDispatcherBackend"]]:
    """Load a backend class from a specification.

    Args:
        spec: Either a class or a string path like "module:ClassName"

    Returns:
        The backend class, or None if loading failed
    """
    from .base import ThreadDispatcherBackend

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
        if name.endswith("Dispatcher"):
            name = name[:-10]
        return name
    if isinstance(spec, str) and ":" in spec:
        class_name = spec.rsplit(":", 1)[1]
        if class_name.endswith("Backend"):
            class_name = class_name[:-7]
        if class_name.endswith("Dispatcher"):
            class_name = class_name[:-10]
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

    Args:
        backend: Either a ThreadDispatcherBackend subclass or a string path
                 in "module:ClassName" format
        priority: Priority value (higher = tried first)
        name: Optional display name for the backend
    """
    global _DISPATCHER_BACKENDS, _cached_backend

    _cached_backend = None

    if isinstance(backend, type):
        identifier = backend
    else:
        identifier = backend

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
    """Clear all registered backends and reset to initial state."""
    global _DISPATCHER_BACKENDS, _cached_backend, _builtins_registered
    _DISPATCHER_BACKENDS.clear()
    _cached_backend = None
    _builtins_registered = False


def _register_builtin_backends() -> None:
    """Register built-in backends lazily."""
    global _builtins_registered

    if _builtins_registered:
        return

    _builtins_registered = True

    from .backends import (
        BlenderDispatcherBackend,
        FallbackDispatcherBackend,
        HoudiniDispatcherBackend,
        MayaDispatcherBackend,
        MaxDispatcherBackend,
        NukeDispatcherBackend,
        QtDispatcherBackend,
        UnrealDispatcherBackend,
    )

    register_dispatcher_backend(
        MayaDispatcherBackend, priority=DispatcherPriority.MAYA, name="Maya"
    )
    register_dispatcher_backend(
        HoudiniDispatcherBackend, priority=DispatcherPriority.HOUDINI, name="Houdini"
    )
    register_dispatcher_backend(
        NukeDispatcherBackend, priority=DispatcherPriority.NUKE, name="Nuke"
    )
    register_dispatcher_backend(
        BlenderDispatcherBackend, priority=DispatcherPriority.BLENDER, name="Blender"
    )
    register_dispatcher_backend(MaxDispatcherBackend, priority=DispatcherPriority.MAX, name="Max")
    register_dispatcher_backend(
        UnrealDispatcherBackend, priority=DispatcherPriority.UNREAL, name="Unreal"
    )
    register_dispatcher_backend(QtDispatcherBackend, priority=DispatcherPriority.QT, name="Qt")
    register_dispatcher_backend(
        FallbackDispatcherBackend, priority=DispatcherPriority.FALLBACK, name="Fallback"
    )


def get_dispatcher_backend() -> "ThreadDispatcherBackend":
    """Get the first available thread dispatcher backend.

    Returns:
        First available ThreadDispatcherBackend instance.

    Raises:
        RuntimeError: If no backend is available.
    """
    global _cached_backend

    if _cached_backend is not None:
        return _cached_backend

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
    """
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


def is_dcc_environment() -> bool:
    """Check if we're running inside a DCC application.

    Returns:
        True if a DCC-specific backend is available (priority >= DCC_THRESHOLD),
        False if only generic backends (Qt, Fallback) are available.
    """
    _register_builtin_backends()

    for priority, spec, _ in _DISPATCHER_BACKENDS:
        if priority < DispatcherPriority.DCC_THRESHOLD:
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
        Name of the DCC application, or None if not running inside a DCC.
    """
    _register_builtin_backends()

    for priority, spec, _ in _DISPATCHER_BACKENDS:
        if priority < DispatcherPriority.DCC_THRESHOLD:
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
