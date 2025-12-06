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

        # Set focus policy based on Qt version
        # Qt6 is stricter about focus handling
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

        # Qt6-specific: ensure proper window activation and layout
        if is_qt6():
            # Qt6 requires explicit window activation in some cases
            container.setAttribute(QtCore.WA_NativeWindow, True)
            # Also set WA_InputMethodEnabled for proper keyboard input
            container.setAttribute(QtCore.WA_InputMethodEnabled, True)
            # Ensure no extra margins from container
            container.setContentsMargins(0, 0, 0, 0)
            # NOTE: Do NOT set WA_OpaquePaintEvent on container!
            # This causes black screen in Houdini and other Qt6 DCCs.
            # The container must remain transparent to show embedded WebView content.
            if _VERBOSE_LOGGING:
                logger.debug("[Qt Compat] Applied Qt6-specific container settings")

        # Qt5/Qt6 common: ensure container accepts focus properly
        container.setAttribute(QtCore.WA_AcceptTouchEvents, True)

        return container

    except Exception as e:
        logger.error(f"[Qt Compat] Failed to create container: {e}")
        return None


def post_container_setup(container: Any, hwnd: int) -> None:
    """Perform post-creation setup for container widget.

    This handles Qt version-specific quirks that need to be addressed
    after the container is created and added to a layout.

    Args:
        container: The container QWidget from createWindowContainer.
        hwnd: The original native HWND.
    """
    try:
        from qtpy.QtWidgets import QApplication

        # Process events to ensure Qt has completed its internal setup
        # Using processEvents multiple times is more reliable than time.sleep
        # and avoids blocking the thread unnecessarily.
        QApplication.processEvents()

        if is_qt6():
            # Qt6 needs additional event processing for proper window attachment.
            # Instead of a fixed time.sleep(), we process events in small batches
            # which allows the event loop to complete native operations.
            for _ in range(3):
                QApplication.processEvents()

            # Qt6/PySide6 specific: ensure native window is properly reparented
            # This is critical for Houdini where PySide6 behaves differently
            get_backend().ensure_native_child_style(hwnd, container)

        # Force a repaint to ensure the content is visible
        container.update()

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
    """Apply Qt6-specific optimizations to a QDialog.

    This function applies performance and compatibility optimizations
    that are recommended for Qt6 environments. It should be called
    after dialog creation but before showing the dialog.

    Optimizations applied:
    - WA_OpaquePaintEvent: Force opaque painting (better performance)
    - WA_TranslucentBackground: Disable translucency (Qt6 performance issue)
    - WA_NoSystemBackground: Ensure proper background handling
    - WA_NativeWindow: Ensure native window creation
    - WA_InputMethodEnabled: Enable input method for keyboard input

    Args:
        dialog: The QDialog to optimize.

    Returns:
        True if optimizations were applied, False if Qt6 not detected or error.

    Example:
        >>> dialog = QDialog()
        >>> apply_qt6_dialog_optimizations(dialog)
        >>> dialog.show()
    """
    if not is_qt6():
        if _VERBOSE_LOGGING:
            logger.debug("[Qt Compat] Not Qt6, skipping dialog optimizations")
        return False

    try:
        from qtpy.QtCore import Qt

        # NOTE: Do NOT set WA_OpaquePaintEvent on dialogs containing WebView!
        # This causes black screen in Houdini and other Qt6 DCCs because
        # Qt assumes the widget will paint its entire background, but
        # the WebView container needs transparency to show the embedded content.

        # Performance optimization: Disable translucent background
        # (Qt6 has significant performance issues with translucency)
        dialog.setAttribute(Qt.WA_TranslucentBackground, False)

        # Ensure proper background handling
        dialog.setAttribute(Qt.WA_NoSystemBackground, False)

        # Qt6 compatibility: Ensure native window
        dialog.setAttribute(Qt.WA_NativeWindow, True)

        # Qt6 compatibility: Enable input method for keyboard
        dialog.setAttribute(Qt.WA_InputMethodEnabled, True)

        if _VERBOSE_LOGGING:
            logger.debug("[Qt Compat] Applied Qt6 dialog optimizations")
        return True

    except Exception as e:
        logger.error(f"[Qt Compat] Failed to apply Qt6 optimizations: {e}")
        return False
