"""Qt backend - WebView integrated with Qt framework.

This module provides a Qt WebEngine-based WebView implementation that
integrates seamlessly with DCC applications that already have Qt loaded
(e.g., Maya, Houdini, Nuke).

This backend avoids Windows HWND-related issues and provides better
integration with Qt-based DCC applications.

**Requirements**:
    Install with Qt support: `pip install auroraview[qt]`

    This will install qtpy and compatible Qt bindings (PySide2, PySide6, PyQt5, or PyQt6).

Example:
    >>> from auroraview import QtWebView
    >>>
    >>> # Create WebView as Qt widget
    >>> webview = QtWebView(
    ...     parent=maya_main_window(),
    ...     title="My Tool",
    ...     width=800,
    ...     height=600
    ... )
    >>>
    >>> # Register event handler
    >>> @webview.on('export_scene')
    >>> def handle_export(data):
    ...     print(f"Exporting to: {data['path']}")
    >>>
    >>> # Load HTML
    >>> webview.load_html("<html><body>Hello!</body></html>")
    >>>
    >>> # Show window
    >>> webview.show()
"""

import json
import logging
from typing import Any, Callable, Dict, Optional

try:
    from qtpy.QtCore import QObject, QUrl, Signal, Slot
    from qtpy.QtWebChannel import QWebChannel
    from qtpy.QtWebEngineWidgets import QWebEngineView
except ImportError as e:
    raise ImportError(
        "Qt backend requires qtpy and Qt bindings. Install with: pip install auroraview[qt]"
    ) from e

logger = logging.getLogger(__name__)


class EventBridge(QObject):
    """JavaScript ↔ Python event bridge using Qt WebChannel."""

    # Signal to send events to JavaScript
    python_to_js = Signal(str, str)  # (event_name, json_data)

    def __init__(self):
        super().__init__()
        self._handlers: Dict[str, list[Callable]] = {}
        logger.debug("EventBridge initialized")

    @Slot(str, str)
    def js_to_python(self, event_name: str, json_data: str):
        """Receive events from JavaScript.

        Args:
            event_name: Name of the event
            json_data: JSON-serialized event data
        """
        try:
            data = json.loads(json_data) if json_data else {}
            logger.debug(f"Event received from JS: {event_name}, data: {data}")

            # Call registered handlers
            if event_name in self._handlers:
                for handler in self._handlers[event_name]:
                    try:
                        handler(data)
                    except Exception as e:
                        logger.error(f"Error in event handler for {event_name}: {e}")
            else:
                logger.warning(f"No handler registered for event: {event_name}")

        except json.JSONDecodeError as e:
            logger.error(f"Failed to decode JSON data for event {event_name}: {e}")
        except Exception as e:
            logger.error(f"Error handling event {event_name}: {e}")

    def register_handler(self, event_name: str, handler: Callable):
        """Register Python event handler.

        Args:
            event_name: Name of the event to listen for
            handler: Callback function to call when event occurs
        """
        if event_name not in self._handlers:
            self._handlers[event_name] = []
        self._handlers[event_name].append(handler)
        logger.debug(f"Handler registered for event: {event_name}")

    def emit_to_js(self, event_name: str, data: Any):
        """Send event to JavaScript.

        Args:
            event_name: Name of the event
            data: Event data (will be JSON-serialized)
        """
        try:
            json_data = json.dumps(data) if data else "{}"
            self.python_to_js.emit(event_name, json_data)
            logger.debug(f"Event sent to JS: {event_name}, data: {data}")
        except Exception as e:
            logger.error(f"Failed to send event {event_name} to JS: {e}")


class QtWebView(QWebEngineView):
    """Qt backend WebView implementation.

    This class provides a Qt WebEngine-based WebView that can be used as
    a Qt widget in DCC applications. It's ideal for applications that
    already have Qt loaded (Maya, Houdini, Nuke, etc.).

    Args:
        parent: Parent Qt widget (optional)
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        dev_tools: Enable developer tools (default: True)

    Example:
        >>> from auroraview import QtWebView
        >>>
        >>> # Create WebView as Qt widget
        >>> webview = QtWebView(
        ...     parent=maya_main_window(),
        ...     title="My Tool",
        ...     width=800,
        ...     height=600
        ... )
        >>>
        >>> # Register event handler
        >>> @webview.on('export_scene')
        >>> def handle_export(data):
        ...     print(f"Exporting to: {data['path']}")
        >>>
        >>> # Load HTML content
        >>> webview.load_html("<html><body>Hello!</body></html>")
        >>>
        >>> # Show window
        >>> webview.show()
    """

    def __init__(
        self,
        parent=None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        dev_tools: bool = True,
    ):
        super().__init__(parent)

        self.setWindowTitle(title)
        self.resize(width, height)

        # Enable developer tools
        if dev_tools:
            from PySide2.QtWebEngineWidgets import QWebEngineSettings

            settings = self.settings()
            settings.setAttribute(QWebEngineSettings.JavascriptEnabled, True)
            settings.setAttribute(QWebEngineSettings.LocalContentCanAccessRemoteUrls, True)

        # Create event bridge
        self._bridge = EventBridge()
        self._channel = QWebChannel()
        self._channel.registerObject("auroraview_bridge", self._bridge)
        self.page().setWebChannel(self._channel)

        # Inject bridge script after page load
        self.loadFinished.connect(self._inject_bridge)

        logger.info(f"AuroraViewQt created: {title} ({width}x{height})")

    def _inject_bridge(self, ok: bool):
        """Inject JavaScript bridge after page load.

        Args:
            ok: Whether the page loaded successfully
        """
        if not ok:
            logger.error("Page failed to load")
            return

        script = """
        new QWebChannel(qt.webChannelTransport, function(channel) {
            window.auroraview = {
                // Send event to Python
                send_event: function(eventName, data) {
                    var jsonData = JSON.stringify(data || {});
                    channel.objects.auroraview_bridge.js_to_python(eventName, jsonData);
                    console.log('[AuroraView] Event sent to Python:', eventName, data);
                },

                // Receive events from Python
                on: function(eventName, callback) {
                    channel.objects.auroraview_bridge.python_to_js.connect(function(name, jsonData) {
                        if (name === eventName) {
                            var data = JSON.parse(jsonData);
                            console.log('[AuroraView] Event received from Python:', name, data);
                            callback(data);
                        }
                    });
                }
            };

            console.log('[AuroraView] Bridge initialized');
            console.log('[AuroraView] Use window.auroraview.send_event(name, data) to send events to Python');
        });
        """
        self.page().runJavaScript(script)
        logger.debug("JavaScript bridge injected")

    def on(self, event_name: str) -> Callable:
        """Decorator to register event handler (AuroraView API compatibility).

        Args:
            event_name: Name of the event to listen for

        Returns:
            Decorator function

        Example:
            >>> @webview.on('my_event')
            >>> def handle_event(data):
            ...     print(f"Event data: {data}")
        """

        def decorator(func: Callable) -> Callable:
            self._bridge.register_handler(event_name, func)
            return func

        return decorator

    def register_callback(self, event_name: str, callback: Callable):
        """Register event handler (AuroraView API compatibility).

        Args:
            event_name: Name of the event
            callback: Function to call when event occurs
        """
        self._bridge.register_handler(event_name, callback)

    def emit(self, event_name: str, data: Any = None):
        """Send event to JavaScript (AuroraView API compatibility).

        Args:
            event_name: Name of the event
            data: Event data (will be JSON-serialized)
        """
        self._bridge.emit_to_js(event_name, data)

    def load_url(self, url: str):
        """Load URL.

        Args:
            url: URL to load
        """
        self.setUrl(QUrl(url))
        logger.info(f"Loading URL: {url}")

    def load_html(self, html: str, base_url: Optional[str] = None):
        """Load HTML content.

        Args:
            html: HTML content to load
            base_url: Base URL for resolving relative URLs (optional)
        """
        if base_url:
            self.setHtml(html, QUrl(base_url))
        else:
            self.setHtml(html)
        logger.info(f"Loading HTML ({len(html)} bytes)")

    def eval_js(self, script: str):
        """Execute JavaScript code.

        Args:
            script: JavaScript code to execute
        """
        self.page().runJavaScript(script)
        logger.debug(f"Executing JavaScript: {script[:100]}...")

    @property
    def title(self) -> str:
        """Get window title."""
        return self.windowTitle()

    @title.setter
    def title(self, value: str):
        """Set window title."""
        self.setWindowTitle(value)

    def __repr__(self) -> str:
        """String representation."""
        return f"QtWebView(title='{self.windowTitle()}', size={self.width()}x{self.height()})"


__all__ = ["QtWebView", "EventBridge"]
