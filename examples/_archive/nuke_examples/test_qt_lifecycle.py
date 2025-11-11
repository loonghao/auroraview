"""
Test Qt WebView lifecycle management in Nuke.

This example tests the proper cleanup of QtWebView when:
1. Closing the window
2. Exiting Nuke
3. Creating nodes (to test event processing)

Usage:
    In Nuke Script Editor:
        import sys
        from pathlib import Path

        examples_dir = Path(r'C:\\path\\to\\dcc_webview\\examples')
        sys.path.insert(0, str(examples_dir))

        import nuke_examples.test_qt_lifecycle as example
        example.run()
"""

import sys

try:
    import nuke

    NUKE_AVAILABLE = True
except ImportError:
    print("Warning: nuke module not available. This example requires Nuke.")
    NUKE_AVAILABLE = False
    nuke = None


def run():
    """Run Qt lifecycle test."""
    from auroraview import QtWebView

    print("\n" + "=" * 60)
    print("Qt WebView Lifecycle Test")
    print("=" * 60 + "\n")

    # Create Qt WebView
    webview = QtWebView(
        parent=None,  # Standalone window for testing
        title="Qt Lifecycle Test",
        width=600,
        height=400,
        dev_tools=True,
    )

    # Register event handler for node creation
    @webview.on("create_node")
    def handle_create_node(data):
        """Handle node creation request."""
        if not NUKE_AVAILABLE or not nuke:
            print("[Python] Nuke not available")
            webview.emit("node_created", {"error": "Nuke not available"})
            return

        node_type = data.get("type", "Grade")
        print(f"[Python] Creating {node_type} node...")

        try:
            # Create node
            node = nuke.createNode(node_type)

            # Send success response
            webview.emit(
                "node_created",
                {"success": True, "name": node.name(), "class": node.Class(), "type": node_type},
            )

            print(f"[Python] Node created: {node.name()}")

        except Exception as e:
            print(f"[Python] Error creating node: {e}")
            webview.emit("node_created", {"success": False, "error": str(e)})

    # Load test HTML
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>Qt Lifecycle Test</title>
        <script src="qrc:///qtwebchannel/qwebchannel.js"></script>
        <style>
            body {
                font-family: Arial, sans-serif;
                padding: 20px;
                background: #1e1e1e;
                color: #fff;
            }
            .container {
                max-width: 500px;
                margin: 0 auto;
            }
            h1 { color: #4CAF50; }
            .btn {
                background: #4CAF50;
                color: white;
                border: none;
                padding: 10px 20px;
                margin: 5px;
                cursor: pointer;
                border-radius: 4px;
            }
            .btn:hover { background: #45a049; }
            #status {
                margin-top: 20px;
                padding: 15px;
                background: #333;
                border-radius: 4px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🧪 Qt Lifecycle Test</h1>
            <p>Test proper cleanup and event processing.</p>

            <div>
                <h3>Create Nodes</h3>
                <button class="btn" onclick="createNode('Grade')">🎨 Grade</button>
                <button class="btn" onclick="createNode('Blur')">🌫️ Blur</button>
                <button class="btn" onclick="createNode('ColorCorrect')">🎨 ColorCorrect</button>
            </div>

            <div id="status">Ready</div>
        </div>

        <script>
            function createNode(type) {
                const status = document.getElementById('status');
                status.textContent = 'Creating ' + type + ' node...';
                status.style.background = '#333';

                window.auroraview.send_event('create_node', { type: type });
            }

            // Listen for node creation response
            window.auroraview.on('node_created', function(data) {
                const status = document.getElementById('status');
                if (data.success) {
                    status.textContent = '✅ Node created: ' + data.name;
                    status.style.background = '#2e7d32';
                } else {
                    status.textContent = '❌ Error: ' + (data.error || 'Unknown error');
                    status.style.background = '#c62828';
                }
            });

            console.log('[Test] Qt WebView initialized');
        </script>
    </body>
    </html>
    """

    webview.load_html(html)
    webview.show()

    print("\n✅ Qt WebView created")
    print("📝 Instructions:")
    print("   1. Click buttons to create nodes - verify UI updates immediately")
    print("   2. Close the window - should cleanup properly without errors")
    print("   3. Exit Nuke - window should close automatically, no errors")
    print("\n🔍 What to verify:")
    print("   ✓ No 'RuntimeError: Internal C++ object already deleted'")
    print("   ✓ No 'Cannot read property of undefined' in console")
    print("   ✓ No 'QEventDispatcherWin32::wakeUp: Failed to post a message'")
    print("   ✓ Window closes when Nuke exits")
    print()


if __name__ == "__main__":
    run()

