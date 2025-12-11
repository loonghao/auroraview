"""Window lifecycle events for AuroraView.

This module defines standard window events that can be listened to using
the WebView.on() decorator or register_callback() method.

Example:
    >>> from auroraview import WebView
    >>> from auroraview.core.events import WindowEvent
    >>>
    >>> webview = WebView.create("My App")
    >>>
    >>> @webview.on(WindowEvent.LOADED)
    >>> def on_loaded(data):
    ...     print("Page loaded!")
    >>>
    >>> @webview.on(WindowEvent.CLOSING)
    >>> def on_closing(data):
    ...     # Return False to prevent closing
    ...     return True
"""

from enum import Enum
from typing import Any, Callable, Dict, Optional


class WindowEvent(str, Enum):
    """Standard window lifecycle events.

    These events are emitted by the WebView at various points in its lifecycle.
    Use webview.on(event_name) to register handlers.
    """

    # Page lifecycle events
    LOADED = "loaded"
    """Emitted when the page has finished loading (DOMContentLoaded)."""

    LOAD_STARTED = "load_started"
    """Emitted when navigation to a new page begins."""

    LOAD_FINISHED = "load_finished"
    """Emitted when the page has completely finished loading (including resources)."""

    # Window visibility events
    SHOWN = "shown"
    """Emitted when the window becomes visible."""

    HIDDEN = "hidden"
    """Emitted when the window is hidden."""

    # Window close events
    CLOSING = "closing"
    """Emitted before the window closes. Return False to prevent closing."""

    CLOSED = "closed"
    """Emitted after the window has closed."""

    # Window state events
    FOCUSED = "focused"
    """Emitted when the window gains focus."""

    BLURRED = "blurred"
    """Emitted when the window loses focus."""

    MINIMIZED = "minimized"
    """Emitted when the window is minimized."""

    MAXIMIZED = "maximized"
    """Emitted when the window is maximized."""

    RESTORED = "restored"
    """Emitted when the window is restored from minimized/maximized state."""

    # Window geometry events
    RESIZED = "resized"
    """Emitted when the window is resized. Data includes {width, height}."""

    MOVED = "moved"
    """Emitted when the window is moved. Data includes {x, y}."""

    # Navigation events
    NAVIGATION_STARTED = "navigation_started"
    """Emitted when navigation begins. Data includes {url}."""

    NAVIGATION_FINISHED = "navigation_finished"
    """Emitted when navigation completes. Data includes {url}."""

    # File drop events (handled natively by Rust/wry for full path access)
    FILE_DROP = "file_drop"
    """Emitted when files are dropped onto the window.

    Data includes:
        - paths: List of full file paths (native access, not limited by browser security)
        - position: Drop position {x, y}
        - timestamp: Unix timestamp in milliseconds

    Example:
        @webview.on(WindowEvent.FILE_DROP)
        def on_file_drop(data):
            for path in data['paths']:
                print(f"Dropped: {path}")
    """

    FILE_DROP_HOVER = "file_drop_hover"
    """Emitted when files are dragged over the window.

    Data includes:
        - hovering: True when files enter the window
        - paths: List of full file paths being dragged
        - position: Current drag position {x, y}
    """

    FILE_DROP_CANCELLED = "file_drop_cancelled"
    """Emitted when drag operation is cancelled or leaves the window.

    Data includes:
        - hovering: False
        - reason: 'left_window' when drag leaves the window area
    """

    FILE_PASTE = "file_paste"
    """Emitted when files are pasted from clipboard.

    Note: Unlike drag-drop, clipboard paste cannot provide full file paths
    due to browser security restrictions.

    Data includes:
        - files: List of file info dicts {name, size, type, lastModified}
        - timestamp: Unix timestamp in milliseconds
    """

    def __str__(self) -> str:
        """Return the event name string."""
        return self.value


class WindowEventData:
    """Data structure for window events.

    Provides typed access to event data with sensible defaults.
    """

    def __init__(self, data: Optional[Dict[str, Any]] = None):
        """Initialize event data.

        Args:
            data: Raw event data dictionary
        """
        self._data = data or {}

    @property
    def url(self) -> Optional[str]:
        """URL for navigation events."""
        return self._data.get("url")

    @property
    def width(self) -> Optional[int]:
        """Window width for resize events."""
        return self._data.get("width")

    @property
    def height(self) -> Optional[int]:
        """Window height for resize events."""
        return self._data.get("height")

    @property
    def x(self) -> Optional[int]:
        """Window X position for move events."""
        return self._data.get("x")

    @property
    def y(self) -> Optional[int]:
        """Window Y position for move events."""
        return self._data.get("y")

    @property
    def focused(self) -> Optional[bool]:
        """Focus state for focus events."""
        return self._data.get("focused")

    @property
    def files(self) -> Optional[list]:
        """List of file info dicts for file paste events.

        Each file info contains: name, size, type, lastModified
        Note: Only available for FILE_PASTE events (clipboard).
        """
        return self._data.get("files")

    @property
    def paths(self) -> Optional[list]:
        """List of full file paths for file drop events.

        These are native file system paths (e.g., 'C:\\Users\\...\\file.txt').
        Available for FILE_DROP and FILE_DROP_HOVER events.
        """
        return self._data.get("paths")

    @property
    def position(self) -> Optional[Dict[str, int]]:
        """Drop/drag position for file drop events.

        Contains: x, y (relative to WebView top-left corner)
        """
        return self._data.get("position")

    @property
    def hovering(self) -> Optional[bool]:
        """Hover state for file drop hover events."""
        return self._data.get("hovering")

    @property
    def reason(self) -> Optional[str]:
        """Reason for file drop cancelled events."""
        return self._data.get("reason")

    @property
    def timestamp(self) -> Optional[int]:
        """Timestamp for events that include it."""
        return self._data.get("timestamp")

    def get(self, key: str, default: Any = None) -> Any:
        """Get a value from the event data.

        Args:
            key: Key to look up
            default: Default value if key not found

        Returns:
            Value or default
        """
        return self._data.get(key, default)

    def __getitem__(self, key: str) -> Any:
        """Get a value from the event data."""
        return self._data[key]

    def __contains__(self, key: str) -> bool:
        """Check if key exists in event data."""
        return key in self._data

    def __repr__(self) -> str:
        """Return string representation."""
        return f"WindowEventData({self._data})"


# Convenience type alias
EventHandler = Callable[[Dict[str, Any]], Any]
