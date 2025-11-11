"""Houdini DCC Integration Example.

Requirements:
    pip install auroraview[qt]  # Installs QtPy for Qt compatibility

This example demonstrates how to integrate AuroraView into Houdini using QtPy
as a middleware layer to handle Qt version compatibility.
"""

import logging

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# HTML content for the WebView
HTML_CONTENT = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Houdini WebView (DCC Mode)</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 500px;
            width: 100%;
        }
        h1 {
            color: #333;
            margin-bottom: 10px;
            font-size: 28px;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 14px;
        }
        .badge {
            display: inline-block;
            background: #10b981;
            color: white;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 600;
            margin-bottom: 20px;
        }
        button {
            background: #667eea;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 8px;
            font-size: 16px;
            cursor: pointer;
            width: 100%;
            margin-bottom: 10px;
            transition: all 0.3s;
        }
        button:hover {
            background: #5568d3;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }
        #output {
            margin-top: 20px;
            padding: 15px;
            background: #f3f4f6;
            border-radius: 8px;
            min-height: 60px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            color: #374151;
        }
        .features {
            margin-top: 20px;
            padding: 15px;
            background: #eff6ff;
            border-radius: 8px;
            border-left: 4px solid #3b82f6;
        }
        .features h3 {
            color: #1e40af;
            font-size: 14px;
            margin-bottom: 10px;
        }
        .features ul {
            list-style: none;
            padding: 0;
        }
        .features li {
            color: #1e3a8a;
            font-size: 13px;
            padding: 4px 0;
            padding-left: 20px;
            position: relative;
        }
        .features li:before {
            content: "✓";
            position: absolute;
            left: 0;
            color: #10b981;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="container">
        <span class="badge">DCC Integration Mode</span>
        <h1>Houdini WebView</h1>
        <p class="subtitle">Pure Rust • No Event Loop • No PySide Dependency</p>

        <button onclick="sendToHoudini()">Send Message to Houdini</button>
        <button onclick="getHoudiniInfo()">Get Houdini Info</button>

        <div id="output">Click a button to test communication...</div>

        <div class="features">
            <h3>Key Features:</h3>
            <ul>
                <li>No event loop conflicts</li>
                <li>No PySide2/PySide6 required</li>
                <li>Non-blocking UI</li>
                <li>Uses Qt message pump</li>
            </ul>
        </div>
    </div>

    <script>
        function sendToHoudini() {
            const message = { action: 'test', timestamp: new Date().toISOString() };
            window.emit('houdini_message', message);
            document.getElementById('output').textContent = 'Sent: ' + JSON.stringify(message, null, 2);
        }

        function getHoudiniInfo() {
            window.emit('get_info', {});
            document.getElementById('output').textContent = 'Requesting Houdini info...';
        }

        // Listen for responses from Houdini
        window.addEventListener('houdini_response', (event) => {
            document.getElementById('output').textContent = 'Response: ' + JSON.stringify(event.detail, null, 2);
        });
    </script>
</body>
</html>
"""

# Global reference to prevent garbage collection
_webview_instance = None
_timer_instance = None


def show():
    """Show the WebView window using DCC integration mode."""
    global _webview_instance, _timer_instance

    try:
        import hou
    except ImportError:
        logger.error("This example must be run inside Houdini")
        return

    # Import WebView
    try:
        from auroraview import WebView
    except ImportError:
        logger.error("AuroraView not found. Please install it first.")
        return

    # Get Houdini main window HWND
    main_window = hou.qt.mainWindow()
    hwnd = int(main_window.winId())
    logger.info(f"Houdini main window HWND: {hwnd}")

    # Create WebView using DCC integration mode
    logger.info("Creating WebView for DCC integration...")
    webview = WebView.for_dcc(
        parent_hwnd=hwnd,
        title="Houdini WebView (DCC Mode)",
        width=650,
        height=600
    )

    # Register event handlers
    @webview.on("houdini_message")
    def handle_message(data):
        logger.info(f"Received message from WebView: {data}")
        # Send response back
        webview.emit("houdini_response", {
            "status": "received",
            "original": data,
            "houdini_version": hou.applicationVersionString()
        })

    @webview.on("get_info")
    def handle_get_info(data):
        logger.info("WebView requested Houdini info")
        webview.emit("houdini_response", {
            "version": hou.applicationVersionString(),
            "platform": hou.applicationPlatformInfo(),
            "license": hou.licenseCategory().name()
        })

    # Load HTML content
    webview.load_html(HTML_CONTENT)

    # Setup Qt timer to process messages (REQUIRED for DCC mode!)
    # Using QtPy for Qt version compatibility
    try:
        from qtpy.QtCore import QTimer
        logger.info("Using QtPy for Qt compatibility")
    except ImportError:
        logger.error("QtPy required! Install with: pip install auroraview[qt]")
        logger.error("QtPy provides compatibility across PySide2/PySide6/PyQt5/PyQt6")
        return

    timer = QTimer()
    timer.timeout.connect(webview.process_messages)
    timer.start(16)  # 60 FPS

    # Store references to prevent garbage collection
    _webview_instance = webview
    _timer_instance = timer

    logger.info("[OK] WebView created successfully!")
    logger.info("[OK] Houdini UI should remain responsive")
    logger.info("[OK] You can interact with both Houdini and WebView simultaneously")

    return webview


if __name__ == "__main__":
    show()

