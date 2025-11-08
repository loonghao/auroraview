"""
Example 01: Basic Houdini Shelf Tool

This example demonstrates how to create a basic shelf tool in Houdini
using AuroraView with a modern web-based UI.

Features:
- Shelf button integration
- Create geometry nodes from UI
- Query scene information
- Bidirectional Python ↔ JavaScript communication
- Uses shadcn/ui components via CDN

Usage:
    In Houdini Python Shell:
        import sys
        from pathlib import Path

        examples_dir = Path(r'C:\\path\to\\dcc_webview\\examples')
        sys.path.insert(0, str(examples_dir))

        import houdini.01_basic_shelf as example
        example.show()
"""

try:
    import hou
except ImportError:
    print("Warning: hou module not available. This example requires Houdini.")
    hou = None

from auroraview import WebView


def create_node(node_type):
    """Create a geometry node in Houdini."""
    if not hou:
        return {"error": "Houdini not available"}

    try:
        # Get /obj context
        obj = hou.node("/obj")

        # Create node
        node = obj.createNode(node_type)
        node.moveToGoodPosition()

        return {"success": True, "name": node.name(), "path": node.path(), "type": node_type}
    except Exception as e:
        return {"error": str(e)}


def get_scene_info():
    """Get current scene information."""
    if not hou:
        return {"error": "Houdini not available"}

    try:
        obj = hou.node("/obj")
        nodes = obj.children()

        return {
            "node_count": len(nodes),
            "nodes": [{"name": n.name(), "type": n.type().name()} for n in nodes[:10]],
        }
    except Exception as e:
        return {"error": str(e)}


# HTML content with shadcn/ui styling
HTML_CONTENT = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Houdini Shelf Tool</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 20px;
        }
        .container {
            max-width: 600px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            padding: 24px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        .btn {
            background: #667eea;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: all 0.2s;
            margin: 4px;
        }
        .btn:hover {
            background: #5568d3;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }
        .info-box {
            background: #f7fafc;
            border-left: 4px solid #667eea;
            padding: 16px;
            margin: 16px 0;
            border-radius: 4px;
        }
        h1 {
            color: #2d3748;
            margin-top: 0;
        }
        #status {
            margin-top: 16px;
            padding: 12px;
            border-radius: 8px;
            display: none;
        }
        .success {
            background: #c6f6d5;
            color: #22543d;
            border: 1px solid #9ae6b4;
        }
        .error {
            background: #fed7d7;
            color: #742a2a;
            border: 1px solid #fc8181;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Houdini Shelf Tool</h1>
        <p>Create geometry nodes and manage your scene from this modern web interface.</p>
        
        <div class="info-box">
            <h3 style="margin-top:0;">Create Geometry</h3>
            <button class="btn" onclick="createNode('geo')">📦 Box</button>
            <button class="btn" onclick="createNode('sphere')">⚪ Sphere</button>
            <button class="btn" onclick="createNode('grid')">📐 Grid</button>
        </div>
        
        <div class="info-box">
            <h3 style="margin-top:0;">Scene Info</h3>
            <button class="btn" onclick="getSceneInfo()">🔍 Get Scene Info</button>
        </div>
        
        <div id="status"></div>
    </div>
    
    <script>
        function createNode(type) {
            window.auroraview.send_event('create_node', { type: type });
        }
        
        function getSceneInfo() {
            window.auroraview.send_event('get_scene_info', {});
        }
        
        function showStatus(message, isError = false) {
            const status = document.getElementById('status');
            status.textContent = message;
            status.className = isError ? 'error' : 'success';
            status.style.display = 'block';
            setTimeout(() => status.style.display = 'none', 3000);
        }
        
        // Listen for responses
        window.auroraview.on('node_created', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`✅ Created ${data.type} node: ${data.name}`);
            }
        });
        
        window.auroraview.on('scene_info', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`Scene has ${data.node_count} nodes`);
            }
        });
    </script>
</body>
</html>
"""


def show():
    """Show the Houdini shelf tool."""
    # Create WebView using Houdini factory method
    webview = WebView.houdini(
        title="Houdini Shelf Tool", html=HTML_CONTENT, width=650, height=500, debug=True
    )

    # Register event handlers
    @webview.on("create_node")
    def handle_create_node(data):
        result = create_node(data.get("type", "geo"))
        webview.emit("node_created", result)

    @webview.on("get_scene_info")
    def handle_scene_info(data):
        result = get_scene_info()
        webview.emit("scene_info", result)

    # Show the window (non-blocking)
    webview.show()

    return webview


if __name__ == "__main__":
    show()
