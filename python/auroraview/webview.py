"""High-level Python API for WebView."""

from typing import Optional, Dict, Any, Callable, Union
import json
import logging

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

    def show(self) -> None:
        """Show the WebView window.

        This method displays the WebView window and starts the event loop.
        """
        logger.info(f"Showing WebView: {self._title}")
        logger.info(f"Calling _core.show()...")
        try:
            self._core.show()
            logger.info("_core.show() returned")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise

    def load_url(self, url: str) -> None:
        """Load a URL in the WebView.
        
        Args:
            url: The URL to load
            
        Example:
            >>> webview.load_url("https://example.com")
        """
        logger.info(f"Loading URL: {url}")
        self._core.load_url(url)

    def load_html(self, html: str) -> None:
        """Load HTML content in the WebView.
        
        Args:
            html: HTML content to load
            
        Example:
            >>> webview.load_html("<h1>Hello, World!</h1>")
        """
        logger.info(f"Loading HTML ({len(html)} bytes)")
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

    def close(self) -> None:
        """Close the WebView window."""
        logger.info("Closing WebView")
        self._core.close()

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

