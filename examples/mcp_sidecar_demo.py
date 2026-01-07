"""MCP Sidecar Integration Demo - Real IPC communication test.

This example demonstrates the full MCP Sidecar architecture:
1. Main process runs IPC Server (via SidecarBridge)
2. Sidecar process runs HTTP MCP Server
3. AI agents connect via HTTP and call tools via IPC

Features:
- Rust IPC Server with LocalSocket
- Tool registration with Python handlers
- Real HTTP MCP endpoint for AI agents

Use Cases:
- Testing MCP integration in Gallery
- Debugging IPC communication
- Validating tool call flow

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import logging
import time
from typing import Any, Dict

from auroraview import AuroraView, ok

# Configure logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)


def create_sample_tools(view: AuroraView) -> None:
    """Register sample tools for MCP testing."""

    @view.bind_call("mcp.echo")
    def echo(message: str) -> Dict[str, Any]:
        """Echo back the input message.

        A simple tool for testing MCP communication.

        Args:
            message: The message to echo back.

        Returns:
            The echoed message with timestamp.
        """
        return ok({
            "original": message,
            "echoed": f"Echo: {message}",
            "timestamp": time.time(),
        })

    @view.bind_call("mcp.calculate")
    def calculate(a: float, b: float, operation: str = "add") -> Dict[str, Any]:
        """Perform basic arithmetic operations.

        Args:
            a: First number.
            b: Second number.
            operation: One of: add, subtract, multiply, divide.

        Returns:
            The calculation result.
        """
        ops = {
            "add": lambda x, y: x + y,
            "subtract": lambda x, y: x - y,
            "multiply": lambda x, y: x * y,
            "divide": lambda x, y: x / y if y != 0 else float("inf"),
        }
        if operation not in ops:
            return {"error": f"Unknown operation: {operation}"}

        result = ops[operation](a, b)
        return ok({
            "a": a,
            "b": b,
            "operation": operation,
            "result": result,
        })

    @view.bind_call("mcp.get_system_info")
    def get_system_info() -> Dict[str, Any]:
        """Get current system information.

        Returns:
            System info including Python version, platform, etc.
        """
        import platform
        import sys

        return ok({
            "python_version": sys.version,
            "platform": platform.platform(),
            "processor": platform.processor(),
            "hostname": platform.node(),
        })


def main():
    """Run the MCP Sidecar demo."""
    from auroraview import McpConfig, WebView

    # Configure MCP with sidecar mode
    mcp_config = McpConfig(
        name="mcp-sidecar-demo",
        host="127.0.0.1",
        port=0,  # Auto-assign port
        auto_expose_api=True,
    )

    # Create WebView with MCP enabled
    # Note: Use WebView constructor with mcp parameter, not WebView.create()
    view = WebView(
        title="MCP Sidecar Demo",
        width=1200,
        height=800,
        mcp=mcp_config,  # Pass McpConfig to enable MCP
    )

    # Register sample tools
    create_sample_tools(view)

    # Get MCP info after showing (MCP server starts on show)
    # Set initial HTML content
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>MCP Sidecar Demo</title>
        <style>
            body { font-family: system-ui; padding: 2rem; background: #1a1a2e; color: #eee; }
            h1 { color: #00d9ff; }
            .status { background: #16213e; padding: 1rem; border-radius: 8px; margin: 1rem 0; }
            .success { border-left: 4px solid #00ff88; }
            code { background: #0f3460; padding: 0.2rem 0.5rem; border-radius: 4px; }
            #port { color: #00ff88; }
        </style>
    </head>
    <body>
        <h1>🚀 MCP Sidecar Demo</h1>
        <div class="status success">
            <h3>✅ MCP Server Running</h3>
            <p>Port: <code id="port">Starting...</code></p>
            <p>Endpoint: <code id="endpoint">Starting...</code></p>
        </div>
        <h2>Available Tools</h2>
        <ul>
            <li><code>mcp.echo</code> - Echo back messages</li>
            <li><code>mcp.calculate</code> - Arithmetic operations</li>
            <li><code>mcp.get_system_info</code> - System information</li>
        </ul>
        <h2>Test with curl</h2>
        <pre id="curl-cmd">curl http://127.0.0.1:PORT/mcp -d '{"method":"tools/list"}'</pre>
        <script>
            // Will be updated by Python after MCP server starts
            window.updateMcpInfo = function(port) {
                document.getElementById('port').textContent = port;
                document.getElementById('endpoint').textContent = 'http://127.0.0.1:' + port + '/mcp';
                document.getElementById('curl-cmd').textContent =
                    'curl http://127.0.0.1:' + port + '/mcp -d \\'{\"method\":\"tools/list\"}\\'';
            };
        </script>
    </body>
    </html>
    """
    view.load_html(html)

    # Show the WebView (MCP server starts here)
    logger.info("Starting WebView with MCP server...")
    view.show()


if __name__ == "__main__":
    main()

