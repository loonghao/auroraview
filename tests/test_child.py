"""Tests for auroraview.child module.

This module tests the child window support functionality including:
- Mode detection functions
- ChildInfo dataclass
- ParentBridge communication
- ChildContext context manager

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import json
import os
import socket
import threading
import time
from typing import List  # noqa: F401 - used for type hints
from unittest.mock import MagicMock, patch

import pytest

from auroraview.child import (
    ENV_CHILD_ID,
    ENV_EXAMPLE_NAME,
    ENV_PARENT_ID,
    ENV_PARENT_PORT,
    ChildContext,
    ChildInfo,
    ParentBridge,
    get_child_id,
    get_example_name,
    get_parent_id,
    is_child_mode,
    run_example,
)


class TestModeDetection:
    """Tests for mode detection functions."""

    def test_is_child_mode_false_when_no_env(self):
        """Test is_child_mode returns False when env var not set."""
        with patch.dict(os.environ, {}, clear=True):
            # Ensure the env var is not set
            os.environ.pop(ENV_PARENT_ID, None)
            assert is_child_mode() is False

    def test_is_child_mode_true_when_env_set(self):
        """Test is_child_mode returns True when env var is set."""
        with patch.dict(os.environ, {ENV_PARENT_ID: "parent-123"}):
            assert is_child_mode() is True

    def test_get_parent_id_returns_none_when_not_set(self):
        """Test get_parent_id returns None when not in child mode."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_PARENT_ID, None)
            assert get_parent_id() is None

    def test_get_parent_id_returns_value_when_set(self):
        """Test get_parent_id returns the env var value."""
        with patch.dict(os.environ, {ENV_PARENT_ID: "parent-456"}):
            assert get_parent_id() == "parent-456"

    def test_get_child_id_returns_none_when_not_set(self):
        """Test get_child_id returns None when not set."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_CHILD_ID, None)
            assert get_child_id() is None

    def test_get_child_id_returns_value_when_set(self):
        """Test get_child_id returns the env var value."""
        with patch.dict(os.environ, {ENV_CHILD_ID: "child-789"}):
            assert get_child_id() == "child-789"

    def test_get_example_name_returns_none_when_not_set(self):
        """Test get_example_name returns None when not set."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_EXAMPLE_NAME, None)
            assert get_example_name() is None

    def test_get_example_name_returns_value_when_set(self):
        """Test get_example_name returns the env var value."""
        with patch.dict(os.environ, {ENV_EXAMPLE_NAME: "my_demo"}):
            assert get_example_name() == "my_demo"


class TestChildInfo:
    """Tests for ChildInfo dataclass."""

    def test_default_values(self):
        """Test ChildInfo default values."""
        info = ChildInfo()
        assert info.is_child is False
        assert info.parent_id is None
        assert info.child_id is None
        assert info.example_name is None
        assert info.parent_port is None

    def test_from_env_standalone_mode(self):
        """Test ChildInfo.from_env in standalone mode."""
        with patch.dict(os.environ, {}, clear=True):
            # Clear all relevant env vars
            for var in [ENV_PARENT_ID, ENV_PARENT_PORT, ENV_CHILD_ID, ENV_EXAMPLE_NAME]:
                os.environ.pop(var, None)

            info = ChildInfo.from_env()
            assert info.is_child is False
            assert info.parent_id is None
            assert info.parent_port is None

    def test_from_env_child_mode(self):
        """Test ChildInfo.from_env in child mode."""
        env = {
            ENV_PARENT_ID: "parent-abc",
            ENV_PARENT_PORT: "12345",
            ENV_CHILD_ID: "child-xyz",
            ENV_EXAMPLE_NAME: "test_demo",
        }
        with patch.dict(os.environ, env, clear=True):
            info = ChildInfo.from_env()
            assert info.is_child is True
            assert info.parent_id == "parent-abc"
            assert info.parent_port == 12345
            assert info.child_id == "child-xyz"
            assert info.example_name == "test_demo"

    def test_from_env_partial_child_mode(self):
        """Test ChildInfo.from_env with only parent_id set."""
        with patch.dict(os.environ, {ENV_PARENT_ID: "parent-only"}, clear=True):
            info = ChildInfo.from_env()
            assert info.is_child is True
            assert info.parent_id == "parent-only"
            assert info.parent_port is None


class TestParentBridge:
    """Tests for ParentBridge class."""

    def test_init(self):
        """Test ParentBridge initialization."""
        bridge = ParentBridge(port=12345)
        assert bridge._port == 12345
        assert bridge._connected is False
        assert bridge._socket is None

    def test_connect_failure(self):
        """Test ParentBridge connect failure when no server."""
        bridge = ParentBridge(port=59999)  # Unlikely to be in use
        result = bridge.connect()
        assert result is False
        assert bridge._connected is False

    def test_send_when_not_connected(self):
        """Test ParentBridge send returns False when not connected."""
        bridge = ParentBridge(port=12345)
        result = bridge.send("test_event", {"data": "value"})
        assert result is False

    def test_disconnect(self):
        """Test ParentBridge disconnect."""
        bridge = ParentBridge(port=12345)
        bridge._connected = True
        bridge._socket = MagicMock()

        bridge.disconnect()

        assert bridge._connected is False
        assert bridge._socket is None

    def test_on_handler_registration(self):
        """Test ParentBridge event handler registration."""
        bridge = ParentBridge(port=12345)

        handler = MagicMock()
        unsubscribe = bridge.on("test_event", handler)

        assert "test_event" in bridge._handlers
        assert handler in bridge._handlers["test_event"]

        # Test unsubscribe
        unsubscribe()
        assert handler not in bridge._handlers["test_event"]

    def test_handle_message_valid(self):
        """Test ParentBridge message handling."""
        bridge = ParentBridge(port=12345)

        handler = MagicMock()
        bridge.on("test_event", handler)

        message = json.dumps({"event": "test_event", "data": {"key": "value"}})
        bridge._handle_message(message)

        handler.assert_called_once_with({"key": "value"})

    def test_handle_message_invalid_json(self):
        """Test ParentBridge handles invalid JSON gracefully."""
        bridge = ParentBridge(port=12345)

        # Should not raise
        bridge._handle_message("not valid json")

    def test_handle_message_no_event(self):
        """Test ParentBridge handles message without event."""
        bridge = ParentBridge(port=12345)

        handler = MagicMock()
        bridge.on("test_event", handler)

        message = json.dumps({"data": {"key": "value"}})
        bridge._handle_message(message)

        handler.assert_not_called()


class TestParentBridgeWithServer:
    """Tests for ParentBridge with actual socket server."""

    @pytest.fixture
    def mock_server(self):
        """Create a mock server for testing."""
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        port = server.getsockname()[1]

        received_messages: List[str] = []
        client_socket = None
        stop_event = threading.Event()

        def accept_connection():
            nonlocal client_socket
            server.settimeout(1.0)
            try:
                client_socket, _ = server.accept()
                client_socket.settimeout(0.5)
                buffer = ""
                while not stop_event.is_set():
                    try:
                        data = client_socket.recv(4096)
                        if not data:
                            break
                        buffer += data.decode("utf-8")
                        while "\n" in buffer:
                            line, buffer = buffer.split("\n", 1)
                            if line.strip():
                                received_messages.append(line)
                    except socket.timeout:
                        continue
                    except Exception:
                        break
            except socket.timeout:
                pass

        accept_thread = threading.Thread(target=accept_connection, daemon=True)
        accept_thread.start()

        yield {"port": port, "messages": received_messages, "server": server, "stop": stop_event}

        # Cleanup
        stop_event.set()
        if client_socket:
            try:
                client_socket.close()
            except Exception:
                pass
        server.close()
        accept_thread.join(timeout=1.0)

    @pytest.mark.timeout(5)
    def test_connect_success(self, mock_server):
        """Test ParentBridge successful connection."""
        bridge = ParentBridge(port=mock_server["port"])
        result = bridge.connect()

        assert result is True
        assert bridge._connected is True

        bridge.disconnect()

    @pytest.mark.timeout(5)
    def test_send_message(self, mock_server):
        """Test ParentBridge send message."""
        with patch.dict(os.environ, {ENV_CHILD_ID: "test-child"}):
            bridge = ParentBridge(port=mock_server["port"])
            bridge.connect()

            result = bridge.send("hello", {"message": "world"})
            assert result is True

            # Wait for message to be received
            time.sleep(0.2)

            assert len(mock_server["messages"]) > 0
            msg = json.loads(mock_server["messages"][0])
            assert msg["event"] == "hello"
            assert msg["data"]["message"] == "world"
            assert msg["child_id"] == "test-child"

            bridge.disconnect()


class TestChildContext:
    """Tests for ChildContext class."""

    def test_standalone_mode(self):
        """Test ChildContext in standalone mode."""
        with patch.dict(os.environ, {}, clear=True):
            for var in [ENV_PARENT_ID, ENV_PARENT_PORT, ENV_CHILD_ID, ENV_EXAMPLE_NAME]:
                os.environ.pop(var, None)

            with ChildContext() as ctx:
                assert ctx.is_child is False
                assert ctx.parent_id is None
                assert ctx.child_id is None
                assert ctx.bridge is None

    def test_child_mode_properties(self):
        """Test ChildContext properties in child mode."""
        env = {
            ENV_PARENT_ID: "parent-test",
            ENV_CHILD_ID: "child-test",
            ENV_EXAMPLE_NAME: "demo_example",
        }
        with patch.dict(os.environ, env, clear=True):
            ctx = ChildContext()
            assert ctx.is_child is True
            assert ctx.parent_id == "parent-test"
            assert ctx.child_id == "child-test"
            assert ctx.example_name == "demo_example"

    def test_emit_to_parent_standalone(self):
        """Test emit_to_parent in standalone mode returns True."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_PARENT_ID, None)

            ctx = ChildContext()
            result = ctx.emit_to_parent("test", {"data": "value"})
            assert result is True  # No-op success in standalone mode

    def test_on_parent_event_standalone(self):
        """Test on_parent_event in standalone mode returns no-op."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_PARENT_ID, None)

            ctx = ChildContext()
            handler = MagicMock()
            unsubscribe = ctx.on_parent_event("test", handler)

            # Should return a callable
            assert callable(unsubscribe)
            unsubscribe()  # Should not raise

    def test_create_webview_standalone(self):
        """Test create_webview in standalone mode."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_PARENT_ID, None)

            with patch("auroraview.child.WebView") as MockWebView:
                mock_webview = MagicMock()
                MockWebView.return_value = mock_webview

                ctx = ChildContext()
                ctx.create_webview(title="Test", width=800, height=600, html="<h1>Test</h1>")

                MockWebView.assert_called_once()
                call_kwargs = MockWebView.call_args[1]
                assert call_kwargs["title"] == "Test"
                assert call_kwargs["width"] == 800
                assert call_kwargs["height"] == 600

    def test_create_webview_child_mode_title_prefix(self):
        """Test create_webview adds title prefix in child mode."""
        env = {
            ENV_PARENT_ID: "parent-test",
            ENV_EXAMPLE_NAME: "MyDemo",
        }
        with patch.dict(os.environ, env, clear=True):
            with patch("auroraview.child.WebView") as MockWebView:
                mock_webview = MagicMock()
                MockWebView.return_value = mock_webview

                ctx = ChildContext()
                ctx.create_webview(title="Original Title")

                call_kwargs = MockWebView.call_args[1]
                assert call_kwargs["title"] == "MyDemo - Original Title"

    def test_handle_parent_command_close(self):
        """Test _handle_parent_command with close command."""
        ctx = ChildContext()
        mock_webview = MagicMock()
        ctx._webview = mock_webview

        ctx._handle_parent_command({"command": "close"})

        mock_webview.close.assert_called_once()

    def test_handle_parent_command_eval(self):
        """Test _handle_parent_command with eval command."""
        ctx = ChildContext()
        mock_webview = MagicMock()
        ctx._webview = mock_webview

        ctx._handle_parent_command({"command": "eval", "args": {"js": "console.log('test')"}})

        mock_webview.eval.assert_called_once_with("console.log('test')")

    def test_handle_parent_command_emit(self):
        """Test _handle_parent_command with emit command."""
        ctx = ChildContext()
        mock_webview = MagicMock()
        ctx._webview = mock_webview

        ctx._handle_parent_command(
            {"command": "emit", "args": {"event": "test_event", "data": {"key": "value"}}}
        )

        mock_webview.emit.assert_called_once_with("test_event", {"key": "value"})

    def test_handle_parent_command_no_webview(self):
        """Test _handle_parent_command when no webview."""
        ctx = ChildContext()
        ctx._webview = None

        # Should not raise
        ctx._handle_parent_command({"command": "close"})


class TestRunExample:
    """Tests for run_example function."""

    def test_run_example_standalone(self):
        """Test run_example in standalone mode."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop(ENV_PARENT_ID, None)

            mock_webview = MagicMock()
            create_func = MagicMock(return_value=mock_webview)
            on_ready = MagicMock()

            run_example(create_func, on_ready=on_ready)

            create_func.assert_called_once()
            on_ready.assert_called_once()
            mock_webview.show.assert_called_once()

    def test_run_example_child_mode(self):
        """Test run_example in child mode."""
        env = {
            ENV_PARENT_ID: "parent-run",
            ENV_CHILD_ID: "child-run",
        }
        with patch.dict(os.environ, env, clear=True):
            mock_webview = MagicMock()
            create_func = MagicMock(return_value=mock_webview)

            run_example(create_func)

            create_func.assert_called_once()
            # Verify context was passed
            ctx = create_func.call_args[0][0]
            assert ctx.is_child is True
            mock_webview.show.assert_called_once()


class TestEnvConstants:
    """Tests for environment variable constants."""

    def test_env_constants_defined(self):
        """Test that all env constants are defined."""
        assert ENV_PARENT_ID == "AURORAVIEW_PARENT_ID"
        assert ENV_PARENT_PORT == "AURORAVIEW_PARENT_PORT"
        assert ENV_CHILD_ID == "AURORAVIEW_CHILD_ID"
        assert ENV_EXAMPLE_NAME == "AURORAVIEW_EXAMPLE_NAME"
