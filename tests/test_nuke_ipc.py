"""
Integration tests for Nuke IPC communication.

Tests the native WebView IPC bridge for creating nodes and communicating
with Nuke through the window.auroraview API.
"""

import time

import pytest

# Check if Nuke is available
try:
    import nuke

    NUKE_AVAILABLE = True
except ImportError:
    NUKE_AVAILABLE = False
    nuke = None


@pytest.mark.skipif(not NUKE_AVAILABLE, reason="Nuke not available")
class TestNukeIPC:
    """Test Nuke IPC communication through native WebView."""

    def test_create_webview_and_node(self):
        """Test creating WebView, creating a node via IPC, and closing."""
        from auroraview import WebView

        # Track created nodes
        created_nodes = []
        test_completed = False

        # Create WebView with IPC handler
        webview = WebView.create(title="Nuke IPC Test", width=400, height=300, debug=True)

        # Register handler for node creation
        @webview.on("create_node")
        def handle_create_node(data):
            """Handle node creation request from JavaScript."""
            nonlocal created_nodes, test_completed

            node_type = data.get("type", "Grade")
            print(f"[Python] Creating {node_type} node...")

            try:
                # Create node in Nuke
                node = nuke.createNode(node_type)
                created_nodes.append(node)

                # Send success response back to JavaScript
                webview.emit(
                    "node_created",
                    {
                        "success": True,
                        "name": node.name(),
                        "class": node.Class(),
                        "type": node_type,
                    },
                )

                print(f"[Python] Node created: {node.name()}")

                # Mark test as completed
                test_completed = True

            except Exception as e:
                print(f"[Python] Error creating node: {e}")
                webview.emit("node_created", {"success": False, "error": str(e)})

        # HTML with test UI
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Nuke IPC Test</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    padding: 20px;
                    background: #2b2b2b;
                    color: #fff;
                }
                button {
                    padding: 10px 20px;
                    font-size: 16px;
                    cursor: pointer;
                    background: #4CAF50;
                    color: white;
                    border: none;
                    border-radius: 4px;
                    margin: 5px;
                }
                button:hover {
                    background: #45a049;
                }
                #status {
                    margin-top: 20px;
                    padding: 10px;
                    background: #333;
                    border-radius: 4px;
                }
            </style>
        </head>
        <body>
            <h1>Nuke IPC Test</h1>
            <button id="createBtn">Create Grade Node</button>
            <button id="closeBtn">Close</button>
            <div id="status">Ready</div>

            <script>
                console.log('[Test] Initializing...');
                
                // Wait for window.auroraview to be ready
                function waitForBridge() {
                    if (window.auroraview && window.auroraview.send_event) {
                        console.log('[Test] Bridge ready!');
                        initTest();
                    } else {
                        console.log('[Test] Waiting for bridge...');
                        setTimeout(waitForBridge, 50);
                    }
                }
                
                function initTest() {
                    const status = document.getElementById('status');
                    
                    // Listen for node creation response
                    window.auroraview.on('node_created', function(data) {
                        console.log('[Test] Node created:', data);
                        if (data.success) {
                            status.textContent = 'Node created: ' + data.name;
                            status.style.background = '#4CAF50';
                        } else {
                            status.textContent = 'Error: ' + data.error;
                            status.style.background = '#f44336';
                        }
                    });
                    
                    // Create node button
                    document.getElementById('createBtn').onclick = function() {
                        console.log('[Test] Sending create_node signal...');
                        status.textContent = 'Creating node...';
                        status.style.background = '#333';
                        
                        window.auroraview.send_event('create_node', {
                            type: 'Grade'
                        });
                    };
                    
                    // Close button
                    document.getElementById('closeBtn').onclick = function() {
                        window.close();
                    };
                    
                    status.textContent = 'Bridge connected - Ready to test';
                }
                
                waitForBridge();
            </script>
        </body>
        </html>
        """

        # Load HTML
        webview.load_html(html)

        # Show WebView (non-blocking in Nuke)
        webview.show()

        # Wait for test to complete (with timeout)
        timeout = 10  # seconds
        start_time = time.time()

        while not test_completed and (time.time() - start_time) < timeout:
            # Process events
            webview.process_events()
            time.sleep(0.1)

        # Verify node was created
        assert test_completed, "Test did not complete within timeout"
        assert len(created_nodes) > 0, "No nodes were created"
        assert created_nodes[0].Class() == "Grade", f"Wrong node type: {created_nodes[0].Class()}"

        # Cleanup
        webview.close()
        for node in created_nodes:
            nuke.delete(node)

        print("[Test] ✓ Nuke IPC test passed!")
