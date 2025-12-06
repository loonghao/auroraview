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

    def apply_clip_styles_to_parent(self, parent_hwnd: int) -> bool:
        """Apply WS_CLIPCHILDREN and WS_CLIPSIBLINGS to parent container."""
        try:
            style = user32.GetWindowLongW(parent_hwnd, GWL_STYLE)
            new_style = style | WS_CLIPCHILDREN | WS_CLIPSIBLINGS

            if new_style != style:
                user32.SetWindowLongW(parent_hwnd, GWL_STYLE, new_style)
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
            style = user32.GetWindowLongW(hwnd, GWL_STYLE)
            ex_style = user32.GetWindowLongW(hwnd, GWL_EXSTYLE)

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
            user32.SetWindowLongW(hwnd, GWL_STYLE, style)
            user32.SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style)

            # Force Windows to apply the style changes (single call)
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
            ex_style = user32.GetWindowLongW(hwnd, GWL_EXSTYLE)

            # Add WS_EX_LAYERED
            new_ex_style = ex_style | WS_EX_LAYERED
            user32.SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style)

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
            ex_style = user32.GetWindowLongW(hwnd, GWL_EXSTYLE)
            new_ex_style = ex_style & ~WS_EX_LAYERED
            user32.SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style)

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
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                )

                if _VERBOSE_LOGGING:
                    logger.debug(
                        f"[Win32] Re-applied WS_CHILD style for Qt6: "
                        f"HWND 0x{hwnd:X} -> parent 0x{container_hwnd:X}"
                    )

        except Exception as e:
            if _VERBOSE_LOGGING:
                logger.debug(f"[Win32] ensure_native_child_style warning: {e}")
