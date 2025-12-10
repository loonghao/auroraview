# -*- coding: utf-8 -*-
"""Shared State System for Python <-> JavaScript synchronization.

This module provides a reactive state system inspired by PyWebView's state mechanism,
allowing automatic synchronization between Python and JavaScript.

Example:
    >>> from auroraview import WebView
    >>>
    >>> webview = WebView(title="State Demo")
    >>>
    >>> # Set state from Python - auto-syncs to JS
    >>> webview.state["user"] = {"name": "Alice", "age": 30}
    >>> webview.state["theme"] = "dark"
    >>>
    >>> # Subscribe to state changes
    >>> @webview.state.on_change
    >>> def handle_change(key, value, source):
    ...     print(f"State changed: {key} = {value} (from {source})")
    >>>
    >>> # In JavaScript:
    >>> # window.auroraview.state.user  // {"name": "Alice", "age": 30}
    >>> # window.auroraview.state.theme = "light"  // Syncs back to Python
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)

# Type alias for state change handlers
StateChangeHandler = Callable[[str, Any, str], None]


class State:
    """Reactive shared state container for Python ↔ JavaScript sync.

    This class provides a dict-like interface for managing shared state between
    Python and JavaScript. Changes made on either side are automatically
    synchronized to the other.

    Attributes:
        _data: Internal state storage
        _webview: Associated WebView instance
        _handlers: List of change handlers
        _sync_enabled: Whether sync is enabled
    """

    def __init__(self, webview: Optional[WebView] = None):
        """Initialize the State container.

        Args:
            webview: Associated WebView instance for synchronization
        """
        self._data: Dict[str, Any] = {}
        self._webview: Optional[WebView] = webview
        self._handlers: List[StateChangeHandler] = []
        self._sync_enabled: bool = True

    def _attach_webview(self, webview: WebView) -> None:
        """Attach a WebView instance for synchronization.

        Args:
            webview: WebView instance to attach
        """
        self._webview = webview
        # Register handler for JS state changes
        webview.register_callback("__state_update__", self._handle_js_update)
        # Sync initial state to JS
        self._sync_to_js()

    def _handle_js_update(self, data: Dict[str, Any]) -> None:
        """Handle state update from JavaScript.

        Args:
            data: Update data with 'key' and 'value' fields
        """
        if not isinstance(data, dict):
            return

        key = data.get("key")
        value = data.get("value")

        if key is None:
            return

        # Update internal state without triggering JS sync
        self._sync_enabled = False
        try:
            self._data[key] = value
            self._notify_handlers(key, value, "javascript")
        finally:
            self._sync_enabled = True

    def _sync_to_js(self, key: Optional[str] = None) -> None:
        """Synchronize state to JavaScript.

        Args:
            key: Specific key to sync, or None for full sync
        """
        if not self._webview or not self._sync_enabled:
            return

        if key is not None:
            # Sync single key
            self._webview.emit(
                "__state_sync__", {"type": "set", "key": key, "value": self._data.get(key)}
            )
        else:
            # Full sync
            self._webview.emit("__state_sync__", {"type": "full", "data": self._data})

    def _notify_handlers(self, key: str, value: Any, source: str) -> None:
        """Notify all registered change handlers.

        Args:
            key: Changed key
            value: New value
            source: Change source ('python' or 'javascript')
        """
        for handler in self._handlers:
            try:
                handler(key, value, source)
            except Exception as e:
                logger.error(f"State change handler error: {e}")

    def on_change(self, handler: StateChangeHandler) -> StateChangeHandler:
        """Register a state change handler (decorator).

        Args:
            handler: Function to call on state changes.
                    Signature: (key: str, value: Any, source: str) -> None

        Returns:
            The handler function (for decorator use)

        Example:
            >>> @webview.state.on_change
            >>> def handle_change(key, value, source):
            ...     print(f"{key} changed to {value} from {source}")
        """
        self._handlers.append(handler)
        return handler

    def off_change(self, handler: StateChangeHandler) -> None:
        """Remove a state change handler.

        Args:
            handler: Handler to remove
        """
        if handler in self._handlers:
            self._handlers.remove(handler)

    # Dict-like interface
    def __getitem__(self, key: str) -> Any:
        """Get a state value by key."""
        return self._data[key]

    def __setitem__(self, key: str, value: Any) -> None:
        """Set a state value by key (auto-syncs to JS)."""
        self._data[key] = value
        self._sync_to_js(key)
        self._notify_handlers(key, value, "python")

    def __delitem__(self, key: str) -> None:
        """Delete a state key (auto-syncs to JS)."""
        if key in self._data:
            del self._data[key]
            if self._webview and self._sync_enabled:
                self._webview.emit("__state_sync__", {"type": "delete", "key": key})
            self._notify_handlers(key, None, "python")

    def __contains__(self, key: str) -> bool:
        """Check if a key exists in state."""
        return key in self._data

    def __iter__(self):
        """Iterate over state keys."""
        return iter(self._data)

    def __len__(self) -> int:
        """Return number of state keys."""
        return len(self._data)

    def get(self, key: str, default: Any = None) -> Any:
        """Get a state value with default."""
        return self._data.get(key, default)

    def keys(self):
        """Return state keys."""
        return self._data.keys()

    def values(self):
        """Return state values."""
        return self._data.values()

    def items(self):
        """Return state items."""
        return self._data.items()

    def update(self, data: Dict[str, Any], notify: bool = True) -> None:
        """Update multiple state values at once.

        This is the recommended way to update multiple state values as it
        reduces IPC overhead by batching all updates into a single message.

        Args:
            data: Dictionary of key-value pairs to update
            notify: Whether to notify change handlers (default: True)
        """
        for key, value in data.items():
            self._data[key] = value
            if notify:
                self._notify_handlers(key, value, "python")
        # Batch sync to JS - single IPC call for all updates
        if self._webview and self._sync_enabled:
            self._webview.emit("__state_sync__", {"type": "batch", "data": data})

    def batch_update(self) -> "BatchUpdate":
        """Create a batch update context for efficient state updates.

        Use this context manager when you need to make many state updates
        and want to minimize IPC overhead. All updates within the context
        are collected and sent as a single batch when the context exits.

        Example:
            >>> with webview.state.batch_update() as batch:
            ...     batch["user"] = {"name": "Alice"}
            ...     batch["theme"] = "dark"
            ...     batch["settings"] = {"volume": 80}
            >>> # All updates sent in one IPC call

        Returns:
            BatchUpdate context manager
        """
        return BatchUpdate(self)

    def clear(self) -> None:
        """Clear all state."""
        keys = list(self._data.keys())
        self._data.clear()
        if self._webview and self._sync_enabled:
            self._webview.emit("__state_sync__", {"type": "clear"})
        for key in keys:
            self._notify_handlers(key, None, "python")

    def to_dict(self) -> Dict[str, Any]:
        """Return a copy of the state as a dictionary."""
        return dict(self._data)

    def __repr__(self) -> str:
        """String representation of state."""
        return f"State({self._data})"


class BatchUpdate:
    """Context manager for batching multiple state updates.

    Collects all state updates and sends them as a single IPC message
    when the context exits. This significantly reduces IPC overhead
    when making many state changes.

    Example:
        >>> with webview.state.batch_update() as batch:
        ...     batch["user"] = {"name": "Alice"}
        ...     batch["theme"] = "dark"
        ...     batch["count"] = 42
        >>> # All 3 updates sent in one IPC call
    """

    def __init__(self, state: State):
        """Initialize batch update context.

        Args:
            state: Parent State instance
        """
        self._state = state
        self._updates: Dict[str, Any] = {}

    def __enter__(self) -> "BatchUpdate":
        """Enter the batch update context."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit the batch update context and send all updates."""
        if self._updates:
            self._state.update(self._updates)
        return None

    def __setitem__(self, key: str, value: Any) -> None:
        """Queue a state update for batching."""
        self._updates[key] = value

    def __getitem__(self, key: str) -> Any:
        """Get a value (from pending updates or current state)."""
        if key in self._updates:
            return self._updates[key]
        return self._state[key]

    def set(self, key: str, value: Any) -> "BatchUpdate":
        """Queue a state update (chainable).

        Args:
            key: State key
            value: New value

        Returns:
            Self for method chaining
        """
        self._updates[key] = value
        return self

    def get_pending(self) -> Dict[str, Any]:
        """Get all pending updates.

        Returns:
            Dictionary of pending updates
        """
        return dict(self._updates)
