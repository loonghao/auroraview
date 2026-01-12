# -*- coding: utf-8 -*-
"""Unified WebView API - Single entry point for all use cases.

This module provides a unified API that automatically selects the appropriate
WebView implementation based on the parent type:

- No parent (None) → Standalone WebView
- QWidget parent → QtWebView (Qt-integrated)
- int parent (HWND) → Embedded WebView

## Quick Start

```python
from auroraview import create_webview

# 1. Standalone window
webview = create_webview(url="http://localhost:3000")
webview.show()

# 2. Qt integration (Maya, Houdini, Nuke)
webview = create_webview(parent=maya_main_window(), url="http://localhost:3000")
webview.show()

# 3. HWND integration (Unreal Engine)
webview = create_webview(parent=unreal_hwnd, url="http://localhost:3000")
hwnd = webview.get_hwnd()
```

## Unified Parameters

All WebView types share these common parameters:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| parent | QWidget/int/None | None | Parent widget or HWND |
| title | str | "AuroraView" | Window title |
| width | int | 800 | Window width |
| height | int | 600 | Window height |
| url | str | None | URL to load |
| html | str | None | HTML content to load |
| debug | bool | True | Enable DevTools |
| context_menu | bool | True | Enable right-click menu |
| frame | bool | True | Show window frame |
| transparent | bool | False | Transparent background |
| background_color | str | None | Background color |
| asset_root | str | None | Custom protocol root |
| allow_file_protocol | bool | False | Enable file:// |
| mode | str | "auto" | Embedding mode |

## Migration Guide

### From WebView

```python
# Before
from auroraview.core import WebView
webview = WebView(title="Tool", parent_hwnd=hwnd, embed_mode="owner")

# After
from auroraview import create_webview
webview = create_webview(title="Tool", parent=hwnd, mode="owner")
```

### From QtWebView

```python
# Before
from auroraview import QtWebView
webview = QtWebView(parent=widget, dev_tools=True, frameless=True)

# After
from auroraview import create_webview
webview = create_webview(parent=widget, debug=True, frame=False)
```

### From AuroraView

```python
# Before
from auroraview import AuroraView
webview = AuroraView(url="http://localhost:3000", debug=True)

# After
from auroraview import create_webview
webview = create_webview(url="http://localhost:3000", debug=True)
```
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, Optional, Union

if TYPE_CHECKING:
    from auroraview.core.webview import WebView
    from auroraview.integration.qt._core import QtWebView

logger = logging.getLogger(__name__)

# Type alias for parent parameter
ParentType = Union[None, int, Any]  # None, HWND (int), or QWidget


def _is_qwidget(obj: Any) -> bool:
    """Check if object is a QWidget without importing Qt.

    Args:
        obj: Object to check

    Returns:
        True if obj is a QWidget instance
    """
    if obj is None:
        return False

    # Check class hierarchy for QWidget
    for cls in type(obj).__mro__:
        if cls.__name__ == "QWidget" and cls.__module__.startswith(("PyQt", "PySide", "qtpy")):
            return True
    return False


def _get_mode_for_parent(parent: ParentType, mode: Optional[str]) -> str:
    """Determine the appropriate mode based on parent type.

    Args:
        parent: Parent widget, HWND, or None
        mode: Explicitly specified mode (overrides auto-detection)

    Returns:
        Embedding mode string: "none", "child", or "owner"
    """
    if mode is not None:
        return mode

    if parent is None:
        return "none"
    elif _is_qwidget(parent):
        return "child"  # Qt widgets default to child mode
    elif isinstance(parent, int):
        return "owner"  # HWND defaults to owner mode (safer)
    else:
        return "none"


def create_webview(
    parent: ParentType = None,
    *,
    title: str = "AuroraView",
    width: int = 800,
    height: int = 600,
    url: Optional[str] = None,
    html: Optional[str] = None,
    debug: bool = True,
    context_menu: bool = True,
    frame: bool = True,
    resizable: bool = True,
    transparent: bool = False,
    background_color: Optional[str] = None,
    asset_root: Optional[str] = None,
    data_directory: Optional[str] = None,
    allow_file_protocol: bool = False,
    always_on_top: bool = False,
    mode: Optional[str] = None,
    ipc_batch_size: int = 0,
    icon: Optional[str] = None,
    tool_window: bool = False,
    allow_new_window: bool = False,
    new_window_mode: Optional[str] = None,
    remote_debugging_port: Optional[int] = None,
    api: Optional[Any] = None,
    **kwargs: Any,
) -> Union["WebView", "QtWebView"]:
    """Create a WebView with automatic mode selection.

    This is the unified entry point for creating WebViews. The implementation
    is automatically selected based on the parent type:

    - ``parent=None`` → Standalone WebView
    - ``parent=QWidget`` → QtWebView (Qt-integrated)
    - ``parent=int`` → Embedded WebView (HWND mode)

    Args:
        parent: Parent widget (QWidget), window handle (int), or None for standalone
        title: Window title
        width: Window width in pixels
        height: Window height in pixels
        url: URL to load
        html: HTML content to load (ignored if url is set)
        debug: Enable developer tools (F12)
        context_menu: Enable native right-click menu
        frame: Show window frame (title bar, borders)
        resizable: Allow window resizing
        transparent: Enable transparent background
        background_color: Background color (CSS format)
        asset_root: Root directory for auroraview:// protocol
        data_directory: User data directory for WebView
        allow_file_protocol: Enable file:// protocol (security risk!)
        always_on_top: Keep window always on top
        mode: Embedding mode ("none", "child", "owner"). Auto-detected if None.
        ipc_batch_size: Max IPC messages per tick (0=unlimited)
        icon: Window icon path
        tool_window: Tool window style (hide from taskbar)
        allow_new_window: Allow window.open()
        new_window_mode: New window behavior
        remote_debugging_port: CDP debugging port
        api: API object to expose to JavaScript
        **kwargs: Additional backend-specific arguments

    Returns:
        WebView or QtWebView instance

    Example:
        >>> # Standalone
        >>> webview = create_webview(url="http://localhost:3000")
        >>> webview.show()
        >>>
        >>> # Qt integration
        >>> webview = create_webview(parent=maya_window, url="http://localhost:3000")
        >>> webview.show()
        >>>
        >>> # HWND integration
        >>> webview = create_webview(parent=hwnd, url="http://localhost:3000")
        >>> webview.show()
    """
    resolved_mode = _get_mode_for_parent(parent, mode)

    if _is_qwidget(parent):
        # Qt mode - use QtWebView
        from auroraview.integration.qt._core import QtWebView

        logger.debug("Creating QtWebView (Qt parent detected)")

        return QtWebView(
            parent=parent,
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=debug,
            context_menu=context_menu,
            frameless=not frame,
            transparent=transparent,
            background_color=background_color,
            asset_root=asset_root,
            data_directory=data_directory,
            allow_file_protocol=allow_file_protocol,
            always_on_top=always_on_top,
            embed_mode=resolved_mode,
            ipc_batch_size=ipc_batch_size,
            icon=icon,
            tool_window=tool_window,
            allow_new_window=allow_new_window,
            new_window_mode=new_window_mode,
            remote_debugging_port=remote_debugging_port,
            **kwargs,
        )
    else:
        # Standalone or HWND mode - use core WebView
        from auroraview.core.webview import WebView

        logger.debug(f"Creating WebView (mode={resolved_mode})")

        webview = WebView(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            debug=debug,
            context_menu=context_menu,
            resizable=resizable,
            frame=frame,
            parent=parent if isinstance(parent, int) else None,
            mode=resolved_mode,
            asset_root=asset_root,
            data_directory=data_directory,
            allow_file_protocol=allow_file_protocol,
            always_on_top=always_on_top,
            transparent=transparent,
            background_color=background_color,
            ipc_batch_size=ipc_batch_size,
            icon=icon,
            tool_window=tool_window,
            allow_new_window=allow_new_window,
            new_window_mode=new_window_mode,
            remote_debugging_port=remote_debugging_port,
            **kwargs,
        )

        # Bind API if provided
        if api is not None:
            webview.bind_api(api)

        return webview


# Convenience alias
webview = create_webview


def run_app(
    url: Optional[str] = None,
    html: Optional[str] = None,
    title: str = "AuroraView",
    width: int = 800,
    height: int = 600,
    debug: bool = True,
    **kwargs: Any,
) -> None:
    """Run a standalone WebView application (blocking).

    This is a convenience function for simple standalone applications.
    It creates a WebView and runs the event loop until the window is closed.

    Args:
        url: URL to load
        html: HTML content to load
        title: Window title
        width: Window width
        height: Window height
        debug: Enable developer tools
        **kwargs: Additional WebView arguments

    Example:
        >>> from auroraview import run_app
        >>> run_app(url="http://localhost:3000", title="My App")
    """
    wv = create_webview(
        url=url,
        html=html,
        title=title,
        width=width,
        height=height,
        debug=debug,
        **kwargs,
    )
    wv.show()
