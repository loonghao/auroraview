#!/usr/bin/env python
"""
Maya Test Tool - Complete Example for Testing show_async()

This is a complete, ready-to-use example that demonstrates the non-blocking
WebView functionality in Maya. You can copy this script directly into Maya's
Python console or save it as a file and execute it.

Features:
- Non-blocking WebView that doesn't freeze Maya
- Interactive UI with real-time updates
- Event communication between Maya and WebView
- Scene information display
- Cube creation with custom size

Usage in Maya:
1. Open Maya 2022
2. Open Script Editor (Windows > General Editors > Script Editor)
3. Copy this entire script into the Python tab
4. Click "Execute" or press Ctrl+Enter
5. The WebView window will open in the background
6. Maya remains fully responsive - you can continue working!
"""

import sys
import logging
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Try to import auroraview
try:
    from auroraview import WebView
except ImportError:
    logger.error("AuroraView not installed. Please install it first:")
    logger.error("  pip install auroraview")
    sys.exit(1)

# Try to import Maya
try:
    import maya.cmds as cmds
    import maya.api.OpenMaya as om
except ImportError:
    logger.error("This script must be run inside Maya")
    sys.exit(1)


def create_test_tool():
    """Create a complete test tool for Maya."""
    logger.info("=" * 80)
    logger.info("AuroraView - Maya Test Tool (Non-Blocking)")
    logger.info("=" * 80)
    logger.info("")
    
    # Create WebView
    logger.info("Creating WebView...")
    webview = WebView(
        title="AuroraView - Maya Test Tool",
        width=700,
        height=600
    )
    logger.info("✓ WebView created")
    logger.info("")
    
    # Register event handlers
    logger.info("Registering event handlers...")
    
    @webview.on("get_scene_info")
    def handle_get_scene_info(data):
        """Get current scene information."""
        try:
            nodes = cmds.ls()
            meshes = cmds.ls(type="mesh")
            cameras = cmds.ls(type="camera")
            lights = cmds.ls(type="light")
            
            info = {
                "total_nodes": len(nodes),
                "meshes": len(meshes),
                "cameras": len(cameras),
                "lights": len(lights),
                "status": "✓ Scene info retrieved"
            }
            
            logger.info(f"Scene info: {info}")
            webview.emit("scene_info_response", info)
        except Exception as e:
            logger.error(f"Error getting scene info: {e}")
            webview.emit("error", {"message": str(e)})
    
    @webview.on("create_cube")
    def handle_create_cube(data):
        """Create a cube in Maya."""
        try:
            size = float(data.get("size", 1.0))
            name = data.get("name", "pCube")
            
            # Create cube
            cube = cmds.polyCube(w=size, h=size, d=size, name=name)
            logger.info(f"✓ Created cube: {cube[0]} (size: {size})")
            
            # Get scene info after creation
            nodes = cmds.ls()
            webview.emit("cube_created", {
                "cube_name": cube[0],
                "size": size,
                "total_nodes": len(nodes),
                "status": f"✓ Cube '{cube[0]}' created with size {size}"
            })
        except Exception as e:
            logger.error(f"Error creating cube: {e}")
            webview.emit("error", {"message": str(e)})
    
    @webview.on("create_sphere")
    def handle_create_sphere(data):
        """Create a sphere in Maya."""
        try:
            radius = float(data.get("radius", 1.0))
            name = data.get("name", "pSphere")
            
            # Create sphere
            sphere = cmds.polySphere(r=radius, name=name)
            logger.info(f"✓ Created sphere: {sphere[0]} (radius: {radius})")
            
            nodes = cmds.ls()
            webview.emit("sphere_created", {
                "sphere_name": sphere[0],
                "radius": radius,
                "total_nodes": len(nodes),
                "status": f"✓ Sphere '{sphere[0]}' created with radius {radius}"
            })
        except Exception as e:
            logger.error(f"Error creating sphere: {e}")
            webview.emit("error", {"message": str(e)})
    
    @webview.on("delete_selected")
    def handle_delete_selected(data):
        """Delete selected objects."""
        try:
            selected = cmds.ls(selection=True)
            if selected:
                cmds.delete(selected)
                logger.info(f"✓ Deleted {len(selected)} object(s)")
                webview.emit("objects_deleted", {
                    "deleted_count": len(selected),
                    "status": f"✓ Deleted {len(selected)} object(s)"
                })
            else:
                webview.emit("warning", {"message": "No objects selected"})
        except Exception as e:
            logger.error(f"Error deleting objects: {e}")
            webview.emit("error", {"message": str(e)})
    
    @webview.on("clear_scene")
    def handle_clear_scene(data):
        """Clear the entire scene."""
        try:
            cmds.select(all=True)
            cmds.delete()
            cmds.select(clear=True)
            logger.info("✓ Scene cleared")
            webview.emit("scene_cleared", {
                "status": "✓ Scene cleared successfully"
            })
        except Exception as e:
            logger.error(f"Error clearing scene: {e}")
            webview.emit("error", {"message": str(e)})
    
    logger.info("✓ Event handlers registered")
    logger.info("")
    
    # Create HTML UI
    logger.info("Loading UI...")
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>AuroraView - Maya Test Tool</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
                color: #e0e0e0;
                padding: 20px;
                min-height: 100vh;
            }
            
            .container {
                max-width: 600px;
                margin: 0 auto;
            }
            
            .header {
                background: linear-gradient(135deg, #00cc00 0%, #00aa00 100%);
                color: #000;
                padding: 20px;
                border-radius: 8px;
                margin-bottom: 20px;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
            }
            
            .header h1 {
                font-size: 1.8em;
                margin-bottom: 5px;
            }
            
            .header p {
                font-size: 0.9em;
                opacity: 0.9;
            }
            
            .section {
                background: #1a1a1a;
                border: 1px solid #333;
                padding: 15px;
                border-radius: 6px;
                margin-bottom: 15px;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
            }
            
            .section h2 {
                font-size: 1.1em;
                margin-bottom: 12px;
                color: #00cc00;
                border-bottom: 2px solid #00cc00;
                padding-bottom: 8px;
            }
            
            .input-group {
                margin-bottom: 12px;
                display: flex;
                gap: 8px;
            }
            
            input[type="number"], input[type="text"] {
                flex: 1;
                padding: 8px 12px;
                background: #2d2d2d;
                border: 1px solid #444;
                color: #e0e0e0;
                border-radius: 4px;
                font-size: 0.9em;
            }
            
            input[type="number"]:focus, input[type="text"]:focus {
                outline: none;
                border-color: #00cc00;
                box-shadow: 0 0 8px rgba(0, 204, 0, 0.3);
            }
            
            .button-group {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 8px;
                margin-bottom: 12px;
            }
            
            button {
                padding: 10px 16px;
                background: #00cc00;
                color: #000;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-weight: 600;
                font-size: 0.9em;
                transition: all 0.2s;
            }
            
            button:hover {
                background: #00aa00;
                transform: translateY(-2px);
                box-shadow: 0 4px 8px rgba(0, 204, 0, 0.3);
            }
            
            button:active {
                transform: translateY(0);
            }
            
            button.danger {
                background: #cc0000;
                grid-column: 1 / -1;
            }
            
            button.danger:hover {
                background: #aa0000;
                box-shadow: 0 4px 8px rgba(204, 0, 0, 0.3);
            }
            
            .status-box {
                background: #0a3a0a;
                border: 1px solid #0d5d0d;
                color: #90ee90;
                padding: 12px;
                border-radius: 4px;
                font-size: 0.85em;
                font-family: monospace;
                margin-top: 10px;
                max-height: 150px;
                overflow-y: auto;
            }
            
            .status-box.error {
                background: #3a0a0a;
                border-color: #5d0d0d;
                color: #ff6b6b;
            }
            
            .status-box.warning {
                background: #3a3a0a;
                border-color: #5d5d0d;
                color: #ffff6b;
            }
            
            .info-grid {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 10px;
                margin-top: 10px;
            }
            
            .info-item {
                background: #2d2d2d;
                padding: 10px;
                border-radius: 4px;
                border-left: 3px solid #00cc00;
            }
            
            .info-label {
                font-size: 0.8em;
                color: #999;
                margin-bottom: 4px;
            }
            
            .info-value {
                font-size: 1.2em;
                color: #00cc00;
                font-weight: 600;
            }
            
            .footer {
                text-align: center;
                font-size: 0.8em;
                color: #666;
                margin-top: 20px;
                padding-top: 15px;
                border-top: 1px solid #333;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🎨 AuroraView - Maya Test Tool</h1>
                <p>Non-blocking WebView Integration Demo</p>
            </div>
            
            <div class="section">
                <h2>📊 Scene Information</h2>
                <button onclick="getSceneInfo()" style="width: 100%; margin-bottom: 10px;">
                    Refresh Scene Info
                </button>
                <div class="info-grid">
                    <div class="info-item">
                        <div class="info-label">Total Nodes</div>
                        <div class="info-value" id="nodeCount">-</div>
                    </div>
                    <div class="info-item">
                        <div class="info-label">Meshes</div>
                        <div class="info-value" id="meshCount">-</div>
                    </div>
                    <div class="info-item">
                        <div class="info-label">Cameras</div>
                        <div class="info-value" id="cameraCount">-</div>
                    </div>
                    <div class="info-item">
                        <div class="info-label">Lights</div>
                        <div class="info-value" id="lightCount">-</div>
                    </div>
                </div>
                <div class="status-box" id="sceneStatus">Ready</div>
            </div>
            
            <div class="section">
                <h2>🎲 Create Objects</h2>
                
                <div>
                    <label style="display: block; margin-bottom: 8px; font-size: 0.9em;">Cube Size:</label>
                    <div class="input-group">
                        <input type="number" id="cubeSize" value="1.0" min="0.1" step="0.1">
                        <button onclick="createCube()">Create Cube</button>
                    </div>
                </div>
                
                <div>
                    <label style="display: block; margin-bottom: 8px; font-size: 0.9em;">Sphere Radius:</label>
                    <div class="input-group">
                        <input type="number" id="sphereRadius" value="1.0" min="0.1" step="0.1">
                        <button onclick="createSphere()">Create Sphere</button>
                    </div>
                </div>
                
                <div class="status-box" id="createStatus">Ready to create objects</div>
            </div>
            
            <div class="section">
                <h2>🗑️ Scene Management</h2>
                <div class="button-group">
                    <button onclick="deleteSelected()">Delete Selected</button>
                    <button class="danger" onclick="clearScene()">Clear Scene</button>
                </div>
                <div class="status-box" id="deleteStatus">Ready</div>
            </div>
            
            <div class="section">
                <h2>ℹ️ Information</h2>
                <p style="font-size: 0.9em; line-height: 1.6;">
                    <strong>✓ Non-blocking:</strong> Maya remains responsive while this window is open<br>
                    <strong>✓ Real-time:</strong> All changes are reflected immediately<br>
                    <strong>✓ Thread-safe:</strong> WebView runs in a background thread<br>
                    <strong>✓ Interactive:</strong> Full communication with Maya
                </p>
            </div>
            
            <div class="footer">
                <p>AuroraView - Rust-powered WebView for Python apps & DCC embedding</p>
                <p>Close this window to finish</p>
            </div>
        </div>
        
        <script>
            function getSceneInfo() {
                updateStatus('sceneStatus', 'Fetching scene info...');
                window.dispatchEvent(new CustomEvent('get_scene_info', { detail: {} }));
            }
            
            function createCube() {
                const size = document.getElementById('cubeSize').value;
                updateStatus('createStatus', `Creating cube with size ${size}...`);
                window.dispatchEvent(new CustomEvent('create_cube', {
                    detail: { size: parseFloat(size), name: 'testCube' }
                }));
            }
            
            function createSphere() {
                const radius = document.getElementById('sphereRadius').value;
                updateStatus('createStatus', `Creating sphere with radius ${radius}...`);
                window.dispatchEvent(new CustomEvent('create_sphere', {
                    detail: { radius: parseFloat(radius), name: 'testSphere' }
                }));
            }
            
            function deleteSelected() {
                updateStatus('deleteStatus', 'Deleting selected objects...');
                window.dispatchEvent(new CustomEvent('delete_selected', { detail: {} }));
            }
            
            function clearScene() {
                if (confirm('Are you sure you want to clear the entire scene?')) {
                    updateStatus('deleteStatus', 'Clearing scene...');
                    window.dispatchEvent(new CustomEvent('clear_scene', { detail: {} }));
                }
            }
            
            function updateStatus(elementId, message) {
                const element = document.getElementById(elementId);
                if (element) {
                    element.textContent = message;
                }
            }
            
            // Listen for responses from Maya
            window.addEventListener('scene_info_response', (e) => {
                const data = e.detail;
                document.getElementById('nodeCount').textContent = data.total_nodes;
                document.getElementById('meshCount').textContent = data.meshes;
                document.getElementById('cameraCount').textContent = data.cameras;
                document.getElementById('lightCount').textContent = data.lights;
                updateStatus('sceneStatus', data.status);
            });
            
            window.addEventListener('cube_created', (e) => {
                const data = e.detail;
                updateStatus('createStatus', data.status);
            });
            
            window.addEventListener('sphere_created', (e) => {
                const data = e.detail;
                updateStatus('createStatus', data.status);
            });
            
            window.addEventListener('objects_deleted', (e) => {
                const data = e.detail;
                updateStatus('deleteStatus', data.status);
            });
            
            window.addEventListener('scene_cleared', (e) => {
                const data = e.detail;
                updateStatus('deleteStatus', data.status);
            });
            
            window.addEventListener('error', (e) => {
                const data = e.detail;
                const statusBox = document.querySelector('.status-box');
                statusBox.textContent = '❌ Error: ' + data.message;
                statusBox.classList.add('error');
            });
            
            window.addEventListener('warning', (e) => {
                const data = e.detail;
                const statusBox = document.querySelector('.status-box');
                statusBox.textContent = '⚠️ Warning: ' + data.message;
                statusBox.classList.add('warning');
            });
            
            // Initial scene info on load
            window.addEventListener('load', () => {
                getSceneInfo();
            });
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html_content)
    logger.info("✓ UI loaded")
    logger.info("")
    
    # Show the tool in background thread (non-blocking)
    logger.info("Starting WebView in background thread...")
    logger.info("")
    
    try:
        webview.show_async()
        logger.info("=" * 80)
        logger.info("✓ WebView started in background thread!")
        logger.info("=" * 80)
        logger.info("")
        logger.info("🎉 SUCCESS! Maya is now responsive!")
        logger.info("")
        logger.info("You can now:")
        logger.info("  • Use the WebView UI to interact with Maya")
        logger.info("  • Continue working in Maya normally")
        logger.info("  • Create cubes and spheres from the UI")
        logger.info("  • View scene information in real-time")
        logger.info("  • Close the WebView window when done")
        logger.info("")
        logger.info("=" * 80)
        
        # Wait for the WebView to close
        webview.wait()
        
    except Exception as e:
        logger.error(f"Error showing WebView: {e}", exc_info=True)
        return 1
    
    logger.info("")
    logger.info("=" * 80)
    logger.info("Maya Test Tool closed.")
    logger.info("=" * 80)
    
    return 0


if __name__ == "__main__":
    sys.exit(create_test_tool())

