"""
Example 01: Basic Nuke Panel

This example demonstrates how to create a basic panel in Nuke
using AuroraView with a modern web-based UI.

Features:
- Panel integration
- Create nodes from UI
- Query node graph
- Bidirectional Python ↔ JavaScript communication
- Uses shadcn/ui components via CDN

Usage:
    In Nuke Script Editor:
        import sys
        from pathlib import Path

        examples_dir = Path(r'C:\\path\to\\dcc_webview\\examples')
        sys.path.insert(0, str(examples_dir))

        import nuke.01_basic_panel as example
        example.show()
"""

try:
    import nuke
except ImportError:
    print("Warning: nuke module not available. This example requires Nuke.")
    nuke = None

from auroraview import WebView


def create_node(node_type):
    """Create a node in Nuke."""
    if not nuke:
        return {"error": "Nuke not available"}

    try:
        # Create node
        node = nuke.createNode(node_type)

        return {"success": True, "name": node.name(), "class": node.Class(), "type": node_type}
    except Exception as e:
        return {"error": str(e)}


def get_graph_info():
    """Get current node graph information."""
    if not nuke:
        return {"error": "Nuke not available"}

    try:
        all_nodes = nuke.allNodes()
        selected = nuke.selectedNodes()

        return {
            "total_nodes": len(all_nodes),
            "selected_count": len(selected),
            "selected_nodes": [{"name": n.name(), "class": n.Class()} for n in selected[:5]],
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
    <title>Nuke Panel</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
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
            background: #f5576c;
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
            background: #e04858;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(245, 87, 108, 0.4);
        }
        .info-box {
            background: #f7fafc;
            border-left: 4px solid #f5576c;
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
        <h1>🎬 Nuke Panel</h1>
        <p>Create nodes and manage your comp from this modern web interface.</p>
        
        <div class="info-box">
            <h3 style="margin-top:0;">Create Nodes</h3>
            <button class="btn" onclick="createNode('Read')">📁 Read</button>
            <button class="btn" onclick="createNode('Write')">💾 Write</button>
            <button class="btn" onclick="createNode('Grade')">🎨 Grade</button>
            <button class="btn" onclick="createNode('Blur')">🌫️ Blur</button>
        </div>
        
        <div class="info-box">
            <h3 style="margin-top:0;">Graph Info</h3>
            <button class="btn" onclick="getGraphInfo()">🔍 Get Graph Info</button>
        </div>
        
        <div id="status"></div>
    </div>
    
    <script>
        function createNode(type) {
            window.auroraview.send_event('create_node', { type: type });
        }
        
        function getGraphInfo() {
            window.auroraview.send_event('get_graph_info', {});
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
                showStatus(`✅ Created ${data.class} node: ${data.name}`);
            }
        });
        
        window.auroraview.on('graph_info', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`Graph: ${data.total_nodes} nodes, ${data.selected_count} selected`);
            }
        });
    </script>
</body>
</html>
"""


def show():
    """Show the Nuke panel."""
    # Create WebView
    webview = WebView.create(
        title="Nuke Panel", html=HTML_CONTENT, width=650, height=500, debug=True
    )

    # Register event handlers
    @webview.on("create_node")
    def handle_create_node(data):
        result = create_node(data.get("type", "Grade"))
        webview.emit("node_created", result)

    @webview.on("get_graph_info")
    def handle_graph_info(data):
        result = get_graph_info()
        webview.emit("graph_info", result)

    # Show the window
    webview.show()

    return webview


if __name__ == "__main__":
    show()
