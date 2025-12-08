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
    def supports_direct_embedding(self) -> bool:
        """Check if this platform supports direct window embedding without createWindowContainer.

        Direct embedding uses platform-native APIs (SetParent on Windows) instead of
        Qt's createWindowContainer. This can be more reliable on Qt6 where
        createWindowContainer has known issues.

        Returns:
            True if direct embedding is supported, False otherwise.
        """
        pass

    @abstractmethod
    def embed_window_directly(
        self, child_hwnd: int, parent_hwnd: int, width: int, height: int
    ) -> bool:
        """Embed a native window directly into a parent window without createWindowContainer.

        This is an alternative to Qt's createWindowContainer that uses platform-native
        APIs for window embedding. On Windows, this uses SetParent() + WS_CHILD.

        Args:
            child_hwnd: The child window handle (WebView HWND).
            parent_hwnd: The parent window handle (Qt widget's winId()).
            width: Initial width of the embedded window.
            height: Initial height of the embedded window.

        Returns:
            True if successful, False otherwise.
        """
        pass

    @abstractmethod
    def update_embedded_window_geometry(
        self, child_hwnd: int, x: int, y: int, width: int, height: int
    ) -> bool:
        """Update the geometry of a directly embedded window.

        This should be called when the parent Qt widget is resized to keep
        the embedded window in sync.

        Args:
            child_hwnd: The child window handle.
            x: X position relative to parent.
            y: Y position relative to parent.
            width: New width.
            height: New height.

        Returns:
            True if successful, False otherwise.
        """
        pass

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

    def supports_direct_embedding(self) -> bool:
        """No-op: returns False (direct embedding not supported)."""
        return False

    def embed_window_directly(
        self, child_hwnd: int, parent_hwnd: int, width: int, height: int
    ) -> bool:
        """No-op: returns False."""
        return False

    def update_embedded_window_geometry(
        self, child_hwnd: int, x: int, y: int, width: int, height: int
    ) -> bool:
        """No-op: returns False."""
        return False

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
