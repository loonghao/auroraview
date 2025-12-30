"""Resource providers for AuroraView MCP Server."""

from __future__ import annotations

import json

from auroraview_mcp.server import get_connection_manager, get_discovery, mcp
from auroraview_mcp.tools.gallery import get_sample_info, get_examples_dir


@mcp.resource("auroraview://instances")
async def get_instances_resource() -> str:
    """Get list of running AuroraView instances.

    Returns:
        JSON string of instance list.
    """
    discovery = get_discovery()
    instances = await discovery.discover()
    return json.dumps([inst.to_dict() for inst in instances], indent=2)


@mcp.resource("auroraview://page/{page_id}")
async def get_page_resource(page_id: str) -> str:
    """Get detailed information about a specific page.

    Args:
        page_id: Page ID to get info for.

    Returns:
        JSON string of page information.
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        return json.dumps({"error": "Not connected"})

    pages = await manager.get_pages()
    for page in pages:
        if page.id == page_id:
            return json.dumps(page.to_dict(), indent=2)

    return json.dumps({"error": f"Page not found: {page_id}"})


@mcp.resource("auroraview://samples")
async def get_samples_resource() -> str:
    """Get list of available samples.

    Returns:
        JSON string of sample list.
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError:
        return json.dumps([])

    samples = []
    for item in examples_dir.iterdir():
        if item.is_dir() and not item.name.startswith(("_", ".")):
            info = get_sample_info(item)
            if info:
                samples.append(info)

    return json.dumps(sorted(samples, key=lambda x: x["name"]), indent=2)


@mcp.resource("auroraview://sample/{name}/source")
async def get_sample_source_resource(name: str) -> str:
    """Get source code of a sample.

    Args:
        name: Sample name.

    Returns:
        Python source code.
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError:
        return f"# Error: Examples directory not found"

    sample_dir = examples_dir / name
    if not sample_dir.exists():
        return f"# Error: Sample not found: {name}"

    info = get_sample_info(sample_dir)
    if not info:
        return f"# Error: Invalid sample: {name}"

    from pathlib import Path
    main_file = Path(info["main_file"])
    return main_file.read_text(encoding="utf-8")


@mcp.resource("auroraview://logs")
async def get_logs_resource() -> str:
    """Get recent console logs.

    Returns:
        JSON string of log entries.
    """
    manager = get_connection_manager()
    if not manager.is_connected:
        return json.dumps({"error": "Not connected"})

    if manager.current_page is None:
        return json.dumps({"error": "No page selected"})

    try:
        conn = await manager.get_page_connection()
        script = """
        (() => {
            return window.__auroraview_console_logs || [];
        })()
        """
        logs = await conn.evaluate(script)
        return json.dumps(logs if isinstance(logs, list) else [], indent=2)
    except Exception as e:
        return json.dumps({"error": str(e)})
