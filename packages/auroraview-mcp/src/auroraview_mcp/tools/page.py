"""Page tools for AuroraView MCP Server."""

from __future__ import annotations

from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def list_pages() -> list[dict[str, Any]]:
    """List all pages in the connected AuroraView instance.

    Returns all available pages/targets from the currently connected
    CDP instance.

    Returns:
        List of pages, each containing:
        - id: Page/target ID
        - url: Page URL
        - title: Page title
        - type: Target type (usually "page")
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    pages = await manager.get_pages()
    return [page.to_dict() for page in pages]


@mcp.tool()
async def select_page(page_id: str | None = None, url_pattern: str | None = None) -> dict[str, Any]:
    """Select a page to operate on.

    Selects a specific page for subsequent operations. You can select
    by page ID or by URL pattern (supports wildcards).

    Args:
        page_id: Exact page ID to select.
        url_pattern: URL pattern to match (supports * wildcards).

    Returns:
        Selected page information:
        - id: Page ID
        - url: Page URL
        - title: Page title
        - selected: True if selection successful
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    page = await manager.select_page(page_id=page_id, url_pattern=url_pattern)

    if page is None:
        return {
            "selected": False,
            "error": "No matching page found",
        }

    return {
        "selected": True,
        "id": page.id,
        "url": page.url,
        "title": page.title,
    }


@mcp.tool()
async def get_page_info() -> dict[str, Any]:
    """Get detailed information about the current page.

    Returns comprehensive information about the currently selected page,
    including AuroraView-specific status.

    Returns:
        Page information:
        - id: Page ID
        - url: Page URL
        - title: Page title
        - auroraview_ready: Whether AuroraView bridge is ready
        - api_methods: List of available API methods (if AuroraView)
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    page = manager.current_page
    if page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    # Get page connection
    conn = await manager.get_page_connection()

    # Check if AuroraView is ready
    auroraview_ready = False
    api_methods: list[str] = []

    try:
        ready = await conn.evaluate(
            "typeof window.auroraview !== 'undefined' && window.auroraview !== null"
        )
        auroraview_ready = bool(ready)

        if auroraview_ready:
            # Try to get API methods
            methods = await conn.evaluate("window.__auroraview_api_methods || []")
            if isinstance(methods, list):
                api_methods = methods
    except Exception:
        pass

    return {
        "id": page.id,
        "url": page.url,
        "title": page.title,
        "auroraview_ready": auroraview_ready,
        "api_methods": api_methods,
    }


@mcp.tool()
async def reload_page(hard: bool = False) -> dict[str, Any]:
    """Reload the current page.

    Reloads the currently selected page. Use hard reload to bypass cache.

    Args:
        hard: If True, performs a hard reload (ignores cache).

    Returns:
        Reload status:
        - status: "reloaded"
        - hard: Whether hard reload was performed
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()
    await conn.send("Page.reload", {"ignoreCache": hard})

    return {
        "status": "reloaded",
        "hard": hard,
    }
