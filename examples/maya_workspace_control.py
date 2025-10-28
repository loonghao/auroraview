#!/usr/bin/env python
"""
Maya Workspace Control with WebView - Advanced Integration

This example shows how to create a proper Maya workspace control
that contains an embedded WebView.

This is the recommended approach for integrating WebView into Maya.

Usage:
1. Open Maya 2022
2. Open Script Editor (Windows > General Editors > Script Editor)
3. Switch to Python tab
4. Copy this entire script
5. Paste into the Python tab
6. Click "Execute" or press Ctrl+Enter
7. A new dockable panel will appear!
"""

import logging
import maya.cmds as cmds
import maya.OpenMayaUI as omui

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

try:
    from auroraview import WebView
except ImportError as e:
    logger.error(f"Import error: {e}")
    raise


def get_maya_main_window_hwnd():
    """Get the HWND of Maya's main window."""
    try:
        main_window_ptr = omui.MQtUtil.mainWindow()
        if main_window_ptr is None:
            logger.error("Could not get Maya main window pointer")
            return None
        hwnd = int(main_window_ptr)
        logger.info(f"✓ Got Maya main window HWND: {hwnd}")
        return hwnd
    except Exception as e:
        logger.error(f"Error getting Maya main window HWND: {e}", exc_info=True)
        return None


def create_auroraview_workspace():
    """Create a workspace control with embedded WebView."""
    
    logger.info("=" * 70)
    logger.info("Creating AuroraView Workspace Control...")
    logger.info("=" * 70)
    
    # Get Maya's main window HWND
    hwnd = get_maya_main_window_hwnd()
    if hwnd is None:
        logger.error("Failed to get Maya main window HWND")
        return False
    
    # Create WebView instance
    webview = WebView(
        title="AuroraView Workspace",
        width=600,
        height=500
    )
    
    # Register event handlers
    @webview.on("create_cube")
    def handle_create_cube(data):
        size = float(data.get("size", 1.0))
        cube = cmds.polyCube(w=size, h=size, d=size)
        logger.info(f"✓ Created cube: {cube[0]}")
        webview.emit("status", {"message": f"✓ Created cube: {cube[0]}"})
    
    @webview.on("create_sphere")
    def handle_create_sphere(data):
        radius = float(data.get("radius", 1.0))
        sphere = cmds.polySphere(r=radius)
        logger.info(f"✓ Created sphere: {sphere[0]}")
        webview.emit("status", {"message": f"✓ Created sphere: {sphere[0]}"})
    
    @webview.on("delete_selected")
    def handle_delete_selected(data):
        selected = cmds.ls(selection=True)
        if selected:
            cmds.delete(selected)
            logger.info(f"✓ Deleted {len(selected)} objects")
            webview.emit("status", {"message": f"✓ Deleted {len(selected)} objects"})
        else:
            webview.emit("status", {"message": "No objects selected"})
    
    @webview.on("get_info")
    def handle_get_info(data):
        nodes = cmds.ls()
        meshes = cmds.ls(type="mesh")
        cameras = cmds.ls(type="camera")
        lights = cmds.ls(type="light")
        logger.info(f"Scene: {len(nodes)} nodes, {len(meshes)} meshes")
        webview.emit("info_response", {
            "nodes": len(nodes),
            "meshes": len(meshes),
            "cameras": len(cameras),
            "lights": len(lights)
        })
    
    # Create HTML UI
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            * { box-sizing: border-box; }
            body {
                font-family: Arial, sans-serif;
                background: #2b2b2b;
                color: #e0e0e0;
                padding: 15px;
                margin: 0;
            }
            h1 {
                color: #00cc00;
                margin: 0 0 15px 0;
                font-size: 1.2em;
            }
            .section {
                background: #1a1a1a;
                padding: 12px;
                margin: 10px 0;
                border-radius: 4px;
                border-left: 3px solid #00cc00;
            }
            .section h2 {
                margin: 0 0 10px 0;
                font-size: 0.95em;
                color: #00cc00;
            }
            input {
                padding: 6px;
                width: 100%;
                margin: 8px 0;
                background: #2d2d2d;
                border: 1px solid #444;
                color: #e0e0e0;
                border-radius: 3px;
            }
            button {
                padding: 8px 15px;
                background: #00cc00;
                color: #000;
                border: none;
                border-radius: 3px;
                cursor: pointer;
                font-weight: bold;
                width: 100%;
                margin: 8px 0;
                font-size: 0.9em;
            }
            button:hover {
                background: #00aa00;
            }
            .status {
                background: #0a3a0a;
                border: 1px solid #0d5d0d;
                color: #90ee90;
                padding: 8px;
                border-radius: 3px;
                margin-top: 8px;
                font-family: monospace;
                font-size: 0.85em;
            }
            .info {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 8px;
                margin-top: 8px;
            }
            .info-item {
                background: #2d2d2d;
                padding: 8px;
                border-radius: 3px;
                text-align: center;
            }
            .info-label {
                font-size: 0.75em;
                color: #999;
            }
            .info-value {
                font-size: 1.3em;
                color: #00cc00;
                font-weight: bold;
            }
        </style>
    </head>
    <body>
        <h1>🎨 AuroraView</h1>
        
        <div class="section">
            <h2>Create Objects</h2>
            <label>Cube Size:</label>
            <input type="number" id="cubeSize" value="1.0" min="0.1" step="0.1">
            <button onclick="createCube()">Create Cube</button>
            
            <label>Sphere Radius:</label>
            <input type="number" id="sphereRadius" value="1.0" min="0.1" step="0.1">
            <button onclick="createSphere()">Create Sphere</button>
            
            <button onclick="deleteSelected()" style="background: #cc0000;">Delete Selected</button>
            <div class="status" id="status1">Ready</div>
        </div>
        
        <div class="section">
            <h2>Scene Info</h2>
            <button onclick="getInfo()">Refresh Info</button>
            <div class="info">
                <div class="info-item">
                    <div class="info-label">Nodes</div>
                    <div class="info-value" id="nodes">-</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Meshes</div>
                    <div class="info-value" id="meshes">-</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Cameras</div>
                    <div class="info-value" id="cameras">-</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Lights</div>
                    <div class="info-value" id="lights">-</div>
                </div>
            </div>
            <div class="status" id="status2">Ready</div>
        </div>
        
        <script>
            function createCube() {
                const size = document.getElementById('cubeSize').value;
                document.getElementById('status1').textContent = 'Creating cube...';
                window.dispatchEvent(new CustomEvent('create_cube', {
                    detail: { size: parseFloat(size) }
                }));
            }
            
            function createSphere() {
                const radius = document.getElementById('sphereRadius').value;
                document.getElementById('status1').textContent = 'Creating sphere...';
                window.dispatchEvent(new CustomEvent('create_sphere', {
                    detail: { radius: parseFloat(radius) }
                }));
            }
            
            function deleteSelected() {
                document.getElementById('status1').textContent = 'Deleting...';
                window.dispatchEvent(new CustomEvent('delete_selected', { detail: {} }));
            }
            
            function getInfo() {
                document.getElementById('status2').textContent = 'Fetching...';
                window.dispatchEvent(new CustomEvent('get_info', { detail: {} }));
            }
            
            window.addEventListener('status', (e) => {
                document.getElementById('status1').textContent = e.detail.message;
            });
            
            window.addEventListener('info_response', (e) => {
                document.getElementById('nodes').textContent = e.detail.nodes;
                document.getElementById('meshes').textContent = e.detail.meshes;
                document.getElementById('cameras').textContent = e.detail.cameras;
                document.getElementById('lights').textContent = e.detail.lights;
                document.getElementById('status2').textContent = '✓ Updated';
            });
            
            // Auto-load scene info on startup
            window.addEventListener('load', () => {
                getInfo();
            });
        </script>
    </body>
    </html>
    """
    
    # Load HTML
    webview.load_html(html)
    logger.info("✓ HTML loaded")
    
    try:
        # Create embedded WebView
        logger.info(f"Creating embedded WebView with HWND: {hwnd}")
        webview._core.create_embedded(hwnd, 600, 500)
        logger.info("✓ Embedded WebView created successfully!")
        logger.info("")
        logger.info("✓ WebView is now embedded in Maya!")
        logger.info("✓ You can use it like any other Maya panel")
        logger.info("✓ Try creating objects and checking scene info")
        
        return True
    except Exception as e:
        logger.error(f"Error creating embedded WebView: {e}", exc_info=True)
        return False


if __name__ == "__main__":
    logger.info("")
    logger.info("╔" + "=" * 68 + "╗")
    logger.info("║" + " " * 68 + "║")
    logger.info("║" + "  AuroraView - Workspace Control Integration".center(68) + "║")
    logger.info("║" + " " * 68 + "║")
    logger.info("╚" + "=" * 68 + "╝")
    logger.info("")
    
    success = create_auroraview_workspace()
    
    if success:
        logger.info("")
        logger.info("=" * 70)
        logger.info("✓ SUCCESS! WebView is now embedded in Maya")
        logger.info("=" * 70)
    else:
        logger.error("✗ Failed to create embedded WebView")

