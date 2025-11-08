"""
Manual IPC test for Nuke integration.

This script tests the complete IPC workflow:
1. Create WebView with native backend
2. Send create_node signal from JavaScript
3. Create node in Nuke
4. Send response back to JavaScript
5. Close WebView

Usage:
    In Nuke Script Editor:
    >>> import sys
    >>> sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
    >>> from nuke_examples import test_ipc_manual
    >>> test_ipc_manual.run_test()
"""

import sys
from pathlib import Path

# Check if running in Nuke
try:
    import nuke
    NUKE_AVAILABLE = True
    print("[Test] ✓ Nuke detected")
except ImportError:
    NUKE_AVAILABLE = False
    print("[Test] ✗ Nuke not available - this test must run inside Nuke")
    sys.exit(1)

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python"))

from auroraview import WebView


def run_test():
    """Run the IPC test."""
    print("\n" + "="*60)
    print("Nuke IPC Communication Test")
    print("="*60)
    
    # Track test results
    results = {
        "nodes_created": [],
        "errors": []
    }
    
    # Create WebView
    print("\n[1/5] Creating WebView...")
    webview = WebView.create(
        title="Nuke IPC Test",
        width=500,
        height=400,
        debug=True
    )
    print("      ✓ WebView created")
    
    # Register IPC handler
    print("\n[2/5] Registering IPC handlers...")
    
    @webview.on("create_node")
    def handle_create_node(data):
        """Handle node creation from JavaScript."""
        node_type = data.get("type", "Grade")
        print(f"\n[IPC] Received create_node signal: {node_type}")
        
        try:
            # Create node
            node = nuke.createNode(node_type)
            results["nodes_created"].append(node)
            
            # Send success response
            response = {
                "success": True,
                "name": node.name(),
                "class": node.Class(),
                "type": node_type
            }
            webview.emit("node_created", response)
            
            print(f"[IPC] ✓ Node created: {node.name()}")
            
        except Exception as e:
            error_msg = str(e)
            results["errors"].append(error_msg)
            
            # Send error response
            webview.emit("node_created", {
                "success": False,
                "error": error_msg
            })
            
            print(f"[IPC] ✗ Error: {error_msg}")
    
    @webview.on("test_complete")
    def handle_test_complete(data):
        """Handle test completion."""
        print(f"\n[IPC] Test completed: {data}")
        webview.close()
    
    print("      ✓ Handlers registered")
    
    # Load test HTML
    print("\n[3/5] Loading test UI...")
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>Nuke IPC Test</title>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                font-family: 'Segoe UI', Arial, sans-serif;
                padding: 30px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: #fff;
                min-height: 100vh;
            }
            .container {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                border-radius: 12px;
                padding: 30px;
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            }
            h1 {
                margin-bottom: 20px;
                font-size: 24px;
            }
            .test-section {
                margin: 20px 0;
                padding: 15px;
                background: rgba(255, 255, 255, 0.05);
                border-radius: 8px;
            }
            button {
                padding: 12px 24px;
                font-size: 14px;
                font-weight: 600;
                cursor: pointer;
                background: #4CAF50;
                color: white;
                border: none;
                border-radius: 6px;
                margin: 5px;
                transition: all 0.3s;
            }
            button:hover {
                background: #45a049;
                transform: translateY(-2px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
            }
            button:active {
                transform: translateY(0);
            }
            #status {
                margin-top: 15px;
                padding: 12px;
                background: rgba(0, 0, 0, 0.3);
                border-radius: 6px;
                border-left: 4px solid #2196F3;
                font-family: 'Consolas', monospace;
                font-size: 13px;
            }
            .success { border-left-color: #4CAF50; }
            .error { border-left-color: #f44336; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 Nuke IPC Communication Test</h1>
            
            <div class="test-section">
                <h3>Test Steps:</h3>
                <ol style="margin-left: 20px; margin-top: 10px;">
                    <li>Click "Create Grade Node"</li>
                    <li>Check Nuke node graph for new node</li>
                    <li>Verify status message below</li>
                    <li>Click "Complete Test" to finish</li>
                </ol>
            </div>
            
            <div class="test-section">
                <button id="createBtn">Create Grade Node</button>
                <button id="completeBtn" style="background: #2196F3;">Complete Test</button>
                <div id="status">Waiting for bridge...</div>
            </div>
        </div>

        <script src="auroraview-bridge.js"></script>
        <script>
            const bridge = new AuroraViewBridge();
            const status = document.getElementById('status');
            
            // Update status
            function updateStatus(message, type = 'info') {
                status.textContent = message;
                status.className = type;
                console.log(`[UI] ${message}`);
            }
            
            // Wait for bridge ready
            bridge.connect('bridge_ready', () => {
                updateStatus('✓ Bridge connected - Ready to test', 'success');
            });
            
            // Listen for node creation response
            bridge.connect('node_created', (data) => {
                console.log('[UI] Node created response:', data);
                if (data.success) {
                    updateStatus(`✓ Node created: ${data.name} (${data.class})`, 'success');
                } else {
                    updateStatus(`✗ Error: ${data.error}`, 'error');
                }
            });
            
            // Create node button
            document.getElementById('createBtn').onclick = () => {
                updateStatus('Creating Grade node...', 'info');
                bridge.emit('create_node', { type: 'Grade' });
            };
            
            // Complete test button
            document.getElementById('completeBtn').onclick = () => {
                updateStatus('Test completed!', 'success');
                bridge.emit('test_complete', { status: 'passed' });
            };
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    print("      ✓ HTML loaded")
    
    # Show WebView
    print("\n[4/5] Showing WebView...")
    print("      → Click 'Create Grade Node' to test IPC")
    print("      → Click 'Complete Test' when done")
    webview.show()
    
    # Print results
    print("\n[5/5] Test Results:")
    print(f"      Nodes created: {len(results['nodes_created'])}")
    print(f"      Errors: {len(results['errors'])}")
    
    if results["nodes_created"]:
        print("\n      Created nodes:")
        for node in results["nodes_created"]:
            print(f"        - {node.name()} ({node.Class()})")
    
    if results["errors"]:
        print("\n      Errors:")
        for error in results["errors"]:
            print(f"        - {error}")
    
    print("\n" + "="*60)
    print("Test completed!")
    print("="*60 + "\n")
    
    return webview, results


if __name__ == "__main__":
    if NUKE_AVAILABLE:
        run_test()
    else:
        print("This script must be run inside Nuke!")

