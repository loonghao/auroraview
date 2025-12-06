"""Platform abstraction base classes for Qt WebView integration.

This module defines the abstract interfaces for platform-specific operations
that are needed to embed WebView windows into Qt containers.
"""

from abc import ABC, abstractmethod
from typing import Any


class PlatformBackend(ABC):
    """Abstract base class for platform-specific operations.

    This interface defines all platform-dependent operations needed for
    embedding WebView windows into Qt's createWindowContainer.

    Implementations exist for:
    - Windows (win.py): Full implementation using Win32 API
    - Other platforms: No-op implementations (placeholder for future)
    """

    @abstractmethod
    def apply_clip_styles_to_parent(self, parent_hwnd: int) -> bool:
        """Apply clip styles to parent window to reduce flicker.

        On Windows, this applies WS_CLIPCHILDREN and WS_CLIPSIBLINGS
        to prevent parent from drawing over child windows.

        Args:
            parent_hwnd: The parent window handle.

        Returns:
            True if successful, False otherwise.
        """
        pass

    @abstractmethod
    def prepare_hwnd_for_container(self, hwnd: int) -> bool:
        """Prepare a native window for Qt's createWindowContainer.

        This modifies window styles to make the native window suitable
        for embedding. On Windows, this removes borders/frames and adds
        WS_CHILD style.

        Args:
            hwnd: The native window handle.

        Returns:
            True if successful, False otherwise.
        """
        pass

    @abstractmethod
    def hide_window_for_init(self, hwnd: int) -> bool:
        """Hide a window during initialization to prevent flicker.

        On Windows, this uses WS_EX_LAYERED with zero alpha.

        Args:
            hwnd: The window handle to hide.

        Returns:
            True if successful, False otherwise.
        """
        pass

    @abstractmethod
    def show_window_after_init(self, hwnd: int) -> bool:
        """Restore window visibility after initialization.

        On Windows, this removes WS_EX_LAYERED and restores alpha.

        Args:
            hwnd: The window handle to show.

        Returns:
            True if successful, False otherwise.
        """
        pass

    @abstractmethod
    def ensure_native_child_style(self, hwnd: int, container: Any) -> None:
        """Ensure native window has proper child style after Qt setup.

        This is especially important for Qt6/PySide6 where reparenting
        may not fully complete the WS_CHILD style application.

        Args:
            hwnd: The native window handle.
            container: The Qt container widget.
        """
        pass


class NullPlatformBackend(PlatformBackend):
    """No-op implementation for unsupported platforms.

    This implementation does nothing and returns False/None for all operations.
    It's used on platforms where native window embedding is not supported
    or not needed.
    """

    def apply_clip_styles_to_parent(self, parent_hwnd: int) -> bool:
        """No-op: returns False."""
        return False

    def prepare_hwnd_for_container(self, hwnd: int) -> bool:
        """No-op: returns False."""
        return False

    def hide_window_for_init(self, hwnd: int) -> bool:
        """No-op: returns False."""
        return False

    def show_window_after_init(self, hwnd: int) -> bool:
        """No-op: returns False."""
        return False

    def ensure_native_child_style(self, hwnd: int, container: Any) -> None:
        """No-op: does nothing."""
        pass
