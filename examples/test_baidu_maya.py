#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Simple test script for loading Baidu in Maya.

This script is specifically designed to work in Maya environment.

Usage in Maya Script Editor:
    import sys
    sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
    
    from test_baidu_maya import test_baidu
    test_baidu()
"""

import sys
import time
import logging
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def test_baidu():
    """Test loading Baidu in Maya."""
    logger.info("=" * 60)
    logger.info("Testing Baidu.com in Maya")
    logger.info("=" * 60)
    
    # Check if in Maya
    try:
        import maya.cmds as cmds
        logger.info("✓ Running in Maya")
    except ImportError:
        logger.warning("⚠ Not running in Maya, but will continue...")
    
    # Create WebView
    logger.info("Creating WebView...")
    try:
        webview = WebView(
            title="Test: Baidu.com",
            width=1200,
            height=800,
            debug=True
        )
        logger.info("✓ WebView created successfully")
    except Exception as e:
        logger.error(f"✗ Failed to create WebView: {e}")
        logger.error("")
        logger.error("TROUBLESHOOTING:")
        logger.error("1. Make sure WebView2 runtime is installed and up-to-date")
        logger.error("2. Try restarting Maya")
        logger.error("3. Run diagnostic: from test_maya_remote_url import diagnose; diagnose()")
        return None
    
    # Load HTML first (to avoid initialization issues)
    logger.info("Loading initial HTML...")
    webview.load_html("""
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <style>
                body {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    font-family: Arial, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                }
                .loader {
                    text-align: center;
                }
                .spinner {
                    border: 4px solid rgba(255,255,255,0.3);
                    border-top: 4px solid white;
                    border-radius: 50%;
                    width: 40px;
                    height: 40px;
                    animation: spin 1s linear infinite;
                    margin: 0 auto 20px;
                }
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
            </style>
        </head>
        <body>
            <div class="loader">
                <div class="spinner"></div>
                <h2>正在加载百度...</h2>
                <p>请稍候</p>
            </div>
        </body>
        </html>
    """)
    logger.info("✓ Initial HTML loaded")
    
    # Show window (show() now uses show_async() internally)
    logger.info("Showing window...")
    try:
        webview.show()
        logger.info("✓ Window shown (non-blocking)")
    except Exception as e:
        logger.error(f"✗ Failed to show window: {e}")
        return None
    
    # Wait a bit for window to appear
    time.sleep(0.5)
    
    # Load Baidu
    logger.info("Loading Baidu.com...")
    try:
        webview.load_url("https://www.baidu.com")
        logger.info("✓ Baidu.com loaded")
    except Exception as e:
        logger.error(f"✗ Failed to load Baidu: {e}")
        return webview
    
    logger.info("")
    logger.info("=" * 60)
    logger.info("SUCCESS!")
    logger.info("=" * 60)
    logger.info("The Baidu website should now be visible in the WebView window.")
    logger.info("You can:")
    logger.info("  - Search on Baidu")
    logger.info("  - Open DevTools (F12) to inspect the page")
    logger.info("  - Close the window when done")
    logger.info("=" * 60)
    
    return webview


def test_with_retry(max_retries=3):
    """Test with retry logic."""
    for attempt in range(max_retries):
        logger.info(f"Attempt {attempt + 1}/{max_retries}")
        
        webview = test_baidu()
        
        if webview:
            return webview
        
        if attempt < max_retries - 1:
            logger.info(f"Retrying in 1 second...")
            time.sleep(1)
    
    logger.error("Failed after all retries")
    return None


if __name__ == "__main__":
    # For standalone testing
    test_baidu()
    input("\nPress Enter to exit...")

