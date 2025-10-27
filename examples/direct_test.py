#!/usr/bin/env python
"""
Direct Test - Call show() directly on core WebView
"""

import sys
import logging
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from dcc_webview._core import WebView as CoreWebView

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function to run the direct test."""
    logger.info("=" * 70)
    logger.info("Direct Core WebView Test")
    logger.info("=" * 70)
    
    try:
        # Create core WebView directly
        logger.info("Creating core WebView...")
        webview = CoreWebView(
            title="Direct Test",
            width=400,
            height=300
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
        logger.info("Calling show()...")
        logger.info("Please wait for the window to appear...")
        logger.info("Close the window to exit.")
        logger.info("")
        
        webview.show()
        
        logger.info("show() returned.")
        
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

