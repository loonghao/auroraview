"""UI tools for AuroraView MCP Server."""

from __future__ import annotations

import base64
from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def take_screenshot(
    selector: str | None = None,
    full_page: bool = False,
    path: str | None = None,
) -> str:
    """Take a screenshot of the current page.

    Captures a screenshot of the page or a specific element.

    Args:
        selector: CSS selector to capture a specific element.
        full_page: If True, captures the entire scrollable page.
        path: File path to save the screenshot. If None, returns base64.

    Returns:
        File path if saved, or base64 data URL.
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    params: dict[str, Any] = {
        "format": "png",
        "captureBeyondViewport": full_page,
    }

    if selector:
        # Get element bounds
        bounds_script = f"""
        (() => {{
            const el = document.querySelector("{selector}");
            if (!el) return null;
            const rect = el.getBoundingClientRect();
            return {{
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                scale: window.devicePixelRatio || 1
            }};
        }})()
        """
        bounds = await conn.evaluate(bounds_script)
        if bounds:
            params["clip"] = {
                "x": bounds["x"],
                "y": bounds["y"],
                "width": bounds["width"],
                "height": bounds["height"],
                "scale": bounds.get("scale", 1),
            }

    result = await conn.send("Page.captureScreenshot", params)
    data = result.get("data", "")

    if path:
        with open(path, "wb") as f:
            f.write(base64.b64decode(data))
        return path

    return f"data:image/png;base64,{data}"


@mcp.tool()
async def get_snapshot() -> dict[str, Any]:
    """Get the accessibility tree snapshot of the current page.

    Returns a structured representation of the page's accessibility tree,
    useful for understanding page structure and finding interactive elements.

    Returns:
        Accessibility tree structure with element UIDs.
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Get accessibility tree
    result = await conn.send("Accessibility.getFullAXTree")

    nodes = result.get("nodes", [])

    # Simplify the tree for easier consumption
    simplified = []
    for node in nodes:
        if node.get("ignored"):
            continue

        simplified_node = {
            "uid": node.get("nodeId"),
            "role": node.get("role", {}).get("value", ""),
            "name": node.get("name", {}).get("value", ""),
        }

        # Add properties
        props = node.get("properties", [])
        for prop in props:
            name = prop.get("name", "")
            value = prop.get("value", {}).get("value")
            if name and value is not None:
                simplified_node[name] = value

        simplified.append(simplified_node)

    return {
        "nodes": simplified,
        "count": len(simplified),
    }


@mcp.tool()
async def click(selector: str | None = None, uid: str | None = None) -> dict[str, Any]:
    """Click on an element.

    Clicks on an element identified by CSS selector or accessibility UID.

    Args:
        selector: CSS selector of the element to click.
        uid: Accessibility tree UID of the element.

    Returns:
        Click result:
        - status: "clicked"
        - selector: The selector used (if any)
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    if not selector and not uid:
        raise ValueError("Either selector or uid must be provided")

    conn = await manager.get_page_connection()

    if selector:
        # Click using selector
        script = f"""
        (() => {{
            const el = document.querySelector("{selector}");
            if (!el) return {{ ok: false, error: "Element not found" }};
            el.click();
            return {{ ok: true }};
        }})()
        """
        result = await conn.evaluate(script)

        if not isinstance(result, dict) or not result.get("ok"):
            error = (
                result.get("error", "Click failed") if isinstance(result, dict) else "Click failed"
            )
            raise RuntimeError(error)

        return {"status": "clicked", "selector": selector}

    # Click using accessibility UID
    # This requires DOM.focus and Input.dispatchMouseEvent
    raise NotImplementedError("Click by UID not yet implemented")


@mcp.tool()
async def fill(selector: str, value: str) -> dict[str, Any]:
    """Fill an input element with text.

    Types text into an input, textarea, or selects an option from a select element.

    Args:
        selector: CSS selector of the input element.
        value: Text value to fill.

    Returns:
        Fill result:
        - status: "filled"
        - selector: The selector used
        - value: The value filled
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Escape value for JavaScript
    escaped_value = value.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")

    script = f"""
    (() => {{
        const el = document.querySelector("{selector}");
        if (!el) return {{ ok: false, error: "Element not found" }};

        // Handle different input types
        if (el.tagName === "SELECT") {{
            el.value = "{escaped_value}";
            el.dispatchEvent(new Event("change", {{ bubbles: true }}));
        }} else {{
            el.focus();
            el.value = "{escaped_value}";
            el.dispatchEvent(new Event("input", {{ bubbles: true }}));
            el.dispatchEvent(new Event("change", {{ bubbles: true }}));
        }}
        return {{ ok: true }};
    }})()
    """

    result = await conn.evaluate(script)

    if not isinstance(result, dict) or not result.get("ok"):
        error = result.get("error", "Fill failed") if isinstance(result, dict) else "Fill failed"
        raise RuntimeError(error)

    return {
        "status": "filled",
        "selector": selector,
        "value": value,
    }


@mcp.tool()
async def evaluate(script: str) -> Any:
    """Execute JavaScript code in the page context.

    Evaluates arbitrary JavaScript code and returns the result.

    Args:
        script: JavaScript code to execute.

    Returns:
        Evaluation result (must be JSON-serializable).
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()
    return await conn.evaluate(script)


@mcp.tool()
async def hover(selector: str) -> dict[str, Any]:
    """Hover over an element.

    Moves the mouse cursor over an element, triggering hover effects.

    Args:
        selector: CSS selector of the element to hover.

    Returns:
        Hover result:
        - status: "hovered"
        - selector: The selector used
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        raise RuntimeError("Not connected to any instance. Use connect() first.")

    if manager.current_page is None:
        raise RuntimeError("No page selected. Use select_page() first.")

    conn = await manager.get_page_connection()

    # Get element center coordinates
    script = f"""
    (() => {{
        const el = document.querySelector("{selector}");
        if (!el) return null;
        const rect = el.getBoundingClientRect();
        return {{
            x: rect.x + rect.width / 2,
            y: rect.y + rect.height / 2
        }};
    }})()
    """

    coords = await conn.evaluate(script)
    if not coords:
        raise RuntimeError(f"Element not found: {selector}")

    # Dispatch mouse move event
    await conn.send(
        "Input.dispatchMouseEvent",
        {
            "type": "mouseMoved",
            "x": coords["x"],
            "y": coords["y"],
        },
    )

    return {"status": "hovered", "selector": selector}
