"""
Example: Basic Nuke Panel

This example demonstrates how to create a basic panel in Nuke
using AuroraView with a modern web-based UI.

Features:
- Panel integration
- Create nodes from UI
- Query node graph
- Bidirectional Python ↔ JavaScript communication
- Uses shadcn/ui components via CDN

Prerequisites:
    1. Install AuroraView in Nuke's Python environment:
       - Find Nuke's Python: nuke.EXE_PATH (e.g., C:/Program Files/Nuke15.2v1/python.exe)
       - Install: "C:/Program Files/Nuke15.2v1/python.exe" -m pip install auroraview

    2. Or use a virtual environment and launch Nuke from it

Usage:
    In Nuke Script Editor:
        import sys
        from pathlib import Path

        examples_dir = Path(r'C:\\path\to\\dcc_webview\\examples')
        sys.path.insert(0, str(examples_dir))

        import nuke_examples.basic_panel as example
        example.show()
"""

import sys

try:
    import nuke
    NUKE_AVAILABLE = True
except ImportError:
    print("Warning: nuke module not available. This example requires Nuke.")
    NUKE_AVAILABLE = False
    nuke = None

try:
    from auroraview import WebView
    AURORAVIEW_AVAILABLE = True
except ImportError as e:
    AURORAVIEW_AVAILABLE = False
    print(f"Error: AuroraView not installed in Nuke's Python environment.")
    print(f"Import error: {e}")
    print(f"\nTo fix this:")
    print(f"1. Find Nuke's Python executable:")
    if NUKE_AVAILABLE:
        print(f"   Nuke Python: {sys.executable}")
    print(f"2. Install AuroraView:")
    print(f'   "{sys.executable}" -m pip install auroraview')
    print(f"\nOr use a virtual environment with AuroraView installed and launch Nuke from it.")
    WebView = None


def create_node(node_type):
    """Create a node in Nuke."""
    if not NUKE_AVAILABLE or not nuke:
        return {"error": "Nuke not available"}

    try:
        # Create node
        node = nuke.createNode(node_type)

        return {"success": True, "name": node.name(), "class": node.Class(), "type": node_type}
    except Exception as e:
        return {"error": str(e)}


def get_graph_info():
    """Get current node graph information."""
    if not NUKE_AVAILABLE or not nuke:
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
        // AuroraView Signal/Slot System (Qt-style)
        // Ensures safe event binding even if bridge isn't ready yet
        class AuroraViewBridge {
            constructor() {
                this.ready = false;
                this.pendingCalls = [];
                this.eventHandlers = new Map();
                this.init();
            }

            init() {
                // Wait for AuroraView bridge to be ready
                if (window.auroraview && window.auroraview.send_event) {
                    this.ready = true;
                    console.log('[AuroraView] Bridge ready');
                    this.processPendingCalls();
                } else {
                    console.log('[AuroraView] Waiting for bridge...');
                    setTimeout(() => this.init(), 50);
                }
            }

            // Qt-style signal emission (Python → JavaScript)
            connect(signal, slot) {
                if (!this.eventHandlers.has(signal)) {
                    this.eventHandlers.set(signal, []);
                }
                this.eventHandlers.get(signal).push(slot);

                // Register with native bridge when ready
                if (this.ready) {
                    window.auroraview.on(signal, (data) => {
                        this.eventHandlers.get(signal).forEach(handler => {
                            try {
                                handler(data);
                            } catch (e) {
                                console.error(`[AuroraView] Error in slot for signal '${signal}':`, e);
                            }
                        });
                    });
                }

                console.log(`[AuroraView] Connected slot to signal: ${signal}`);
            }

            // Qt-style slot invocation (JavaScript → Python)
            emit(signal, data = {}) {
                if (this.ready) {
                    try {
                        window.auroraview.send_event(signal, data);
                        console.log(`[AuroraView] Emitted signal: ${signal}`, data);
                    } catch (e) {
                        console.error(`[AuroraView] Error emitting signal '${signal}':`, e);
                    }
                } else {
                    console.log(`[AuroraView] Queuing signal: ${signal}`);
                    this.pendingCalls.push({ signal, data });
                }
            }

            processPendingCalls() {
                console.log(`[AuroraView] Processing ${this.pendingCalls.length} pending calls`);
                this.pendingCalls.forEach(({ signal, data }) => {
                    this.emit(signal, data);
                });
                this.pendingCalls = [];
            }
        }

        // Global bridge instance
        const bridge = new AuroraViewBridge();

        // User-friendly API
        function createNode(type) {
            bridge.emit('create_node', { type: type });
        }

        function getGraphInfo() {
            bridge.emit('get_graph_info', {});
        }

        function showStatus(message, isError = false) {
            const status = document.getElementById('status');
            status.textContent = message;
            status.className = isError ? 'error' : 'success';
            status.style.display = 'block';
            setTimeout(() => status.style.display = 'none', 3000);
        }

        // Connect signals to slots (Qt-style)
        bridge.connect('node_created', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`✅ Created ${data.class} node: ${data.name}`);
            }
        });

        bridge.connect('graph_info', (data) => {
            if (data.error) {
                showStatus('Error: ' + data.error, true);
            } else {
                showStatus(`Graph: ${data.total_nodes} nodes, ${data.selected_count} selected`);
            }
        });

        console.log('[AuroraView] UI initialized');
    </script>
</body>
</html>
"""


# Global reference to keep webview alive
_active_webview = None


def show():
    """Show the Nuke panel.

    Note: When closing the window, you may see Qt-related warnings from Nuke/Hiero's
    status bar. These are harmless and can be safely ignored. They occur because
    Nuke's UI tries to update after the WebView window is closed.

    Returns:
        WebView instance or None if failed
    """
    global _active_webview

    if not AURORAVIEW_AVAILABLE:
        print("\n" + "="*60)
        print("ERROR: AuroraView is not installed!")
        print("="*60)
        print(f"\nCurrent Python: {sys.executable}")
        print(f"\nTo install AuroraView, run:")
        print(f'  "{sys.executable}" -m pip install auroraview')
        print("\nOr install in Nuke's Python environment.")
        print("="*60 + "\n")
        return None

    if not NUKE_AVAILABLE:
        print("Warning: Nuke module not available. Some features may not work.")

    # Close existing webview if any
    if _active_webview is not None:
        try:
            print("Closing existing WebView...")
            _active_webview.close()
        except Exception as e:
            print(f"Error closing existing WebView: {e}")
        _active_webview = None

    # Suppress Qt warnings during window close
    # These warnings come from Nuke/Hiero's status bar trying to update
    # after the window is closed, and are harmless
    import warnings
    warnings.filterwarnings('ignore', category=RuntimeWarning, message='.*Internal C\\+\\+ object.*already deleted.*')

    # Create WebView with singleton mode to prevent multiple instances
    webview = WebView.create(
        title="Nuke Panel",
        html=HTML_CONTENT,
        width=650,
        height=500,
        debug=True,
        singleton="nuke_panel",  # Use singleton to prevent multiple instances
        auto_timer=True  # Auto-start event timer
    )

    # Register event handlers
    @webview.on("create_node")
    def handle_create_node(data):
        try:
            result = create_node(data.get("type", "Grade"))
            webview.emit("node_created", result)
        except Exception as e:
            print(f"Error creating node: {e}")
            webview.emit("node_created", {"error": str(e)})

    @webview.on("get_graph_info")
    def handle_graph_info(data):
        try:
            result = get_graph_info()
            webview.emit("graph_info", result)
        except Exception as e:
            print(f"Error getting graph info: {e}")
            webview.emit("graph_info", {"error": str(e)})

    # Show the window (non-blocking in Nuke)
    try:
        webview.show()
        _active_webview = webview
        print("WebView shown successfully. Close the window when done.")
    except Exception as e:
        print(f"Error showing WebView: {e}")
        return None

    return webview


def close():
    """Close the active WebView panel."""
    global _active_webview

    if _active_webview is not None:
        try:
            print("Closing WebView...")
            _active_webview.close()
            print("WebView closed successfully")
        except Exception as e:
            print(f"Error closing WebView: {e}")
        finally:
            _active_webview = None
    else:
        print("No active WebView to close")


if __name__ == "__main__":
    show()
