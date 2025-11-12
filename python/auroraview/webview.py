"""High-level Python API for WebView."""

from __future__ import annotations

import logging
import threading
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional, Union

try:
    from typing import Literal  # py38+
except ImportError:  # pragma: no cover - only for py37
    from typing_extensions import Literal  # type: ignore

if TYPE_CHECKING:
    from .bridge import Bridge

try:
    from ._core import WebView as _CoreWebView
except ImportError:
    _CoreWebView = None

logger = logging.getLogger(__name__)


class WebView:
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
        resizable: bool = True,
        frame: Optional[bool] = None,
        parent: Optional[int] = None,
        mode: Optional[str] = None,
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        dev_tools: Optional[bool] = None,
        decorations: Optional[bool] = None,
    ) -> None:
        """Initialize the WebView.

        Args:
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            debug: Enable developer tools (default: True)
            resizable: Make window resizable (default: True)
            frame: Show window frame (title bar, borders) (default: True)
            parent: Parent window handle for embedding (optional)
            mode: Embedding mode - "child" or "owner" (optional)
            bridge: Bridge instance for DCC integration
                   - Bridge instance: Use provided bridge
                   - True: Auto-create bridge with default settings
                   - None: No bridge (default)
        """
        if _CoreWebView is None:
            raise RuntimeError(
                "AuroraView core library not found. "
                "Please ensure the package is properly installed."
            )

        # Backward-compat parameter aliases
        if dev_tools is not None and debug is None:
            debug = dev_tools
        if decorations is not None and frame is None:
            frame = decorations
        if debug is None:
            debug = True
        if frame is None:
            frame = True

        # Map new parameter names to Rust core (which still uses old names)
        self._core = _CoreWebView(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            dev_tools=debug,  # debug -> dev_tools
            resizable=resizable,
            decorations=frame,  # frame -> decorations
            parent_hwnd=parent,  # parent -> parent_hwnd
            parent_mode=mode,  # mode -> parent_mode
        )
        self._event_handlers: Dict[str, list[Callable]] = {}
        self._title = title
        self._width = width
        self._height = height
        self._debug = debug
        self._resizable = resizable
        self._frame = frame
        self._parent = parent
        self._mode = mode
        self._show_thread: Optional[threading.Thread] = None
        self._is_running = False
        self._auto_timer = None  # Will be set by create() factory method
        # Store content for async mode
        self._stored_url: Optional[str] = None
        self._stored_html: Optional[str] = None
        # Store the background thread's core instance
        self._async_core: Optional[Any] = None
        self._async_core_lock = threading.Lock()

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
        # DCC integration
        parent: Optional[int] = None,
        mode: Literal["auto", "owner", "child"] = "auto",
        # Bridge integration
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        # Development options
        debug: bool = True,
        # Automation
        auto_show: bool = False,
        auto_timer: bool = True,
        # Singleton control
        singleton: Optional[str] = None,
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
        instance = cls(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            resizable=resizable,
            frame=frame,
            parent=parent,
            mode=actual_mode,
            debug=debug,
            bridge=bridge,
        )

        # Auto timer (embedded mode)
        if is_embedded and auto_timer:
            try:
                from .event_timer import EventTimer

                instance._auto_timer = EventTimer(instance, interval_ms=16)
                instance._auto_timer.on_close(lambda: instance._auto_timer.stop())
                logger.info("Auto timer created for embedded mode")
            except ImportError:
                logger.warning("EventTimer not available, auto_timer disabled")
                instance._auto_timer = None
        else:
            instance._auto_timer = None

        # Register singleton
        if singleton is not None:
            cls._singleton_registry[singleton] = instance
            logger.info(f"Registered singleton instance: '{singleton}'")

        # Auto show
        if auto_show:
            instance.show()

        return instance

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
                from ._core import WebView as _CoreWebView

                core = _CoreWebView(
                    title=self._title,
                    width=self._width,
                    height=self._height,
                    dev_tools=self._debug,  # Use new parameter name
                    resizable=self._resizable,
                    decorations=self._frame,  # Use new parameter name
                    parent_hwnd=self._parent,  # Use new parameter name
                    parent_mode=self._mode,  # Use new parameter name
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

        try:
            self._core.show()
            logger.info("_core.show() returned successfully")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise

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

    def load_url(self, url: str) -> None:
        """Load a URL in the WebView.

        Args:
            url: The URL to load

        Example:
            >>> webview.load_url("https://example.com")
        """
        logger.info(f"Loading URL: {url}")
        self._stored_url = url
        self._stored_html = None
        self._core.load_url(url)

    def load_html(self, html: str) -> None:
        """Load HTML content in the WebView.

        Args:
            html: HTML content to load

        Example:
            >>> webview.load_html("<h1>Hello, World!</h1>")
        """
        logger.info(f"Loading HTML ({len(html)} bytes)")
        self._stored_html = html
        self._stored_url = None
        self._core.load_html(html)

    def eval_js(self, script: str) -> None:
        """Execute JavaScript code in the WebView.

        Args:
            script: JavaScript code to execute

        Example:
            >>> webview.eval_js("console.log('Hello from Python')")
        """
        logger.debug(f"Executing JavaScript: {script[:100]}...")

        # Use the async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core

        core.eval_js(script)

    def emit(self, event_name: str, data: Union[Dict[str, Any], Any] = None) -> None:
        """Emit an event to JavaScript.

        Args:
            event_name: Name of the event
            data: Data to send with the event (will be JSON serialized)

        Example:
            >>> webview.emit("update_scene", {"objects": ["cube", "sphere"]})
        """
        if data is None:
            data = {}

        logger.debug(f"[SEND] [WebView.emit] START - Event: {event_name}")
        logger.debug(f"[SEND] [WebView.emit] Data type: {type(data)}")
        logger.debug(f"[SEND] [WebView.emit] Data: {data}")

        # Convert data to dict if needed
        if not isinstance(data, dict):
            logger.debug("[SEND] [WebView.emit] Converting non-dict data to dict")
            data = {"value": data}

        # Use the async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core

        try:
            logger.debug("[SEND] [WebView.emit] Calling core.emit()...")
            core.emit(event_name, data)
            logger.debug(f"[OK] [WebView.emit] Event emitted successfully: {event_name}")
        except Exception as e:
            logger.error(f"[ERROR] [WebView.emit] Failed to emit event {event_name}: {e}")
            logger.error(f"[ERROR] [WebView.emit] Data was: {data}")
            import traceback

            logger.error(f"[ERROR] [WebView.emit] Traceback: {traceback.format_exc()}")
            raise

    def on(self, event_name: str) -> Callable:
        """Decorator to register a Python callback for JavaScript events.

        Args:
            event_name: Name of the event to listen for

        Returns:
            Decorator function

        Example:
            >>> @webview.on("export_scene")
            >>> def handle_export(data):
            >>>     print(f"Exporting to: {data['path']}")
        """

        def decorator(func: Callable) -> Callable:
            self.register_callback(event_name, func)
            return func

        return decorator

    def register_callback(self, event_name: str, callback: Callable) -> None:
        """Register a callback for an event.

        Args:
            event_name: Name of the event
            callback: Function to call when event occurs
        """
        if event_name not in self._event_handlers:
            self._event_handlers[event_name] = []

        self._event_handlers[event_name].append(callback)
        logger.debug(f"Registered callback for event: {event_name}")

        # Register with core
        self._core.on(event_name, callback)

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

        logger.info("✅ Bridge ↔ WebView integration complete")

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
