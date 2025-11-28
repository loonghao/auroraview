"""Tests for auroraview.bridge module."""

from unittest.mock import MagicMock


class TestBridgeCreation:
    """Tests for Bridge class creation and initialization."""

    def test_bridge_import(self):
        """Test that Bridge can be imported."""
        from auroraview import Bridge

        assert Bridge is not None

    def test_bridge_creation_default(self):
        """Test Bridge creation with default parameters."""
        from auroraview import Bridge

        bridge = Bridge()
        assert bridge.host == "localhost"
        assert bridge.port == 9001
        assert bridge.protocol == "json"
        assert bridge.is_running is False
        assert bridge.client_count == 0

    def test_bridge_creation_custom_port(self):
        """Test Bridge creation with custom port."""
        from auroraview import Bridge

        bridge = Bridge(port=8888)
        assert bridge.port == 8888

    def test_bridge_creation_custom_host(self):
        """Test Bridge creation with custom host."""
        from auroraview import Bridge

        bridge = Bridge(host="0.0.0.0", port=9002)
        assert bridge.host == "0.0.0.0"
        assert bridge.port == 9002

    def test_bridge_repr(self):
        """Test Bridge string representation."""
        from auroraview import Bridge

        bridge = Bridge(port=9003)
        repr_str = repr(bridge)
        assert "Bridge" in repr_str
        assert "9003" in repr_str
        assert "stopped" in repr_str

    def test_bridge_clients_property(self):
        """Test Bridge clients property."""
        from auroraview import Bridge

        bridge = Bridge(port=9004)
        assert isinstance(bridge.clients, set)
        assert len(bridge.clients) == 0


class TestBridgeHandlers:
    """Tests for Bridge handler registration."""

    def test_register_handler(self):
        """Test registering a message handler."""
        from auroraview import Bridge

        bridge = Bridge(port=9005)

        async def my_handler(data, client):
            return {"status": "ok"}

        bridge.register_handler("test_action", my_handler)
        assert "test_action" in bridge._handlers

    def test_on_decorator(self):
        """Test @bridge.on() decorator."""
        from auroraview import Bridge

        bridge = Bridge(port=9006)

        @bridge.on("layer_created")
        async def handle_layer(data, client):
            return {"status": "ok"}

        assert "layer_created" in bridge._handlers
        assert bridge._handlers["layer_created"] == handle_layer

    def test_multiple_handlers(self):
        """Test registering multiple handlers."""
        from auroraview import Bridge

        bridge = Bridge(port=9007)

        @bridge.on("action1")
        async def handler1(data, client):
            pass

        @bridge.on("action2")
        async def handler2(data, client):
            pass

        assert len(bridge._handlers) == 2
        assert "action1" in bridge._handlers
        assert "action2" in bridge._handlers


class TestBridgeWebViewCallback:
    """Tests for WebView callback integration."""

    def test_set_webview_callback(self):
        """Test setting WebView callback."""
        from auroraview import Bridge

        bridge = Bridge(port=9008)

        def my_callback(action, data, result):
            pass

        bridge.set_webview_callback(my_callback)
        assert bridge._webview_callback == my_callback


class TestBridgeExecuteCommand:
    """Tests for execute_command method."""

    def test_execute_command_without_loop(self):
        """Test execute_command when event loop is not running."""
        from auroraview import Bridge

        bridge = Bridge(port=9009)
        # Should not raise, just log warning
        bridge.execute_command("test_command", {"param1": "value1"})

    def test_execute_command_with_params(self):
        """Test execute_command with parameters."""
        from auroraview import Bridge

        bridge = Bridge(port=9010)
        # Mock the loop
        bridge._loop = MagicMock()
        bridge._loop.is_running.return_value = False

        bridge.execute_command("create_layer", {"name": "Test Layer"})
        # Should not raise


class TestBridgeServiceDiscovery:
    """Tests for service discovery integration."""

    def test_service_discovery_property(self):
        """Test service_discovery property."""
        from auroraview import Bridge

        bridge = Bridge(port=9011)
        # Without service discovery enabled, should be None
        assert bridge.service_discovery is None


class TestBridgeProtocol:
    """Tests for Bridge protocol handling."""

    def test_default_protocol(self):
        """Test default protocol is json."""
        from auroraview import Bridge

        bridge = Bridge(port=9012)
        assert bridge.protocol == "json"

    def test_custom_protocol(self):
        """Test custom protocol setting."""
        from auroraview import Bridge

        bridge = Bridge(port=9013, protocol="msgpack")
        assert bridge.protocol == "msgpack"


class TestBridgeIsRunning:
    """Tests for Bridge is_running property."""

    def test_is_running_initially_false(self):
        """Test is_running is False initially."""
        from auroraview import Bridge

        bridge = Bridge(port=9014)
        assert bridge.is_running is False


class TestBridgeClientCount:
    """Tests for Bridge client_count property."""

    def test_client_count_initially_zero(self):
        """Test client_count is 0 initially."""
        from auroraview import Bridge

        bridge = Bridge(port=9015)
        assert bridge.client_count == 0


class TestBridgeHandlerRegistration:
    """Tests for handler registration logging."""

    def test_register_handler_logs(self):
        """Test that register_handler logs the action."""
        from auroraview import Bridge

        bridge = Bridge(port=9016)

        async def handler(data, client):
            pass

        bridge.register_handler("test_action", handler)
        assert "test_action" in bridge._handlers


class TestBridgeWebViewCallbackLogging:
    """Tests for WebView callback logging."""

    def test_set_webview_callback_logs(self):
        """Test that set_webview_callback logs."""
        from auroraview import Bridge

        bridge = Bridge(port=9017)

        def callback(action, data, result):
            pass

        bridge.set_webview_callback(callback)
        assert bridge._webview_callback is callback


class TestBridgeLoop:
    """Tests for Bridge event loop handling."""

    def test_loop_initially_none(self):
        """Test _loop is None initially."""
        from auroraview import Bridge

        bridge = Bridge(port=9018)
        assert bridge._loop is None


class TestBridgeThread:
    """Tests for Bridge thread handling."""

    def test_thread_initially_none(self):
        """Test _thread is None initially."""
        from auroraview import Bridge

        bridge = Bridge(port=9019)
        assert bridge._thread is None
