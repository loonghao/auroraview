"""
Simple IPC test - minimal example for debugging.

This is the simplest possible test to verify window.auroraview API works.

Usage in Nuke:
    >>> import sys
    >>> sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
    >>> from nuke_examples import test_simple
    >>> test_simple.run()
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
    """Run simple IPC test."""
    from auroraview import WebView
    
    print("\n" + "="*60)
    print("Simple IPC Test")
    print("="*60 + "\n")
    
    # Create WebView
    webview = WebView.create(
        title="Simple IPC Test",
        width=600,
        height=400,
        debug=True
    )
    
    # Register handler
    @webview.on("test_signal")
    def handle_test(data):
        print(f"[Python] Received signal: {data}")
        
        # Send response
        webview.emit("test_response", {
            "message": "Hello from Python!",
            "received": data
        })
        
        # Create node if in Nuke
        if NUKE_AVAILABLE and nuke:
            try:
                node = nuke.createNode("Grade")
                print(f"[Python] Created node: {node.name()}")
                webview.emit("node_created", {
                    "name": node.name(),
                    "class": node.Class()
                })
            except Exception as e:
                print(f"[Python] Error: {e}")
    
    # Simple HTML
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <style>
            body {
                font-family: Arial, sans-serif;
                padding: 20px;
                background: #1e1e1e;
                color: #fff;
            }
            button {
                padding: 15px 30px;
                font-size: 16px;
                background: #007acc;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                margin: 10px 0;
            }
            button:hover {
                background: #005a9e;
            }
            #log {
                margin-top: 20px;
                padding: 15px;
                background: #252526;
                border-radius: 4px;
                font-family: 'Consolas', monospace;
                font-size: 13px;
                white-space: pre-wrap;
            }
        </style>
    </head>
    <body>
        <h1>Simple IPC Test</h1>
        <button onclick="testIPC()">Test IPC</button>
        <button onclick="createNode()">Create Node</button>
        <div id="log">Waiting for bridge...</div>

        <script>
            const log = document.getElementById('log');
            
            function addLog(msg) {
                log.textContent += msg + '\\n';
                console.log(msg);
            }
            
            // Wait for bridge
            function waitForBridge() {
                if (window.auroraview && window.auroraview.send_event) {
                    addLog('✓ Bridge ready!');
                    addLog('✓ window.auroraview.send_event exists');
                    addLog('✓ window.auroraview.on exists');
                    
                    // Register listeners
                    window.auroraview.on('test_response', function(data) {
                        addLog('✓ Received: ' + JSON.stringify(data));
                    });
                    
                    window.auroraview.on('node_created', function(data) {
                        addLog('✓ Node created: ' + data.name);
                    });
                } else {
                    addLog('Waiting for bridge...');
                    setTimeout(waitForBridge, 100);
                }
            }
            
            function testIPC() {
                addLog('→ Sending test_signal...');
                window.auroraview.send_event('test_signal', {
                    message: 'Hello from JavaScript!',
                    timestamp: Date.now()
                });
            }
            
            function createNode() {
                addLog('→ Requesting node creation...');
                window.auroraview.send_event('test_signal', {
                    action: 'create_node'
                });
            }
            
            waitForBridge();
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    webview.show()
    
    print("WebView shown - test the buttons!")
    print("="*60 + "\n")
    
    return webview


if __name__ == "__main__":
    run()

