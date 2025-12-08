"""Windows platform implementation for Qt WebView integration.

This module provides Windows-specific window manipulation using Win32 API
for embedding WebView2 windows into Qt containers.
"""

import ctypes
import logging
import os
from ctypes import wintypes
from typing import Any

from .base import PlatformBackend

logger = logging.getLogger(__name__)

# Performance optimization: Check verbose logging once at import time
_VERBOSE_LOGGING = os.environ.get("AURORAVIEW_LOG_VERBOSE", "").lower() in (
    "1",
    "true",
    "yes",
    "on",
)

# Windows API setup
user32 = ctypes.windll.user32

# Configure SetParent function signature
user32.SetParent.argtypes = [wintypes.HWND, wintypes.HWND]
user32.SetParent.restype = wintypes.HWND

# Configure GetWindowLongW/SetWindowLongW function signatures
# On 64-bit Windows, use GetWindowLongPtrW/SetWindowLongPtrW
if ctypes.sizeof(ctypes.c_void_p) == 8:
    # 64-bit Windows
    user32.GetWindowLongPtrW.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.GetWindowLongPtrW.restype = ctypes.c_longlong
    user32.SetWindowLongPtrW.argtypes = [wintypes.HWND, ctypes.c_int, ctypes.c_longlong]
    user32.SetWindowLongPtrW.restype = ctypes.c_longlong
    GetWindowLong = user32.GetWindowLongPtrW
    SetWindowLong = user32.SetWindowLongPtrW
else:
    # 32-bit Windows
    user32.GetWindowLongW.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.GetWindowLongW.restype = wintypes.LONG
    user32.SetWindowLongW.argtypes = [wintypes.HWND, ctypes.c_int, wintypes.LONG]
    user32.SetWindowLongW.restype = wintypes.LONG
    GetWindowLong = user32.GetWindowLongW
    SetWindowLong = user32.SetWindowLongW

# Configure SetWindowPos function signature
user32.SetWindowPos.argtypes = [
    wintypes.HWND,  # hWnd
    wintypes.HWND,  # hWndInsertAfter
    ctypes.c_int,  # X
    ctypes.c_int,  # Y
    ctypes.c_int,  # cx
    ctypes.c_int,  # cy
    wintypes.UINT,  # uFlags
]
user32.SetWindowPos.restype = wintypes.BOOL

# Configure SetLayeredWindowAttributes function signature
user32.SetLayeredWindowAttributes.argtypes = [
    wintypes.HWND,
    wintypes.DWORD,  # COLORREF
    wintypes.BYTE,  # alpha
    wintypes.DWORD,  # flags
]
user32.SetLayeredWindowAttributes.restype = wintypes.BOOL

# Window style constants
GWL_STYLE = -16
GWL_EXSTYLE = -20

# Basic window styles
WS_CHILD = 0x40000000
WS_POPUP = 0x80000000
WS_CAPTION = 0x00C00000
WS_THICKFRAME = 0x00040000
WS_MINIMIZEBOX = 0x00020000
WS_MAXIMIZEBOX = 0x00010000
WS_SYSMENU = 0x00080000
WS_BORDER = 0x00800000
WS_DLGFRAME = 0x00400000
WS_OVERLAPPEDWINDOW = 0x00CF0000

# Extended window styles
WS_EX_WINDOWEDGE = 0x00000100
WS_EX_CLIENTEDGE = 0x00000200
WS_EX_APPWINDOW = 0x00040000
WS_EX_TOOLWINDOW = 0x00000080
WS_EX_STATICEDGE = 0x00020000
WS_EX_DLGMODALFRAME = 0x00000001
WS_EX_LAYERED = 0x00080000

# Clipping styles for reducing flicker
WS_CLIPCHILDREN = 0x02000000
WS_CLIPSIBLINGS = 0x04000000

# SetWindowPos flags
SWP_FRAMECHANGED = 0x0020
SWP_NOMOVE = 0x0002
SWP_NOSIZE = 0x0001
SWP_NOZORDER = 0x0004
SWP_NOACTIVATE = 0x0010

# Layered window alpha flag
LWA_ALPHA = 0x00000002


class WindowsPlatformBackend(PlatformBackend):
    """Windows-specific implementation using Win32 API.

    This class implements all window manipulation operations needed
    for embedding WebView2 windows into Qt containers on Windows.
    """

    def supports_direct_embedding(self) -> bool:
        """Windows supports direct embedding via SetParent()."""
        return True

    def embed_window_directly(
        self, child_hwnd: int, parent_hwnd: int, width: int, height: int
    ) -> bool:
        """Embed a native window directly into a parent window using SetParent().

        This is an alternative to Qt's createWindowContainer that uses Win32 API
        directly. This can be more reliable on Qt6 where createWindowContainer
        has known issues with WebView2.

        The process:
        1. Remove all frame/border styles from child
        2. Add WS_CHILD style
        3. Call SetParent() to establish parent-child relationship
        4. Position and size the child window
        5. Apply clip styles to parent

        Args:
            child_hwnd: The child window handle (WebView HWND).
            parent_hwnd: The parent window handle (Qt widget's winId()).
            width: Initial width of the embedded window.
            height: Initial height of the embedded window.

        Returns:
            True if successful, False otherwise.
        """
        try:
            # Step 1: Get current styles
            style = GetWindowLong(child_hwnd, GWL_STYLE)
            ex_style = GetWindowLong(child_hwnd, GWL_EXSTYLE)

            old_style = style
            old_ex_style = ex_style

            # Step 2: Remove all frame/border styles
            style &= ~(
                WS_POPUP
                | WS_CAPTION
                | WS_THICKFRAME
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX
                | WS_SYSMENU
                | WS_BORDER
                | WS_DLGFRAME
                | WS_OVERLAPPEDWINDOW
            )

            # Step 3: Add WS_CHILD and WS_CLIPSIBLINGS
            style |= WS_CHILD | WS_CLIPSIBLINGS

            # Step 4: Remove extended styles that can cause issues
            ex_style &= ~(
                WS_EX_WINDOWEDGE
                | WS_EX_CLIENTEDGE
                | WS_EX_APPWINDOW
                | WS_EX_TOOLWINDOW
                | WS_EX_STATICEDGE
                | WS_EX_DLGMODALFRAME
            )

            # Step 5: Apply new styles BEFORE SetParent
            SetWindowLong(child_hwnd, GWL_STYLE, style)
            SetWindowLong(child_hwnd, GWL_EXSTYLE, ex_style)

            # Step 6: Call SetParent to establish parent-child relationship
            old_parent = user32.SetParent(child_hwnd, parent_hwnd)
            if old_parent == 0:
                # SetParent failed
                error_code = ctypes.get_last_error()
                logger.error(
                    f"[Win32] SetParent failed: child=0x{child_hwnd:X}, "
                    f"parent=0x{parent_hwnd:X}, error={error_code}"
                )
                return False

            # Step 7: Position and size the child window at (0, 0)
            # Use SWP_SHOWWINDOW to make it visible
            SWP_SHOWWINDOW = 0x0040
            result = user32.SetWindowPos(
                child_hwnd,
                None,
                0,  # X position - top-left of parent
                0,  # Y position - top-left of parent
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            )

            if not result:
                logger.warning(f"[Win32] SetWindowPos failed for child 0x{child_hwnd:X}")

            # Step 8: Apply clip styles to parent
            self.apply_clip_styles_to_parent(parent_hwnd)

            # Step 9: CRITICAL - Fix all WebView2 child windows recursively
            # WebView2 creates multiple child windows (Chrome_WidgetWin_0, etc.)
            # that may still be draggable. We need to fix ALL of them.
            self._fix_all_child_windows_recursive(child_hwnd)

            if _VERBOSE_LOGGING:
                logger.info(
                    f"[Win32] Direct embedding successful: "
                    f"child=0x{child_hwnd:X} -> parent=0x{parent_hwnd:X} "
                    f"(style=0x{old_style:08X}->0x{style:08X}, "
                    f"ex_style=0x{old_ex_style:08X}->0x{ex_style:08X}, "
                    f"size={width}x{height})"
                )
            return True

        except Exception as e:
            logger.error(f"[Win32] embed_window_directly failed: {e}")
            return False

    def _fix_all_child_windows_recursive(self, parent_hwnd: int, depth: int = 0) -> int:
        """Recursively fix all child windows to prevent independent dragging.

        WebView2 creates a hierarchy of child windows:
        - Main WebView window
          - Chrome_WidgetWin_0
            - Chrome_WidgetWin_1
              - Intermediate D3D Window
              - ...

        All of these need WS_CHILD style and proper positioning to prevent
        them from being dragged independently.

        Args:
            parent_hwnd: The parent window to enumerate children from.
            depth: Current recursion depth (for logging).

        Returns:
            Number of windows fixed.
        """
        fixed_count = 0
        max_depth = 10  # Prevent infinite recursion

        if depth > max_depth:
            return 0

        try:
            # EnumChildWindows callback
            WNDENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

            child_windows = []

            def enum_callback(hwnd, lparam):
                child_windows.append(hwnd)
                return True  # Continue enumeration

            callback = WNDENUMPROC(enum_callback)
            user32.EnumChildWindows(parent_hwnd, callback, 0)

            for child_hwnd in child_windows:
                try:
                    # Get current style
                    style = GetWindowLong(child_hwnd, GWL_STYLE)
                    ex_style = GetWindowLong(child_hwnd, GWL_EXSTYLE)

                    # Check if already a proper child
                    if not (style & WS_CHILD):
                        # Add WS_CHILD, remove WS_POPUP
                        new_style = (style | WS_CHILD | WS_CLIPSIBLINGS) & ~WS_POPUP
                        SetWindowLong(child_hwnd, GWL_STYLE, new_style)

                        # Remove problematic extended styles
                        new_ex_style = ex_style & ~(WS_EX_APPWINDOW | WS_EX_TOOLWINDOW)
                        if new_ex_style != ex_style:
                            SetWindowLong(child_hwnd, GWL_EXSTYLE, new_ex_style)

                        # Apply changes
                        user32.SetWindowPos(
                            child_hwnd,
                            None,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE
                            | SWP_NOSIZE
                            | SWP_NOZORDER
                            | SWP_NOACTIVATE
                            | SWP_FRAMECHANGED,
                        )

                        fixed_count += 1
                        if _VERBOSE_LOGGING:
                            logger.debug(
                                f"[Win32] Fixed child window: HWND=0x{child_hwnd:X} "
                                f"(depth={depth}, style=0x{style:08X}->0x{new_style:08X})"
                            )

                except Exception as e:
                    if _VERBOSE_LOGGING:
                        logger.debug(f"[Win32] Failed to fix child 0x{child_hwnd:X}: {e}")

            if fixed_count > 0:
                logger.info(
                    f"[Win32] Fixed {fixed_count} child windows at depth {depth} "
                    f"for parent 0x{parent_hwnd:X}"
                )

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] _fix_all_child_windows_recursive failed: {e}")

        return fixed_count

    def update_embedded_window_geometry(
        self, child_hwnd: int, x: int, y: int, width: int, height: int
    ) -> bool:
        """Update the geometry of a directly embedded window."""
        try:
            result = user32.SetWindowPos(
                child_hwnd,
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
            if _VERBOSE_LOGGING:
                logger.debug(
                    f"[Win32] Updated geometry: HWND=0x{child_hwnd:X}, "
                    f"pos=({x},{y}), size={width}x{height}"
                )
            return bool(result)
        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] update_embedded_window_geometry failed: {e}")
            return False

    def apply_clip_styles_to_parent(self, parent_hwnd: int) -> bool:
        """Apply WS_CLIPCHILDREN and WS_CLIPSIBLINGS to parent container."""
        try:
            style = GetWindowLong(parent_hwnd, GWL_STYLE)
            new_style = style | WS_CLIPCHILDREN | WS_CLIPSIBLINGS

            if new_style != style:
                SetWindowLong(parent_hwnd, GWL_STYLE, new_style)
                user32.SetWindowPos(
                    parent_hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                )
                if _VERBOSE_LOGGING:
                    logger.debug(f"[Win32] Applied clip styles to parent HWND 0x{parent_hwnd:X}")
            return True

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] Failed to apply clip styles: {e}")
            return False

    def prepare_hwnd_for_container(self, hwnd: int) -> bool:
        """Prepare a native HWND for Qt's createWindowContainer."""
        try:
            # Get current styles
            style = GetWindowLong(hwnd, GWL_STYLE)
            ex_style = GetWindowLong(hwnd, GWL_EXSTYLE)

            old_style = style
            old_ex_style = ex_style

            # Remove all frame/border styles (comprehensive)
            style &= ~(
                WS_POPUP
                | WS_CAPTION
                | WS_THICKFRAME
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX
                | WS_SYSMENU
                | WS_BORDER
                | WS_DLGFRAME
                | WS_OVERLAPPEDWINDOW
            )

            # Add WS_CHILD - critical for proper embedding
            # Also add WS_CLIPSIBLINGS for child window
            style |= WS_CHILD | WS_CLIPSIBLINGS

            # Remove extended styles that can cause issues (comprehensive)
            ex_style &= ~(
                WS_EX_WINDOWEDGE
                | WS_EX_CLIENTEDGE
                | WS_EX_APPWINDOW
                | WS_EX_TOOLWINDOW
                | WS_EX_STATICEDGE
                | WS_EX_DLGMODALFRAME
            )

            # Apply new styles
            SetWindowLong(hwnd, GWL_STYLE, style)
            SetWindowLong(hwnd, GWL_EXSTYLE, ex_style)

            # Force Windows to apply the style changes
            # CRITICAL: Do NOT use SWP_NOMOVE - we need to force position to (0, 0)
            # This prevents the WebView from being dragged/offset within the Qt container
            user32.SetWindowPos(
                hwnd,
                None,
                0,  # X position - top-left of parent
                0,  # Y position - top-left of parent
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            )

            if _VERBOSE_LOGGING:
                logger.debug(
                    f"[Win32] Prepared HWND 0x{hwnd:X} for container "
                    f"(style=0x{old_style:08X}->0x{style:08X}, "
                    f"ex_style=0x{old_ex_style:08X}->0x{ex_style:08X})"
                )
            return True

        except Exception as e:
            logger.error(f"[Win32] Failed to prepare HWND: {e}")
            return False

    def hide_window_for_init(self, hwnd: int) -> bool:
        """Hide a window during initialization using WS_EX_LAYERED with zero alpha."""
        try:
            # Get current extended style
            ex_style = GetWindowLong(hwnd, GWL_EXSTYLE)

            # Add WS_EX_LAYERED
            new_ex_style = ex_style | WS_EX_LAYERED
            SetWindowLong(hwnd, GWL_EXSTYLE, new_ex_style)

            # Set zero alpha (completely invisible)
            user32.SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA)

            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] Hidden window HWND 0x{hwnd:X} for init")
            return True

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] Failed to hide window: {e}")
            return False

    def show_window_after_init(self, hwnd: int) -> bool:
        """Restore window visibility by removing WS_EX_LAYERED."""
        try:
            # First restore full alpha
            user32.SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA)

            # Remove WS_EX_LAYERED style
            ex_style = GetWindowLong(hwnd, GWL_EXSTYLE)
            new_ex_style = ex_style & ~WS_EX_LAYERED
            SetWindowLong(hwnd, GWL_EXSTYLE, new_ex_style)

            # Apply changes
            user32.SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            )

            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] Restored window HWND 0x{hwnd:X} visibility")
            return True

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] Failed to show window: {e}")
            return False

    def ensure_native_child_style(self, hwnd: int, container: Any) -> None:
        """Ensure native window has proper child style after Qt setup.

        This is critical for Qt6/PySide6 where reparenting may not
        fully complete the WS_CHILD style application.
        """
        try:
            # Get the container's HWND
            container_hwnd = int(container.winId())
            if not container_hwnd:
                return

            # Re-apply WS_CHILD and set proper parent
            style = GetWindowLong(hwnd, GWL_STYLE)

            # Check if WS_CHILD is already set
            if not (style & WS_CHILD):
                style |= WS_CHILD
                style &= ~WS_POPUP
                SetWindowLong(hwnd, GWL_STYLE, style)

                # Set the container as parent
                user32.SetParent(hwnd, container_hwnd)

                # Apply changes and force position to (0, 0)
                # CRITICAL: Do NOT use SWP_NOMOVE - we need to force position to (0, 0)
                user32.SetWindowPos(
                    hwnd,
                    None,
                    0,  # X position - top-left of parent
                    0,  # Y position - top-left of parent
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                )

                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[Win32] Re-applied WS_CHILD style for Qt6: "
                        f"HWND 0x{hwnd:X} -> parent 0x{container_hwnd:X}"
                    )

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] ensure_native_child_style warning: {e}")
