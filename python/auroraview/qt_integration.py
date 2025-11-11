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

    def __init__(self, parent=None):
        super().__init__(parent)
        self._handlers: Dict[str, list[Callable]] = {}
        self._is_destroyed = False
        logger.debug("EventBridge initialized")

    @Slot(str, str)
    def js_to_python(self, event_name: str, json_data: str):
        """Receive events from JavaScript.

        Args:
            event_name: Name of the event
            json_data: JSON-serialized event data
        """
        # Check if bridge is destroyed
        if self._is_destroyed:
            logger.debug(f"Ignoring event '{event_name}' - bridge is destroyed")
            return

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
        # Check if bridge is destroyed
        if self._is_destroyed:
            logger.debug(f"Ignoring emit '{event_name}' - bridge is destroyed")
            return

        try:
            json_data = json.dumps(data) if data else "{}"
            self.python_to_js.emit(event_name, json_data)
            logger.debug(f"Event sent to JS: {event_name}, data: {data}")
        except Exception as e:
            logger.error(f"Failed to send event {event_name} to JS: {e}")

    def cleanup(self):
        """Cleanup bridge resources and mark as destroyed."""
        logger.debug("EventBridge cleanup started")
        self._is_destroyed = True
        self._handlers.clear()
        logger.debug("EventBridge cleanup completed")


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

        # Set window to delete on close to prevent orphaned windows
        from qtpy.QtCore import Qt

        self.setAttribute(Qt.WA_DeleteOnClose, True)

        # Enable developer tools
        if dev_tools:
            from qtpy.QtWebEngineWidgets import QWebEngineSettings

            settings = self.settings()
            settings.setAttribute(QWebEngineSettings.JavascriptEnabled, True)
            settings.setAttribute(QWebEngineSettings.LocalContentCanAccessRemoteUrls, True)

        # Create event bridge with this widget as parent for proper cleanup
        self._bridge = EventBridge(parent=self)
        self._channel = QWebChannel(self)
        self._channel.registerObject("auroraview_bridge", self._bridge)
        self.page().setWebChannel(self._channel)

        # Inject bridge script after page load
        self.loadFinished.connect(self._inject_bridge)

        # Track cleanup state
        self._is_closing = False

        # Register application quit handler to ensure cleanup
        self._register_app_quit_handler()

        logger.info(f"AuroraViewQt created: {title} ({width}x{height})")

    def _register_app_quit_handler(self):
        """Register handler to close window when application quits.

        This ensures the WebView is properly closed when the DCC application
        (like Nuke, Maya, Houdini) exits, preventing orphaned windows.
        """
        try:
            from qtpy.QtCore import QCoreApplication

            app = QCoreApplication.instance()
            if app:
                # Connect to aboutToQuit signal
                app.aboutToQuit.connect(self._on_app_quit)
                logger.debug("Registered application quit handler")
        except Exception as e:
            logger.warning(f"Could not register app quit handler: {e}")

    def _on_app_quit(self):
        """Handle application quit event.

        This is called when the DCC application is about to quit.
        We need to cleanup immediately to prevent errors during shutdown.
        """
        logger.info("Application quitting - cleaning up QtWebView")

        if self._is_closing:
            return

        self._is_closing = True

        try:
            # Immediate cleanup without waiting for closeEvent
            # This prevents errors during application shutdown

            # 1. Destroy JavaScript bridge first
            try:
                cleanup_js = """
                (function() {
                    if (window.auroraview && window.auroraview._destroy) {
                        window.auroraview._destroy();
                    }
                    window.auroraview = null;
                })();
                """
                self.page().runJavaScript(cleanup_js)
            except Exception:
                pass

            # 2. Cleanup Python bridge
            if hasattr(self, "_bridge") and self._bridge:
                self._bridge.cleanup()

            # 3. Stop page immediately
            try:
                self.stop()
                self.setHtml("")
            except Exception:
                pass

            # 4. Unregister web channel
            if hasattr(self, "_channel") and self._channel:
                try:
                    self._channel.deregisterObject(self._bridge)
                except (RuntimeError, AttributeError):
                    pass

            # 5. Hide window (don't call close() to avoid triggering closeEvent again)
            try:
                self.hide()
            except Exception:
                pass

            logger.info("QtWebView cleanup on app quit completed")
        except Exception as e:
            logger.error(f"Error during app quit cleanup: {e}")

    def _inject_bridge(self, ok: bool):
        """Inject JavaScript bridge after page load.

        Args:
            ok: Whether the page loaded successfully
        """
        # Check if we're closing - don't inject if so
        if self._is_closing:
            logger.debug("Skipping bridge injection - window is closing")
            return

        if not ok:
            logger.error("Page failed to load")
            return

        script = """
        (function() {
            // Prevent multiple initializations
            if (window.auroraview) {
                console.log('[AuroraView] Bridge already initialized');
                return;
            }

            // Check if QWebChannel is available
            if (typeof QWebChannel === 'undefined') {
                console.error('[AuroraView] QWebChannel not available');
                return;
            }

            // Check if qt.webChannelTransport is available
            if (typeof qt === 'undefined' || !qt.webChannelTransport) {
                console.error('[AuroraView] qt.webChannelTransport not available');
                return;
            }

            try {
                new QWebChannel(qt.webChannelTransport, function(channel) {
                    // Check if bridge object exists
                    if (!channel.objects || !channel.objects.auroraview_bridge) {
                        console.error('[AuroraView] Bridge object not found');
                        return;
                    }

                    // Create safe wrapper functions
                    var bridge = channel.objects.auroraview_bridge;
                    var isDestroyed = false;

                    // Listen for window unload to mark as destroyed
                    window.addEventListener('beforeunload', function() {
                        isDestroyed = true;
                        console.log('[AuroraView] Window unloading - marking bridge as destroyed');
                    });

                    window.auroraview = {
                        // Send event to Python
                        send_event: function(eventName, data) {
                            if (isDestroyed) {
                                console.warn('[AuroraView] Bridge destroyed - ignoring send_event');
                                return;
                            }
                            try {
                                if (!bridge || !bridge.js_to_python) {
                                    console.error('[AuroraView] Bridge not available');
                                    return;
                                }
                                var jsonData = JSON.stringify(data || {});
                                bridge.js_to_python(eventName, jsonData);
                                console.log('[AuroraView] Event sent to Python:', eventName, data);
                            } catch (e) {
                                console.error('[AuroraView] Error sending event:', e);
                            }
                        },

                        // Receive events from Python
                        on: function(eventName, callback) {
                            if (isDestroyed) {
                                console.warn('[AuroraView] Bridge destroyed - ignoring on');
                                return;
                            }
                            try {
                                if (!bridge || !bridge.python_to_js) {
                                    console.error('[AuroraView] Bridge not available');
                                    return;
                                }
                                bridge.python_to_js.connect(function(name, jsonData) {
                                    if (isDestroyed) {
                                        return;
                                    }
                                    if (name === eventName) {
                                        try {
                                            var data = JSON.parse(jsonData);
                                            console.log('[AuroraView] Event received from Python:', name, data);
                                            callback(data);
                                        } catch (e) {
                                            console.error('[AuroraView] Error parsing event data:', e);
                                        }
                                    }
                                });
                            } catch (e) {
                                console.error('[AuroraView] Error registering event handler:', e);
                            }
                        },

                        // Cleanup method
                        _destroy: function() {
                            isDestroyed = true;
                            console.log('[AuroraView] Bridge destroyed');
                        }
                    };

                    console.log('[AuroraView] Bridge initialized');
                    console.log('[AuroraView] Use window.auroraview.send_event(name, data) to send events to Python');
                });
            } catch (e) {
                console.error('[AuroraView] Error initializing QWebChannel:', e);
            }
        })();
        """

        try:
            self.page().runJavaScript(script)
            logger.debug("JavaScript bridge injected")
        except Exception as e:
            logger.error(f"Failed to inject JavaScript bridge: {e}")

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
        # Force process events to ensure immediate delivery
        self._process_pending_events()

    def _process_pending_events(self):
        """Process pending Qt events to ensure immediate UI updates.

        This is particularly important in DCC applications where the event
        loop might be busy with other tasks.
        """
        try:
            from qtpy.QtCore import QCoreApplication

            # Process all pending events
            QCoreApplication.processEvents()
        except Exception as e:
            logger.debug(f"Could not process pending events: {e}")

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

    def closeEvent(self, event):
        """Handle Qt close event.

        This ensures proper cleanup of the event bridge and web channel
        before the widget is destroyed.

        Args:
            event: QCloseEvent
        """
        if self._is_closing:
            event.accept()
            return

        logger.info("QtWebView closeEvent triggered")
        self._is_closing = True

        try:
            # Step 1: Destroy JavaScript bridge FIRST
            try:
                # Call JavaScript cleanup to mark bridge as destroyed
                cleanup_js = """
                (function() {
                    if (window.auroraview && window.auroraview._destroy) {
                        window.auroraview._destroy();
                    }
                    window.auroraview = null;
                })();
                """
                self.page().runJavaScript(cleanup_js)
                logger.debug("JavaScript bridge destroyed")
            except Exception as e:
                logger.debug(f"Could not destroy JavaScript bridge: {e}")

            # Step 2: Cleanup Python bridge to stop all event processing
            if hasattr(self, "_bridge") and self._bridge:
                self._bridge.cleanup()
                logger.debug("Python bridge cleanup completed")

            # Step 3: Disconnect app quit handler to prevent double cleanup
            try:
                from qtpy.QtCore import QCoreApplication

                app = QCoreApplication.instance()
                if app:
                    try:
                        app.aboutToQuit.disconnect(self._on_app_quit)
                        logger.debug("Disconnected app quit handler")
                    except (RuntimeError, TypeError):
                        pass
            except (AttributeError, ImportError):
                pass

            # Step 4: Unregister web channel
            if hasattr(self, "_channel") and self._channel:
                try:
                    self._channel.deregisterObject(self._bridge)
                    logger.debug("Deregistered web channel object")
                except (RuntimeError, AttributeError):
                    pass

            # Step 5: Stop page loading and clear content
            try:
                # Stop any ongoing page loads
                self.stop()

                # Load blank page to stop all JavaScript execution
                self.setHtml("")
                logger.debug("Stopped page and cleared content")
            except Exception as e:
                logger.debug(f"Could not stop page: {e}")

            # Step 5: Disconnect all signals
            try:
                self.loadFinished.disconnect(self._inject_bridge)
                logger.debug("Disconnected loadFinished signal")
            except (RuntimeError, TypeError):
                pass

            # Step 6: Delete web channel to release resources
            if hasattr(self, "_channel") and self._channel:
                try:
                    self._channel.deleteLater()
                    self._channel = None
                    logger.debug("Scheduled channel deletion")
                except Exception as e:
                    logger.debug(f"Could not delete channel: {e}")

            logger.info("QtWebView cleanup completed")
        except Exception as e:
            logger.error(f"Error during QtWebView cleanup: {e}", exc_info=True)
        finally:
            # Always accept the close event
            event.accept()
            super().closeEvent(event)

    def __del__(self):
        """Destructor - ensure cleanup on deletion."""
        try:
            if hasattr(self, "_bridge") and self._bridge and not self._is_closing:
                logger.debug("QtWebView __del__ triggered - cleaning up")
                self._bridge.cleanup()
        except Exception as e:
            logger.error(f"Error in QtWebView __del__: {e}")

    def __repr__(self) -> str:
        """String representation."""
        try:
            return f"QtWebView(title='{self.windowTitle()}', size={self.width()}x{self.height()})"
        except RuntimeError:
            # Widget already deleted
            return "QtWebView(<deleted>)"


# Backward-compatibility alias
AuroraViewQt = QtWebView

__all__ = ["QtWebView", "AuroraViewQt", "EventBridge"]
