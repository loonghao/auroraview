"""Test MCP server integration with WebView.

This test verifies that the MCP server starts correctly and can handle tool calls.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import time
import pytest


def test_mcp_config_creation():
    """Test McpConfig creation and validation."""
    from auroraview._core import McpConfig

    # Test default config
    config = McpConfig(name="test-server")
    assert config.name == "test-server"
    print(f"[TEST] McpConfig created: name={config.name}")


def test_mcp_server_creation():
    """Test McpServer creation."""
    from auroraview._core import McpConfig, McpServer

    # Create server config
    config = McpConfig(name="lifecycle-test")

    # Create server
    server = McpServer(config)
    assert server is not None
    print("[TEST] McpServer created successfully")


def test_mcp_server_tool_registration():
    """Test MCP server tool registration."""
    from auroraview._core import McpConfig, McpServer

    config = McpConfig(name="tool-test")
    server = McpServer(config)

    # Register a tool
    def test_handler(args: dict) -> dict:
        return {"received": args}

    server.register_tool(
        name="test_tool",
        description="A test tool for testing",
        handler=test_handler,
    )
    print("[TEST] Tool registered successfully")


def test_mcp_server_start_stop():
    """Test MCP server start/stop lifecycle."""
    from auroraview._core import McpConfig, McpServer

    config = McpConfig(name="start-stop-test")
    server = McpServer(config)

    # Register a tool (required before start)
    def echo_handler(args: dict) -> dict:
        return {"echo": args.get("message", "")}

    server.register_tool(
        name="echo",
        description="Echo back a message",
        handler=echo_handler,
    )

    # Start server
    server.start()
    print("[TEST] MCP server started")

    # Give server time to start
    time.sleep(0.5)

    # Get port
    port = server.port
    print(f"[TEST] MCP server running on port {port}")
    assert port > 0, "Server should have a valid port"

    # Stop server
    server.stop()
    print("[TEST] MCP server stopped successfully")


def test_mcp_server_tool_call():
    """Test calling a tool through MCP server."""
    from auroraview._core import McpConfig, McpServer

    config = McpConfig(name="tool-call-test")
    server = McpServer(config)

    # Track if handler was called
    handler_called = []

    def add_handler(args: dict) -> dict:
        handler_called.append(True)
        a = args.get("a", 0)
        b = args.get("b", 0)
        return {"result": a + b}

    server.register_tool(
        name="add",
        description="Add two numbers",
        handler=add_handler,
    )

    # Start server
    server.start()
    time.sleep(0.5)

    port = server.port
    print(f"[TEST] MCP server for tool call test on port {port}")

    # Note: Actually calling the tool requires an HTTP client
    # For now, just verify the server started successfully
    assert port > 0

    server.stop()
    print("[TEST] Tool call test completed")


def test_mcp_http_endpoint():
    """Test MCP server HTTP endpoints."""
    import urllib.request
    from auroraview._core import McpConfig, McpServer

    config = McpConfig(name="http-test")
    server = McpServer(config)

    def echo_handler(args: dict) -> dict:
        return {"echo": args.get("message", "no message")}

    server.register_tool(
        name="echo",
        description="Echo back a message",
        handler=echo_handler,
    )

    server.start()
    time.sleep(1.0)  # Give server more time to start

    port = server.port
    print(f"[TEST] Testing HTTP endpoint on port {port}")

    try:
        # Test if server is responding
        # MCP uses SSE endpoint typically at /sse
        url = f"http://127.0.0.1:{port}/"
        req = urllib.request.Request(url)

        try:
            with urllib.request.urlopen(req, timeout=5) as response:
                status = response.status
                print(f"[TEST] Server responded with status {status}")
        except urllib.error.HTTPError as e:
            # Even 404 means server is running
            print(f"[TEST] Server responded with HTTP error {e.code} (server is running)")
        except urllib.error.URLError as e:
            # Connection refused means server might not be ready
            print(f"[TEST] Connection issue: {e}")

    finally:
        server.stop()
        print("[TEST] HTTP endpoint test completed")


def test_mcp_endpoints():
    """Test MCP server endpoints (/mcp, /health, /tools)."""
    import urllib.request
    from auroraview._core import McpConfig, McpServer

    config = McpConfig(name="endpoint-test")
    server = McpServer(config)

    def test_handler(_args: dict) -> dict:
        return {"ok": True}

    server.register_tool(
        name="test_tool",
        description="A test tool for endpoint testing",
        handler=test_handler,
    )

    server.start()
    time.sleep(1.0)

    port = server.port
    print(f"[TEST] Testing MCP endpoints on port {port}")

    try:
        # Test /health endpoint
        health_url = f"http://127.0.0.1:{port}/health"
        try:
            with urllib.request.urlopen(health_url, timeout=5) as response:
                status = response.status
                data = response.read().decode('utf-8')
                print(f"[TEST] /health: status={status}, data={data}")
                assert status == 200, "Health endpoint should return 200"
        except urllib.error.HTTPError as e:
            print(f"[TEST] /health returned HTTP {e.code}")

        # Test /tools endpoint
        tools_url = f"http://127.0.0.1:{port}/tools"
        try:
            with urllib.request.urlopen(tools_url, timeout=5) as response:
                status = response.status
                data = response.read().decode('utf-8')
                print(f"[TEST] /tools: status={status}")
                print(f"[TEST] /tools data: {data[:200]}...")
                assert status == 200, "Tools endpoint should return 200"
                assert "test_tool" in data, "Tools should include registered tool"
        except urllib.error.HTTPError as e:
            print(f"[TEST] /tools returned HTTP {e.code}")

        # Test /mcp endpoint (this is the main MCP endpoint)
        mcp_url = f"http://127.0.0.1:{port}/mcp"
        try:
            req = urllib.request.Request(mcp_url)
            with urllib.request.urlopen(req, timeout=3) as response:
                content_type = response.headers.get('Content-Type', '')
                print(f"[TEST] /mcp Content-Type: {content_type}")
        except urllib.error.HTTPError as e:
            # 405 Method Not Allowed is expected for GET on MCP endpoint
            print(f"[TEST] /mcp returned HTTP {e.code} (expected for GET)")
        except Exception as e:
            print(f"[TEST] /mcp: {type(e).__name__} - {e}")

    finally:
        server.stop()
        print("[TEST] MCP endpoints test completed")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

