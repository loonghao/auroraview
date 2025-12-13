# -*- coding: utf-8 -*-
"""High-level Python API for WebView."""

from __future__ import annotations

import logging
import threading
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Union

try:
    from typing import Literal  # py38+
except ImportError:  # pragma: no cover - only for py37
    from typing_extensions import Literal  # type: ignore

# Import Mixin classes
from auroraview.core.mixins import (
    WebViewApiMixin,
    WebViewContentMixin,
    WebViewDOMMixin,
    WebViewEventMixin,
    WebViewJSMixin,
    WebViewWindowMixin,
)

if TYPE_CHECKING:
    from .bridge import Bridge
    from .channel import Channel, ChannelManager
    from .commands import CommandRegistry
    from .state import State

_CORE_IMPORT_ERROR = None
try:
    from auroraview._core import WebView as _CoreWebView
except ImportError as e:
    _CoreWebView = None
    _CORE_IMPORT_ERROR = str(e)

logger = logging.getLogger(__name__)


class WebView(
    WebViewWindowMixin,
    WebViewContentMixin,
    WebViewJSMixin,
    WebViewEventMixin,
    WebViewApiMixin,
    WebViewDOMMixin,
):
    """High-level WebView class with enhanced Python API.

    This class wraps the Rust core WebView implementation and provides
    a more Pythonic interface with additional features.

    Args:
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        url: URL to load (optional)
        html: HTML content to load (optional)
        debug: Enable developer tools (default: True)
        context_menu: Enable native context menu (default: True)
        resizable: Make window resizable (default: True)
        frame: Show window frame (title bar, borders) (default: True)
        parent: Parent window handle for embedding (optional)
        mode: Embedding mode - "child" or "owner" (optional, Windows only)
              "owner" is safer for cross-thread usage
              "child" requires same-thread parenting

    Example:
        >>> # Standalone window
        >>> webview = WebView(title="My Tool", width=1024, height=768)
        >>> webview.load_url("http://localhost:3000")
        >>> webview.show()

        >>> # DCC integration (e.g., Maya)
        >>> import maya.OpenMayaUI as omui
        >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
        >>> webview = WebView(title="My Tool", parent=maya_hwnd, mode="owner")
        >>> webview.show()

        >>> # Disable native context menu for custom menu
        >>> webview = WebView(title="My Tool", context_menu=False)
        >>> webview.show()
    """

    # Class-level singleton registry using weak references
    _singleton_registry: Dict[str, "WebView"] = {}

    def __init__(
        self,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        debug: Optional[bool] = None,
        context_menu: bool = True,
        resizable: bool = True,
        frame: Optional[bool] = None,
        parent: Optional[int] = None,
        mode: Optional[str] = None,
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        dev_tools: Optional[bool] = None,
        decorations: Optional[bool] = None,
        asset_root: Optional[str] = None,
        data_directory: Optional[str] = None,
        allow_file_protocol: bool = False,
        always_on_top: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        auto_show: bool = True,
        ipc_batch_size: int = 0,
        icon: Optional[str] = None,
        tool_window: bool = False,
        undecorated_shadow: bool = True,
        allow_new_window: bool = False,
        # Aliases for more intuitive API
        parent_hwnd: Optional[int] = None,
        embed_mode: Optional[str] = None,
    ) -> None:
        r"""Initialize the WebView.

        Args:
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            debug: Enable developer tools (default: True). Press F12 or right-click
                > Inspect to open DevTools.
            context_menu: Enable native context menu (default: True)
            resizable: Make window resizable (default: True)
            frame: Show window frame (title bar, borders) (default: True)
            parent: Parent window handle for embedding (optional)
            mode: Embedding mode - "child" or "owner" (optional)
            bridge: Bridge instance for DCC integration
                   - Bridge instance: Use provided bridge
                   - True: Auto-create bridge with default settings
                   - None: No bridge (default)
            asset_root: Root directory for auroraview:// protocol.
                When set, enables the auroraview:// custom protocol for secure
                local resource loading. Files under this directory can be accessed
                using URLs like ``auroraview://path/to/file``.

                **Platform-specific URL format**:

                - Windows: ``https://auroraview.localhost/path``
                - macOS/Linux: ``auroraview://path``

                **Security**: Uses ``.localhost`` TLD (IANA reserved, RFC 6761)
                which cannot be registered and is treated as a local address.
                Requests are intercepted before DNS resolution.

                **Recommended** over ``allow_file_protocol=True`` because access
                is restricted to the specified directory only.

            data_directory: User data directory for WebView (cookies, cache, localStorage).
                If None, uses system default (usually %LOCALAPPDATA%\...\EBWebView on Windows).
                Set this to isolate WebView data per application or user profile.

            allow_file_protocol: Enable file:// protocol support (default: False).
                **WARNING**: Enabling this allows access to ANY file on the system
                that the process can read. Only use with trusted content.
                Prefer using ``asset_root`` for secure local resource loading.

            always_on_top: Keep window always on top of other windows (default: False).
                Useful for floating tool panels or overlay windows.

            tool_window: Apply tool window style (default: False, Windows only).
                When enabled, the window:
                - Does NOT appear in the taskbar
                - Does NOT appear in Alt+Tab window switcher
                - Has a smaller title bar (if decorations are enabled)

                This is commonly used with ``embed_mode="owner"`` for floating tool windows.

                See: https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles

            undecorated_shadow: Show shadow for frameless windows (default: True, Windows only).
                When ``frame=False``, Windows can still show a subtle shadow around the window.
                Set this to ``False`` to disable the shadow completely, which is required for
                truly transparent frameless windows (e.g., floating logo buttons).

                Example::

                    # Transparent logo button with no shadow
                    webview = WebView(
                        html=LOGO_HTML,
                        width=64,
                        height=64,
                        frame=False,
                        transparent=True,
                        undecorated_shadow=False,
                        tool_window=True,
                    )

            parent_hwnd: Alias for ``parent`` parameter (for backward compatibility).
            embed_mode: Alias for ``mode`` parameter. Values: "child", "owner".

                - **child**: Embed WebView inside a Qt widget (WS_CHILD).
                  Window is clipped to parent bounds, cannot be moved independently.

                - **owner**: Create floating tool window (GWLP_HWNDPARENT).
                  Window stays above owner in Z-order, hidden when owner minimizes.

                See: https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features
        """
        if _CoreWebView is None:
            import sys

            error_details = [
                "AuroraView core library not found.",
                f"Import error: {_CORE_IMPORT_ERROR}",
                f"Python version: {sys.version}",
                f"Platform: {sys.platform}",
            ]
            # Check if _core.pyd exists in expected locations
            try:
                import auroraview

                pkg_dir = Path(auroraview.__file__).parent
                pyd_path = pkg_dir / "_core.pyd"
                so_path = pkg_dir / "_core.so"
                if pyd_path.exists():
                    error_details.append(f"Found: {pyd_path}")
                elif so_path.exists():
                    error_details.append(f"Found: {so_path}")
                else:
                    error_details.append(f"_core.pyd not found in: {pkg_dir}")
            except Exception:
                pass
            raise RuntimeError("\n".join(error_details))

        # Backward-compat parameter aliases
        if dev_tools is not None and debug is None:
            debug = dev_tools
        if decorations is not None and frame is None:
            frame = decorations
        if debug is None:
            debug = True
        if frame is None:
            frame = True

        # Handle parameter aliases for more intuitive API
        # parent_hwnd is alias for parent
        if parent_hwnd is not None and parent is None:
            parent = parent_hwnd
        # embed_mode is alias for mode
        if embed_mode is not None and mode is None:
            mode = embed_mode

        # Map new parameter names to Rust core (which still uses old names)
        self._core = _CoreWebView(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=debug,  # debug -> dev_tools
            context_menu=context_menu,
            resizable=resizable,
            decorations=frame,  # frame -> decorations
            parent_hwnd=parent,  # parent -> parent_hwnd
            parent_mode=mode,  # mode -> parent_mode
            asset_root=asset_root,  # Custom protocol asset root
            data_directory=data_directory,  # User data directory (cookies, cache, etc.)
            allow_file_protocol=allow_file_protocol,  # Enable file:// protocol
            always_on_top=always_on_top,  # Keep window always on top
            transparent=transparent,  # Enable transparent window
            background_color=background_color,  # Window background color
            auto_show=auto_show,  # Control window visibility on creation
            ipc_batch_size=ipc_batch_size,  # Max messages per tick (0=unlimited)
            icon=icon,  # Custom window icon path
            tool_window=tool_window,  # Tool window style (hide from taskbar/Alt+Tab)
            undecorated_shadow=undecorated_shadow,  # Show shadow for frameless windows
            allow_new_window=allow_new_window,  # Allow window.open() to create new windows
        )
        self._event_handlers: Dict[str, List[Callable]] = {}
        self._title = title
        self._width = width
        self._height = height
        self._debug = debug
        self._resizable = resizable
        self._frame = frame
        self._parent = parent
        self._mode = mode
        self._always_on_top = always_on_top
        self._transparent = transparent
        self._background_color = background_color
        self._tool_window = tool_window
        self._undecorated_shadow = undecorated_shadow
        self._allow_new_window = allow_new_window
        self._show_thread: Optional[threading.Thread] = None
        self._is_running = False
        self._auto_timer = None  # Will be set by create() factory method
        self._auto_show = auto_show  # Store auto_show setting
        # Store content for async mode
        self._stored_url: Optional[str] = None
        self._stored_html: Optional[str] = None
        # Store the background thread's core instance
        self._async_core: Optional[Any] = None
        self._async_core_lock = threading.Lock()
        # Track if running in blocking event loop (for HWND mode)
        self._in_blocking_event_loop = False

        # Event processor (strategy pattern for UI framework integration)
        self._event_processor: Optional[Any] = None

        # Post eval_js hook (for Qt integration and testing)
        self._post_eval_js_hook: Optional[Callable[[], None]] = None

        # Bridge integration
        self._bridge: Optional["Bridge"] = None  # type: ignore
        if bridge is not None:
            if bridge is True:
                # Auto-create bridge with default settings
                from .bridge import Bridge

                self._bridge = Bridge(port=9001)
                logger.info("Auto-created Bridge on port 9001")
            else:
                # Use provided bridge instance
                self._bridge = bridge
                logger.info(f"Using provided Bridge: {bridge}")

            # Setup bidirectional communication
            if self._bridge:
                self._setup_bridge_integration()

        # Shared state system (lazy initialization)
        self._state: Optional["State"] = None

        # Command registry (lazy initialization)
        self._commands: Optional["CommandRegistry"] = None

        # Channel manager (lazy initialization)
        self._channels: Optional["ChannelManager"] = None

    @property
    def state(self) -> "State":
        """Get the shared state container for Python ↔ JavaScript sync.

        Returns:
            State container with dict-like interface

        Example:
            >>> webview.state["user"] = {"name": "Alice"}
            >>> webview.state["theme"] = "dark"
            >>>
            >>> @webview.state.on_change
            >>> def handle_change(key, value, source):
            ...     print(f"{key} = {value} from {source}")
        """
        if self._state is None:
            from .state import State

            self._state = State(self)
        return self._state

    @property
    def commands(self) -> "CommandRegistry":
        """Get the command registry for Python ↔ JavaScript RPC.

        Returns:
            CommandRegistry instance

        Example:
            >>> @webview.commands.register
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
        """
        if self._commands is None:
            from .commands import CommandRegistry

            self._commands = CommandRegistry(self)
        return self._commands

    def command(self, func_or_name=None):
        """Decorator to register a command callable from JavaScript.

        This is a convenience shortcut for `webview.commands.register`.

        Args:
            func_or_name: Function to register or custom command name

        Returns:
            Decorated function

        Example:
            >>> @webview.command
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
            >>>
            >>> @webview.command("add_numbers")
            >>> def add(x: int, y: int) -> int:
            ...     return x + y
            >>>
            >>> # In JavaScript:
            >>> # const msg = await auroraview.invoke("greet", {name: "World"});
            >>> # const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
        """
        return self.commands.register(func_or_name)

    @property
    def channels(self) -> "ChannelManager":
        """Get the channel manager for streaming data.

        Returns:
            ChannelManager instance

        Example:
            >>> channel = webview.channels.create()
            >>> channel.send({"progress": 50})
            >>> channel.send({"progress": 100})
            >>> channel.close()
        """
        if self._channels is None:
            from .channel import ChannelManager

            self._channels = ChannelManager(self)
        return self._channels

    def create_channel(self, channel_id: Optional[str] = None) -> "Channel":
        """Create a new streaming channel.

        This is a convenience shortcut for `webview.channels.create()`.

        Args:
            channel_id: Optional custom channel ID

        Returns:
            New Channel instance

        Example:
            >>> with webview.create_channel() as channel:
            ...     for chunk in read_large_file():
            ...         channel.send(chunk)
        """
        return self.channels.create(channel_id)

    @classmethod
    def create(
        cls,
        title: str = "AuroraView",
        *,
        # Content
        url: Optional[str] = None,
        html: Optional[str] = None,
        # Window properties
        width: int = 800,
        height: int = 600,
        resizable: bool = True,
        frame: bool = True,
        always_on_top: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        # DCC integration
        parent: Optional[int] = None,
        mode: Literal["auto", "owner", "child", "container"] = "auto",
        # Bridge integration
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        # Development options
        debug: bool = True,
        context_menu: bool = True,
        # Custom protocol
        asset_root: Optional[str] = None,
        data_directory: Optional[str] = None,
        allow_file_protocol: bool = False,
        # Automation
        auto_show: bool = False,
        auto_timer: bool = True,
        # Singleton control
        singleton: Optional[str] = None,
        # IPC performance tuning
        ipc_batch_size: int = 0,
        # Custom icon
        icon: Optional[str] = None,
    ) -> "WebView":
        """Create WebView instance (recommended way).

        Args:
            title: Window title
            url: URL to load
            html: HTML content to load
            width: Window width in pixels
            height: Window height in pixels
            resizable: Make window resizable
            frame: Show window frame (title bar, borders)
            always_on_top: Keep window always on top of other windows (default: False)
            parent: Parent window handle for DCC embedding
            mode: Embedding mode
                - "auto": Auto-select (recommended)
                - "owner": Owner mode (cross-thread safe)
                - "child": Child window mode (same-thread)
            bridge: Bridge for DCC/Web integration
                - Bridge instance: Use provided bridge
                - True: Auto-create bridge (port 9001)
                - None: No bridge (default)
            debug: Enable developer tools
            context_menu: Enable native context menu (default: True)
            asset_root: Root directory for auroraview:// protocol
            data_directory: User data directory for WebView (cookies, cache, localStorage).
                If None, uses system default. Set to isolate data per app/user profile.
            allow_file_protocol: Enable file:// protocol support (default: False)
                WARNING: Enabling this bypasses WebView's default security restrictions
            auto_show: Automatically show after creation
            auto_timer: Auto-start event timer for embedded mode (recommended)
            singleton: Singleton key. If provided, only one instance with this key
                      can exist at a time. Calling create() again with the same key
                      returns the existing instance.

        Returns:
            WebView instance

        Examples:
            >>> # Standalone window
            >>> webview = WebView.create("My App", url="http://localhost:3000")
            >>> webview.show()

            >>> # DCC embedding (Maya)
            >>> webview = WebView.create("Maya Tool", parent=maya_hwnd)
            >>> webview.show()

            >>> # With Bridge integration
            >>> webview = WebView.create("Photoshop Tool", bridge=True)
            >>> @webview.bridge.on('layer_created')
            >>> async def handle_layer(data, client):
            ...     return {"status": "ok"}
            >>> webview.show()

            >>> # Auto-show
            >>> webview = WebView.create("App", auto_show=True)

            >>> # Singleton mode - only one instance allowed

        Note:
            For Qt-based DCC applications (Maya, Houdini, Nuke), consider using
            QtWebView instead for automatic event processing and better integration:

            >>> from auroraview import QtWebView
            >>> webview = QtWebView(parent=maya_main_window(), title="My Tool")
            >>> webview.load_url("http://localhost:3000")
            >>> webview.show()  # Automatic event processing!
            >>> webview1 = WebView.create("Tool", singleton="my_tool")
            >>> webview2 = WebView.create("Tool", singleton="my_tool")  # Returns webview1
            >>> assert webview1 is webview2
        """
        # Check singleton registry
        if singleton is not None:
            if singleton in cls._singleton_registry:
                existing = cls._singleton_registry[singleton]
                logger.info(f"Returning existing singleton instance: '{singleton}'")
                return existing
            logger.info(f"Creating new singleton instance: '{singleton}'")
        # Detect mode
        is_embedded = parent is not None

        # Auto-select mode
        if mode == "auto":
            actual_mode = "owner" if is_embedded else None
            if is_embedded:
                logger.info(f"[AUTO-DETECT] parent={parent} detected, auto-selecting mode='owner'")
        else:
            actual_mode = mode if is_embedded else None
            if is_embedded:
                logger.info(f"[MANUAL] Using user-specified mode='{mode}'")

        logger.info(f"[MODE] Final mode: {actual_mode} (embedded={is_embedded})")

        # Create instance
        # For embedded mode, always set auto_show=False to let Qt control visibility
        # For standalone mode, use the user-provided auto_show value
        rust_auto_show = False if is_embedded else auto_show
        instance = cls(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            resizable=resizable,
            frame=frame,
            always_on_top=always_on_top,
            transparent=transparent,
            background_color=background_color,
            parent=parent,
            mode=actual_mode,
            debug=debug,
            context_menu=context_menu,
            bridge=bridge,
            asset_root=asset_root,
            data_directory=data_directory,
            allow_file_protocol=allow_file_protocol,
            auto_show=rust_auto_show,  # Pass to Rust layer
            ipc_batch_size=ipc_batch_size,  # Max messages per tick (0=unlimited)
            icon=icon,  # Custom window icon path
        )

        # Auto timer (embedded mode)
        if is_embedded and auto_timer:
            try:
                from auroraview.utils.event_timer import EventTimer

                instance._auto_timer = EventTimer(instance, interval_ms=16)
                instance._auto_timer.on_close(lambda: instance._auto_timer.stop())
                logger.info("Auto timer created for embedded mode")
            except ImportError as e:
                logger.warning("EventTimer not available: %s, auto_timer disabled", e)
                instance._auto_timer = None
        else:
            instance._auto_timer = None

        # Register singleton
        if singleton is not None:
            cls._singleton_registry[singleton] = instance
            logger.info(f"Registered singleton instance: '{singleton}'")

        # Auto show (only for standalone mode, embedded mode is controlled by Qt)
        if auto_show and not is_embedded:
            instance.show()

        return instance

    @classmethod
    def run_embedded(
        cls,
        title: str = "AuroraView",
        *,
        url: Optional[str] = None,
        html: Optional[str] = None,
        width: int = 800,
        height: int = 600,
        resizable: bool = True,
        frame: bool = True,
        parent: Optional[int] = None,
        mode: Literal["auto", "owner", "child"] = "owner",
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        debug: bool = True,
        context_menu: bool = True,
        auto_timer: bool = True,
    ) -> "WebView":
        """Create and show an embedded WebView with auto timer (non-blocking).

        This is a convenience helper equivalent to:
            WebView.create(..., parent=..., mode=..., auto_timer=True, auto_show=True)

        Returns:
            WebView: The created instance (kept alive by your reference)
        """
        instance = cls.create(
            title=title,
            url=url,
            html=html,
            width=width,
            height=height,
            resizable=resizable,
            frame=frame,
            parent=parent,
            mode=mode,
            bridge=bridge,
            debug=debug,
            context_menu=context_menu,
            auto_show=True,
            auto_timer=auto_timer,
        )
        return instance

    @classmethod
    def create_embedded(
        cls,
        parent_hwnd: int,
        *,
        title: str = "Embedded WebView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        asset_root: Optional[str] = None,
        debug: bool = True,
    ) -> "WebView":
        """Create a WebView directly embedded into a parent window's HWND.

        This is the fastest way to embed a WebView into a host application because:
        1. No Qt Widget intermediate layer
        2. WebView2 is created synchronously on the calling thread
        3. Uses host's native message loop directly

        This method is ideal when you have the host window's HWND and want
        maximum performance with minimal overhead. Works with DCC applications
        (Maya, 3ds Max, Houdini, etc.) or any Windows application with a HWND.

        Args:
            parent_hwnd: The HWND of the parent window (e.g., from Qt winId())
            title: Window title (for debugging/identification)
            width: Width in pixels
            height: Height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            asset_root: Root directory for auroraview:// protocol (optional)
            debug: Enable developer tools (default: True)

        Returns:
            WebView: A configured WebView instance ready to use

        Example (Houdini):
            >>> import hou
            >>> from auroraview import WebView
            >>> from PySide2.QtCore import QTimer
            >>>
            >>> # Get Houdini main window HWND
            >>> main_window = hou.qt.mainWindow()
            >>> hwnd = int(main_window.winId())
            >>>
            >>> # Create WebView directly embedded
            >>> webview = WebView.create_embedded(
            ...     parent_hwnd=hwnd,
            ...     title="My Tool",
            ...     width=650,
            ...     height=500,
            ...     url="https://example.com"
            ... )
            >>>
            >>> # Set up timer to process messages
            >>> timer = QTimer()
            >>> timer.timeout.connect(webview.process_events_ipc_only)
            >>> timer.start(16)  # 60 FPS
        """
        from auroraview._core import WebView as _CoreWebView

        logger.info(f"[create_embedded] Creating WebView for parent HWND: {parent_hwnd}")

        # Create core WebView using create_embedded static method
        core = _CoreWebView.create_embedded(
            parent_hwnd=parent_hwnd,
            title=title,
            width=width,
            height=height,
        )

        # Create Python wrapper
        instance = cls.__new__(cls)
        instance._core = core
        instance._parent = parent_hwnd
        instance._mode = "child"
        instance._bridge = None
        instance._auto_timer = None
        instance._show_thread = None
        instance._async_core = None
        instance._async_core_lock = threading.Lock()
        instance._event_processor = None
        instance._post_eval_js_hook = None
        instance._config = {
            "title": title,
            "width": width,
            "height": height,
            "url": url,
            "html": html,
            "asset_root": asset_root,
            "debug": debug,
        }

        # Configure asset root
        if asset_root:
            core.set_asset_root(asset_root)

        # Load content
        if url:
            core.load_url(url)
        elif html:
            core.load_html(html)

        logger.info("[create_embedded] WebView created successfully")
        logger.info("[create_embedded] Remember to call process_events_ipc_only() periodically!")

        return instance

    @classmethod
    def create_for_dcc(
        cls,
        parent_hwnd: int,
        *,
        title: str = "DCC WebView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        asset_root: Optional[str] = None,
        debug: bool = True,
    ) -> "WebView":
        """Create a WebView for DCC integration (deprecated alias).

        .. deprecated:: 0.4.0
            Use :meth:`create_embedded` instead. This method will be removed in a future version.

        This is an alias for :meth:`create_embedded` for backward compatibility.
        """
        import warnings

        warnings.warn(
            "create_for_dcc is deprecated, use create_embedded instead",
            DeprecationWarning,
            stacklevel=2,
        )
        return cls.create_embedded(
            parent_hwnd,
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            asset_root=asset_root,
            debug=debug,
        )

    def show(self, *, wait: Optional[bool] = None) -> None:
        """Show the WebView window (smart mode).

        Automatically detects standalone/embedded mode and chooses the best behavior:
        - Standalone window: Blocks until closed (unless wait=False)
        - Embedded window: Non-blocking, auto-starts timer if available

        Args:
            wait: Whether to wait for window to close
                - None: Auto-detect (standalone=True, embedded=False)
                - True: Block until window closes
                - False: Return immediately (background thread)

        Examples:
            >>> # Standalone window - auto-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show()  # Blocks until closed

            >>> # Standalone window - force non-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show(wait=False)  # Returns immediately
            >>> input("Press Enter to exit...")

            >>> # Embedded window - auto non-blocking
            >>> webview = WebView(title="Tool", parent=maya_hwnd)
            >>> webview.show()  # Returns immediately, timer auto-runs
        """
        # Detect mode
        is_embedded = self._parent is not None

        # Auto-detect wait behavior
        if wait is None:
            wait = not is_embedded  # Standalone defaults to blocking

        logger.info(f"Showing WebView: embedded={is_embedded}, wait={wait}")

        # Start Bridge if present
        if self._bridge and not self._bridge.is_running:
            logger.info("Starting Bridge in background...")
            self._bridge.start_background()

        if is_embedded:
            # Embedded mode: non-blocking + auto timer
            logger.info("Embedded mode: non-blocking with auto timer")
            self._show_non_blocking()
            # Start timer immediately - it will wait for WebView to be ready
            if self._auto_timer is not None:
                self._auto_timer.start()
                logger.info("Auto timer started (will wait for WebView initialization)")
        else:
            # Standalone mode
            if wait:
                # Blocking
                logger.info("Standalone mode: blocking until window closes")
                self.show_blocking()
            else:
                # Non-blocking (background thread)
                logger.info("Standalone mode: non-blocking (background thread)")
                logger.warning("⚠️  Window will close when script exits!")
                logger.warning("⚠️  Use wait=True or keep script running with input()")
                self._show_non_blocking()

    def show_async(self) -> None:
        """Show the WebView window in non-blocking mode (compatibility helper).

        Equivalent to calling show(wait=False). Safe to call multiple times; if the
        WebView is already running, the call is ignored.
        """
        self._show_non_blocking()

    def _show_non_blocking(self) -> None:
        """Internal method: non-blocking show (background thread)."""
        if self._is_running:
            logger.warning("WebView is already running")
            return

        logger.info(f"Showing WebView in background thread: {self._title}")
        self._is_running = True

        def _run_webview():
            """Run the WebView in a background thread.

            Note: We create a new WebView instance in the background thread
            because the Rust core requires the WebView to be created and shown
            in the same thread due to GUI event loop requirements.
            """
            try:
                logger.info("Background thread: Creating WebView instance")
                # Create a new WebView instance in this thread
                # This is necessary because the Rust core is not Send/Sync
                from auroraview._core import WebView as _CoreWebView

                core = _CoreWebView(
                    title=self._title,
                    width=self._width,
                    height=self._height,
                    dev_tools=self._debug,  # Use new parameter name
                    resizable=self._resizable,
                    decorations=self._frame,  # Use new parameter name
                    parent_hwnd=self._parent,  # Use new parameter name
                    parent_mode=self._mode,  # Use new parameter name
                    always_on_top=self._always_on_top,  # Keep window always on top
                    transparent=self._transparent,  # Enable transparent window
                    background_color=self._background_color,  # Window background color
                    tool_window=self._tool_window,  # Tool window style
                    undecorated_shadow=self._undecorated_shadow,  # Shadow for frameless
                    allow_new_window=self._allow_new_window,  # Allow window.open()
                )

                # Store the core instance for use by emit() and other methods
                with self._async_core_lock:
                    self._async_core = core

                # Re-register all event handlers in the background thread
                logger.info(
                    f"Background thread: Re-registering {len(self._event_handlers)} event handlers"
                )
                for event_name, handlers in self._event_handlers.items():
                    for handler in handlers:
                        logger.debug(f"Background thread: Registering handler for '{event_name}'")
                        core.on(event_name, handler)

                # Load the same content that was loaded in the main thread
                if self._stored_html:
                    logger.info("Background thread: Loading stored HTML")
                    core.load_html(self._stored_html)
                elif self._stored_url:
                    logger.info("Background thread: Loading stored URL")
                    core.load_url(self._stored_url)
                else:
                    logger.warning("Background thread: No content loaded")

                logger.info("Background thread: Starting WebView event loop")
                core.show()
                logger.info("Background thread: WebView event loop exited")
            except Exception as e:
                logger.error(f"Error in background WebView: {e}", exc_info=True)
            finally:
                # Clear the async core reference
                with self._async_core_lock:
                    self._async_core = None
                self._is_running = False
                logger.info("Background thread: WebView thread finished")

        # Create and start the background thread as daemon
        # CRITICAL: daemon=True allows Maya to exit cleanly when user closes Maya
        # The event loop now uses run_return() instead of run(), which prevents
        # the WebView from calling std::process::exit() and terminating Maya
        self._show_thread = threading.Thread(target=_run_webview, daemon=True)
        self._show_thread.start()
        logger.info("WebView background thread started (daemon=True)")

    def show_blocking(self) -> None:
        """Show the WebView window (blocking - for standalone scripts).

        This method blocks until the window is closed. Use this in standalone scripts
        where you want the script to wait for the user to close the window.

        NOT recommended for DCC integration (Maya, Houdini, etc.) as it will freeze
        the main application.

        Example:
            >>> webview = WebView(title="My App", width=800, height=600)
            >>> webview.load_html("<h1>Hello</h1>")
            >>> webview.show_blocking()  # Blocks until window closes
            >>> print("Window was closed")
        """
        logger.info(f"Showing WebView (blocking): {self._title}")
        logger.info("Calling _core.show()...")

        # Check if we're in embedded mode
        is_embedded = self._parent is not None  # Use new parameter name

        # Mark that we're entering blocking event loop
        # This tells eval_js to skip _auto_process_events since the event loop
        # will handle message queue processing automatically
        self._in_blocking_event_loop = True

        try:
            self._core.show()
            logger.info("_core.show() returned successfully")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise
        finally:
            # Clear the flag when event loop exits
            self._in_blocking_event_loop = False

        # IMPORTANT: Only cleanup in standalone mode
        # In embedded mode, the window should stay open until explicitly closed
        if not is_embedded:
            logger.info("Standalone mode: WebView show_blocking() completed, cleaning up...")
            try:
                self.close()
            except Exception as cleanup_error:
                logger.warning(f"Error during cleanup: {cleanup_error}")
        else:
            logger.info("Embedded mode: WebView window is now open (non-blocking)")
            logger.info("IMPORTANT: Keep this Python object alive to prevent window from closing")
            logger.info("Example: __main__.webview = webview")

    # =========================================================================
    # Window Control Methods - Now provided by WebViewWindowMixin
    # =========================================================================

    # NOTE: Content loading methods (load_url, get_current_url, load_html, load_file, load_local_html)
    # are now provided by WebViewContentMixin

    # =========================================================================
    # JavaScript Interaction Methods - Now provided by WebViewJSMixin
    # =========================================================================

    # NOTE: JavaScript methods (eval_js, eval_js_async, eval_js_awaitable, get_proxy)
    # are now provided by WebViewJSMixin

    # NOTE: DOM methods (dom, dom_all, dom_by_id, dom_by_class)
    # are now provided by WebViewDOMMixin

    # NOTE: Event system methods (emit, on, register_callback, on_loaded, on_shown, etc.)
    # are now provided by WebViewEventMixin

    # NOTE: API binding methods (register_protocol, bind_call, bind_api, _inject_api_methods_via_js)
    # are now provided by WebViewApiMixin

    def wait(self, timeout: Optional[float] = None) -> bool:
        """Wait for the WebView to close.

        This method blocks until the WebView window is closed or the timeout expires.
        Useful when using show_async() to wait for user interaction.

        Args:
            timeout: Maximum time to wait in seconds (None = wait indefinitely)

        Returns:
            True if the WebView closed, False if timeout expired

        Example:
            >>> webview.show_async()
            >>> if webview.wait(timeout=60):
            ...     print("WebView closed by user")
            ... else:
            ...     print("Timeout waiting for WebView")
        """
        if self._show_thread is None:
            logger.warning("WebView is not running")
            return True

        logger.info(f"Waiting for WebView to close (timeout={timeout})")
        self._show_thread.join(timeout=timeout)

        if self._show_thread.is_alive():
            logger.warning("Timeout waiting for WebView to close")
            return False

        logger.info("WebView closed")
        return True

    def set_event_processor(self, processor: Any) -> None:
        """Set event processor (strategy pattern for UI framework integration).

        Args:
            processor: Event processor object with a `process()` method.
                      This allows UI frameworks (Qt, Tk, etc.) to inject their
                      event processing logic.

        Example:
            >>> class QtEventProcessor:
            ...     def process(self):
            ...         QCoreApplication.processEvents()
            ...         webview._core.process_events()
            >>>
            >>> processor = QtEventProcessor()
            >>> webview.set_event_processor(processor)
        """
        self._event_processor = processor
        logger.debug(f"Event processor set: {type(processor).__name__}")

    def _auto_process_events(self) -> None:
        """Automatically process events after emit() or eval_js().

        This method uses the strategy pattern:
        1. If in blocking event loop (HWND mode), skip - event loop handles it
        2. If an event processor is set, use it (UI framework integration)
        3. Otherwise, use default implementation (direct Rust call)

        Subclasses can still override this method for custom behavior.
        """
        # Skip if we're in a blocking event loop (e.g., HWND mode background thread)
        # The event loop automatically processes the message queue
        if self._in_blocking_event_loop:
            logger.debug("Skipping _auto_process_events - in blocking event loop")
            return

        try:
            if self._event_processor is not None:
                # Use strategy pattern: delegate to event processor
                self._event_processor.process()
            else:
                # Default implementation: direct Rust call
                self._core.process_events()
        except Exception as e:
            logger.debug(f"Auto process events failed (non-critical): {e}")

    def process_events(self) -> bool:
        """Process pending window events.

        This method should be called periodically in embedded mode to handle
        window messages and user interactions. Returns True if the window
        should be closed.

        Returns:
            True if the window should close, False otherwise

        Example:
            >>> # In Maya, use a scriptJob to process events
            >>> def process_webview_events():
            ...     if webview.process_events():
            ...         # Window should close
            ...         cmds.scriptJob(kill=job_id)
            ...
            >>> job_id = cmds.scriptJob(event=["idle", process_webview_events])
        """
        return self._core.process_events()

    def process_events_ipc_only(self) -> bool:
        """Process only internal AuroraView IPC without touching host event loop.

        This variant is intended for host-driven embedding scenarios (Qt/DCC)
        where the native window message pump is owned by the host application.
        It only drains the internal WebView message queue and respects
        lifecycle close requests.
        """
        return self._core.process_ipc_only()

    def is_alive(self) -> bool:
        """Check if WebView is still running.

        Returns:
            True if WebView is running, False otherwise

        Example:
            >>> webview.show(wait=False)
            >>> while webview.is_alive():
            ...     time.sleep(0.1)
        """
        if self._show_thread is None:
            return False
        return self._show_thread.is_alive()

    def get_hwnd(self) -> Optional[int]:
        """Get the native window handle (HWND on Windows).

        This is useful for integrating with external applications that need
        the native window handle, such as:
        - Unreal Engine: `unreal.parent_external_window_to_slate(hwnd)`
        - Windows API: Direct window manipulation
        - Other DCC tools with HWND-based integration

        Returns:
            int: The native window handle (HWND), or None if not available.

        Raises:
            RuntimeError: If WebView is not initialized (call show() first).

        Example:
            >>> webview = WebView.create(...)
            >>> webview.show()
            >>> hwnd = webview.get_hwnd()
            >>> if hwnd:
            ...     print(f"Window HWND: 0x{hwnd:x}")
            ...     # Use with Unreal Engine
            ...     # unreal.parent_external_window_to_slate(hwnd)
        """
        return self._core.get_hwnd()

    def get_proxy(self) -> Any:
        """Get a thread-safe proxy for cross-thread operations.

        Returns a WebViewProxy that can be safely shared across threads.
        Use this when you need to call `eval_js`, `emit`, etc. from a different
        thread than the one that created the WebView.

        This is essential for HWND mode where the WebView runs in a background
        thread but you need to call methods from the DCC main thread.

        Returns:
            WebViewProxy: A thread-safe proxy for WebView operations.
                The proxy supports:
                - eval_js(script): Execute JavaScript
                - eval_js_async(script, callback, timeout_ms): Async JavaScript
                - emit(event_name, data): Emit events to JavaScript
                - load_url(url): Load a URL
                - load_html(html): Load HTML content
                - reload(): Reload the current page

        Example:
            >>> # In HWND mode - WebView runs in background thread
            >>> def create_webview_thread():
            ...     webview = WebView(...)
            ...     proxy = webview.get_proxy()  # Get thread-safe proxy
            ...     self._proxy = proxy          # Store for cross-thread access
            ...     webview.show_blocking()
            ...
            >>> # From DCC main thread - safe!
            >>> self._proxy.eval_js("console.log('Hello from DCC!')")
            >>> self._proxy.emit("update", {"status": "ready"})

        Note:
            The proxy uses a message queue internally. Operations are queued
            and processed by the WebView's event loop on the correct thread.
        """
        # Use async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core
        return core.get_proxy()

    def create_emitter(self) -> Any:
        """Create a thread-safe event emitter.

        Returns an EventEmitter that can be safely used from any thread to emit
        events to the JavaScript frontend. This is useful for plugin callbacks
        that run on background threads (e.g., ProcessPlugin).

        Unlike `emit()` which must be called from the main thread, the emitter
        returned by this method can be called from any thread safely.

        Returns:
            EventEmitter: A thread-safe emitter with an `emit(event_name, data)` method.

        Example:
            >>> # Create emitter for cross-thread event emission
            >>> emitter = webview.create_emitter()
            >>>
            >>> # Use with PluginManager (ProcessPlugin runs on background threads)
            >>> plugins = PluginManager.permissive()
            >>> plugins.set_emit_callback(emitter.emit)
            >>>
            >>> # ProcessPlugin will emit events like:
            >>> # - process:stdout - { pid, data }
            >>> # - process:stderr - { pid, data }
            >>> # - process:exit - { pid, code }
        """
        return self._core.create_emitter()

    def close(self) -> None:
        """Close the WebView window and remove from singleton registry."""
        logger.info("Closing WebView")

        try:
            # Close the core WebView
            self._core.close()
            logger.info("Core WebView closed")
        except Exception as e:
            logger.warning(f"Error closing core WebView: {e}")

        # Wait for background thread if running
        if self._show_thread is not None and self._show_thread.is_alive():
            logger.info("Waiting for background thread to finish...")
            self._show_thread.join(timeout=5.0)
            if self._show_thread.is_alive():
                logger.warning("Background thread did not finish within timeout")
            else:
                logger.info("Background thread finished successfully")

        # Remove from singleton registry
        for key, instance in list(self._singleton_registry.items()):
            if instance is self:
                del self._singleton_registry[key]
                logger.info(f"Removed from singleton registry: '{key}'")
                break

        logger.info("WebView closed successfully")

    @property
    def title(self) -> str:
        """Get the window title."""
        return self._core.title

    @title.setter
    def title(self, value: str) -> None:
        """Set the window title."""
        self._core.set_title(value)
        self._title = value

    @property
    def width(self) -> int:
        """Get the window width."""
        return self._width

    @property
    def height(self) -> int:
        """Get the window height."""
        return self._height

    @property
    def x(self) -> Optional[int]:
        """Get the window x position."""
        return self._x

    @property
    def y(self) -> Optional[int]:
        """Get the window y position."""
        return self._y

    def __repr__(self) -> str:
        """String representation of the WebView."""
        return f"WebView(title='{self._title}', width={self._width}, height={self._height})"

    def __enter__(self) -> "WebView":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:  # noqa: ARG002
        """Context manager exit."""
        self.close()

    # Bridge integration methods

    def _setup_bridge_integration(self):
        """Setup bidirectional communication between Bridge and WebView.

        This method is called automatically when a Bridge is associated with the WebView.
        It sets up:
        1. Bridge → WebView: Forward bridge events to WebView UI
        2. WebView → Bridge: Register handler to send commands to bridge clients
        """
        if not self._bridge:
            return

        logger.info("Setting up Bridge ↔ WebView integration")

        # Bridge → WebView: Forward events
        def bridge_callback(action: str, data: Dict, result: Any):
            """Forward bridge events to WebView UI."""
            logger.debug(f"Bridge event: {action}")
            # Emit event to JavaScript with 'bridge:' prefix
            self.emit(f"bridge:{action}", {"action": action, "data": data, "result": result})

        self._bridge.set_webview_callback(bridge_callback)

        # WebView → Bridge: Register command sender
        @self.on("send_to_bridge")
        def handle_send_to_bridge(data):
            """Send command from WebView to Bridge clients."""
            command = data.get("command")
            params = data.get("params", {})
            logger.debug(f"WebView → Bridge: {command}")
            if self._bridge:
                self._bridge.execute_command(command, params)
            return {"status": "sent"}

        logger.info("Bridge <-> WebView integration complete")

    @property
    def bridge(self) -> Optional["Bridge"]:  # type: ignore
        """Get the associated Bridge instance.

        Returns:
            Bridge instance or None if no bridge is associated

        Example:
            >>> webview = WebView.create("Tool", bridge=True)
            >>> print(webview.bridge)  # Bridge(ws://localhost:9001, ...)
            >>>
            >>> # Register handlers on the bridge
            >>> @webview.bridge.on('custom_event')
            >>> async def handle_custom(data, client):
            ...     return {"status": "ok"}
        """
        return self._bridge

    def send_to_bridge(self, command: str, params: Dict[str, Any] = None):
        """Send command to Bridge clients (convenience method).

        Args:
            command: Command name
            params: Command parameters

        Example:
            >>> webview.send_to_bridge('create_layer', {'name': 'New Layer'})
        """
        if not self._bridge:
            logger.warning("No bridge associated with this WebView")
            return

        self._bridge.execute_command(command, params)
