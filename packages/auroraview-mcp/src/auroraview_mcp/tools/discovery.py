"""Discovery tools for AuroraView MCP Server."""

from __future__ import annotations

from typing import Any

from auroraview_mcp.server import get_connection_manager, get_discovery, mcp


@mcp.tool()
async def discover_instances(ports: list[int] | None = None) -> list[dict[str, Any]]:
    """Discover all running AuroraView instances.

    Scans specified ports (or default ports 9222-9225) to find running
    AuroraView/WebView2 instances with CDP enabled.

    Args:
        ports: List of ports to scan. If None, uses default ports [9222, 9223, 9224, 9225].

    Returns:
        List of discovered instances, each containing:
        - port: CDP port number
        - browser: Browser/WebView version string
        - ws_url: WebSocket debugger URL
        - user_agent: User agent string
        - protocol_version: CDP protocol version
    """
    discovery = get_discovery()
    instances = await discovery.discover(ports)
    return [instance.to_dict() for instance in instances]


@mcp.tool()
async def connect(port: int = 9222) -> dict[str, Any]:
    """Connect to an AuroraView instance.

    Establishes a CDP connection to the specified port. After connecting,
    you can use other tools to interact with the instance.

    Args:
        port: CDP port number. Default is 9222.

    Returns:
        Connection status and instance information:
        - status: "connected"
        - port: Connected port number
        - pages: List of available pages
    """
    manager = get_connection_manager()
    await manager.connect(port)

    # Get available pages
    pages = await manager.get_pages()

    # Auto-select first page if available
    if pages:
        await manager.select_page(page_id=pages[0].id)

    return {
        "status": "connected",
        "port": port,
        "pages": [page.to_dict() for page in pages],
        "current_page": manager.current_page.to_dict() if manager.current_page else None,
    }


@mcp.tool()
async def disconnect() -> dict[str, Any]:
    """Disconnect from the current AuroraView instance.

    Closes the CDP connection to the currently connected instance.

    Returns:
        Disconnection status:
        - status: "disconnected"
        - port: Previously connected port (or None)
    """
    manager = get_connection_manager()
    port = manager.current_port
    await manager.disconnect()

    return {
        "status": "disconnected",
        "port": port,
    }


@mcp.tool()
async def list_dcc_instances(ports: list[int] | None = None) -> list[dict[str, Any]]:
    """Discover AuroraView instances in DCC environments.

    Scans for AuroraView instances and enriches them with DCC context
    information (Maya, Blender, Houdini, Nuke, Unreal, etc.).

    Args:
        ports: List of ports to scan. If None, uses default ports.

    Returns:
        List of DCC instances, each containing:
        - port: CDP port number
        - browser: Browser/WebView version
        - dcc_type: DCC application type (maya, blender, houdini, etc.)
        - dcc_version: DCC version (if detected)
        - panel_name: Panel/window name
        - title: Page title
        - url: Page URL
    """
    discovery = get_discovery()
    instances = await discovery.discover_dcc_instances(ports)
    return [instance.to_dict() for instance in instances]
