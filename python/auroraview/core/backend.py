"""Backend abstraction for AuroraView.

This module provides a unified interface for selecting and configuring
WebView backends, inspired by Qt WebView's factory pattern.

Environment Variables:
    AURORAVIEW_BACKEND: Override the default backend selection.
        Valid values: "wry", "webview2" (Windows), "wkwebview" (macOS)

Example:
    >>> from auroraview.backend import BackendType, get_backend_type
    >>> print(get_backend_type())  # Returns current backend
    BackendType.WRY
"""

from __future__ import annotations

import os
import sys
from enum import Enum
from typing import TYPE_CHECKING, List

if TYPE_CHECKING:
    pass


class BackendType(Enum):
    """Available WebView backend types.

    Attributes:
        WRY: Cross-platform backend using wry/tao (default)
        WEBVIEW2: Windows native WebView2 backend
        WKWEBVIEW: macOS native WKWebView backend
        WEBKITGTK: Linux native WebKitGTK backend
    """

    WRY = "wry"
    WEBVIEW2 = "webview2"
    WKWEBVIEW = "wkwebview"
    WEBKITGTK = "webkitgtk"

    @classmethod
    def from_string(cls, value: str) -> "BackendType":
        """Parse backend type from string.

        Args:
            value: Backend name (case-insensitive)

        Returns:
            BackendType enum value

        Raises:
            ValueError: If backend name is not recognized
        """
        value_lower = value.lower().strip()
        mapping = {
            "wry": cls.WRY,
            "webview2": cls.WEBVIEW2,
            "wv2": cls.WEBVIEW2,
            "webview_2": cls.WEBVIEW2,
            "wkwebview": cls.WKWEBVIEW,
            "wk": cls.WKWEBVIEW,
            "webkit": cls.WKWEBVIEW,
            "webkitgtk": cls.WEBKITGTK,
            "gtk": cls.WEBKITGTK,
        }
        if value_lower not in mapping:
            valid = ", ".join(sorted(set(mapping.keys())))
            raise ValueError(f"Unknown backend: '{value}'. Valid backends: {valid}")
        return mapping[value_lower]


# Environment variable for backend override
ENV_BACKEND = "AURORAVIEW_BACKEND"


def get_default_backend() -> BackendType:
    """Get the default backend for the current platform.

    Returns:
        BackendType: Default backend for current OS
    """
    # Currently using WRY as default on all platforms
    # Future: Use platform-native backends by default
    return BackendType.WRY


def get_available_backends() -> List[BackendType]:
    """Get list of backends available on the current platform.

    Returns:
        List of available BackendType values
    """
    backends = [BackendType.WRY]

    if sys.platform == "win32":
        backends.append(BackendType.WEBVIEW2)
    elif sys.platform == "darwin":
        backends.append(BackendType.WKWEBVIEW)
    elif sys.platform.startswith("linux"):
        backends.append(BackendType.WEBKITGTK)

    return backends


def get_backend_type() -> BackendType:
    """Get the current backend type, respecting environment override.

    Checks AURORAVIEW_BACKEND environment variable first, then
    falls back to platform default.

    Returns:
        BackendType: Selected backend

    Raises:
        ValueError: If AURORAVIEW_BACKEND contains invalid value
    """
    env_value = os.environ.get(ENV_BACKEND)
    if env_value:
        return BackendType.from_string(env_value)
    return get_default_backend()


def set_backend_type(backend: BackendType) -> None:
    """Set the backend type via environment variable.

    This affects future WebView instances. Existing instances
    are not affected.

    Args:
        backend: Backend type to use
    """
    os.environ[ENV_BACKEND] = backend.value


def is_backend_available(backend: BackendType) -> bool:
    """Check if a specific backend is available on this platform.

    Args:
        backend: Backend type to check

    Returns:
        True if backend is available
    """
    return backend in get_available_backends()


__all__ = [
    "BackendType",
    "ENV_BACKEND",
    "get_default_backend",
    "get_available_backends",
    "get_backend_type",
    "set_backend_type",
    "is_backend_available",
]
