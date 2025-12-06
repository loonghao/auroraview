"""Unit tests for the Channel and ChannelManager classes."""

from __future__ import annotations

from auroraview.core.channel import Channel, ChannelManager


class TestChannelBasic:
    """Test basic Channel functionality."""

    def test_init_default(self):
        """Test Channel initialization with defaults."""
        channel = Channel()
        assert channel.id.startswith("channel_")
        assert not channel.is_closed

    def test_init_custom_id(self):
        """Test Channel initialization with custom ID."""
        channel = Channel(channel_id="my_channel")
        assert channel.id == "my_channel"

    def test_send_without_webview(self):
        """Test sending data without webview (buffers)."""
        channel = Channel()
        result = channel.send({"data": "test"})
        assert result is True
        assert len(channel._buffer) == 1

    def test_send_on_closed_channel(self):
        """Test sending on closed channel fails."""
        channel = Channel()
        channel.close()
        result = channel.send({"data": "test"})
        assert result is False

    def test_close(self):
        """Test closing a channel."""
        channel = Channel()
        assert not channel.is_closed
        channel.close()
        assert channel.is_closed

    def test_close_idempotent(self):
        """Test closing multiple times is safe."""
        channel = Channel()
        channel.close()
        channel.close()  # Should not raise
        assert channel.is_closed

    def test_on_close_handler(self):
        """Test close handler is called."""
        channel = Channel()
        closed = []

        @channel.on_close
        def handler():
            closed.append(True)

        channel.close()
        assert len(closed) == 1

    def test_context_manager(self):
        """Test channel as context manager."""
        with Channel() as channel:
            channel.send({"data": 1})
            assert not channel.is_closed
        assert channel.is_closed

    def test_repr(self):
        """Test string representation."""
        channel = Channel(channel_id="test_ch")
        assert "test_ch" in repr(channel)
        assert "open" in repr(channel)

        channel.close()
        assert "closed" in repr(channel)


class TestChannelManager:
    """Test ChannelManager functionality."""

    def test_init(self):
        """Test ChannelManager initialization."""
        manager = ChannelManager()
        assert len(manager) == 0
        assert manager.active_count == 0

    def test_create_channel(self):
        """Test creating a channel."""
        manager = ChannelManager()
        channel = manager.create()

        assert len(manager) == 1
        assert channel.id in manager

    def test_create_with_custom_id(self):
        """Test creating channel with custom ID."""
        manager = ChannelManager()
        channel = manager.create("my_channel")

        assert channel.id == "my_channel"
        assert "my_channel" in manager

    def test_get_channel(self):
        """Test getting a channel by ID."""
        manager = ChannelManager()
        channel = manager.create("test")

        retrieved = manager.get("test")
        assert retrieved is channel

        assert manager.get("nonexistent") is None

    def test_close_all(self):
        """Test closing all channels."""
        manager = ChannelManager()
        ch1 = manager.create()
        ch2 = manager.create()

        manager.close_all()

        assert ch1.is_closed
        assert ch2.is_closed
        assert len(manager) == 0

    def test_auto_remove_on_close(self):
        """Test channel is removed when closed."""
        manager = ChannelManager()
        channel = manager.create("auto_remove")

        assert "auto_remove" in manager
        channel.close()
        assert "auto_remove" not in manager

    def test_contains(self):
        """Test 'in' operator."""
        manager = ChannelManager()
        manager.create("exists")

        assert "exists" in manager
        assert "missing" not in manager

    def test_repr(self):
        """Test string representation."""
        manager = ChannelManager()
        manager.create()
        manager.create()

        assert "ChannelManager" in repr(manager)
        assert "2" in repr(manager)
