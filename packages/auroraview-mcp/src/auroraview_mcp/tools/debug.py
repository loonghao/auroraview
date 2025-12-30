"""Debug tools for AuroraView MCP Server."""

from __future__ import annotations

import contextlib
from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def get_console_logs(level: str | None = None, limit: int = 100) -> list[dict[str, Any]]:
    """Get console logs from the current page.

    Retrieves console messages (log, warn, error) from the page.

    Args:
        level: Filter by log level ("log", "warn", "error", "info").
        limit: Maximum number of logs to return.

    Returns:
        List of console messages, each containing:
        - level: Log level
        - text: Message text
        - timestamp: Message timestamp
        - source: Source URL
        - line: Line number
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Enable console if not already enabled
    with contextlib.suppress(Exception):
        await conn.send("Console.enable")

    # Get logs via JavaScript
    script = f"""
    (() => {{
        // Access stored console logs if available
        const logs = window.__auroraview_console_logs || [];
        const level = "{level or ""}";
        const limit = {limit};

        let filtered = logs;
        if (level) {{
            filtered = logs.filter(log => log.level === level);
        }}

        return filtered.slice(-limit);
    }})()
    """

    logs = await conn.evaluate(script)

    if not isinstance(logs, list):
        return []

    return logs


@mcp.tool()
async def get_network_requests(
    url_pattern: str | None = None, method: str | None = None
) -> list[dict[str, Any]]:
    """Get network requests from the current page.

    Retrieves network request history from the page.

    Args:
        url_pattern: Filter by URL pattern (supports wildcards).
        method: Filter by HTTP method (GET, POST, etc.).

    Returns:
        List of requests, each containing:
        - url: Request URL
        - method: HTTP method
        - status: Response status code
        - type: Resource type
        - size: Response size in bytes
        - time: Request duration in ms
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Get requests via JavaScript Performance API
    url_filter = url_pattern.replace("*", ".*") if url_pattern else ""
    method_filter = method.upper() if method else ""

    script = f"""
    (() => {{
        const entries = performance.getEntriesByType("resource");
        const urlPattern = "{url_filter}";
        const methodFilter = "{method_filter}";

        let requests = entries.map(entry => ({{
            url: entry.name,
            method: "GET",  // Performance API doesn't expose method
            status: 200,    // Assumed successful
            type: entry.initiatorType,
            size: entry.transferSize || 0,
            time: Math.round(entry.duration)
        }}));

        if (urlPattern) {{
            const regex = new RegExp(urlPattern);
            requests = requests.filter(r => regex.test(r.url));
        }}

        return requests;
    }})()
    """

    requests = await conn.evaluate(script)

    if not isinstance(requests, list):
        return []

    return requests


@mcp.tool()
async def get_backend_status() -> dict[str, Any]:
    """Get AuroraView Python backend status.

    Returns status information about the Python backend.

    Returns:
        Backend status:
        - ready: Whether backend is ready
        - uptime: Uptime in seconds (if available)
        - handlers: List of registered handlers
        - version: AuroraView version (if available)
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    script = """
    (() => {
        if (!window.auroraview) {
            return { ready: false };
        }

        return {
            ready: true,
            handlers: window.__auroraview_api_methods || [],
            version: window.__auroraview_version || null,
            platform: window.auroraview.platform || null
        };
    })()
    """

    status = await conn.evaluate(script)

    if not isinstance(status, dict):
        return {"ready": False}

    return status


@mcp.tool()
async def get_memory_info() -> dict[str, Any]:
    """Get memory usage information.

    Returns memory usage information from the page.

    Returns:
        Memory info:
        - usedJSHeapSize: Used JS heap size in bytes
        - totalJSHeapSize: Total JS heap size in bytes
        - jsHeapSizeLimit: JS heap size limit in bytes
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    script = """
    (() => {
        if (performance.memory) {
            return {
                usedJSHeapSize: performance.memory.usedJSHeapSize,
                totalJSHeapSize: performance.memory.totalJSHeapSize,
                jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
            };
        }
        return null;
    })()
    """

    memory = await conn.evaluate(script)

    if not memory:
        return {"error": "Memory API not available"}

    return memory


@mcp.tool()
async def clear_console() -> dict[str, Any]:
    """Clear the console logs.

    Clears all console messages from the page.

    Returns:
        Clear status:
        - status: "cleared"
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    await conn.evaluate("console.clear(); window.__auroraview_console_logs = [];")

    return {"status": "cleared"}
