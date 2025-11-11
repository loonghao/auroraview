"""Maya DCC Integration Example.

Requirements:
    pip install auroraview[qt]  # Installs QtPy for Qt compatibility

This example demonstrates how to integrate AuroraView into Maya using QtPy
as a middleware layer to handle Qt version compatibility.
"""

import logging
from auroraview import WebView

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
    <title>Maya WebView</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0;
        }
        .container {
            background: white;
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 500px;
        }
        h1 { color: #333; margin-bottom: 20px; }
        button {
            background: #667eea;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 16px;
            margin: 5px;
        }
        button:hover { background: #5568d3; }
        #status { margin-top: 20px; padding: 10px; background: #f0f0f0; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Maya WebView Tool</h1>
        <p>This WebView is integrated with Maya using QtPy!</p>
        <button onclick="sendToMaya()">Send Message to Maya</button>
        <button onclick="getMayaInfo()">Get Maya Info</button>
        <div id="status"></div>
    </div>
    
    <script>
        function sendToMaya() {
            window.auroraview.send_event('maya_message', {
                text: 'Hello from WebView!',
                timestamp: new Date().toISOString()
            });
            document.getElementById('status').innerHTML = '✓ Message sent to Maya';
        }
        
        function getMayaInfo() {
            window.auroraview.send_event('get_info', {});
            document.getElementById('status').innerHTML = '⏳ Requesting Maya info...';
        }
        
        // Listen for responses from Maya
        window.auroraview.on('maya_response', function(data) {
            document.getElementById('status').innerHTML = 
                '<strong>Maya Response:</strong><br>' + JSON.stringify(data, null, 2);
        });
    </script>
</body>
</html>
"""

# Global references to prevent garbage collection
_webview_instance = None
_timer_instance = None


def show():
    """Show the WebView integrated with Maya.
    
    This function creates a WebView that integrates with Maya's Qt event loop
    using QtPy for Qt version compatibility.
    """
    global _webview_instance, _timer_instance
    
    try:
        import maya.OpenMayaUI as omui
    except ImportError:
        logger.error("This example must be run inside Maya")
        return
    
    # Get Maya main window HWND
    try:
        from qtpy import QtWidgets
        try:
            from shiboken2 import wrapInstance
        except ImportError:
            from shiboken6 import wrapInstance
        
        maya_main_window_ptr = omui.MQtUtil.mainWindow()
        maya_main_window = wrapInstance(int(maya_main_window_ptr), QtWidgets.QWidget)
        hwnd = int(maya_main_window.winId())
        
        logger.info(f"Maya main window HWND: {hwnd}")
    except Exception as e:
        logger.error(f"Failed to get Maya main window: {e}")
        return
    
    # Create WebView for DCC integration
    logger.info("Creating WebView...")
    webview = WebView.for_dcc(
        parent_hwnd=hwnd,
        title="Maya WebView (DCC Mode)",
        width=650,
        height=600
    )
    
    # Register event handlers
    @webview.on("maya_message")
    def handle_message(data):
        logger.info(f"Received message from WebView: {data}")
        import maya.cmds as cmds
        # Send response back
        webview.emit("maya_response", {
            "status": "received",
            "maya_version": cmds.about(version=True),
            "message": data.get("text", "")
        })
    
    @webview.on("get_info")
    def handle_get_info(data):
        logger.info("WebView requested Maya info")
        import maya.cmds as cmds
        webview.emit("maya_response", {
            "version": cmds.about(version=True),
            "api_version": cmds.about(apiVersion=True),
            "os": cmds.about(operatingSystem=True)
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
    logger.info("[OK] Maya UI should remain responsive")
    
    return webview


if __name__ == "__main__":
    show()

