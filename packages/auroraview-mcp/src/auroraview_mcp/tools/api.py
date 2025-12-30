"""API tools for AuroraView MCP Server."""

from __future__ import annotations

import json
from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def call_api(method: str, params: dict[str, Any] | None = None) -> Any:
    """Call an AuroraView Python API method.

    Invokes a method on the AuroraView Python backend through the JS bridge.
    The method name should follow the format "namespace.method" (e.g., "api.echo").

    Args:
        method: API method name (e.g., "api.get_samples", "tool.apply").
        params: Method parameters as a dictionary.

    Returns:
        API method return value.

    Example:
        call_api("api.echo", {"message": "hello"})
        call_api("api.get_samples", {"category": "getting-started"})
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Build the call script
    if params:
        params_json = json.dumps(params)
        script = f"""
        (async () => {{
            try {{
                if (!window.auroraview) {{
                    return {{ ok: false, error: "AuroraView bridge not ready" }};
                }}
                const result = await window.auroraview.call("{method}", {params_json});
                return {{ ok: true, result }};
            }} catch (e) {{
                return {{ ok: false, error: e.message || String(e) }};
            }}
        }})()
        """
    else:
        # For simple API calls without params, try the api.* shorthand
        parts = method.split(".")
        if len(parts) == 2 and parts[0] == "api":
            method_name = parts[1]
            script = f"""
            (async () => {{
                try {{
                    if (!window.auroraview || !window.auroraview.api) {{
                        return {{ ok: false, error: "AuroraView bridge not ready" }};
                    }}
                    const result = await window.auroraview.api.{method_name}();
                    return {{ ok: true, result }};
                }} catch (e) {{
                    return {{ ok: false, error: e.message || String(e) }};
                }}
            }})()
            """
        else:
            script = f"""
            (async () => {{
                try {{
                    if (!window.auroraview) {{
                        return {{ ok: false, error: "AuroraView bridge not ready" }};
                    }}
                    const result = await window.auroraview.call("{method}");
                    return {{ ok: true, result }};
                }} catch (e) {{
                    return {{ ok: false, error: e.message || String(e) }};
                }}
            }})()
            """

    result = await conn.evaluate(script)

    if not isinstance(result, dict):
        raise RuntimeError(f"Unexpected response format: {result}")

    if not result.get("ok"):
        raise RuntimeError(result.get("error", "Unknown error"))

    return result.get("result")


@mcp.tool()
async def list_api_methods() -> list[dict[str, Any]]:
    """List all available AuroraView API methods.

    Returns a list of all methods exposed by the AuroraView Python backend
    through the JS bridge.

    Returns:
        List of methods, each containing:
        - name: Method name
        - signature: Method signature (if available)
        - docstring: Method documentation (if available)
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
            return [];
        }
        // Return registered API methods
        return window.__auroraview_api_methods || [];
    })()
    """

    methods = await conn.evaluate(script)

    if not isinstance(methods, list):
        return []

    # Format as list of dicts
    return [{"name": m, "signature": None, "docstring": None} for m in methods]


@mcp.tool()
async def emit_event(event: str, data: dict[str, Any] | None = None) -> dict[str, Any]:
    """Emit an event to the AuroraView frontend.

    Triggers a custom event that can be listened to by the frontend
    using window.auroraview.on(event, handler).

    Args:
        event: Event name.
        data: Event payload data.

    Returns:
        Emit status:
        - status: "emitted"
        - event: Event name
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    data_json = json.dumps(data) if data else "null"
    script = f"""
    (() => {{
        if (window.auroraview && window.auroraview.trigger) {{
            window.auroraview.trigger("{event}", {data_json});
            return {{ ok: true }};
        }}
        return {{ ok: false, error: "AuroraView bridge not ready" }};
    }})()
    """

    result = await conn.evaluate(script)

    if not isinstance(result, dict) or not result.get("ok"):
        error = (
            result.get("error", "Unknown error") if isinstance(result, dict) else "Unknown error"
        )
        raise RuntimeError(error)

    return {
        "status": "emitted",
        "event": event,
    }
