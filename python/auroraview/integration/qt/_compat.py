"""Qt version compatibility layer for AuroraView.

This module provides a unified API for handling differences between:
- Qt5 (PySide2/PyQt5) and Qt6 (PySide6/PyQt6)
- createWindowContainer behavior differences
- Window style handling differences

The main purpose is to ensure consistent WebView embedding behavior
across different DCC applications that may use different Qt versions.

Platform-specific implementations are in the `platforms/` subdirectory:
- platforms/base.py: Abstract interface definitions
- platforms/win.py: Windows implementation (Win32 API)
- platforms/__init__.py: Platform detection and backend selection
"""

import logging
import os
from typing import Any, Optional, Tuple

from auroraview.integration.qt.platforms import get_backend

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
# In DCC environments, excessive logging causes severe UI performance issues
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)

# Detect Qt version and binding
_QT_VERSION: Optional[int] = None  # 5 or 6
_QT_BINDING: Optional[str] = None  # "PySide2", "PySide6", "PyQt5", "PyQt6"

try:
    from qtpy import API_NAME, QT_VERSION

    _QT_BINDING = API_NAME
    # Parse major version from QT_VERSION (e.g., "5.15.2" -> 5)
    _QT_VERSION = int(QT_VERSION.split(".")[0]) if QT_VERSION else None
    if _VERBOSE_LOGGING:
        logger.debug(f"Qt detected: {_QT_BINDING} (Qt {_QT_VERSION})")
except ImportError:
    logger.warning("qtpy not available, Qt compatibility layer disabled")
except Exception as e:
    logger.warning(f"Failed to detect Qt version: {e}")


def get_qt_info() -> Tuple[Optional[str], Optional[int]]:
    """Get Qt binding and version information.

    Returns:
        Tuple of (binding_name, major_version).
        Example: ("PySide6", 6) or ("PySide2", 5)
    """
    return (_QT_BINDING, _QT_VERSION)


def is_qt6() -> bool:
    """Check if running on Qt6."""
    return _QT_VERSION == 6


def is_qt5() -> bool:
    """Check if running on Qt5."""
    return _QT_VERSION == 5


def is_pyside() -> bool:
    """Check if running on PySide (2 or 6)."""
    return _QT_BINDING in ("PySide2", "PySide6")


def is_pyqt() -> bool:
    """Check if running on PyQt (5 or 6)."""
    return _QT_BINDING in ("PyQt5", "PyQt6")


# =============================================================================
# Platform-specific window operations (delegated to platform backend)
# =============================================================================


def apply_clip_styles_to_parent(parent_hwnd: int) -> bool:
    """Apply WS_CLIPCHILDREN and WS_CLIPSIBLINGS to parent container.

    These styles reduce flicker by preventing parent from drawing over
    child windows and siblings from drawing over each other.

    Args:
        parent_hwnd: The parent window handle (Qt container's HWND).

    Returns:
        True if successful, False otherwise.
    """
    return get_backend().apply_clip_styles_to_parent(parent_hwnd)


def prepare_hwnd_for_container(hwnd: int) -> bool:
    """Prepare a native HWND for Qt's createWindowContainer.

    This function applies all necessary platform-specific style modifications
    to make a native window work properly with Qt's createWindowContainer.

    On Windows:
    - Removes all frame/border styles (WS_POPUP, WS_CAPTION, WS_THICKFRAME, etc.)
    - Adds WS_CHILD style (required for container embedding)
    - Adds WS_CLIPSIBLINGS (reduces flicker)
    - Removes extended styles that can cause issues

    Args:
        hwnd: The native window handle (HWND) to prepare.

    Returns:
        True if successful, False otherwise.
    """
    return get_backend().prepare_hwnd_for_container(hwnd)


def create_container_widget(
    qwindow: Any,
    parent: Any,
    *,
    focus_policy: Optional[str] = "strong",
) -> Optional[Any]:
    """Create a Qt container widget from a QWindow with version-specific handling.

    This wrapper handles differences between Qt5 and Qt6 in how
    createWindowContainer works.

    Args:
        qwindow: The QWindow to wrap.
        parent: The parent QWidget.
        focus_policy: Focus policy - "strong", "click", "tab", "wheel", or None.

    Returns:
        The container QWidget, or None if creation failed.
    """
    try:
        from qtpy.QtCore import Qt as QtCore
        from qtpy.QtWidgets import QSizePolicy, QWidget

        container = QWidget.createWindowContainer(qwindow, parent)
        if container is None:
            logger.error("[Qt Compat] createWindowContainer returned None")
            return None

        # Qt5 minimal settings - only essential configuration
        if focus_policy:
            policy_map = {
                "strong": QtCore.StrongFocus,
                "click": QtCore.ClickFocus,
                "tab": QtCore.TabFocus,
                "wheel": QtCore.WheelFocus,
                "none": QtCore.NoFocus,
            }
            container.setFocusPolicy(policy_map.get(focus_policy, QtCore.StrongFocus))

        # Set size policy to expanding
        container.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

        # Set minimum size to 0 to allow container to shrink
        container.setMinimumSize(0, 0)

        if _VERBOSE_LOGGING:
            logger.debug("[Qt Compat] Applied Qt5-style minimal container settings")

        return container

    except Exception as e:
        logger.error(f"[Qt Compat] Failed to create container: {e}")
        return None


def post_container_setup(container: Any, hwnd: int) -> None:
    """Perform post-creation setup for container widget.

    Qt5-style minimal setup - just process events once.

    Args:
        container: The container QWidget from createWindowContainer.
        hwnd: The original native HWND.
    """
    try:
        from qtpy.QtWidgets import QApplication

        # Qt5-style: single processEvents call
        QApplication.processEvents()

        if _VERBOSE_LOGGING:
            logger.debug(f"[Qt Compat] Post-container setup complete for HWND 0x{hwnd:X}")

    except Exception as e:
        if _VERBOSE_LOGGING:
            logger.debug(f"[Qt Compat] Post-container setup warning: {e}")


def hide_window_for_init(hwnd: int) -> bool:
    """Hide a window during initialization to prevent flicker.

    This applies platform-specific techniques to make the window
    completely invisible during WebView initialization.

    On Windows, this uses WS_EX_LAYERED with zero alpha.

    Args:
        hwnd: The window handle to hide.

    Returns:
        True if successful, False otherwise.
    """
    return get_backend().hide_window_for_init(hwnd)


def show_window_after_init(hwnd: int) -> bool:
    """Restore window visibility after initialization.

    On Windows, this removes the WS_EX_LAYERED style and restores full alpha.

    Args:
        hwnd: The window handle to show.

    Returns:
        True if successful, False otherwise.
    """
    return get_backend().show_window_after_init(hwnd)


def apply_qt6_dialog_optimizations(dialog: Any) -> bool:
    """Apply dialog optimizations - Qt5 style (minimal/no-op).

    Currently disabled for testing - Qt5 doesn't need special optimizations.

    Args:
        dialog: The QDialog to optimize.

    Returns:
        True (no-op for Qt5 compatibility testing).
    """
    # Qt5-style: no special dialog optimizations needed
    if _VERBOSE_LOGGING:
        logger.debug("[Qt Compat] Qt5-style: skipping dialog optimizations")
    return True


# =============================================================================
# Direct embedding (alternative to createWindowContainer)
# =============================================================================


def supports_direct_embedding() -> bool:
    """Check if the current platform supports direct window embedding.

    Direct embedding uses platform-native APIs (SetParent on Windows) instead of
    Qt's createWindowContainer. This can be more reliable on Qt6 where
    createWindowContainer has known issues with WebView2.

    Returns:
        True if direct embedding is supported, False otherwise.
    """
    return get_backend().supports_direct_embedding()


def embed_window_directly(child_hwnd: int, parent_hwnd: int, width: int, height: int) -> bool:
    """Embed a native window directly into a parent window without createWindowContainer.

    This is an alternative to Qt's createWindowContainer that uses platform-native
    APIs for window embedding. On Windows, this uses SetParent() + WS_CHILD.

    This approach bypasses Qt's createWindowContainer entirely, which can help
    avoid Qt6-specific issues with WebView2 embedding.

    Args:
        child_hwnd: The child window handle (WebView HWND).
        parent_hwnd: The parent window handle (Qt widget's winId()).
        width: Initial width of the embedded window.
        height: Initial height of the embedded window.

    Returns:
        True if successful, False otherwise.
    """
    return get_backend().embed_window_directly(child_hwnd, parent_hwnd, width, height)


def update_embedded_window_geometry(
    child_hwnd: int, x: int, y: int, width: int, height: int
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
    return get_backend().update_embedded_window_geometry(child_hwnd, x, y, width, height)
