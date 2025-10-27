#!/usr/bin/env python
"""
Test Window Example - Minimal WebView Window

This is a minimal example to test if the WebView window can be displayed.
"""

import sys
import logging
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function."""
    logger.info("=" * 70)
    logger.info("AuroraView - Test Window")
    logger.info("=" * 70)
    logger.info("")
    
    try:
        # Step 1: Create WebView
        logger.info("Step 1: Creating WebView instance...")
        webview = WebView(
            title="AuroraView Test Window",
            width=600,
            height=400
        )
        logger.info(f"✓ WebView created: {webview}")
        logger.info("")
        
        # Step 2: Create simple HTML
        logger.info("Step 2: Creating HTML content...")
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Window</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    height: 100vh;
                    margin: 0;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                }
                .container {
                    background: white;
                    padding: 40px;
                    border-radius: 10px;
                    text-align: center;
                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                }
                h1 {
                    color: #333;
                    margin: 0 0 20px 0;
                }
                p {
                    color: #666;
                    margin: 10px 0;
                }
                button {
                    background: #667eea;
                    color: white;
                    border: none;
                    padding: 10px 20px;
                    border-radius: 5px;
                    cursor: pointer;
                    font-size: 16px;
                    margin-top: 20px;
                }
                button:hover {
                    background: #5568d3;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🚀 AuroraView Test</h1>
                <p>If you can see this, the WebView is working!</p>
                <p>Close this window to exit.</p>
                <button onclick="alert('Button clicked!')">Click Me</button>
            </div>
        </body>
        </html>
        """
        logger.info("✓ HTML content created")
        logger.info("")
        
        # Step 3: Load HTML
        logger.info("Step 3: Loading HTML content...")
        webview.load_html(html)
        logger.info("✓ HTML loaded")
        logger.info("")
        
        # Step 4: Register event handler
        logger.info("Step 4: Registering event handler...")
        @webview.on("test_event")
        def handle_test_event(data):
            logger.info(f"✓ Event received: {data}")
        logger.info("✓ Event handler registered")
        logger.info("")
        
        # Step 5: Show window
        logger.info("Step 5: Showing WebView window...")
        logger.info("=" * 70)
        logger.info("Window should appear now. Close it to exit.")
        logger.info("=" * 70)
        logger.info("")
        
        webview.show()
        
        logger.info("")
        logger.info("=" * 70)
        logger.info("Window closed. Test completed.")
        logger.info("=" * 70)
        
        return 0
        
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())

