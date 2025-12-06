"""Channel System for streaming data between Python and JavaScript.

This module provides a streaming channel system inspired by Tauri's Channel API,
allowing efficient transfer of large data in chunks.

Example:
    >>> from auroraview import WebView
    >>>
    >>> webview = WebView(title="Channel Demo")
    >>>
    >>> # Create a channel for streaming data
    >>> @webview.command
    >>> async def stream_file(path: str, channel: Channel):
    ...     with open(path, 'rb') as f:
    ...         while chunk := f.read(8192):
    ...             await channel.send(chunk)
    ...     channel.close()
    >>>
    >>> # In JavaScript:
    >>> # const channel = await auroraview.invoke("stream_file", {path: "/data.bin"});
    >>> # channel.onMessage((chunk) => console.log("Received chunk:", chunk.length));
    >>> # channel.onClose(() => console.log("Stream complete"));
"""

from __future__ import annotations

import logging
import uuid
from typing import TYPE_CHECKING, Callable, Dict, Generic, List, Optional, TypeVar

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)

T = TypeVar("T")


class Channel(Generic[T]):
    """Streaming channel for sending data chunks to JavaScript.

    This class provides a unidirectional channel for streaming data from
    Python to JavaScript. Useful for large file transfers, real-time data,
    or progress updates.

    Attributes:
        id: Unique channel identifier
        _webview: Associated WebView instance
        _closed: Whether the channel is closed
        _on_close_handlers: Handlers called when channel closes
    """

    def __init__(self, webview: Optional[WebView] = None, channel_id: Optional[str] = None):
        """Initialize a Channel.

        Args:
            webview: Associated WebView instance
            channel_id: Optional custom channel ID
        """
        self.id: str = channel_id or f"channel_{uuid.uuid4().hex[:8]}"
        self._webview: Optional[WebView] = webview
        self._closed: bool = False
        self._on_close_handlers: List[Callable[[], None]] = []
        self._buffer: List[T] = []

    def send(self, data: T) -> bool:
        """Send data through the channel.

        Args:
            data: Data to send (will be JSON serialized)

        Returns:
            True if sent successfully, False if channel is closed
        """
        if self._closed:
            logger.warning(f"Cannot send on closed channel: {self.id}")
            return False

        if self._webview:
            self._webview.emit("__channel_message__", {"channel_id": self.id, "data": data})
        else:
            # Buffer if no webview attached
            self._buffer.append(data)

        return True

    async def send_async(self, data: T) -> bool:
        """Async version of send.

        Args:
            data: Data to send

        Returns:
            True if sent successfully
        """
        return self.send(data)

    def close(self) -> None:
        """Close the channel.

        Notifies JavaScript that no more data will be sent.
        """
        if self._closed:
            return

        self._closed = True

        if self._webview:
            self._webview.emit("__channel_close__", {"channel_id": self.id})

        # Call close handlers
        for handler in self._on_close_handlers:
            try:
                handler()
            except Exception as e:
                logger.error(f"Channel close handler error: {e}")

    def on_close(self, handler: Callable[[], None]) -> Callable[[], None]:
        """Register a close handler.

        Args:
            handler: Function to call when channel closes

        Returns:
            The handler function
        """
        self._on_close_handlers.append(handler)
        return handler

    @property
    def is_closed(self) -> bool:
        """Check if channel is closed."""
        return self._closed

    def _attach_webview(self, webview: WebView) -> None:
        """Attach a WebView and flush buffered data.

        Args:
            webview: WebView instance to attach
        """
        self._webview = webview

        # Notify JS about new channel
        webview.emit("__channel_open__", {"channel_id": self.id})

        # Flush buffer
        for data in self._buffer:
            self.send(data)
        self._buffer.clear()

    def __enter__(self) -> "Channel[T]":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit - auto-close channel."""
        self.close()

    def __repr__(self) -> str:
        """String representation."""
        status = "closed" if self._closed else "open"
        return f"Channel({self.id}, {status})"


class ChannelManager:
    """Manager for multiple streaming channels.

    This class manages the lifecycle of multiple channels and provides
    factory methods for creating new channels.

    Attributes:
        _channels: Dictionary of active channels
        _webview: Associated WebView instance
    """

    def __init__(self, webview: Optional[WebView] = None):
        """Initialize the ChannelManager.

        Args:
            webview: Associated WebView instance
        """
        self._channels: Dict[str, Channel] = {}
        self._webview: Optional[WebView] = webview

    def _attach_webview(self, webview: WebView) -> None:
        """Attach a WebView instance.

        Args:
            webview: WebView instance to attach
        """
        self._webview = webview
        # Attach to existing channels
        for channel in self._channels.values():
            channel._attach_webview(webview)

    def create(self, channel_id: Optional[str] = None) -> Channel:
        """Create a new channel.

        Args:
            channel_id: Optional custom channel ID

        Returns:
            New Channel instance
        """
        channel = Channel(self._webview, channel_id)
        self._channels[channel.id] = channel

        # Auto-remove on close
        @channel.on_close
        def remove_channel():
            if channel.id in self._channels:
                del self._channels[channel.id]

        return channel

    def get(self, channel_id: str) -> Optional[Channel]:
        """Get a channel by ID.

        Args:
            channel_id: Channel ID to look up

        Returns:
            Channel instance or None if not found
        """
        return self._channels.get(channel_id)

    def close_all(self) -> None:
        """Close all active channels."""
        for channel in list(self._channels.values()):
            channel.close()

    @property
    def active_count(self) -> int:
        """Return number of active channels."""
        return len(self._channels)

    def __len__(self) -> int:
        """Return number of channels."""
        return len(self._channels)

    def __contains__(self, channel_id: str) -> bool:
        """Check if channel exists."""
        return channel_id in self._channels

    def __repr__(self) -> str:
        """String representation."""
        return f"ChannelManager({len(self._channels)} channels)"
