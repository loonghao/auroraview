"""Qt version compatibility layer for AuroraView.

This module provides a unified API for handling differences between:
- Qt5 (PySide2/PyQt5) and Qt6 (PySide6/PyQt6)
- createWindowContainer behavior differences
- Window style handling differences

The main purpose is to ensure consistent WebView embedding behavior
across different DCC applications that may use different Qt versions.
"""

import logging
import sys
from typing import Any, Optional, Tuple

logger = logging.getLogger(__name__)

# Detect Qt version and binding
_QT_VERSION: Optional[int] = None  # 5 or 6
_QT_BINDING: Optional[str] = None  # "PySide2", "PySide6", "PyQt5", "PyQt6"

try:
    from qtpy import API_NAME, QT_VERSION

    _QT_BINDING = API_NAME
    # Parse major version from QT_VERSION (e.g., "5.15.2" -> 5)
    _QT_VERSION = int(QT_VERSION.split(".")[0]) if QT_VERSION else None
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


if sys.platform == "win32":
    import ctypes
    from ctypes import wintypes

    user32 = ctypes.windll.user32

    # Configure SetParent function signature
    user32.SetParent.argtypes = [wintypes.HWND, wintypes.HWND]
    user32.SetParent.restype = wintypes.HWND

    # Window style constants
    GWL_STYLE = -16
    GWL_EXSTYLE = -20
    WS_CHILD = 0x40000000
    WS_POPUP = 0x80000000
    WS_CAPTION = 0x00C00000
    WS_THICKFRAME = 0x00040000
    WS_MINIMIZEBOX = 0x00020000
    WS_MAXIMIZEBOX = 0x00010000
    WS_SYSMENU = 0x00080000
    WS_EX_WINDOWEDGE = 0x00000100
    WS_EX_CLIENTEDGE = 0x00000200
    WS_EX_APPWINDOW = 0x00040000
    WS_EX_TOOLWINDOW = 0x00000080
    SWP_FRAMECHANGED = 0x0020
    SWP_NOMOVE = 0x0002
    SWP_NOSIZE = 0x0001
    SWP_NOZORDER = 0x0004
    SWP_NOACTIVATE = 0x0010

    # Clipping styles for reducing flicker
    WS_CLIPCHILDREN = 0x02000000
    WS_CLIPSIBLINGS = 0x04000000


def apply_clip_styles_to_parent(parent_hwnd: int) -> bool:
    """Apply WS_CLIPCHILDREN and WS_CLIPSIBLINGS to parent container.

    These styles reduce flicker by preventing parent from drawing over
    child windows and siblings from drawing over each other.

    Args:
        parent_hwnd: The parent window handle (Qt container's HWND).

    Returns:
        True if successful, False otherwise.
    """
    if sys.platform != "win32":
        return False

    try:
        style = user32.GetWindowLongW(parent_hwnd, GWL_STYLE)
        new_style = style | WS_CLIPCHILDREN | WS_CLIPSIBLINGS

        if new_style != style:
            user32.SetWindowLongW(parent_hwnd, GWL_STYLE, new_style)
            user32.SetWindowPos(
                parent_hwnd, None, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
            )
            logger.debug(
                f"[Qt Compat] Applied clip styles to parent HWND 0x{parent_hwnd:X}"
            )
        return True

    except Exception as e:
        logger.debug(f"[Qt Compat] Failed to apply clip styles: {e}")
        return False


def prepare_hwnd_for_container(hwnd: int) -> bool:
    """Prepare a native HWND for Qt's createWindowContainer.

    This function applies the necessary Win32 style modifications to make
    a native window work properly with Qt's createWindowContainer.

    Qt6 is stricter about window styles, so we need to be more aggressive
    about removing styles that can cause issues.

    Args:
        hwnd: The native window handle (HWND) to prepare.

    Returns:
        True if successful, False otherwise.
    """
    if sys.platform != "win32":
        return False

    try:
        # Get current styles
        style = user32.GetWindowLongW(hwnd, GWL_STYLE)
        ex_style = user32.GetWindowLongW(hwnd, GWL_EXSTYLE)

        # Remove all frame/border styles
        style &= ~(WS_POPUP | WS_CAPTION | WS_THICKFRAME |
                   WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU)

        # Add WS_CHILD - critical for proper embedding
        # Also add WS_CLIPSIBLINGS for child window
        style |= WS_CHILD | WS_CLIPSIBLINGS

        # Remove extended styles that can cause issues
        ex_style &= ~(WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE |
                      WS_EX_APPWINDOW | WS_EX_TOOLWINDOW)

        # Apply new styles
        user32.SetWindowLongW(hwnd, GWL_STYLE, style)
        user32.SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style)

        # Force Windows to apply the style changes
        user32.SetWindowPos(
            hwnd, None, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
        )

        logger.debug(f"[Qt Compat] Prepared HWND 0x{hwnd:X} for container")
        return True

    except Exception as e:
        logger.error(f"[Qt Compat] Failed to prepare HWND: {e}")
        return False


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
        from qtpy.QtWidgets import QWidget, QSizePolicy
        from qtpy.QtCore import Qt as QtCore

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
            # Force opaque paint to avoid transparency issues
            container.setAttribute(QtCore.WA_OpaquePaintEvent, True)
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
    if sys.platform != "win32":
        return

    try:
        from qtpy.QtWidgets import QApplication

        # Process events to ensure Qt has completed its internal setup
        QApplication.processEvents()

        # Qt6 sometimes needs a small delay for proper window attachment
        if is_qt6():
            import time
            time.sleep(0.01)  # 10ms delay
            QApplication.processEvents()

            # Qt6/PySide6 specific: ensure native window is properly reparented
            # This is critical for Houdini where PySide6 behaves differently
            _ensure_native_child_style(hwnd, container)

        # Force a repaint to ensure the content is visible
        container.update()

        logger.debug(f"[Qt Compat] Post-container setup complete for HWND 0x{hwnd:X}")

    except Exception as e:
        logger.debug(f"[Qt Compat] Post-container setup warning: {e}")


def _ensure_native_child_style(hwnd: int, container: Any) -> None:
    """Ensure native window has proper child style after Qt takes over.

    In Qt6/PySide6, the window reparenting process may not fully complete
    the WS_CHILD style application. This function ensures the native window
    is properly configured as a child window.

    Args:
        hwnd: The native HWND.
        container: The Qt container widget.
    """
    if sys.platform != "win32":
        return

    try:
        # Get the container's HWND
        container_hwnd = int(container.winId())
        if not container_hwnd:
            return

        # Re-apply WS_CHILD and set proper parent
        style = user32.GetWindowLongW(hwnd, GWL_STYLE)

        # Check if WS_CHILD is already set
        if not (style & WS_CHILD):
            style |= WS_CHILD
            style &= ~WS_POPUP
            user32.SetWindowLongW(hwnd, GWL_STYLE, style)

            # Set the container as parent
            user32.SetParent(hwnd, container_hwnd)

            # Apply changes
            user32.SetWindowPos(
                hwnd, None, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
            )

            logger.debug(
                f"[Qt Compat] Re-applied WS_CHILD style for Qt6: "
                f"HWND 0x{hwnd:X} -> parent 0x{container_hwnd:X}"
            )

    except Exception as e:
        logger.debug(f"[Qt Compat] _ensure_native_child_style warning: {e}")

