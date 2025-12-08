# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Window Control Mixin.

This module provides window control methods for the WebView class.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from typing import Any

logger = logging.getLogger(__name__)


class WebViewWindowMixin:
    """Mixin providing window control methods.

    Provides methods for controlling the WebView window:
    - move: Move window to a new position
    - resize: Resize the window
    - minimize: Minimize the window
    - maximize: Maximize the window
    - restore: Restore window from minimized/maximized
    - toggle_fullscreen: Toggle fullscreen mode
    - set_always_on_top: Set window always on top
    - hide: Hide the window
    - focus: Focus the window
    """

    # Type hints for attributes from main class
    _core: Any
    _x: int
    _y: int
    _width: int
    _height: int

    def move(self, x: int, y: int) -> None:
        """Move the window to a new position.

        Args:
            x: New x position (pixels from left)
            y: New y position (pixels from top)

        Example:
            >>> webview.move(100, 50)
        """
        if hasattr(self._core, "move_to"):
            self._core.move_to(x, y)
        else:
            logger.warning("move() not supported by current backend")
        self._x = x
        self._y = y

    def resize(self, width: int, height: int) -> None:
        """Resize the window.

        Args:
            width: New width in pixels
            height: New height in pixels

        Example:
            >>> webview.resize(1024, 768)
        """
        if hasattr(self._core, "resize"):
            self._core.resize(width, height)
        else:
            logger.warning("resize() not supported by current backend")
        self._width = width
        self._height = height

    def minimize(self) -> None:
        """Minimize the window."""
        if hasattr(self._core, "minimize"):
            self._core.minimize()
        else:
            logger.warning("minimize() not supported by current backend")

    def maximize(self) -> None:
        """Maximize the window."""
        if hasattr(self._core, "maximize"):
            self._core.maximize()
        else:
            logger.warning("maximize() not supported by current backend")

    def restore(self) -> None:
        """Restore the window from minimized/maximized state."""
        if hasattr(self._core, "restore"):
            self._core.restore()
        else:
            logger.warning("restore() not supported by current backend")

    def toggle_fullscreen(self) -> None:
        """Toggle fullscreen mode."""
        if hasattr(self._core, "toggle_fullscreen"):
            self._core.toggle_fullscreen()
        else:
            logger.warning("toggle_fullscreen() not supported by current backend")

    def set_always_on_top(self, on_top: bool = True) -> None:
        """Set whether the window should always be on top.

        Args:
            on_top: True to keep window on top, False otherwise
        """
        if hasattr(self._core, "set_always_on_top"):
            self._core.set_always_on_top(on_top)
        else:
            logger.warning("set_always_on_top() not supported by current backend")

    def hide(self) -> None:
        """Hide the window without closing it."""
        if hasattr(self._core, "hide"):
            self._core.hide()
        else:
            logger.warning("hide() not supported by current backend")

    def focus(self) -> None:
        """Bring the window to the front and give it focus."""
        if hasattr(self._core, "focus"):
            self._core.focus()
        else:
            logger.warning("focus() not supported by current backend")
