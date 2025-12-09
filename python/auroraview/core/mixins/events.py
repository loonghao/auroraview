# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Event System Mixin.

This module provides event handling methods for the WebView class.
It integrates with the signal-slot system for Qt-inspired event handling
while maintaining backward compatibility with the @webview.on() decorator pattern.
"""

from __future__ import annotations

import logging
import threading
import traceback
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Union

from auroraview.core.signals import ConnectionId, WebViewSignals

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)


class WebViewEventMixin:
    """Mixin providing event system methods with signal-slot support.

    Provides methods for event handling:
    - emit: Emit an event to JavaScript
    - on: Decorator to register event callback (backward compatible)
    - register_callback: Register a callback for an event
    - signals: WebViewSignals instance for Qt-style signal connections
    - on_loaded, on_shown, on_closing, on_closed: Lifecycle event decorators
    - on_resized, on_moved, on_focused, on_blurred: Window event decorators
    - on_minimized, on_maximized, on_restored: State event decorators

    Signal-Slot Pattern:
        The mixin supports both the traditional decorator pattern and the
        new Qt-inspired signal-slot pattern:

        # Traditional pattern (still supported)
        @webview.on("my_event")
        def handle_event(data):
            print(data)

        # New signal-slot pattern
        conn_id = webview.signals.custom["my_event"].connect(handle_event)
        webview.signals.custom["my_event"].disconnect(conn_id)
    """

    # Type hints for attributes from main class
    _core: Any
    _async_core: Optional[Any]
    _async_core_lock: threading.Lock
    _event_handlers: Dict[str, List[Callable]]
    _post_eval_js_hook: Optional[Callable[[], None]]
    _auto_process_events: Callable[[], None]
    _signals: Optional[WebViewSignals]

    def _init_signals(self) -> None:
        """Initialize the signal system. Called during WebView initialization."""
        self._signals = WebViewSignals()

    @property
    def signals(self) -> WebViewSignals:
        """Get the WebView signals for Qt-style event handling.

        Returns:
            WebViewSignals instance with pre-defined and custom signals

        Example:
            >>> # Connect to lifecycle signal
            >>> webview.signals.page_loaded.connect(lambda: print("Loaded!"))
            >>>
            >>> # Connect to custom event
            >>> conn = webview.signals.custom["my_event"].connect(handler)
            >>> webview.signals.custom["my_event"].disconnect(conn)
        """
        if not hasattr(self, "_signals") or self._signals is None:
            self._init_signals()
        return self._signals  # type: ignore

    def emit(
        self, event_name: str, data: Union[Dict[str, Any], Any] = None, auto_process: bool = True
    ) -> None:
        """Emit an event to JavaScript.

        Args:
            event_name: Name of the event
            data: Data to send with the event (will be JSON serialized)
            auto_process: Automatically process message queue after emission (default: True).

        Example:
            >>> webview.emit("update_scene", {"objects": ["cube", "sphere"]})

            >>> # Batch multiple events
            >>> webview.emit("event1", {"data": 1}, auto_process=False)
            >>> webview.emit("event2", {"data": 2}, auto_process=False)
            >>> webview.process_events()  # Process all at once
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
            logger.error(f"[ERROR] [WebView.emit] Traceback: {traceback.format_exc()}")
            raise

        # Call post eval_js hook if set (for Qt integration and testing)
        if self._post_eval_js_hook is not None:
            self._post_eval_js_hook()

        # Automatically process events to ensure immediate delivery
        if auto_process:
            self._auto_process_events()

    def emit_batch(
        self,
        events: list,
        auto_process: bool = True,
    ) -> int:
        """Emit multiple events to JavaScript in a single batch.

        This is more efficient than calling emit() multiple times because
        all events are queued together and processed in one go.

        Args:
            events: List of tuples (event_name, data_dict)
            auto_process: Automatically process message queue after emission (default: True).

        Returns:
            Number of events emitted

        Example:
            >>> webview.emit_batch([
            ...     ("update", {"field": "name", "value": "John"}),
            ...     ("update", {"field": "email", "value": "john@example.com"}),
            ...     ("batch_complete", {"count": 2}),
            ... ])
        """
        if not events:
            return 0

        # Use the async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core

        # Convert events to proper format for Rust
        rust_events = []
        for event_name, data in events:
            if data is None:
                data = {}
            elif not isinstance(data, dict):
                data = {"value": data}
            rust_events.append((event_name, data))

        count = core.emit_batch(rust_events)
        logger.debug(f"[OK] [WebView.emit_batch] Emitted {count} events via Rust")

        if auto_process:
            self._auto_process_events()
        return count

    def on(
        self,
        event_name: str,
        handler: Optional[Callable] = None,
    ) -> Union[Callable, ConnectionId]:
        """Register a Python callback for JavaScript events.

        Can be used as a decorator or as a method call:

            # As decorator (returns the function)
            @webview.on("export_scene")
            def handle_export(data):
                print(f"Exporting to: {data['path']}")

            # As method call (returns ConnectionId for disconnect)
            conn_id = webview.on("export_scene", handle_export)
            webview.disconnect(conn_id)

        Args:
            event_name: Name of the event to listen for
            handler: Optional callback function

        Returns:
            If handler is None (decorator mode): returns a decorator
            If handler is provided: returns ConnectionId for disconnection
        """
        if handler is None:
            # Decorator usage
            def decorator(func: Callable) -> Callable:
                self.register_callback(event_name, func)
                return func

            return decorator

        # Direct call - use signal system
        return self.register_callback(event_name, handler)

    def register_callback(self, event_name: str, callback: Callable) -> ConnectionId:
        """Register a callback for an event.

        Args:
            event_name: Name of the event (can be a string or WindowEvent enum)
            callback: Function to call when event occurs

        Returns:
            ConnectionId that can be used to disconnect the callback
        """
        # Convert WindowEvent enum to string if needed
        event_str = str(event_name)

        # Register with legacy event handlers dict (for backward compatibility)
        if event_str not in self._event_handlers:
            self._event_handlers[event_str] = []
        self._event_handlers[event_str].append(callback)

        # Register with signal system
        conn_id = self.signals.custom.connect(event_str, callback)
        logger.debug(f"Registered callback for event: {event_str} (conn_id: {conn_id})")

        # Register with core
        self._core.on(event_str, callback)

        return conn_id

    def disconnect(self, event_name: str, conn_id: ConnectionId) -> bool:
        """Disconnect a callback by its ConnectionId.

        Args:
            event_name: Name of the event
            conn_id: ConnectionId returned by on() or register_callback()

        Returns:
            True if callback was disconnected
        """
        return self.signals.custom.disconnect(event_name, conn_id)

    # =========================================================================
    # Window Event Convenience Methods
    # These methods connect to both the core and the signal system
    # =========================================================================

    def on_loaded(self, callback: Callable) -> Callable:
        """Register a callback for when the page finishes loading.

        Args:
            callback: Function to call when page loads

        Returns:
            The callback function (for decorator use)

        Example:
            >>> @webview.on_loaded
            >>> def handle_loaded(data):
            ...     print("Page loaded!")
        """
        self.register_callback("loaded", callback)
        # Also connect to lifecycle signal (wraps to handle None arg)
        self.signals.page_loaded.connect(lambda: callback({}))
        return callback

    def on_shown(self, callback: Callable) -> Callable:
        """Register a callback for when the window becomes visible."""
        self.register_callback("shown", callback)
        return callback

    def on_closing(self, callback: Callable) -> Callable:
        """Register a callback for before the window closes.

        The callback can return False to prevent the window from closing.

        Example:
            >>> @webview.on_closing
            >>> def handle_closing(data):
            ...     if has_unsaved_changes():
            ...         return False  # Prevent closing
            ...     return True
        """
        self.register_callback("closing", callback)
        self.signals.closing.connect(lambda: callback({}))
        return callback

    def on_closed(self, callback: Callable) -> Callable:
        """Register a callback for after the window has closed."""
        self.register_callback("closed", callback)
        self.signals.closed.connect(lambda: callback({}))
        return callback

    def on_resized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is resized.

        Args:
            callback: Function to call when window is resized.
                     Data includes {width, height}.

        Example:
            >>> @webview.on_resized
            >>> def handle_resize(data):
            ...     print(f"New size: {data['width']}x{data['height']}")
        """
        self.register_callback("resized", callback)
        self.signals.resized.connect(lambda size: callback({"width": size[0], "height": size[1]}))
        return callback

    def on_moved(self, callback: Callable) -> Callable:
        """Register a callback for when the window is moved.

        Args:
            callback: Function to call when window is moved.
                     Data includes {x, y}.
        """
        self.register_callback("moved", callback)
        self.signals.moved.connect(lambda pos: callback({"x": pos[0], "y": pos[1]}))
        return callback

    def on_focused(self, callback: Callable) -> Callable:
        """Register a callback for when the window gains focus."""
        self.register_callback("focused", callback)
        self.signals.focused.connect(lambda: callback({}))
        return callback

    def on_blurred(self, callback: Callable) -> Callable:
        """Register a callback for when the window loses focus."""
        self.register_callback("blurred", callback)
        self.signals.blurred.connect(lambda: callback({}))
        return callback

    def on_minimized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is minimized."""
        self.register_callback("minimized", callback)
        self.signals.minimized.connect(lambda: callback({}))
        return callback

    def on_maximized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is maximized."""
        self.register_callback("maximized", callback)
        self.signals.maximized.connect(lambda: callback({}))
        return callback

    def on_restored(self, callback: Callable) -> Callable:
        """Register a callback for when the window is restored from minimized/maximized state."""
        self.register_callback("restored", callback)
        self.signals.restored.connect(lambda: callback({}))
        return callback
