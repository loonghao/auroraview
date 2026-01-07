# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Tests for Rust MCP Server implementation."""

from __future__ import annotations

import pytest

import pytest

try:
    from auroraview.mcp import McpConfig, McpServer, _USE_RUST_MCP
except ImportError:
    pytest.skip("Rust MCP not available", allow_module_level=True)


class TestRustMcpServer:

    """Tests for Rust-based MCP Server."""

    def test_config_creation(self):
        """Test McpConfig creation."""
        from auroraview.mcp import McpConfig

        config = McpConfig()
        assert config.host == "127.0.0.1"
        assert config.port == 0

    def test_config_with_options(self):
        """Test McpConfig with custom options."""
        from auroraview.mcp import McpConfig

        config = McpConfig(
            name="test-server",
            port=8765,
            host="0.0.0.0",
        )
        assert config.name == "test-server"
        assert config.port == 8765
        assert config.host == "0.0.0.0"

    def test_server_creation(self):
        """Test McpServer creation."""
        from auroraview.mcp import McpConfig, McpServer

        config = McpConfig(name="test")
        server = McpServer(config)
        assert not server.is_running
        assert server.port == 0

    def test_server_tool_registration(self):
        """Test tool registration."""
        from auroraview.mcp import McpServer

        server = McpServer()

        def echo_handler(args):
            return {"echoed": args.get("message", "")}

        server.register_tool("echo", echo_handler, "Echo back the input")
        assert "echo" in server.list_tools()
        assert len(server) == 1

    def test_server_start_stop(self):
        """Test server start and stop."""
        from auroraview.mcp import McpServer

        server = McpServer()
        port = server.start()

        assert port > 0
        assert server.is_running
        assert server.port == port

        server.stop()
        assert not server.is_running


def test_feature_flag():
    """Test that _USE_RUST_MCP flag is set correctly."""
    from auroraview.mcp import _USE_RUST_MCP

    # Flag should be a boolean
    assert isinstance(_USE_RUST_MCP, bool)
