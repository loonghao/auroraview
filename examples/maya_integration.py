#!/usr/bin/env python
"""
Maya Integration Example

This example demonstrates how to integrate DCC WebView with Autodesk Maya.
It shows how to create a WebView panel that can interact with Maya's scene.

Usage in Maya:
    from examples import maya_integration
    maya_integration.create_maya_tool()
"""

import sys
import logging
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def create_maya_tool():
    """Create a DCC WebView tool for Maya.

    This example demonstrates how to use show_async() to prevent blocking
    Maya's main thread while the WebView is running.
    """
    logger.info("=" * 60)
    logger.info("AuroraView - Maya Integration Example")
    logger.info("=" * 60)
    logger.info("")

    # Create WebView
    logger.info("Creating Maya WebView tool...")
    webview = WebView(
        title="AuroraView - Maya Tool",
        width=600,
        height=500
    )
    logger.info(f"✓ Created: {webview}")
    logger.info("")
    
    # Create HTML UI
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Maya Tool</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: #2b2b2b;
                color: #e0e0e0;
                padding: 20px;
            }
            
            .header {
                background: #1a1a1a;
                padding: 15px;
                border-radius: 5px;
                margin-bottom: 20px;
                border-left: 4px solid #0066cc;
            }
            
            .header h1 {
                font-size: 1.5em;
                margin-bottom: 5px;
            }
            
            .header p {
                font-size: 0.9em;
                color: #999;
            }
            
            .section {
                background: #1a1a1a;
                padding: 15px;
                border-radius: 5px;
                margin-bottom: 15px;
            }
            
            .section h2 {
                font-size: 1.1em;
                margin-bottom: 10px;
                color: #0066cc;
            }
            
            .form-group {
                margin-bottom: 10px;
            }
            
            label {
                display: block;
                margin-bottom: 5px;
                font-size: 0.9em;
                color: #999;
            }
            
            input, select {
                width: 100%;
                padding: 8px;
                background: #333;
                border: 1px solid #444;
                color: #e0e0e0;
                border-radius: 3px;
                font-size: 0.9em;
            }
            
            input:focus, select:focus {
                outline: none;
                border-color: #0066cc;
                box-shadow: 0 0 5px rgba(0, 102, 204, 0.3);
            }
            
            .button-group {
                display: flex;
                gap: 10px;
                margin-top: 15px;
            }
            
            button {
                flex: 1;
                padding: 10px;
                background: #0066cc;
                color: white;
                border: none;
                border-radius: 3px;
                cursor: pointer;
                font-weight: 600;
                transition: all 0.2s;
            }
            
            button:hover {
                background: #0052a3;
            }
            
            button:active {
                transform: scale(0.98);
            }
            
            .status {
                background: #0a3a0a;
                border: 1px solid #0d5d0d;
                color: #90ee90;
                padding: 10px;
                border-radius: 3px;
                margin-top: 10px;
                font-size: 0.9em;
                font-family: monospace;
            }
            
            .status.error {
                background: #3a0a0a;
                border-color: #5d0d0d;
                color: #ff6b6b;
            }
            
            .info {
                background: #0a1a3a;
                border: 1px solid #0d2d5d;
                color: #87ceeb;
                padding: 10px;
                border-radius: 3px;
                font-size: 0.85em;
                margin-top: 10px;
            }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>🎨 Maya Scene Tool</h1>
            <p>Interact with your Maya scene using AuroraView</p>
        </div>
        
        <div class="section">
            <h2>Scene Information</h2>
            <div class="form-group">
                <label>Scene Name:</label>
                <input type="text" id="sceneName" placeholder="untitled" readonly>
            </div>
            <div class="form-group">
                <label>Selected Objects:</label>
                <input type="text" id="selectedObjects" placeholder="None" readonly>
            </div>
            <div class="info">
                💡 In a real Maya plugin, this would show actual scene data
            </div>
        </div>
        
        <div class="section">
            <h2>Object Operations</h2>
            <div class="form-group">
                <label>Object Name:</label>
                <input type="text" id="objectName" placeholder="Enter object name">
            </div>
            <div class="form-group">
                <label>Operation:</label>
                <select id="operation">
                    <option value="select">Select</option>
                    <option value="delete">Delete</option>
                    <option value="duplicate">Duplicate</option>
                    <option value="hide">Hide</option>
                </select>
            </div>
            <div class="button-group">
                <button onclick="executeOperation()">Execute</button>
                <button onclick="clearStatus()">Clear</button>
            </div>
            <div class="status" id="status">Ready</div>
        </div>
        
        <div class="section">
            <h2>Export Options</h2>
            <div class="form-group">
                <label>Export Format:</label>
                <select id="exportFormat">
                    <option value="fbx">FBX</option>
                    <option value="obj">OBJ</option>
                    <option value="abc">Alembic</option>
                    <option value="usd">USD</option>
                </select>
            </div>
            <div class="button-group">
                <button onclick="exportScene()">Export Scene</button>
            </div>
        </div>
        
        <script>
            function executeOperation() {
                const objectName = document.getElementById('objectName').value;
                const operation = document.getElementById('operation').value;
                const status = document.getElementById('status');
                
                if (!objectName) {
                    status.textContent = '❌ Error: Please enter an object name';
                    status.classList.add('error');
                    return;
                }
                
                status.textContent = `✓ Executing ${operation} on "${objectName}"...`;
                status.classList.remove('error');
                
                // In a real implementation, this would send a message to Python
                // which would then execute the Maya command
            }
            
            function exportScene() {
                const format = document.getElementById('exportFormat').value;
                const status = document.getElementById('status');
                
                status.textContent = `✓ Exporting scene as ${format.toUpperCase()}...`;
                status.classList.remove('error');
            }
            
            function clearStatus() {
                document.getElementById('status').textContent = 'Ready';
                document.getElementById('status').classList.remove('error');
            }
            
            // Initialize
            document.getElementById('sceneName').value = 'untitled.ma';
            document.getElementById('selectedObjects').value = 'pCube1, pSphere1';
        </script>
    </body>
    </html>
    """
    
    # Load HTML
    logger.info("Loading Maya tool UI...")
    webview.load_html(html_content)
    logger.info("✓ UI loaded")
    logger.info("")
    
    # Register event handlers
    logger.info("Registering event handlers...")
    
    @webview.on("execute_operation")
    def handle_execute_operation(data):
        """Handle object operation from UI."""
        logger.info(f"✓ Execute operation: {data}")
        # In a real Maya plugin, this would execute Maya commands
        # Example: cmds.select(data['object_name'])
    
    @webview.on("export_scene")
    def handle_export_scene(data):
        """Handle scene export from UI."""
        logger.info(f"✓ Export scene: {data}")
        # In a real Maya plugin, this would export the scene
        # Example: cmds.file(data['path'], save=True, type=data['format'])
    
    logger.info("✓ Event handlers registered")
    logger.info("")
    
    # Show the tool in background thread (non-blocking)
    logger.info("Showing Maya tool in background thread...")
    logger.info("The WebView will run without blocking Maya's main thread.")
    logger.info("")

    try:
        webview.show_async()
        logger.info("✓ WebView started in background thread")
        logger.info("")
        logger.info("Maya is now responsive. You can:")
        logger.info("  - Use the WebView UI")
        logger.info("  - Continue working in Maya")
        logger.info("  - Close the WebView window to finish")
        logger.info("")

        # Wait for the WebView to close
        # This allows the script to complete when the user closes the window
        webview.wait()

    except Exception as e:
        logger.error(f"Error showing WebView: {e}")
        return 1

    logger.info("")
    logger.info("=" * 60)
    logger.info("Maya tool closed.")
    logger.info("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(create_maya_tool())

