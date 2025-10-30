"""High-level Python API for WebView."""

import logging
import threading
from typing import Any, Callable, Dict, Optional, Union

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
        dev_tools: Enable developer tools (default: True)
        resizable: Make window resizable (default: True)
        parent_hwnd: Parent window handle (HWND on Windows) for embedding/ownership (optional)
        parent_mode: "child" | "owner" (Windows only). "owner" is safer for cross-thread usage; "child" requires same-thread parenting with the host window.

    Example:
        >>> webview = WebView(title="My Tool", width=1024, height=768)
        >>> webview.load_url("http://localhost:3000")
        >>> webview.show()

        >>> # For DCC integration (e.g., Maya)
        >>> import maya.OpenMayaUI as omui
        >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
        >>> # Prefer owner mode to avoid cross-thread freezes
        >>> webview = WebView(title="My Tool", parent_hwnd=maya_hwnd, parent_mode="owner")
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
        decorations: bool = True,
        parent_hwnd: Optional[int] = None,
        parent_mode: Optional[str] = None,
    ) -> None:
        """Initialize the WebView.

        Args:
            title: Window title
            width: Window width in pixels
            height: Window height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            dev_tools: Enable developer tools (default: True)
            resizable: Make window resizable (default: True)
            decorations: Show window decorations (title bar, borders) (default: True)
            parent_hwnd: Parent window handle for embedding (optional)
            parent_mode: Embedding mode - "child" or "owner" (optional)
        """
        if _CoreWebView is None:
            raise RuntimeError(
                "AuroraView core library not found. "
                "Please ensure the package is properly installed."
            )

        self._core = _CoreWebView(
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
        self._event_handlers: Dict[str, list[Callable]] = {}
        self._title = title
        self._width = width
        self._height = height
        self._dev_tools = dev_tools
        self._resizable = resizable
        self._decorations = decorations
        self._parent_hwnd = parent_hwnd
        self._parent_mode = parent_mode
        self._show_thread: Optional[threading.Thread] = None
        self._is_running = False
        # Store content for async mode
        self._stored_url: Optional[str] = None
        self._stored_html: Optional[str] = None
        # Store the background thread's core instance
        self._async_core: Optional[Any] = None
        self._async_core_lock = threading.Lock()

    def show(self) -> None:
        """Show the WebView window.

        Behavior depends on the mode:
        - Standalone mode (no parent_hwnd): Blocking call, runs event loop until window closes
        - Embedded mode (with parent_hwnd): Non-blocking call, returns immediately

        In embedded mode, the window remains open until explicitly closed or the Python object is destroyed.
        To keep the window open, store the WebView object in a persistent variable (e.g., global).

        For background thread usage, use show_async() instead.
        """
        logger.info(f"Showing WebView: {self._title}")
        logger.info("Calling _core.show()...")

        # Check if we're in embedded mode
        is_embedded = self._parent_hwnd is not None

        try:
            self._core.show()
            logger.info("_core.show() returned successfully")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise

        # IMPORTANT: Only cleanup in standalone mode
        # In embedded mode, the window should stay open until explicitly closed
        if not is_embedded:
            logger.info("Standalone mode: WebView show() completed, cleaning up...")
            try:
                self.close()
            except Exception as cleanup_error:
                logger.warning(f"Error during cleanup: {cleanup_error}")
        else:
            logger.info("Embedded mode: WebView window is now open (non-blocking)")
            logger.info("IMPORTANT: Keep this Python object alive to prevent window from closing")
            logger.info("Example: __main__.webview = webview")

    def show_async(self) -> None:
        """Show the WebView window in a background thread (non-blocking).

        This method is designed for DCC integration (e.g., Maya, Houdini, Blender).
        It starts the WebView in a separate thread, allowing the main thread to continue.

        This prevents blocking the DCC application's main thread while the WebView is running.

        IMPORTANT: Due to GUI thread requirements, this method uses a workaround:
        - The WebView is created and shown in a separate thread
        - The thread is a daemon thread so it won't prevent the application from exiting
        - Event handlers are re-registered in the background thread
        - The WebView will run until the user closes the window

        Example:
            >>> webview = WebView(title="Maya Tool", width=600, height=500)
            >>> webview.load_html("<h1>Hello Maya</h1>")
            >>> webview.show_async()  # Returns immediately
            >>> # Main thread continues here
            >>> print("WebView is running in background")
        """
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
                    dev_tools=self._dev_tools,
                    resizable=self._resizable,
                    decorations=self._decorations,
                    parent_hwnd=self._parent_hwnd,
                    parent_mode=self._parent_mode,
                )

                # Store the core instance for use by emit() and other methods
                with self._async_core_lock:
                    self._async_core = core

                # Re-register all event handlers in the background thread
                logger.info(f"Background thread: Re-registering {len(self._event_handlers)} event handlers")
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

        logger.debug(f"Emitting event: {event_name}")

        # Convert data to dict if needed
        if not isinstance(data, dict):
            data = {"value": data}

        # Use the async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core

        core.emit(event_name, data)

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

    def close(self) -> None:
        """Close the WebView window."""
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

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit."""
        self.close()

