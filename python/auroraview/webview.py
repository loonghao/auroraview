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

    Example:
        >>> webview = WebView(title="My Tool", width=1024, height=768)
        >>> webview.load_url("http://localhost:3000")
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
    ) -> None:
        """Initialize the WebView."""
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
        )
        self._event_handlers: Dict[str, list[Callable]] = {}
        self._title = title
        self._width = width
        self._height = height
        self._show_thread: Optional[threading.Thread] = None
        self._is_running = False
        # Store content for async mode
        self._stored_url: Optional[str] = None
        self._stored_html: Optional[str] = None

    def show(self) -> None:
        """Show the WebView window.

        This method displays the WebView window and starts the event loop.
        This is a blocking call - the method will not return until the window is closed.

        For non-blocking usage (e.g., in DCC applications like Maya), use show_async() instead.
        """
        logger.info(f"Showing WebView: {self._title}")
        logger.info("Calling _core.show()...")
        try:
            self._core.show()
            logger.info("_core.show() returned")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise

    def show_async(self) -> None:
        """Show the WebView window in a background thread (non-blocking).

        This method is designed for DCC integration (e.g., Maya, Houdini, Blender).
        It starts the WebView in a separate thread, allowing the main thread to continue.

        This prevents blocking the DCC application's main thread while the WebView is running.

        IMPORTANT: Due to GUI thread requirements, this method uses a workaround:
        - The WebView is created and shown in a separate thread
        - The thread must be a daemon thread to avoid blocking the main application
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
                )

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
                self._is_running = False
                logger.info("Background thread: WebView thread finished")

        # Create and start the background thread
        self._show_thread = threading.Thread(target=_run_webview, daemon=False)
        self._show_thread.start()
        logger.info("WebView background thread started")

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
        self._core.eval_js(script)

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

        self._core.emit(event_name, data)

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

    def close(self) -> None:
        """Close the WebView window."""
        logger.info("Closing WebView")
        self._core.close()

        # Wait for background thread if running
        if self._show_thread is not None and self._show_thread.is_alive():
            logger.info("Waiting for background thread to finish...")
            self._show_thread.join(timeout=5.0)
            if self._show_thread.is_alive():
                logger.warning("Background thread did not finish within timeout")

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

