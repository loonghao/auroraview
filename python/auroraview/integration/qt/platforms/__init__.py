"""Platform-specific implementations for Qt WebView integration.

This package provides platform abstraction for window manipulation operations
needed to embed WebView windows into Qt containers.

Usage:
    from auroraview.integration.qt.platforms import get_backend

    backend = get_backend()
    backend.prepare_hwnd_for_container(hwnd)
    backend.hide_window_for_init(hwnd)
    backend.show_window_after_init(hwnd)

Supported Platforms:
    - Windows: Full implementation using Win32 API
    - macOS: No-op (placeholder for future WebKit integration)
    - Linux: No-op (placeholder for future WebKitGTK integration)
"""

import sys
from typing import Optional

from .base import NullPlatformBackend, PlatformBackend

# Singleton backend instance
_backend: Optional[PlatformBackend] = None


def get_platform_backend() -> PlatformBackend:
    """Create the appropriate platform backend for the current OS.

    Returns:
        PlatformBackend: The platform-specific backend instance.
    """
    if sys.platform == "win32":
        from .win import WindowsPlatformBackend

        return WindowsPlatformBackend()
    elif sys.platform == "darwin":
        # macOS - placeholder for future implementation
        # Could potentially use Cocoa/AppKit for window manipulation
        return NullPlatformBackend()
    elif sys.platform.startswith("linux"):
        # Linux - placeholder for future implementation
        # Could potentially use X11/Wayland for window manipulation
        return NullPlatformBackend()
    else:
        return NullPlatformBackend()


def get_backend() -> PlatformBackend:
    """Get the singleton platform backend instance.

    This function returns a cached backend instance, creating it
    on first call. This ensures consistent backend usage throughout
    the application.

    Returns:
        PlatformBackend: The platform-specific backend instance.
    """
    global _backend
    if _backend is None:
        _backend = get_platform_backend()
    return _backend


def reset_backend() -> None:
    """Reset the singleton backend instance.

    This is primarily useful for testing to ensure a fresh backend
    is created between tests.
    """
    global _backend
    _backend = None


__all__ = [
    "PlatformBackend",
    "NullPlatformBackend",
    "get_backend",
    "get_platform_backend",
    "reset_backend",
]
