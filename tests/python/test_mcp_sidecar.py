# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Tests for McpSidecar Python manager."""

from __future__ import annotations

import os
import platform
import pytest

from auroraview.mcp.sidecar import McpSidecar, _generate_channel_name, _generate_auth_token


class TestHelperFunctions:
    """Test helper functions."""

    def test_generate_channel_name_format(self):
        """Channel name should include PID and nonce."""
        name = _generate_channel_name()
        assert name.startswith("auroraview_mcp_")
        parts = name.split("_")
        assert len(parts) == 4
        assert parts[2] == str(os.getpid())

    def test_generate_channel_name_unique(self):
        """Each call should generate a unique name."""
        names = [_generate_channel_name() for _ in range(10)]
        assert len(set(names)) == 10

    def test_generate_auth_token_length(self):
        """Auth token should be reasonably long."""
        token = _generate_auth_token()
        assert len(token) >= 32

    def test_generate_auth_token_unique(self):
        """Each call should generate a unique token."""
        tokens = [_generate_auth_token() for _ in range(10)]
        assert len(set(tokens)) == 10


class TestMcpSidecar:
    """Test McpSidecar class."""

    def test_init_defaults(self):
        """Test default initialization."""
        sidecar = McpSidecar()
        assert sidecar._port == 0
        assert sidecar._log_level == "info"
        assert sidecar._process is None
        assert not sidecar._started

    def test_init_custom_port(self):
        """Test initialization with custom port."""
        sidecar = McpSidecar(port=8080)
        assert sidecar._port == 8080

    def test_get_binary_path_exists(self):
        """Binary path should be found in development environment."""
        path = McpSidecar.get_binary_path()
        # In CI or fresh clone, binary might not exist
        if path is not None:
            assert os.path.isfile(path)
            expected_name = "auroraview-mcp-server"
            if platform.system() == "Windows":
                expected_name += ".exe"
            assert os.path.basename(path) == expected_name

    def test_register_tool(self):
        """Test tool registration."""
        sidecar = McpSidecar()

        def my_handler(x: int) -> int:
            return x * 2

        sidecar.register_tool(
            name="double",
            description="Double a number",
            handler=my_handler,
            input_schema={"type": "object", "properties": {"x": {"type": "integer"}}},
        )

        assert "double" in sidecar._tools
        assert sidecar._tools["double"]["name"] == "double"
        assert sidecar._tools["double"]["handler"] is my_handler

    def test_tools_property(self):
        """Test tools property returns definitions without handlers."""
        sidecar = McpSidecar()
        sidecar.register_tool(
            name="test",
            description="Test tool",
            handler=lambda: None,
        )

        tools = sidecar.tools
        assert len(tools) == 1
        assert tools[0]["name"] == "test"
        assert "handler" not in tools[0]

    def test_is_alive_not_started(self):
        """is_alive should return False when not started."""
        sidecar = McpSidecar()
        assert not sidecar.is_alive()

    def test_port_not_started(self):
        """port should return 0 when not started."""
        sidecar = McpSidecar()
        assert sidecar.port == 0

    def test_stop_not_started(self):
        """stop should be safe when not started."""
        sidecar = McpSidecar()
        sidecar.stop()  # Should not raise


@pytest.mark.skipif(
    McpSidecar.get_binary_path() is None,
    reason="MCP sidecar binary not available",
)
@pytest.mark.skip(reason="Requires IPC server running in main process - full integration test")
class TestMcpSidecarIntegration:
    """Integration tests requiring the sidecar binary and IPC server."""

    def test_start_and_stop(self):
        """Test starting and stopping the sidecar."""
        sidecar = McpSidecar(log_level="debug")
        try:
            port = sidecar.start()
            assert port > 0
            assert sidecar.is_alive()
            assert sidecar.port == port
        finally:
            sidecar.stop()

        assert not sidecar.is_alive()
        assert sidecar.port == 0

    def test_context_manager(self):
        """Test using sidecar as context manager."""
        with McpSidecar(log_level="debug") as sidecar:
            assert sidecar.is_alive()
            port = sidecar.port
            assert port > 0

        assert not sidecar.is_alive()

    def test_double_start(self):
        """Starting twice should return same port."""
        sidecar = McpSidecar(log_level="debug")
        try:
            port1 = sidecar.start()
            port2 = sidecar.start()
            assert port1 == port2
        finally:
            sidecar.stop()
