#!/usr/bin/env python
"""
Houdini Integration Example

This example demonstrates how to integrate DCC WebView with SideFX Houdini.
It shows how to create a WebView panel that can interact with Houdini's node graph.

Usage in Houdini:
    from examples import houdini_integration
    houdini_integration.create_houdini_tool()
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


def create_houdini_tool():
    """Create a DCC WebView tool for Houdini."""
    logger.info("=" * 60)
    logger.info("AuroraView - Houdini Integration Example")
    logger.info("=" * 60)
    logger.info("")
    
    # Create WebView
    logger.info("Creating Houdini WebView tool...")
    webview = WebView(
        title="AuroraView - Houdini Tool",
        width=700,
        height=600
    )
    logger.info(f"✓ Created: {webview}")
    logger.info("")
    
    # Create HTML UI
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Houdini Tool</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: 'Courier New', monospace;
                background: #1a1a1a;
                color: #d0d0d0;
                padding: 15px;
            }
            
            .header {
                background: linear-gradient(135deg, #ff6b35 0%, #f7931e 100%);
                color: white;
                padding: 15px;
                border-radius: 5px;
                margin-bottom: 15px;
            }
            
            .header h1 {
                font-size: 1.4em;
                margin-bottom: 5px;
            }
            
            .section {
                background: #252525;
                border: 1px solid #333;
                padding: 12px;
                border-radius: 3px;
                margin-bottom: 12px;
            }
            
            .section h2 {
                font-size: 1em;
                margin-bottom: 10px;
                color: #ff6b35;
                border-bottom: 1px solid #333;
                padding-bottom: 5px;
            }
            
            .node-list {
                background: #1a1a1a;
                border: 1px solid #333;
                border-radius: 3px;
                max-height: 150px;
                overflow-y: auto;
                margin-bottom: 10px;
            }
            
            .node-item {
                padding: 8px;
                border-bottom: 1px solid #333;
                cursor: pointer;
                transition: background 0.2s;
            }
            
            .node-item:hover {
                background: #333;
            }
            
            .node-item.selected {
                background: #ff6b35;
                color: white;
            }
            
            .form-group {
                margin-bottom: 10px;
            }
            
            label {
                display: block;
                margin-bottom: 4px;
                font-size: 0.85em;
                color: #999;
            }
            
            input, select, textarea {
                width: 100%;
                padding: 6px;
                background: #1a1a1a;
                border: 1px solid #333;
                color: #d0d0d0;
                border-radius: 2px;
                font-family: 'Courier New', monospace;
                font-size: 0.85em;
            }
            
            input:focus, select:focus, textarea:focus {
                outline: none;
                border-color: #ff6b35;
                box-shadow: 0 0 5px rgba(255, 107, 53, 0.3);
            }
            
            .button-group {
                display: flex;
                gap: 8px;
                margin-top: 10px;
            }
            
            button {
                flex: 1;
                padding: 8px;
                background: #ff6b35;
                color: white;
                border: none;
                border-radius: 2px;
                cursor: pointer;
                font-weight: 600;
                font-size: 0.9em;
                transition: all 0.2s;
            }
            
            button:hover {
                background: #f7931e;
            }
            
            button:active {
                transform: scale(0.98);
            }
            
            .status {
                background: #1a1a1a;
                border: 1px solid #333;
                color: #90ee90;
                padding: 8px;
                border-radius: 2px;
                margin-top: 8px;
                font-size: 0.8em;
                font-family: monospace;
                max-height: 100px;
                overflow-y: auto;
            }
            
            .status.error {
                color: #ff6b6b;
            }
            
            .info {
                background: #1a1a1a;
                border-left: 3px solid #ff6b35;
                padding: 8px;
                margin-top: 8px;
                font-size: 0.8em;
            }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>🎬 Houdini Node Manager</h1>
            <p>Manage your Houdini node graph with AuroraView</p>
        </div>
        
        <div class="section">
            <h2>Node Graph</h2>
            <div class="node-list" id="nodeList">
                <div class="node-item">geo1 (Geometry)</div>
                <div class="node-item">file1 (File)</div>
                <div class="node-item">transform1 (Transform)</div>
                <div class="node-item">group1 (Group)</div>
                <div class="node-item">attribwrangle1 (Attribute Wrangle)</div>
            </div>
            <div class="info">
                💡 Click on nodes to select them
            </div>
        </div>
        
        <div class="section">
            <h2>Node Properties</h2>
            <div class="form-group">
                <label>Node Name:</label>
                <input type="text" id="nodeName" placeholder="Select a node" readonly>
            </div>
            <div class="form-group">
                <label>Node Type:</label>
                <input type="text" id="nodeType" placeholder="Type" readonly>
            </div>
            <div class="form-group">
                <label>Parameters:</label>
                <textarea id="parameters" rows="4" placeholder="Node parameters" readonly></textarea>
            </div>
        </div>
        
        <div class="section">
            <h2>Operations</h2>
            <div class="button-group">
                <button onclick="deleteNode()">Delete</button>
                <button onclick="duplicateNode()">Duplicate</button>
                <button onclick="connectNodes()">Connect</button>
            </div>
            <div class="button-group">
                <button onclick="cookNode()">Cook</button>
                <button onclick="inspectNode()">Inspect</button>
            </div>
            <div class="status" id="status">Ready</div>
        </div>
        
        <script>
            let selectedNode = null;
            
            // Setup node list click handlers
            document.querySelectorAll('.node-item').forEach(item => {
                item.addEventListener('click', function() {
                    document.querySelectorAll('.node-item').forEach(n => n.classList.remove('selected'));
                    this.classList.add('selected');
                    selectedNode = this.textContent;
                    updateNodeProperties();
                });
            });
            
            function updateNodeProperties() {
                if (!selectedNode) return;
                
                document.getElementById('nodeName').value = selectedNode.split(' ')[0];
                document.getElementById('nodeType').value = selectedNode.split('(')[1]?.replace(')', '') || 'Unknown';
                document.getElementById('parameters').value = 'param1: value1\\nparam2: value2\\nparam3: value3';
            }
            
            function deleteNode() {
                if (!selectedNode) {
                    updateStatus('❌ No node selected', true);
                    return;
                }
                updateStatus(`✓ Deleted node: ${selectedNode}`);
            }
            
            function duplicateNode() {
                if (!selectedNode) {
                    updateStatus('❌ No node selected', true);
                    return;
                }
                updateStatus(`✓ Duplicated node: ${selectedNode}`);
            }
            
            function connectNodes() {
                if (!selectedNode) {
                    updateStatus('❌ No node selected', true);
                    return;
                }
                updateStatus(`✓ Ready to connect: ${selectedNode}`);
            }
            
            function cookNode() {
                if (!selectedNode) {
                    updateStatus('❌ No node selected', true);
                    return;
                }
                updateStatus(`✓ Cooking node: ${selectedNode}...`);
            }
            
            function inspectNode() {
                if (!selectedNode) {
                    updateStatus('❌ No node selected', true);
                    return;
                }
                updateStatus(`✓ Inspecting node: ${selectedNode}`);
            }
            
            function updateStatus(message, isError = false) {
                const status = document.getElementById('status');
                status.textContent = message;
                status.classList.toggle('error', isError);
            }
        </script>
    </body>
    </html>
    """
    
    # Load HTML
    logger.info("Loading Houdini tool UI...")
    webview.load_html(html_content)
    logger.info("✓ UI loaded")
    logger.info("")
    
    # Register event handlers
    logger.info("Registering event handlers...")
    
    @webview.on("delete_node")
    def handle_delete_node(data):
        """Handle node deletion from UI."""
        logger.info(f"✓ Delete node: {data}")
        # In a real Houdini plugin, this would delete the node
        # Example: hou.node(data['node_path']).destroy()
    
    @webview.on("cook_node")
    def handle_cook_node(data):
        """Handle node cooking from UI."""
        logger.info(f"✓ Cook node: {data}")
        # In a real Houdini plugin, this would cook the node
        # Example: hou.node(data['node_path']).cook()
    
    @webview.on("inspect_node")
    def handle_inspect_node(data):
        """Handle node inspection from UI."""
        logger.info(f"✓ Inspect node: {data}")
        # In a real Houdini plugin, this would inspect the node
    
    logger.info("✓ Event handlers registered")
    logger.info("")
    
    # Show the tool
    logger.info("Showing Houdini tool...")
    logger.info("Close the window to exit.")
    logger.info("")
    
    try:
        webview.show()
    except Exception as e:
        logger.error(f"Error showing WebView: {e}")
        return 1
    
    logger.info("")
    logger.info("=" * 60)
    logger.info("Houdini tool closed.")
    logger.info("=" * 60)
    
    return 0


if __name__ == "__main__":
    sys.exit(create_houdini_tool())

