"""Minimal MCP (Rust) example for IDE/agent usage.

Run in a DCC or standalone environment after building with `--features mcp-server`.
The script starts a WebView, auto-exposes bound calls as MCP tools, and prints the
assigned MCP port for agent configuration.
"""

from __future__ import annotations

from auroraview import WebView


def echo(message: str) -> dict:
    """Echo back incoming message."""
    return {"echo": message}


def get_status() -> dict:
    """Return basic status."""
    return {"status": "ok"}


def main() -> None:
    webview = WebView(
        title="AuroraView MCP Demo",
        url="https://example.com",  # replace with your local UI if needed
        mcp=True,
        mcp_name="auroraview-mcp",
        # mcp_port=8765,  # uncomment to pin the port
    )

    webview.bind_call("api.echo", echo)
    webview.bind_call("api.status", get_status)

    # Show window; MCP server starts automatically when shown
    webview.show()

    # Print MCP connection info for IDEs/agents
    if webview.mcp_port:
        print(f"[MCP] Connect your IDE/agent to: http://127.0.0.1:{webview.mcp_port}/sse")


if __name__ == "__main__":
    main()
