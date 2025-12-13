"""Desktop Events Demo - Demonstrates new desktop event features.

This example showcases:
1. Plugin invoke() method - Call native plugins from JavaScript
2. File drop events - Handle file drag and drop
3. Event cancellation - Cancel closing event
4. Debounce/throttle - Event rate limiting

Note: This example uses the low-level WebView API for demonstration.
For most use cases, prefer QtWebView, AuroraView, or run_desktop.

Run with:
    python examples/desktop_events_demo.py
"""

import os
import sys

# Add parent directory to path for development
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from auroraview import WebView
from auroraview.core.events import WindowEvent

# HTML content demonstrating desktop events
HTML_CONTENT = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Desktop Events Demo</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #fff;
            min-height: 100vh;
        }
        h1 { color: #00d9ff; }
        h2 { color: #ff6b6b; margin-top: 30px; }
        .section {
            background: rgba(255,255,255,0.1);
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
        }
        .drop-zone {
            border: 2px dashed #00d9ff;
            border-radius: 8px;
            padding: 40px;
            text-align: center;
            transition: all 0.3s;
        }
        .drop-zone.hover {
            background: rgba(0, 217, 255, 0.2);
            border-color: #ff6b6b;
        }
        button {
            background: #00d9ff;
            border: none;
            color: #1a1a2e;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px;
            font-weight: bold;
        }
        button:hover { background: #00b8d9; }
        .log {
            background: #0a0a15;
            border-radius: 4px;
            padding: 10px;
            font-family: monospace;
            font-size: 12px;
            max-height: 200px;
            overflow-y: auto;
        }
        .log-entry { margin: 2px 0; }
        .log-entry.info { color: #00d9ff; }
        .log-entry.success { color: #4ade80; }
        .log-entry.error { color: #ff6b6b; }
    </style>
</head>
<body>
    <h1>Desktop Events Demo</h1>

    <div class="section">
        <h2>1. Plugin Invoke</h2>
        <p>Test native plugin commands using auroraview.invoke()</p>
        <button onclick="testFsPlugin()">Test FS Plugin</button>
        <button onclick="testDialogPlugin()">Test Dialog Plugin</button>
        <button onclick="testClipboardPlugin()">Test Clipboard Plugin</button>
    </div>

    <div class="section">
        <h2>2. File Drop</h2>
        <p>Drag and drop files here:</p>
        <div id="dropZone" class="drop-zone">
            Drop files here
        </div>
    </div>

    <div class="section">
        <h2>3. Debounce/Throttle</h2>
        <p>Move your mouse rapidly over this area:</p>
        <div id="mouseArea" style="background: rgba(0,217,255,0.2); padding: 40px; text-align: center;">
            Mouse move area (throttled to 100ms)
        </div>
        <p>Move count: <span id="moveCount">0</span></p>
    </div>

    <div class="section">
        <h2>Event Log</h2>
        <div id="log" class="log"></div>
    </div>

    <script>
        // Logging utility
        function log(message, type = 'info') {
            const logEl = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = 'log-entry ' + type;
            entry.textContent = new Date().toLocaleTimeString() + ' - ' + message;
            logEl.insertBefore(entry, logEl.firstChild);
        }

        // Wait for AuroraView bridge
        window.addEventListener('auroraviewready', function() {
            log('AuroraView bridge ready!', 'success');

            // Subscribe to file drop events
            auroraview.on('file_drop', function(data) {
                log('Files dropped: ' + JSON.stringify(data.files.map(f => f.name)), 'success');
            });

            auroraview.on('file_drop_hover', function(data) {
                const dropZone = document.getElementById('dropZone');
                if (data.hovering) {
                    dropZone.classList.add('hover');
                    dropZone.textContent = 'Release to drop ' + data.files.length + ' file(s)';
                } else {
                    dropZone.classList.remove('hover');
                    dropZone.textContent = 'Drop files here';
                }
            });

            auroraview.on('file_drop_cancelled', function(data) {
                const dropZone = document.getElementById('dropZone');
                dropZone.classList.remove('hover');
                dropZone.textContent = 'Drop files here';
                log('Drop cancelled: ' + data.reason, 'info');
            });

            // Throttled mouse move handler
            var moveCount = 0;
            var throttledHandler = auroraview.utils.throttle(function(e) {
                moveCount++;
                document.getElementById('moveCount').textContent = moveCount;
            }, 100);

            document.getElementById('mouseArea').addEventListener('mousemove', throttledHandler);
        });

        // Plugin test functions
        async function testFsPlugin() {
            log('Testing FS plugin...');
            try {
                // Check if temp directory exists
                const result = await auroraview.invoke('plugin:fs|exists', { path: 'C:\\\\Windows' });
                log('FS exists result: ' + JSON.stringify(result), 'success');
            } catch (e) {
                log('FS error: ' + e.message, 'error');
            }
        }

        async function testDialogPlugin() {
            log('Testing Dialog plugin...');
            try {
                const result = await auroraview.invoke('plugin:dialog|message', {
                    title: 'Hello',
                    message: 'This is a test message from AuroraView!',
                    kind: 'info'
                });
                log('Dialog result: ' + JSON.stringify(result), 'success');
            } catch (e) {
                log('Dialog error: ' + e.message, 'error');
            }
        }

        async function testClipboardPlugin() {
            log('Testing Clipboard plugin...');
            try {
                // Write to clipboard
                await auroraview.invoke('plugin:clipboard|write_text', { text: 'Hello from AuroraView!' });
                log('Clipboard write success', 'success');

                // Read from clipboard
                const result = await auroraview.invoke('plugin:clipboard|read_text', {});
                log('Clipboard read: ' + JSON.stringify(result), 'success');
            } catch (e) {
                log('Clipboard error: ' + e.message, 'error');
            }
        }

        log('Page loaded, waiting for AuroraView bridge...');
    </script>
</body>
</html>
"""


def main():
    """Run the desktop events demo."""
    print("Starting Desktop Events Demo...")
    print("Features demonstrated:")
    print("  1. Plugin invoke() method")
    print("  2. File drop events")
    print("  3. Debounce/throttle utilities")
    print()

    # Create WebView
    webview = WebView(
        title="Desktop Events Demo",
        width=900,
        height=800,
        html=HTML_CONTENT,
        debug=True,
    )

    # Register event handlers
    # File drop events now provide full native file paths
    @webview.on(WindowEvent.FILE_DROP)
    def on_file_drop(data):
        paths = data.get("paths", [])
        position = data.get("position", {})
        print(f"[Python] Files dropped at ({position.get('x')}, {position.get('y')}):")
        for path in paths:
            print(f"  - {path}")

    @webview.on(WindowEvent.FILE_DROP_HOVER)
    def on_file_hover(data):
        if data.get("hovering"):
            paths = data.get("paths", [])
            print(f"[Python] Dragging {len(paths)} file(s) over window")

    @webview.on(WindowEvent.CLOSING)
    def on_closing(data):
        print("[Python] Window closing...")
        return True  # Allow close

    # Show the WebView
    webview.show()


if __name__ == "__main__":
    main()
