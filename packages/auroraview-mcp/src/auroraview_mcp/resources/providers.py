"""Resource providers for AuroraView MCP Server.

Provides MCP resources for:
- Instance information
- Page details
- Sample listings
- Gallery status
- Console logs
"""

from __future__ import annotations

import json

from auroraview_mcp.server import get_connection_manager, get_discovery, mcp
from auroraview_mcp.tools.gallery import (
    _process_manager,
    get_examples_dir,
    get_gallery_dir,
    get_project_root,
    get_sample_info,
    scan_samples,
)


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
        JSON string of sample list with full details.
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError:
        return json.dumps({"error": "Examples directory not found"})

    samples = scan_samples(examples_dir)
    return json.dumps(samples, indent=2)


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
        return "# Error: Examples directory not found"

    # Find sample
    main_file = None

    # Try as .py file
    for suffix in ["", "_demo", "_example"]:
        candidate = examples_dir / f"{name}{suffix}.py"
        if candidate.exists():
            main_file = candidate
            break

    # Try as directory
    if not main_file:
        sample_dir = examples_dir / name
        if sample_dir.is_dir():
            info = get_sample_info(sample_dir)
            if info:
                from pathlib import Path

                main_file = Path(info["main_file"])

    if not main_file or not main_file.exists():
        return f"# Error: Sample not found: {name}"

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


@mcp.resource("auroraview://gallery")
async def get_gallery_resource() -> str:
    """Get Gallery application status and information.

    Returns:
        JSON string of Gallery status:
        - running: Whether Gallery is running
        - pid: Process ID (if running)
        - port: CDP port (if running)
        - gallery_dir: Gallery directory path
        - dist_exists: Whether built dist exists
    """
    try:
        gallery_dir = get_gallery_dir()
        dist_exists = (gallery_dir / "dist" / "index.html").exists()
    except FileNotFoundError:
        return json.dumps(
            {
                "running": False,
                "error": "Gallery directory not found",
            }
        )

    proc_info = _process_manager.get_gallery()

    if proc_info and proc_info.process.poll() is None:
        return json.dumps(
            {
                "running": True,
                "pid": proc_info.pid,
                "port": proc_info.port,
                "gallery_dir": str(gallery_dir),
                "dist_exists": dist_exists,
            },
            indent=2,
        )

    return json.dumps(
        {
            "running": False,
            "gallery_dir": str(gallery_dir),
            "dist_exists": dist_exists,
        },
        indent=2,
    )


@mcp.resource("auroraview://project")
async def get_project_resource() -> str:
    """Get AuroraView project information.

    Returns:
        JSON string of project info:
        - project_root: Project root directory
        - gallery_dir: Gallery directory
        - examples_dir: Examples directory
        - gallery_built: Whether Gallery dist exists
        - sample_count: Number of available samples
    """
    try:
        project_root = get_project_root()
        gallery_dir = get_gallery_dir()
        examples_dir = get_examples_dir()
    except FileNotFoundError as e:
        return json.dumps({"error": str(e)})

    gallery_built = (gallery_dir / "dist" / "index.html").exists()
    samples = scan_samples(examples_dir)

    return json.dumps(
        {
            "project_root": str(project_root),
            "gallery_dir": str(gallery_dir),
            "examples_dir": str(examples_dir),
            "gallery_built": gallery_built,
            "sample_count": len(samples),
        },
        indent=2,
    )


@mcp.resource("auroraview://processes")
async def get_processes_resource() -> str:
    """Get list of running sample processes.

    Returns:
        JSON string of process list.
    """
    _process_manager.cleanup()

    processes = []
    for info in _process_manager.list_all():
        status = "running" if info.process.poll() is None else "terminated"
        processes.append(
            {
                "pid": info.pid,
                "name": info.name,
                "status": status,
                "port": info.port,
                "is_gallery": info.is_gallery,
            }
        )

    return json.dumps(processes, indent=2)
