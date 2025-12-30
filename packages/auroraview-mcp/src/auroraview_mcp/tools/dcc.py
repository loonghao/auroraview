"""DCC (Digital Content Creation) tools for AuroraView MCP Server.

This module provides tools for interacting with AuroraView instances
running inside DCC applications like Maya, Blender, Houdini, Nuke, etc.
"""

from __future__ import annotations

from typing import Any

from auroraview_mcp.server import get_connection_manager, mcp


@mcp.tool()
async def get_dcc_context() -> dict[str, Any]:
    """Get the current DCC environment context.

    Retrieves information about the DCC application context including
    scene information, selection state, current frame, and project path.

    This tool requires an active connection to an AuroraView instance
    running inside a DCC application.

    Returns:
        DCC context information:
        - dcc_type: DCC application type (maya, blender, houdini, nuke, unreal)
        - dcc_version: DCC application version
        - scene_path: Current scene/project file path
        - scene_name: Current scene name
        - selected_objects: List of currently selected objects
        - current_frame: Current timeline frame
        - frame_range: Frame range (start, end)
        - project_path: Project directory path
        - fps: Frames per second
        - units: Scene units (cm, m, etc.)

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    # Execute JavaScript to get DCC context from AuroraView bridge
    result = await page_conn.evaluate("""
    (async () => {
        // Check if AuroraView bridge is available
        if (!window.auroraview || !window.auroraview.call) {
            return {
                error: 'AuroraView bridge not available',
                dcc_type: null
            };
        }

        try {
            // Try to get DCC context via AuroraView API
            const context = await window.auroraview.call('dcc.get_context');
            return context;
        } catch (e) {
            // Fallback: try to detect DCC type from environment
            const dccInfo = {
                dcc_type: null,
                dcc_version: null,
                scene_path: null,
                scene_name: null,
                selected_objects: [],
                current_frame: null,
                frame_range: null,
                project_path: null,
                fps: null,
                units: null,
                error: e.message
            };

            // Try to detect from page title or URL
            const title = document.title.toLowerCase();
            const url = window.location.href.toLowerCase();

            if (title.includes('maya') || url.includes('maya')) {
                dccInfo.dcc_type = 'maya';
            } else if (title.includes('blender') || url.includes('blender')) {
                dccInfo.dcc_type = 'blender';
            } else if (title.includes('houdini') || url.includes('houdini')) {
                dccInfo.dcc_type = 'houdini';
            } else if (title.includes('nuke') || url.includes('nuke')) {
                dccInfo.dcc_type = 'nuke';
            } else if (title.includes('unreal') || url.includes('unreal')) {
                dccInfo.dcc_type = 'unreal';
            } else if (title.includes('3ds max') || url.includes('3dsmax')) {
                dccInfo.dcc_type = '3dsmax';
            }

            return dccInfo;
        }
    })()
    """)

    return result if result else {"error": "Failed to get DCC context", "dcc_type": None}


@mcp.tool()
async def execute_dcc_command(
    command: str,
    args: list[Any] | None = None,
    kwargs: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Execute a DCC native command.

    Executes commands in the DCC application's scripting environment.
    The command format depends on the DCC type:

    - Maya: "maya.cmds.ls", "maya.cmds.select", etc.
    - Blender: "bpy.ops.object.select_all", "bpy.data.objects", etc.
    - Houdini: "hou.node", "hou.selectedNodes", etc.
    - Nuke: "nuke.selectedNodes", "nuke.createNode", etc.

    Args:
        command: Command name with namespace (e.g., "maya.cmds.ls")
        args: Positional arguments for the command
        kwargs: Keyword arguments for the command

    Returns:
        Command execution result:
        - success: Whether the command executed successfully
        - result: Command return value (if any)
        - error: Error message (if failed)

    Raises:
        RuntimeError: If not connected to any instance or page.

    Examples:
        # Maya - list selected objects
        execute_dcc_command("maya.cmds.ls", kwargs={"selection": True})

        # Blender - select all objects
        execute_dcc_command("bpy.ops.object.select_all", kwargs={"action": "SELECT"})

        # Houdini - get selected nodes
        execute_dcc_command("hou.selectedNodes")
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    # Serialize arguments for JavaScript
    import json

    args_json = json.dumps(args or [])
    kwargs_json = json.dumps(kwargs or {})

    result = await page_conn.evaluate(f"""
    (async () => {{
        if (!window.auroraview || !window.auroraview.call) {{
            return {{
                success: false,
                error: 'AuroraView bridge not available'
            }};
        }}

        try {{
            const result = await window.auroraview.call('dcc.execute_command', {{
                command: '{command}',
                args: {args_json},
                kwargs: {kwargs_json}
            }});
            return {{
                success: true,
                result: result
            }};
        }} catch (e) {{
            return {{
                success: false,
                error: e.message
            }};
        }}
    }})()
    """)

    return result if result else {"success": False, "error": "Failed to execute command"}


@mcp.tool()
async def sync_selection() -> dict[str, Any]:
    """Synchronize selection state between DCC and WebView.

    Gets the current selection from both the DCC application and
    the WebView UI, allowing for comparison and synchronization.

    Returns:
        Selection state:
        - dcc_selection: Objects selected in the DCC application
        - webview_selection: Items selected in the WebView UI
        - synced: Whether selections are in sync
        - dcc_type: DCC application type

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    result = await page_conn.evaluate("""
    (async () => {
        if (!window.auroraview || !window.auroraview.call) {
            return {
                dcc_selection: [],
                webview_selection: [],
                synced: false,
                dcc_type: null,
                error: 'AuroraView bridge not available'
            };
        }

        try {
            const syncResult = await window.auroraview.call('dcc.sync_selection');
            return syncResult;
        } catch (e) {
            // Fallback: try to get selections separately
            let dccSelection = [];
            let webviewSelection = [];

            try {
                dccSelection = await window.auroraview.call('dcc.get_selection') || [];
            } catch {}

            try {
                // Try to get WebView UI selection from state
                if (window.auroraview.state && window.auroraview.state.get) {
                    webviewSelection = window.auroraview.state.get('selection') || [];
                }
            } catch {}

            return {
                dcc_selection: dccSelection,
                webview_selection: webviewSelection,
                synced: JSON.stringify(dccSelection) === JSON.stringify(webviewSelection),
                dcc_type: null,
                error: e.message
            };
        }
    })()
    """)

    return (
        result
        if result
        else {
            "dcc_selection": [],
            "webview_selection": [],
            "synced": False,
            "dcc_type": None,
            "error": "Failed to sync selection",
        }
    )


@mcp.tool()
async def set_dcc_selection(objects: list[str]) -> dict[str, Any]:
    """Set the selection in the DCC application.

    Selects the specified objects in the DCC application.

    Args:
        objects: List of object names/paths to select

    Returns:
        Selection result:
        - success: Whether selection was set successfully
        - selected: List of objects that were selected
        - error: Error message (if failed)

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    import json

    objects_json = json.dumps(objects)

    result = await page_conn.evaluate(f"""
    (async () => {{
        if (!window.auroraview || !window.auroraview.call) {{
            return {{
                success: false,
                selected: [],
                error: 'AuroraView bridge not available'
            }};
        }}

        try {{
            const result = await window.auroraview.call('dcc.set_selection', {{
                objects: {objects_json}
            }});
            return {{
                success: true,
                selected: result || {objects_json}
            }};
        }} catch (e) {{
            return {{
                success: false,
                selected: [],
                error: e.message
            }};
        }}
    }})()
    """)

    return (
        result if result else {"success": False, "selected": [], "error": "Failed to set selection"}
    )


@mcp.tool()
async def get_dcc_scene_info() -> dict[str, Any]:
    """Get detailed information about the current DCC scene.

    Retrieves comprehensive scene information including objects,
    materials, cameras, lights, and other scene elements.

    Returns:
        Scene information:
        - scene_path: Full path to scene file
        - scene_name: Scene file name
        - modified: Whether scene has unsaved changes
        - objects_count: Total number of objects
        - selected_count: Number of selected objects
        - cameras: List of cameras in scene
        - lights: List of lights in scene
        - materials: List of materials used
        - render_settings: Current render settings

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    result = await page_conn.evaluate("""
    (async () => {
        if (!window.auroraview || !window.auroraview.call) {
            return {
                error: 'AuroraView bridge not available'
            };
        }

        try {
            const sceneInfo = await window.auroraview.call('dcc.get_scene_info');
            return sceneInfo;
        } catch (e) {
            return {
                scene_path: null,
                scene_name: null,
                modified: false,
                objects_count: 0,
                selected_count: 0,
                cameras: [],
                lights: [],
                materials: [],
                render_settings: {},
                error: e.message
            };
        }
    })()
    """)

    return result if result else {"error": "Failed to get scene info"}


@mcp.tool()
async def get_dcc_timeline() -> dict[str, Any]:
    """Get timeline/animation information from the DCC application.

    Returns:
        Timeline information:
        - current_frame: Current frame number
        - start_frame: Animation start frame
        - end_frame: Animation end frame
        - fps: Frames per second
        - playing: Whether animation is playing
        - time_unit: Time unit (frames, seconds, etc.)

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    result = await page_conn.evaluate("""
    (async () => {
        if (!window.auroraview || !window.auroraview.call) {
            return {
                error: 'AuroraView bridge not available'
            };
        }

        try {
            const timeline = await window.auroraview.call('dcc.get_timeline');
            return timeline;
        } catch (e) {
            return {
                current_frame: null,
                start_frame: null,
                end_frame: null,
                fps: null,
                playing: false,
                time_unit: null,
                error: e.message
            };
        }
    })()
    """)

    return result if result else {"error": "Failed to get timeline info"}


@mcp.tool()
async def set_dcc_frame(frame: int) -> dict[str, Any]:
    """Set the current frame in the DCC application.

    Args:
        frame: Frame number to set

    Returns:
        Result:
        - success: Whether frame was set successfully
        - frame: The frame that was set
        - error: Error message (if failed)

    Raises:
        RuntimeError: If not connected to any instance or page.
    """
    manager = get_connection_manager()
    page_conn = await manager.get_page_connection()

    result = await page_conn.evaluate(f"""
    (async () => {{
        if (!window.auroraview || !window.auroraview.call) {{
            return {{
                success: false,
                error: 'AuroraView bridge not available'
            }};
        }}

        try {{
            await window.auroraview.call('dcc.set_frame', {{ frame: {frame} }});
            return {{
                success: true,
                frame: {frame}
            }};
        }} catch (e) {{
            return {{
                success: false,
                error: e.message
            }};
        }}
    }})()
    """)

    return result if result else {"success": False, "error": "Failed to set frame"}
