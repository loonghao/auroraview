"""Native backend - WebView using platform-specific window embedding.

This module provides the NativeWebView class, which uses native window
parenting (HWND on Windows) to embed the WebView into existing DCC
application windows.

Recommended Usage (Factory Methods):
    >>> from auroraview import NativeWebView
    >>>
    >>> # Standalone window (recommended)
    >>> webview = NativeWebView.standalone(title="My App", width=800, height=600)
    >>> webview.show()
    >>>
    >>> # Embedded in DCC (recommended)
    >>> import maya.OpenMayaUI as omui
    >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
    >>> webview = NativeWebView.embedded(
    ...     parent_hwnd=maya_hwnd,
    ...     title="Maya Tool",
    ...     mode="owner"  # Default, safer for cross-thread
    ... )
    >>> webview.show()

Legacy Usage (Direct Constructor):
    >>> # Still supported for backward compatibility
    >>> webview = NativeWebView(
    ...     title="Maya Tool",
    ...     parent_hwnd=maya_hwnd,
    ...     parent_mode="owner"
    ... )
"""

import logging
from typing import Optional, Literal

from .webview import WebView

logger = logging.getLogger(__name__)


class NativeWebView(WebView):
    """Native backend WebView implementation.

    This class uses platform-specific APIs (e.g., Windows HWND) to embed
    the WebView into existing windows. It's the default and most compatible
    backend for DCC integration.

    **Recommended: Use factory methods instead of direct constructor**

    Factory Methods:
        - `NativeWebView.standalone()` - Create standalone window
        - `NativeWebView.embedded()` - Embed in DCC application

    Direct Constructor Args (Legacy):
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        url: URL to load (optional)
        html: HTML content to load (optional)
        dev_tools: Enable developer tools (default: True)
        resizable: Make window resizable (default: True)
        decorations: Show window decorations/title bar (default: True)
        parent_hwnd: Parent window handle (HWND on Windows) for embedding (optional)
        parent_mode: "child" | "owner" (Windows only, default: "owner")

    Examples:
        >>> # Recommended: Use factory methods
        >>> webview = NativeWebView.standalone(title="My App")
        >>> webview.show()
        >>>
        >>> # Recommended: Embedded in Maya
        >>> import maya.OpenMayaUI as omui
        >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
        >>> webview = NativeWebView.embedded(
        ...     parent_hwnd=maya_hwnd,
        ...     title="Maya Tool"
        ... )
        >>> webview.show()
    """
    
    def __init__(
        self,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        dev_tools: bool = True,
        resizable: bool = True,
        decorations: bool = True,
        parent_hwnd: Optional[int] = None,
        parent_mode: Optional[str] = "owner",
    ) -> None:
        """Initialize the NativeWebView."""
        super().__init__(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=dev_tools,
            resizable=resizable,
            decorations=decorations,
            parent_hwnd=parent_hwnd,
            parent_mode=parent_mode,
        )
        logger.debug(f"NativeWebView initialized (decorations={decorations}, parent_hwnd={parent_hwnd}, mode={parent_mode})")

    @classmethod
    def standalone(
        cls,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        dev_tools: bool = True,
        resizable: bool = True,
        decorations: bool = True,
    ) -> "NativeWebView":
        """Create a standalone WebView window (not embedded in any parent).

        This is the recommended way to create a standalone window.

        Args:
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            dev_tools: Enable developer tools (default: True)
            resizable: Make window resizable (default: True)
            decorations: Show window decorations/title bar (default: True)

        Returns:
            NativeWebView instance configured for standalone mode

        Example:
            >>> webview = NativeWebView.standalone(
            ...     title="My App",
            ...     width=1024,
            ...     height=768,
            ...     decorations=False  # Borderless window
            ... )
            >>> webview.load_url("http://localhost:3000")
            >>> webview.show()  # Blocking until window closes
        """
        logger.info(f"Creating standalone WebView: {title}")
        return cls(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=dev_tools,
            resizable=resizable,
            decorations=decorations,
            parent_hwnd=None,
            parent_mode=None,
        )

    @classmethod
    def embedded(
        cls,
        parent_hwnd: int,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        dev_tools: bool = True,
        resizable: bool = True,
        decorations: bool = True,
        mode: Literal["owner", "child"] = "owner",
    ) -> "NativeWebView":
        """Create a WebView embedded in a parent window (DCC integration).

        This is the recommended way to embed WebView in DCC applications like Maya.

        Args:
            parent_hwnd: Parent window handle (HWND on Windows)
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            dev_tools: Enable developer tools (default: True)
            resizable: Make window resizable (default: True)
            decorations: Show window decorations/title bar (default: True)
            mode: Embedding mode (default: "owner")
                - "owner": Safer for cross-thread usage (RECOMMENDED)
                  Window follows parent minimize/activate
                - "child": Requires same-thread parenting
                  Can freeze if used cross-thread

        Returns:
            NativeWebView instance configured for embedded mode

        Example (Maya):
            >>> import maya.OpenMayaUI as omui
            >>> from shiboken2 import wrapInstance
            >>> from PySide2.QtWidgets import QWidget
            >>>
            >>> # Get Maya main window handle
            >>> main_window_ptr = omui.MQtUtil.mainWindow()
            >>> maya_window = wrapInstance(int(main_window_ptr), QWidget)
            >>> hwnd = maya_window.winId()
            >>>
            >>> # Create embedded WebView
            >>> webview = NativeWebView.embedded(
            ...     parent_hwnd=hwnd,
            ...     title="Maya Tool",
            ...     width=400,
            ...     height=600,
            ...     mode="owner"  # Recommended for Maya
            ... )
            >>>
            >>> # Setup event processing (required for embedded mode)
            >>> import __main__
            >>> __main__.my_webview = webview
            >>>
            >>> def process_events():
            ...     if hasattr(__main__, 'my_webview'):
            ...         should_close = __main__.my_webview._core.process_events()
            ...         if should_close:
            ...             # Cleanup
            ...             pass
            >>>
            >>> import maya.cmds as cmds
            >>> timer_id = cmds.scriptJob(event=["idle", process_events])
            >>> __main__.my_webview_timer = timer_id
            >>>
            >>> # Show window (non-blocking in embedded mode)
            >>> webview.show()
        """
        logger.info(f"Creating embedded WebView: {title} (parent_hwnd={parent_hwnd}, mode={mode})")
        return cls(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=dev_tools,
            resizable=resizable,
            decorations=decorations,
            parent_hwnd=parent_hwnd,
            parent_mode=mode,
        )


# Backward compatibility: AuroraView is an alias for NativeWebView
AuroraView = NativeWebView

__all__ = ["NativeWebView", "AuroraView"]

