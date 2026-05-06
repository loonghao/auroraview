# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Channels Mixin - streaming data channels."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from ..channel import Channel, ChannelManager


class WebViewChannelsMixin:
    """Mixin for streaming channel manager."""

    # Instance variables to be set by WebView.__init__
    _channels: Optional["ChannelManager"]

    @property
    def channels(self) -> "ChannelManager":
        """Get the channel manager for streaming data.

        Returns:
            ChannelManager instance

        Example:
            >>> channel = webview.channels.create()
            >>> channel.send({"progress": 50})
            >>> channel.send({"progress": 100})
            >>> channel.close()
        """
        if self._channels is None:
            from ..channel import ChannelManager

            self._channels = ChannelManager(self)
        return self._channels

    def create_channel(self, channel_id: Optional[str] = None) -> "Channel":
        """Create a new streaming channel.

        This is a convenience shortcut for `webview.channels.create()`.

        Args:
            channel_id: Optional custom channel ID

        Returns:
            New Channel instance

        Example:
            >>> with webview.create_channel() as channel:
            ...     for chunk in read_large_file():
            ...         channel.send(chunk)
        """
        return self.channels.create(channel_id)
