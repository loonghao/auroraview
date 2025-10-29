"""Native backend - WebView using platform-specific window embedding.

This module provides the NativeWebView class, which uses native window
parenting (HWND on Windows) to embed the WebView into existing DCC
application windows.

Example:
    >>> from auroraview import NativeWebView
    >>> 
    >>> # Standalone mode
    >>> webview = NativeWebView(title="My App", width=800, height=600)
    >>> webview.load_url("http://localhost:3000")
    >>> webview.show()
    >>> 
    >>> # Embedded mode (Maya example)
    >>> import maya.OpenMayaUI as omui
    >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
    >>> webview = NativeWebView(
    ...     title="Maya Tool",
    ...     parent_hwnd=maya_hwnd,
    ...     parent_mode="owner"  # Safer for cross-thread usage
    ... )
    >>> webview.show_async()
"""

import logging
from typing import Optional

from .webview import WebView

logger = logging.getLogger(__name__)


class NativeWebView(WebView):
    """Native backend WebView implementation.
    
    This class uses platform-specific APIs (e.g., Windows HWND) to embed
    the WebView into existing windows. It's the default and most compatible
    backend for DCC integration.
    
    Args:
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        url: URL to load (optional)
        html: HTML content to load (optional)
        dev_tools: Enable developer tools (default: True)
        resizable: Make window resizable (default: True)
        parent_hwnd: Parent window handle (HWND on Windows) for embedding (optional)
        parent_mode: "child" | "owner" (Windows only, default: "owner")
            - "owner": Safer for cross-thread usage, window follows parent minimize/activate
            - "child": Requires same-thread parenting, can freeze if used cross-thread
    
    Example:
        >>> # Standalone window
        >>> webview = NativeWebView(title="My App")
        >>> webview.show()
        >>> 
        >>> # Embedded in Maya (owner mode - recommended)
        >>> import maya.OpenMayaUI as omui
        >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
        >>> webview = NativeWebView(
        ...     title="Maya Tool",
        ...     parent_hwnd=maya_hwnd,
        ...     parent_mode="owner"
        ... )
        >>> webview.show_async()
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
            parent_hwnd=parent_hwnd,
            parent_mode=parent_mode,
        )
        logger.debug(f"NativeWebView initialized (parent_hwnd={parent_hwnd}, mode={parent_mode})")


# Backward compatibility: AuroraView is an alias for NativeWebView
AuroraView = NativeWebView

__all__ = ["NativeWebView", "AuroraView"]

