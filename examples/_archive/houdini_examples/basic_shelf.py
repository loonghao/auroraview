"""
Example 01: Basic Houdini Shelf Tool (Qt Integration)

This example demonstrates how to create a shelf tool in Houdini
using AuroraView with Qt backend for better integration.

Features:
- Qt backend integration (no UI blocking!)
- Create geometry nodes from UI
- Query scene information
- Bidirectional Python ↔ JavaScript communication
- Uses shadcn/ui components via CDN

Requirements:
    pip install auroraview[qt]

Usage:
    In Houdini Python Shell:
        import sys
        from pathlib import Path

        examples_dir = Path(r'C:\\path\to\\dcc_webview\\examples')
        sys.path.insert(0, str(examples_dir))

        import houdini_examples.basic_shelf as example
        example.show()
"""

try:
    import hou
except ImportError:
    print("Warning: hou module not available. This example requires Houdini.")
    hou = None

try:
    from auroraview import QtWebView
except ImportError:
    print("[ERROR] Qt backend not available!")
    print("Install with: pip install auroraview[qt]")
    raise

try:
    from PySide2.QtWidgets import QWidget
    import hou
except ImportError:
    try:
        from PySide6.QtWidgets import QWidget
        import hou
    except ImportError:
        print("[ERROR] PySide2/PySide6 not available!")
        raise


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
        <h1>🎨 Houdini Shelf Tool <span style="font-size: 14px; color: #667eea;">Qt Backend</span></h1>
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
        // Qt backend uses window.emit() and window.on()
        function createNode(type) {
            window.emit('create_node', { type: type });
        }

        function getSceneInfo() {
            window.emit('get_scene_info', {});
        }

        function showStatus(message, isError = false) {
            const status = document.getElementById('status');
            status.textContent = message;
            status.className = isError ? 'error' : 'success';
            status.style.display = 'block';
            setTimeout(() => status.style.display = 'none', 3000);
        }

        // Listen for responses from Python
        window.on('node_created', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`✅ Created ${data.type} node: ${data.name}`);
            }
        });

        window.on('scene_info', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`Scene has ${data.node_count} nodes`);
            }
        });

        console.log('[AuroraView] Qt backend initialized');
    </script>
</body>
</html>
"""


def get_houdini_main_window():
    """Get Houdini's main window as a QWidget."""
    return hou.qt.mainWindow()


# Global instance to keep webview alive
_webview_instance = None


def show():
    """Show the Houdini shelf tool using Qt backend."""
    global _webview_instance

    # Close existing instance if any
    if _webview_instance is not None:
        try:
            _webview_instance.close()
        except:
            pass
        _webview_instance = None

    # Get Houdini main window
    houdini_window = get_houdini_main_window()

    # Create WebView with Qt backend
    webview = QtWebView(
        parent=houdini_window,
        title="Houdini Shelf Tool",
        width=650,
        height=500
    )

    # Load HTML content
    webview.load_html(HTML_CONTENT)

    # Register event handlers
    @webview.on("create_node")
    def handle_create_node(data):
        result = create_node(data.get("type", "geo"))
        webview.emit("node_created", result)

    @webview.on("get_scene_info")
    def handle_scene_info(data):
        result = get_scene_info()
        webview.emit("scene_info", result)

    # Show the window (non-blocking with Qt)
    webview.show()

    # Keep reference to prevent garbage collection
    _webview_instance = webview

    return webview


if __name__ == "__main__":
    show()
