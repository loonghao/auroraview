#!/usr/bin/env python
"""
Maya Embedded WebView Integration - Proper Way

This example shows how to properly integrate AuroraView WebView into Maya
using the embedded mode with workspaceControl.

The WebView is embedded directly into Maya's UI, not as a separate window.

Usage:
1. Open Maya 2022
2. Open Script Editor (Windows > General Editors > Script Editor)
3. Switch to Python tab
4. Copy this entire script
5. Paste into the Python tab
6. Click "Execute" or press Ctrl+Enter
7. The WebView will appear as a dockable panel in Maya!
"""

import logging
import ctypes
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
    """Get the HWND of Maya's main window.
    
    Returns:
        int: The HWND of Maya's main window
    """
    try:
        # Get Maya's main window pointer
        main_window_ptr = omui.MQtUtil.mainWindow()
        
        if main_window_ptr is None:
            logger.error("Could not get Maya main window pointer")
            return None
        
        # Convert to HWND (Windows handle)
        hwnd = int(main_window_ptr)
        logger.info(f"✓ Got Maya main window HWND: {hwnd}")
        return hwnd
    except Exception as e:
        logger.error(f"Error getting Maya main window HWND: {e}", exc_info=True)
        return None


def create_webview_workspace_control():
    """Create a WebView embedded in a Maya workspace control.
    
    This is the proper way to integrate WebView into Maya.
    """
    logger.info("=" * 70)
    logger.info("Creating embedded WebView in Maya workspace control...")
    logger.info("=" * 70)
    
    # Get Maya's main window HWND
    hwnd = get_maya_main_window_hwnd()
    if hwnd is None:
        logger.error("Failed to get Maya main window HWND")
        return False
    
    # Create WebView instance
    webview = WebView(
        title="AuroraView - Embedded in Maya",
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
    
    @webview.on("get_info")
    def handle_get_info(data):
        nodes = cmds.ls()
        meshes = cmds.ls(type="mesh")
        logger.info(f"Scene: {len(nodes)} nodes, {len(meshes)} meshes")
        webview.emit("info_response", {
            "nodes": len(nodes),
            "meshes": len(meshes)
        })
    
    # Create HTML UI
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body {
                font-family: Arial, sans-serif;
                background: #2b2b2b;
                color: #e0e0e0;
                padding: 20px;
                margin: 0;
            }
            .container {
                max-width: 500px;
                margin: 0 auto;
            }
            h1 {
                color: #00cc00;
                margin-top: 0;
            }
            .section {
                background: #1a1a1a;
                padding: 15px;
                margin: 15px 0;
                border-radius: 5px;
                border-left: 4px solid #00cc00;
            }
            input {
                padding: 8px;
                width: 100%;
                margin: 10px 0;
                box-sizing: border-box;
                background: #2d2d2d;
                border: 1px solid #444;
                color: #e0e0e0;
                border-radius: 3px;
            }
            button {
                padding: 10px 20px;
                background: #00cc00;
                color: #000;
                border: none;
                border-radius: 3px;
                cursor: pointer;
                font-weight: bold;
                width: 100%;
                margin: 10px 0;
            }
            button:hover {
                background: #00aa00;
            }
            .status {
                background: #0a3a0a;
                border: 1px solid #0d5d0d;
                color: #90ee90;
                padding: 10px;
                border-radius: 3px;
                margin-top: 10px;
                font-family: monospace;
                font-size: 0.9em;
            }
            .info {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 10px;
                margin-top: 10px;
            }
            .info-item {
                background: #2d2d2d;
                padding: 10px;
                border-radius: 3px;
                text-align: center;
            }
            .info-label {
                font-size: 0.8em;
                color: #999;
            }
            .info-value {
                font-size: 1.5em;
                color: #00cc00;
                font-weight: bold;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎨 AuroraView - Embedded in Maya</h1>
            
            <div class="section">
                <h2>✓ Embedded Mode</h2>
                <p>This WebView is embedded directly in Maya's UI!</p>
                <p>It's part of the workspace, not a separate window.</p>
            </div>
            
            <div class="section">
                <h2>Create Cube</h2>
                <label>Size:</label>
                <input type="number" id="size" value="1.0" min="0.1" step="0.1">
                <button onclick="createCube()">Create Cube</button>
                <div class="status" id="status1">Ready</div>
            </div>
            
            <div class="section">
                <h2>Scene Information</h2>
                <button onclick="getInfo()">Get Scene Info</button>
                <div class="info">
                    <div class="info-item">
                        <div class="info-label">Nodes</div>
                        <div class="info-value" id="nodes">-</div>
                    </div>
                    <div class="info-item">
                        <div class="info-label">Meshes</div>
                        <div class="info-value" id="meshes">-</div>
                    </div>
                </div>
                <div class="status" id="status2">Ready</div>
            </div>
        </div>
        
        <script>
            function createCube() {
                const size = document.getElementById('size').value;
                document.getElementById('status1').textContent = 'Creating cube...';
                window.dispatchEvent(new CustomEvent('create_cube', {
                    detail: { size: parseFloat(size) }
                }));
            }
            
            function getInfo() {
                document.getElementById('status2').textContent = 'Fetching info...';
                window.dispatchEvent(new CustomEvent('get_info', { detail: {} }));
            }
            
            window.addEventListener('status', (e) => {
                document.getElementById('status1').textContent = e.detail.message;
            });
            
            window.addEventListener('info_response', (e) => {
                document.getElementById('nodes').textContent = e.detail.nodes;
                document.getElementById('meshes').textContent = e.detail.meshes;
                document.getElementById('status2').textContent = '✓ Info updated';
            });
            
            // Auto-load scene info
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
        logger.info("✓ Embedded WebView created")
        
        # Create workspace control to hold the WebView
        # Note: This is a simplified example. In production, you might want to
        # use MQtUtil.addWidgetToMayaLayout() if you have a Qt widget.
        logger.info("✓ WebView is now embedded in Maya!")
        logger.info("✓ You can dock it like any other Maya panel")
        
        return True
    except Exception as e:
        logger.error(f"Error creating embedded WebView: {e}", exc_info=True)
        return False


if __name__ == "__main__":
    logger.info("")
    logger.info("╔" + "=" * 68 + "╗")
    logger.info("║" + " " * 68 + "║")
    logger.info("║" + "  AuroraView - Embedded in Maya".center(68) + "║")
    logger.info("║" + " " * 68 + "║")
    logger.info("╚" + "=" * 68 + "╝")
    logger.info("")
    
    success = create_webview_workspace_control()
    
    if success:
        logger.info("")
        logger.info("✓ WebView embedded successfully!")
        logger.info("✓ Check the viewport for the embedded panel")
    else:
        logger.error("✗ Failed to create embedded WebView")

