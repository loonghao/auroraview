# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Event System Mixin.

This module provides event handling methods for the WebView class.
"""

from __future__ import annotations

import logging
import threading
import traceback
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional, Union

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)


class WebViewEventMixin:
    """Mixin providing event system methods.

    Provides methods for event handling:
    - emit: Emit an event to JavaScript
    - on: Decorator to register event callback
    - register_callback: Register a callback for an event
    - on_loaded, on_shown, on_closing, on_closed: Lifecycle event decorators
    - on_resized, on_moved, on_focused, on_blurred: Window event decorators
    - on_minimized, on_maximized, on_restored: State event decorators
    """

    # Type hints for attributes from main class
    _core: Any
    _async_core: Optional[Any]
    _async_core_lock: threading.Lock
    _event_handlers: Dict[str, list]
    _post_eval_js_hook: Optional[Callable[[], None]]
    _auto_process_events: Callable[[], None]

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
            event_name: Name of the event (can be a string or WindowEvent enum)
            callback: Function to call when event occurs
        """
        # Convert WindowEvent enum to string if needed
        event_str = str(event_name)

        if event_str not in self._event_handlers:
            self._event_handlers[event_str] = []

        self._event_handlers[event_str].append(callback)
        logger.debug(f"Registered callback for event: {event_str}")

        # Register with core
        self._core.on(event_str, callback)

    # =========================================================================
    # Window Event Convenience Methods
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
        return callback

    def on_closed(self, callback: Callable) -> Callable:
        """Register a callback for after the window has closed."""
        self.register_callback("closed", callback)
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
        return callback

    def on_moved(self, callback: Callable) -> Callable:
        """Register a callback for when the window is moved.

        Args:
            callback: Function to call when window is moved.
                     Data includes {x, y}.
        """
        self.register_callback("moved", callback)
        return callback

    def on_focused(self, callback: Callable) -> Callable:
        """Register a callback for when the window gains focus."""
        self.register_callback("focused", callback)
        return callback

    def on_blurred(self, callback: Callable) -> Callable:
        """Register a callback for when the window loses focus."""
        self.register_callback("blurred", callback)
        return callback

    def on_minimized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is minimized."""
        self.register_callback("minimized", callback)
        return callback

    def on_maximized(self, callback: Callable) -> Callable:
        """Register a callback for when the window is maximized."""
        self.register_callback("maximized", callback)
        return callback

    def on_restored(self, callback: Callable) -> Callable:
        """Register a callback for when the window is restored from minimized/maximized state."""
        self.register_callback("restored", callback)
        return callback

