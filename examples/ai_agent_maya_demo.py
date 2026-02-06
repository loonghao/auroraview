"""
AI Agent Sidebar Demo for Maya

This example demonstrates how to create an AI-powered sidebar in Maya
that can control the DCC application using natural language commands.

The AI automatically discovers all bound APIs and can:
- Export scenes in various formats
- Manage selections
- Create and assign materials
- Control the viewport
- And more...

Usage:
    # In Maya Script Editor (Python)
    import sys
    sys.path.insert(0, "/path/to/auroraview")

    from examples.ai_agent_maya_demo import create_ai_tool_panel
    create_ai_tool_panel()

Requirements:
    - AuroraView with Qt support: pip install auroraview[qt]
    - OpenAI API key: export OPENAI_API_KEY=your-key
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def create_ai_tool_panel():
    """Create an AI-powered tool panel in Maya."""
    try:
        import maya.cmds as cmds
    except ImportError:
        logger.error("This example must be run inside Maya")
        return None

    from auroraview import QtWebView
    from auroraview.ai import AIAgent, AIConfig
    from auroraview.ai.dcc_tools import DCCToolCategory, dcc_tool
    from auroraview.integration.qt import get_maya_main_window

    # Get Maya main window
    maya_window = get_maya_main_window()

    # Create WebView with a simple UI
    webview = QtWebView(
        parent=maya_window,
        title="AI Assistant",
        width=400,
        height=600,
    )

    # ========================================
    # Bind Maya APIs as AI-callable tools
    # ========================================

    @webview.bind_call("scene.get_info")
    @dcc_tool(category=DCCToolCategory.SCENE, description="Get current scene information")
    def get_scene_info() -> Dict[str, Any]:
        """Get information about the current Maya scene."""
        scene_name = cmds.file(q=True, sceneName=True) or "Untitled"
        modified = cmds.file(q=True, modified=True)
        return {
            "name": scene_name,
            "modified": modified,
            "fps": cmds.currentUnit(q=True, time=True),
            "frame_range": [
                cmds.playbackOptions(q=True, min=True),
                cmds.playbackOptions(q=True, max=True),
            ],
        }

    @webview.bind_call("scene.export")
    @dcc_tool(
        category=DCCToolCategory.SCENE,
        confirm=True,
        description="Export the current scene to a file",
    )
    def export_scene(
        format: str = "fbx",
        path: str = "",
        selected_only: bool = False,
    ) -> Dict[str, Any]:
        """Export the Maya scene to the specified format.

        Args:
            format: Export format (fbx, obj, abc, ma, mb)
            path: Output file path (auto-generated if empty)
            selected_only: Export only selected objects
        """
        if not path:
            import tempfile

            path = f"{tempfile.gettempdir()}/maya_export.{format}"

        export_type = {
            "fbx": "FBX export",
            "obj": "OBJexport",
            "abc": "Alembic",
            "ma": "mayaAscii",
            "mb": "mayaBinary",
        }.get(format, "FBX export")

        if selected_only:
            cmds.file(path, exportSelected=True, type=export_type, force=True)
        else:
            cmds.file(path, exportAll=True, type=export_type, force=True)

        return {"success": True, "path": path, "format": format}

    @webview.bind_call("selection.get")
    @dcc_tool(category=DCCToolCategory.SELECTION, description="Get selected objects")
    def get_selection() -> List[str]:
        """Get the list of currently selected objects in Maya."""
        return cmds.ls(selection=True) or []

    @webview.bind_call("selection.select")
    @dcc_tool(category=DCCToolCategory.SELECTION, description="Select objects by name")
    def select_objects(names: List[str], add: bool = False) -> Dict[str, Any]:
        """Select objects by their names.

        Args:
            names: List of object names to select
            add: Add to current selection instead of replacing
        """
        if add:
            cmds.select(names, add=True)
        else:
            cmds.select(names, replace=True)
        return {"selected": cmds.ls(selection=True)}

    @webview.bind_call("selection.clear")
    @dcc_tool(category=DCCToolCategory.SELECTION, description="Clear selection")
    def clear_selection() -> Dict[str, Any]:
        """Clear the current selection."""
        cmds.select(clear=True)
        return {"success": True}

    @webview.bind_call("objects.list")
    @dcc_tool(category=DCCToolCategory.SCENE, description="List objects in scene")
    def list_objects(
        type: Optional[str] = None,
        pattern: str = "*",
    ) -> List[str]:
        """List objects in the scene.

        Args:
            type: Object type filter (mesh, camera, light, etc.)
            pattern: Name pattern filter (supports wildcards)
        """
        kwargs = {"long": False}
        if type:
            kwargs["type"] = type
        return cmds.ls(pattern, **kwargs) or []

    @webview.bind_call("transform.move")
    @dcc_tool(
        category=DCCToolCategory.TRANSFORM,
        requires_selection=True,
        description="Move selected objects",
    )
    def move_objects(
        x: float = 0, y: float = 0, z: float = 0, relative: bool = True
    ) -> Dict[str, Any]:
        """Move selected objects.

        Args:
            x: X translation
            y: Y translation
            z: Z translation
            relative: Move relative to current position
        """
        cmds.move(x, y, z, relative=relative)
        return {"success": True, "translation": [x, y, z]}

    @webview.bind_call("material.create")
    @dcc_tool(category=DCCToolCategory.MATERIAL, description="Create a new material")
    def create_material(
        name: str,
        type: str = "lambert",
        color: Optional[List[float]] = None,
    ) -> Dict[str, Any]:
        """Create a new material.

        Args:
            name: Material name
            type: Material type (lambert, blinn, phong, standardSurface)
            color: RGB color values (0-1 range)
        """
        shader = cmds.shadingNode(type, asShader=True, name=name)
        shading_group = cmds.sets(
            renderable=True, noSurfaceShader=True, empty=True, name=f"{name}SG"
        )
        cmds.connectAttr(f"{shader}.outColor", f"{shading_group}.surfaceShader")

        if color:
            cmds.setAttr(f"{shader}.color", *color, type="double3")

        return {"shader": shader, "shading_group": shading_group}

    @webview.bind_call("material.assign")
    @dcc_tool(
        category=DCCToolCategory.MATERIAL,
        requires_selection=True,
        description="Assign material to selected objects",
    )
    def assign_material(material_name: str) -> Dict[str, Any]:
        """Assign a material to selected objects.

        Args:
            material_name: Name of the material to assign
        """
        selection = cmds.ls(selection=True)
        if not selection:
            return {"success": False, "error": "No objects selected"}

        shading_group = f"{material_name}SG"
        if not cmds.objExists(shading_group):
            return {"success": False, "error": f"Material {material_name} not found"}

        cmds.sets(selection, edit=True, forceElement=shading_group)
        return {"success": True, "assigned_to": selection}

    @webview.bind_call("render.preview")
    @dcc_tool(category=DCCToolCategory.RENDER, description="Render a preview image")
    def render_preview(width: int = 960, height: int = 540) -> Dict[str, Any]:
        """Render a preview image of the current view.

        Args:
            width: Image width in pixels
            height: Image height in pixels
        """
        import tempfile

        output_path = f"{tempfile.gettempdir()}/maya_preview.png"

        cmds.setAttr("defaultRenderGlobals.imageFormat", 32)  # PNG
        cmds.playblast(
            frame=cmds.currentTime(q=True),
            format="image",
            filename=output_path,
            width=width,
            height=height,
            percent=100,
            viewer=False,
        )
        return {"success": True, "path": output_path}

    # ========================================
    # Create AI Agent with auto-discovery
    # ========================================

    # DCC-specific system prompt
    maya_system_prompt = """You are an AI assistant integrated into Autodesk Maya.
You can help artists with:
- Scene management (open, save, export)
- Object selection and manipulation
- Material creation and assignment
- Rendering and previews
- Animation and keyframing

When the user asks you to do something, use the available tools to accomplish the task.
Always confirm destructive operations before executing them.
If you're unsure about something, ask for clarification.

Available tool categories:
- scene.*: Scene operations
- selection.*: Selection management
- objects.*: Object queries
- transform.*: Object transformations
- material.*: Material operations
- render.*: Rendering operations
"""

    # Create AI Agent
    agent = AIAgent.as_sidebar(
        webview,
        config=AIConfig.openai().with_system_prompt(maya_system_prompt).with_temperature(0.7),
        auto_discover_apis=True,  # Automatically discovers all bound tools
    )

    # Show the panel
    webview.show()

    logger.info("AI Agent sidebar created with %d tools", len(agent.tools.all()))
    return webview


if __name__ == "__main__":
    # For testing outside Maya
    print("This example should be run inside Maya.")
    print("See the docstring for usage instructions.")
