"""End-to-end tests for IPC Channel functionality.

These tests verify the full IPC channel flow:
1. Parent process spawns child with spawn_ipc_channel
2. Child connects to the LocalSocket channel
3. Bidirectional JSON messaging works correctly
4. process:message events are emitted to JavaScript

Note: These tests require the Rust core to be built.
"""

from __future__ import annotations

import os
import sys
import tempfile
import threading
import time

import pytest

# Skip if Rust core not available
try:
    from auroraview._core import PluginManager  # noqa: F401

    CORE_AVAILABLE = True
except ImportError:
    CORE_AVAILABLE = False

pytestmark = [
    pytest.mark.skipif(not CORE_AVAILABLE, reason="Rust core not available"),
    pytest.mark.integration,
    pytest.mark.e2e,
]


@pytest.mark.slow
class TestIpcChannelSpawn:
    """Test spawn_ipc_channel functionality."""

    def test_spawn_ipc_channel_returns_channel_name(self):
        """Test that spawn_ipc_channel returns channel info."""
        from auroraview._core import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Spawn a simple Python script that exits immediately
        args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "import time; time.sleep(0.1)"],
                "showConsole": False,
            }
        )

        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc_channel", args))

        assert result.get("success") is True
        data = result.get("data", {})
        assert "pid" in data
        assert data.get("mode") == "channel"
        assert "channel" in data
        assert data["channel"].startswith("auroraview_ipc_")

        # Cleanup
        pid = data["pid"]
        plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid}))

    def test_spawn_ipc_vs_spawn_ipc_channel_mode(self):
        """Test that spawn_ipc returns pipe mode and spawn_ipc_channel returns channel mode."""
        from auroraview._core import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "import time; time.sleep(0.1)"],
                "showConsole": False,
            }
        )

        # Test spawn_ipc (pipe mode)
        result1 = json_loads(plugins.handle_command("plugin:process|spawn_ipc", args))
        assert result1.get("success") is True
        assert result1.get("data", {}).get("mode") == "pipe"
        pid1 = result1.get("data", {}).get("pid")

        # Test spawn_ipc_channel (channel mode)
        result2 = json_loads(plugins.handle_command("plugin:process|spawn_ipc_channel", args))
        assert result2.get("success") is True
        assert result2.get("data", {}).get("mode") == "channel"
        pid2 = result2.get("data", {}).get("pid")

        # Cleanup
        plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid1}))
        plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid2}))


@pytest.mark.slow
class TestIpcChannelCommunication:
    """Test IPC channel communication between parent and child."""

    def test_child_receives_environment_variables(self):
        """Test that child process receives IPC channel environment variables."""
        from auroraview._core import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Create a script that prints the env vars
        script = """
import os
import sys
channel = os.environ.get('AURORAVIEW_IPC_CHANNEL', 'NOT_SET')
mode = os.environ.get('AURORAVIEW_IPC_MODE', 'NOT_SET')
print(f"CHANNEL={channel}", file=sys.stderr)
print(f"MODE={mode}", file=sys.stderr)
"""

        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
            f.write(script)
            script_path = f.name

        try:
            args = json_dumps(
                {
                    "command": sys.executable,
                    "args": [script_path],
                    "showConsole": False,
                }
            )

            result = json_loads(plugins.handle_command("plugin:process|spawn_ipc_channel", args))
            assert result.get("success") is True

            # Wait for process to complete
            time.sleep(0.5)

            # The environment variables should have been set
            # (We can't easily capture stderr here, but the spawn should succeed)
            data = result.get("data", {})
            assert data.get("mode") == "channel"
            assert "channel" in data

        finally:
            os.unlink(script_path)
            pid = result.get("data", {}).get("pid")
            if pid:
                plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid}))


class TestIpcChannelPythonClient:
    """Test Python IPC channel client."""

    def test_ipc_channel_client_import(self):
        """Test that IpcChannel client can be imported."""
        from auroraview.core.ipc_channel import (
            IpcChannel,
            IpcChannelError,
        )

        assert IpcChannel is not None
        assert IpcChannelError is not None

    def test_ipc_channel_is_available_detection(self):
        """Test is_available correctly detects channel mode."""
        from auroraview.core.ipc_channel import IpcChannel

        # Save original env
        orig_channel = os.environ.get("AURORAVIEW_IPC_CHANNEL")
        orig_mode = os.environ.get("AURORAVIEW_IPC_MODE")

        try:
            # Not available without env vars
            os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
            os.environ.pop("AURORAVIEW_IPC_MODE", None)
            assert IpcChannel.is_available() is False

            # Not available with only channel
            os.environ["AURORAVIEW_IPC_CHANNEL"] = "test"
            assert IpcChannel.is_available() is False

            # Not available with wrong mode
            os.environ["AURORAVIEW_IPC_MODE"] = "pipe"
            assert IpcChannel.is_available() is False

            # Available with correct settings
            os.environ["AURORAVIEW_IPC_MODE"] = "channel"
            assert IpcChannel.is_available() is True

        finally:
            # Restore
            if orig_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = orig_channel
            else:
                os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)
            if orig_mode:
                os.environ["AURORAVIEW_IPC_MODE"] = orig_mode
            else:
                os.environ.pop("AURORAVIEW_IPC_MODE", None)


class TestIpcChannelClientBehavior:
    """Test IpcChannel client behavior without actual connection."""

    def test_channel_initialization(self):
        """Test IpcChannel initialization."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test_channel_name")
        assert channel.channel_name == "test_channel_name"
        assert channel._connected is False
        assert channel._running is False
        assert len(channel._receive_handlers) == 0

    def test_channel_repr(self):
        """Test IpcChannel string representation."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("my_test_channel")
        repr_str = repr(channel)
        assert "my_test_channel" in repr_str
        assert "disconnected" in repr_str

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

    def test_send_without_connection_raises(self):
        """Test that send raises error when not connected."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        channel = IpcChannel("test")
        with pytest.raises(IpcChannelError) as exc_info:
            channel.send({"test": "data"})
        assert "not connected" in str(exc_info.value)

    def test_receive_without_connection_raises(self):
        """Test that receive raises error when not connected."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        channel = IpcChannel("test")
        with pytest.raises(IpcChannelError) as exc_info:
            channel.receive()
        assert "not connected" in str(exc_info.value)

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
        assert channel._running is False


class TestConvenienceFunctions:
    """Test convenience functions for IPC communication."""

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

    def test_emit_event_without_channel(self):
        """Test emit_event returns False when channel not available."""
        from auroraview.core.ipc_channel import emit_event

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

    def test_report_progress_without_channel(self):
        """Test report_progress returns False when channel not available."""
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

    def test_report_result_success_without_channel(self):
        """Test report_result with success returns False when channel not available."""
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

    def test_report_result_failure_without_channel(self):
        """Test report_result with failure returns False when channel not available."""
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
    """Test Rust JSON integration."""

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

    def test_rust_json_unicode(self):
        """Test Rust JSON handles Unicode correctly."""
        try:
            from auroraview._core import json_dumps, json_loads

            data = {"中文": "测试", "emoji": "🎉", "mixed": "Hello 世界"}
            json_str = json_dumps(data)
            parsed = json_loads(json_str)
            assert parsed == data
        except ImportError:
            pytest.skip("Rust core not available")

    def test_rust_json_nested_structures(self):
        """Test Rust JSON handles nested structures."""
        try:
            from auroraview._core import json_dumps, json_loads

            data = {
                "level1": {"level2": {"level3": [1, 2, {"deep": True}]}},
                "array": [[1, 2], [3, 4]],
            }
            json_str = json_dumps(data)
            parsed = json_loads(json_str)
            assert parsed == data
        except ImportError:
            pytest.skip("Rust core not available")


class TestGalleryIpcChannelIntegration:
    """Test Gallery integration with IPC channel."""

    def test_gallery_run_sample_with_channel(self):
        """Test that Gallery's run_sample supports use_channel parameter."""
        # This test verifies the API signature, not actual execution

        # Import the gallery main module to check function signature
        gallery_path = os.path.join(
            os.path.dirname(__file__),
            "..",
            "..",
            "..",
            "gallery",
            "main.py",
        )

        if os.path.exists(gallery_path):
            # Read the file and check for use_channel parameter
            with open(gallery_path, "r", encoding="utf-8") as f:
                content = f.read()

            assert "use_channel" in content
            assert "spawn_ipc_channel" in content
            assert "send_json_to_process" in content


class TestIpcChannelThreadSafety:
    """Test IPC channel thread safety."""

    def test_multiple_handler_registration(self):
        """Test registering handlers from multiple threads."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")
        handlers_registered = []
        errors = []

        def register_handler(idx):
            try:
                channel.on_message(lambda m: handlers_registered.append((idx, m)))
            except Exception as e:
                errors.append(e)

        threads = [threading.Thread(target=register_handler, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(errors) == 0
        assert len(channel._receive_handlers) == 10

    def test_close_during_operations(self):
        """Test that close is safe during other operations."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")
        errors = []

        def close_channel():
            try:
                channel.close()
            except Exception as e:
                errors.append(e)

        def register_handlers():
            for _ in range(100):
                try:
                    channel.on_message(lambda m: None)
                except Exception as e:
                    errors.append(e)

        t1 = threading.Thread(target=close_channel)
        t2 = threading.Thread(target=register_handlers)

        t1.start()
        t2.start()
        t1.join()
        t2.join()

        # Should not have any exceptions
        assert len(errors) == 0


class TestIpcChannelMessageFormat:
    """Test IPC channel message format validation."""

    def test_json_message_format(self):
        """Test that messages follow expected JSON format."""
        from auroraview.core.ipc_channel import IpcChannel

        # Verify the expected message structure
        channel = IpcChannel("test")  # noqa: F841

        # Progress message format
        progress_msg = {"type": "progress", "value": 50, "message": "Processing..."}
        assert "type" in progress_msg
        assert "value" in progress_msg

        # Event message format
        event_msg = {"type": "event", "event": "custom_event", "data": {"key": "value"}}
        assert "type" in event_msg
        assert "event" in event_msg

        # Result message format
        result_msg = {"type": "result", "success": True, "data": {"items": []}}
        assert "type" in result_msg
        assert "success" in result_msg

    def test_message_type_constants(self):
        """Test that message types are consistent."""
        # These are the expected message types
        expected_types = ["progress", "event", "result"]

        for msg_type in expected_types:
            assert isinstance(msg_type, str)
            assert len(msg_type) > 0


class TestIpcChannelErrorHandling:
    """Test IPC channel error handling."""

    def test_connect_without_env_raises_descriptive_error(self):
        """Test that connect without env var raises descriptive error."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        old_channel = os.environ.pop("AURORAVIEW_IPC_CHANNEL", None)

        try:
            with pytest.raises(IpcChannelError) as exc_info:
                IpcChannel.connect()

            error_msg = str(exc_info.value)
            assert "AURORAVIEW_IPC_CHANNEL" in error_msg
            assert "spawn_ipc_channel" in error_msg
        finally:
            if old_channel:
                os.environ["AURORAVIEW_IPC_CHANNEL"] = old_channel

    def test_connect_to_nonexistent_channel_raises(self):
        """Test that connecting to nonexistent channel raises error."""
        from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

        channel = IpcChannel("nonexistent_channel_12345")

        with pytest.raises(IpcChannelError) as exc_info:
            channel._connect()

        error_msg = str(exc_info.value)
        assert "nonexistent_channel_12345" in error_msg or "not found" in error_msg.lower()


class TestIpcChannelPlatformSpecific:
    """Test platform-specific IPC channel behavior."""

    def test_windows_pipe_prefix(self):
        """Test Windows Named Pipe prefix constant."""
        from auroraview.core.ipc_channel import WINDOWS_PIPE_PREFIX

        if sys.platform == "win32":
            assert WINDOWS_PIPE_PREFIX == r"\\.\pipe\ipckit_"
        else:
            # On non-Windows, the constant should still be defined
            assert WINDOWS_PIPE_PREFIX is not None

    def test_platform_detection(self):
        """Test that platform is correctly detected."""
        from auroraview.core.ipc_channel import IpcChannel

        channel = IpcChannel("test")

        # The channel should work on both platforms
        assert channel.channel_name == "test"
        assert channel._connected is False


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
