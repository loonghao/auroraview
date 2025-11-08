"""
Simplified IPC Test - No Manual Bridge Waiting Required!

This example shows the new simplified API where:
1. No need to wait for window.auroraview - it's always ready
2. No need to load external bridge.js files
3. Use simple window.aurora.emit() and window.aurora.on()
4. Qt-style API for familiar syntax
"""

import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python"))

try:
    import nuke
    NUKE_AVAILABLE = True
except ImportError:
    NUKE_AVAILABLE = False
    nuke = None


def run():
    """Run simplified IPC test with auto-ready bridge."""
    from auroraview import WebView
    
    print("\n" + "="*60)
    print("Simplified IPC Test - Auto-Ready Bridge")
    print("="*60 + "\n")
    
    # Create WebView
    webview = WebView.create(
        title="Simplified IPC Test",
        width=600,
        height=400,
        debug=True
    )
    
    # Register handler
    @webview.on("create_node")
    def handle_create_node(data):
        print(f"[Python] Creating node: {data}")
        
        # Create node if in Nuke
        if NUKE_AVAILABLE and nuke:
            try:
                node_type = data.get("type", "Grade")
                node = nuke.createNode(node_type)
                print(f"[Python] Created node: {node.name()}")
                webview.emit("node_created", {
                    "name": node.name(),
                    "class": node.Class()
                })
            except Exception as e:
                print(f"[Python] Error: {e}")
                webview.emit("error", {"message": str(e)})
        else:
            # Simulate node creation
            webview.emit("node_created", {
                "name": "Grade1",
                "class": data.get("type", "Grade")
            })
    
    # Simple HTML with new API
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <style>
            body {
                font-family: 'Segoe UI', Arial, sans-serif;
                padding: 30px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: #fff;
                margin: 0;
            }
            .container {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                border-radius: 12px;
                padding: 30px;
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            }
            h1 {
                margin: 0 0 20px 0;
                font-size: 24px;
                font-weight: 600;
            }
            button {
                padding: 12px 24px;
                font-size: 14px;
                background: rgba(255, 255, 255, 0.2);
                color: white;
                border: 2px solid rgba(255, 255, 255, 0.3);
                border-radius: 8px;
                cursor: pointer;
                margin: 8px 8px 8px 0;
                transition: all 0.3s ease;
                font-weight: 500;
            }
            button:hover {
                background: rgba(255, 255, 255, 0.3);
                border-color: rgba(255, 255, 255, 0.5);
                transform: translateY(-2px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
            }
            #log {
                margin-top: 20px;
                padding: 20px;
                background: rgba(0, 0, 0, 0.3);
                border-radius: 8px;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 13px;
                white-space: pre-wrap;
                max-height: 200px;
                overflow-y: auto;
                line-height: 1.6;
            }
            .status {
                display: inline-block;
                padding: 4px 12px;
                background: rgba(76, 175, 80, 0.3);
                border-radius: 12px;
                font-size: 12px;
                margin-bottom: 15px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 Simplified IPC Test</h1>
            <div class="status">✓ Bridge Ready (No Waiting!)</div>
            
            <div>
                <button onclick="createGrade()">Create Grade</button>
                <button onclick="createBlur()">Create Blur</button>
                <button onclick="createMerge()">Create Merge</button>
            </div>
            
            <div id="log">Ready! Click buttons to test IPC.\nNo manual bridge waiting required! 🎉</div>
        </div>

        <script>
            const log = document.getElementById('log');
            
            function addLog(msg) {
                const timestamp = new Date().toLocaleTimeString();
                log.textContent += `[${timestamp}] ${msg}\n`;
                log.scrollTop = log.scrollHeight;
            }
            
            // Register listeners using simplified API
            window.aurora.on('node_created', (data) => {
                addLog(`✓ Node created: ${data.name} (${data.class})`);
            });
            
            window.aurora.on('error', (data) => {
                addLog(`✗ Error: ${data.message}`);
            });
            
            // Helper functions
            function createGrade() {
                addLog('→ Creating Grade node...');
                window.aurora.emit('create_node', { type: 'Grade' });
            }
            
            function createBlur() {
                addLog('→ Creating Blur node...');
                window.aurora.emit('create_node', { type: 'Blur' });
            }
            
            function createMerge() {
                addLog('→ Creating Merge node...');
                window.aurora.emit('create_node', { type: 'Merge' });
            }
            
            // Log that we're ready
            addLog('✓ JavaScript initialized');
            addLog('✓ Using window.aurora API');
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    webview.show()
    
    print("✓ WebView shown - No bridge waiting needed!")
    print("✓ Use window.aurora.emit() and window.aurora.on()")
    print("="*60 + "\n")
    
    return webview


if __name__ == "__main__":
    run()

