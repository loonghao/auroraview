#!/usr/bin/env python
"""
Maya Integration Example - Async Mode

This example demonstrates how to integrate DCC WebView with Autodesk Maya
using the non-blocking show_async() method.

This approach prevents blocking Maya's main thread, allowing the application
to remain responsive while the WebView is running.

Usage in Maya:
    from examples import maya_integration_async
    maya_integration_async.create_maya_tool()
"""

import sys
import logging
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def create_maya_tool():
    """Create a non-blocking DCC WebView tool for Maya."""
    logger.info("=" * 70)
    logger.info("AuroraView - Maya Integration Example (Async Mode)")
    logger.info("=" * 70)
    logger.info("")
    
    # Create WebView
    logger.info("Creating Maya WebView tool...")
    webview = WebView(
        title="AuroraView - Maya Tool (Async)",
        width=600,
        height=500
    )
    logger.info(f"✓ Created: {webview}")
    logger.info("")
    
    # Create HTML UI
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Maya Tool - Async Mode</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: #2b2b2b;
                color: #e0e0e0;
                padding: 20px;
            }
            
            .header {
                background: #1a1a1a;
                padding: 15px;
                border-radius: 5px;
                margin-bottom: 20px;
                border-left: 4px solid #00cc00;
            }
            
            .header h1 {
                font-size: 1.5em;
                margin-bottom: 5px;
            }
            
            .header p {
                font-size: 0.9em;
                color: #999;
            }
            
            .section {
                background: #1a1a1a;
                padding: 15px;
                border-radius: 5px;
                margin-bottom: 15px;
            }
            
            .section h2 {
                font-size: 1.1em;
                margin-bottom: 10px;
                color: #00cc00;
            }
            
            .status-box {
                background: #0a3a0a;
                border: 1px solid #0d5d0d;
                color: #90ee90;
                padding: 15px;
                border-radius: 3px;
                margin-top: 10px;
                font-size: 0.9em;
                font-family: monospace;
            }
            
            .info-box {
                background: #0a1a3a;
                border: 1px solid #0d2d5d;
                color: #87ceeb;
                padding: 10px;
                border-radius: 3px;
                font-size: 0.85em;
                margin-top: 10px;
            }
            
            button {
                padding: 10px 20px;
                background: #00cc00;
                color: #000;
                border: none;
                border-radius: 3px;
                cursor: pointer;
                font-weight: 600;
                margin-top: 10px;
            }
            
            button:hover {
                background: #00aa00;
            }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>🎨 Maya Scene Tool (Async Mode)</h1>
            <p>Non-blocking WebView integration with Maya</p>
        </div>
        
        <div class="section">
            <h2>Status</h2>
            <div class="status-box">
                ✓ WebView is running in background thread<br>
                ✓ Maya main thread is responsive<br>
                ✓ You can continue working in Maya
            </div>
        </div>
        
        <div class="section">
            <h2>Features</h2>
            <div class="info-box">
                <strong>Non-blocking:</strong> Maya remains responsive while WebView is open<br>
                <strong>Thread-safe:</strong> WebView runs in a separate thread<br>
                <strong>Seamless:</strong> Close the window to finish
            </div>
        </div>
        
        <div class="section">
            <h2>Actions</h2>
            <button onclick="testAction()">Test Action</button>
            <div class="status-box" id="actionStatus" style="display:none; margin-top: 10px;">
                Action executed!
            </div>
        </div>
        
        <script>
            function testAction() {
                const status = document.getElementById('actionStatus');
                status.style.display = 'block';
                setTimeout(() => {
                    status.style.display = 'none';
                }, 2000);
            }
        </script>
    </body>
    </html>
    """
    
    # Load HTML
    logger.info("Loading Maya tool UI...")
    webview.load_html(html_content)
    logger.info("✓ UI loaded")
    logger.info("")
    
    # Register event handlers
    logger.info("Registering event handlers...")
    
    @webview.on("test_action")
    def handle_test_action(data):
        """Handle test action from UI."""
        logger.info(f"✓ Test action executed: {data}")
    
    logger.info("✓ Event handlers registered")
    logger.info("")
    
    # Show the tool in background thread (non-blocking)
    logger.info("Showing Maya tool in background thread...")
    logger.info("")
    
    try:
        webview.show_async()
        logger.info("✓ WebView started in background thread")
        logger.info("")
        logger.info("=" * 70)
        logger.info("Maya is now responsive!")
        logger.info("=" * 70)
        logger.info("")
        logger.info("You can:")
        logger.info("  • Use the WebView UI")
        logger.info("  • Continue working in Maya")
        logger.info("  • Close the WebView window to finish")
        logger.info("")
        
        # Wait for the WebView to close
        webview.wait()
        
    except Exception as e:
        logger.error(f"Error showing WebView: {e}")
        return 1
    
    logger.info("")
    logger.info("=" * 70)
    logger.info("Maya tool closed.")
    logger.info("=" * 70)
    
    return 0


if __name__ == "__main__":
    sys.exit(create_maya_tool())

