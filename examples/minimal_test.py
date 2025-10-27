#!/usr/bin/env python
"""
Minimal Test - Debug WebView Display

This is a minimal test to debug why the window is not displaying.
"""

import sys
import logging
import time
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function to run the minimal test."""
    logger.info("=" * 70)
    logger.info("Minimal WebView Test")
    logger.info("=" * 70)
    
    try:
        # Create WebView
        logger.info("Creating WebView...")
        webview = WebView(
            title="Minimal Test",
            width=400,
            height=300,
            dev_tools=True
        )
        logger.info(f"✓ Created: {webview}")
        
        # Load simple HTML
        logger.info("Loading HTML...")
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test</title>
            <style>
                body { 
                    font-family: Arial; 
                    background: #f0f0f0;
                    padding: 20px;
                }
                h1 { color: #333; }
            </style>
        </head>
        <body>
            <h1>Hello from WebView!</h1>
            <p>If you see this, the window is working.</p>
        </body>
        </html>
        """
        webview.load_html(html)
        logger.info("✓ HTML loaded")
        
        # Show window
        logger.info("Showing window...")
        logger.info("Please wait for the window to appear...")
        logger.info("Close the window to exit.")
        logger.info("")
        
        webview.show()
        
        logger.info("Window closed.")
        
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

