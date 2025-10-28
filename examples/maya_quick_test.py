#!/usr/bin/env python
"""
Maya Quick Test - Minimal Example for Testing show_async()

This is a minimal, copy-paste ready example for testing in Maya's Script Editor.

Usage:
1. Open Maya 2022
2. Open Script Editor (Windows > General Editors > Script Editor)
3. Switch to Python tab
4. Copy this entire script
5. Paste into the Python tab
6. Click "Execute" or press Ctrl+Enter
7. The WebView will open - Maya stays responsive!
"""

import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

try:
    from auroraview import WebView
    import maya.cmds as cmds
except ImportError as e:
    logger.error(f"Import error: {e}")
    raise

# Create WebView
webview = WebView(
    title="AuroraView - Quick Test",
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

# Create simple HTML UI
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
        <h1>🎨 AuroraView Quick Test</h1>
        
        <div class="section">
            <h2>Test Non-Blocking Mode</h2>
            <p>This WebView is running in a background thread.</p>
            <p><strong>✓ Maya is fully responsive!</strong></p>
            <p>Try:</p>
            <ul>
                <li>Create objects using the buttons below</li>
                <li>Switch to Maya and work normally</li>
                <li>The UI updates in real-time</li>
            </ul>
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
        
        <div class="section" style="text-align: center; color: #999;">
            <p>Close this window to finish</p>
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

# Load HTML and show
webview.load_html(html)

logger.info("=" * 70)
logger.info("Starting WebView in background thread...")
logger.info("=" * 70)

webview.show_async()

logger.info("")
logger.info("✓ WebView started!")
logger.info("✓ Maya is responsive!")
logger.info("")
logger.info("The WebView window should appear shortly.")
logger.info("You can now:")
logger.info("  • Use the UI to create objects")
logger.info("  • Continue working in Maya")
logger.info("  • Close the window when done")
logger.info("")

# Wait for window to close
webview.wait()

logger.info("")
logger.info("Test completed!")

