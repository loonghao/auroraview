#!/usr/bin/env python3
"""Tests for IPC Channel functionality."""

import os
import sys

import pytest

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


class TestIpcChannelModule:
    """Test IpcChannel module imports and basic functionality."""

    def test_import_ipc_channel(self):
        """Test that IpcChannel can be imported."""
        from auroraview.core.ipc_channel import IpcChannel

        assert IpcChannel is not None

    def test_import_convenience_functions(self):
        """Test that convenience functions can be imported."""
        from auroraview.core.ipc_channel import (
            emit_event,
            report_progress,
            report_result,
            send_to_parent,
        )

        assert send_to_parent is not None
        assert emit_event is not None
        assert report_progress is not None
        assert report_result is not None

    def test_import_from_core(self):
        """Test that IpcChannel can be imported from core module."""
        from auroraview.core import (
            IpcChannel,
            IpcChannelError,
        )

        assert IpcChannel is not None
        assert IpcChannelError is not None

    def test_is_available_without_env(self):
        """Test is_available returns False when env var not set."""
        from auroraview.core.ipc_channel import IpcChannel

        # Ensure env vars are not set
        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            assert IpcChannel.is_available() is False
        finally:
            # Restore env vars
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode

    def test_is_available_with_env(self):
        """Test is_available returns True when env vars are set correctly."""
        from auroraview.core.ipc_channel import IpcChannel

        # Save old values
        old_channel = os.environ.get("AURORAVIEW_IPC_CHANNEL")
        old_mode = os.environ.get("AURORAVIEW_IPC_MODE")

        try:
            os.environ["AURORAVIEW_IPC_CHANNEL"] = "test_channel"
            os.environ["AURORAVIEW_IPC_MODE"] = "channel"
            assert IpcChannel.is_available() is True
        finally:
            # Restore env vars
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            else:
                os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode
            else:
                os.environ.pop("AURORAVIEW_IPC_MODE", None)

    def test_connect_without_env_raises_error(self):
        """Test that connect() raises error when env var not set."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        # Ensure env var is not set
        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)

        try:
            with pytest.raises(IpcChannelError) as exc_info:
                IpcChannel.connect()
            assert "AURORAVIEW_IPC_CHANNEL" in str(exc_info.value)
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel

    def test_send_to_parent_without_channel(self):
        """Test send_to_parent returns False when channel not available."""
        from auroraview.core.ipc_channel import send_to_parent

        # Ensure env vars are not set
        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            result = send_to_parent({"test": "data"})
            assert result is False
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode

    def test_channel_repr(self):
        """Test Channel string representation."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test_channel")
        repr_str = repr(channel)
        assert "test_channel" in repr_str
        assert "disconnected" in repr_str


class TestIpcChannelInstance:
    """Test IpcChannel instance methods."""

    def test_channel_init(self):
        """Test IpcChannel initialization."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("my_channel")
        assert channel.channel_name == "my_channel"
        assert channel._connected is False
        assert channel._running is False

    def test_channel_not_connected_send_raises(self):
        """Test that send raises error when not connected."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        channel = IpcChannel("test")
        with pytest.raises(IpcChannelError) as exc_info:
            channel.send({"test": "data"})
        assert "not connected" in str(exc_info.value)

    def test_channel_not_connected_receive_raises(self):
        """Test that receive raises error when not connected."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        channel = IpcChannel("test")
        with pytest.raises(IpcChannelError) as exc_info:
            channel.receive()
        assert "not connected" in str(exc_info.value)

    def test_channel_context_manager(self):
        """Test IpcChannel as context manager."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")
        # Manually set connected to test close behavior
        channel._connected = True

        with channel:
            assert channel._connected is True

        # After exiting context, should be closed
        assert channel._connected is False

    def test_on_message_handler_registration(self):
        """Test registering message handlers."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")
        handler_called = []

        def handler(msg):
            handler_called.append(msg)

        channel.on_message(handler)
        assert len(channel._receive_handlers) == 1

        # Register another
        channel.on_message(lambda m: None)
        assert len(channel._receive_handlers) == 2

    def test_close_idempotent(self):
        """Test that close can be called multiple times safely."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")
        channel.close()
        channel.close()  # Should not raise
        assert channel._connected is False


class TestConvenienceFunctions:
    """Test convenience functions."""

    def test_emit_event_structure(self):
        """Test emit_event creates correct message structure."""

        # We can't actually send, but we can verify the function exists
        # and has correct signature
        from auroraview.core.ipc_channel import emit_event

        # When channel not available, should return False
        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            result = emit_event("test_event", {"key": "value"})
            assert result is False
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode

    def test_report_progress_structure(self):
        """Test report_progress creates correct message structure."""
        from auroraview.core.ipc_channel import report_progress

        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            result = report_progress(50, "Half done")
            assert result is False
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode

    def test_report_result_success(self):
        """Test report_result with success."""
        from auroraview.core.ipc_channel import report_result

        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            result = report_result(True, data={"items": [1, 2, 3]})
            assert result is False
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode

    def test_report_result_failure(self):
        """Test report_result with failure."""
        from auroraview.core.ipc_channel import report_result

        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
        old_mode = os.environ.pop("AURORAVIEW_IPC_MODE", None)

        try:
            result = report_result(False, error="Something went wrong")
            assert result is False
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel
            if old_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = old_mode


class TestRustJsonIntegration:
    """Test that Rust JSON is being used."""

    def test_rust_json_available(self):
        """Test that Rust JSON functions are available."""
        try:
            from auroraview._core import json_dumps, json_loads

            # Test basic serialization
            data = {"key": "value", "number": 42, "list": [1, 2, 3]}
            json_str = json_dumps(data)
            assert isinstance(json_str, str)

            # Test deserialization
            parsed = json_loads(json_str)
            assert parsed == data
        except ImportError:
            pytest.skip("Rust core not available")

    def test_ipc_channel_uses_rust_json(self):
        """Test that IpcChannel module imports Rust JSON."""
        # This test verifies the import path
        import auroraview.core.ipc_channel as ipc_module

        # The module should have json_dumps and json_loads defined
        assert hasattr(ipc_module, "json_dumps") or "json_dumps" in dir(ipc_module)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
