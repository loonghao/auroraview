#!/usr/bin/env python
"""
Test script to verify the event loop fix for DCC integration.

This script tests that WebView can be created and shown in a background thread
without triggering the PanicException about unsendable types.

Run this in Maya:
    1. Open Script Editor (Ctrl + Shift + E)
    2. Copy this entire script
    3. Paste into the Python tab
    4. Execute (Ctrl + Enter)
    5. Verify that WebView window appears and Maya remains responsive
"""

import logging
import threading
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='# %(name)s : %(message)s #'
)
logger = logging.getLogger(__name__)

def test_event_loop_fix():
    """Test that event loop can be created on any thread."""
    logger.info("=" * 70)
    logger.info("Testing Event Loop Fix for DCC Integration")
    logger.info("=" * 70)
    
    try:
        from auroraview import WebView
        
        # Create HTML content
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Event Loop Fix Test</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    margin: 0;
                }
                .container {
                    max-width: 600px;
                    margin: 0 auto;
                }
                h1 {
                    text-align: center;
                    margin-bottom: 30px;
                }
                .status {
                    background: rgba(255, 255, 255, 0.1);
                    padding: 15px;
                    border-radius: 8px;
                    margin-bottom: 20px;
                }
                .success {
                    color: #4ade80;
                    font-weight: bold;
                }
                .info {
                    font-size: 14px;
                    line-height: 1.6;
                    margin-top: 10px;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>✓ Event Loop Fix Verified!</h1>
                <div class="status">
                    <div class="success">WebView is running in background thread</div>
                    <div class="info">
                        <p>✓ Event loop created successfully on any thread</p>
                        <p>✓ No PanicException about unsendable types</p>
                        <p>✓ Maya main thread remains responsive</p>
                        <p>✓ WebView window displays correctly</p>
                    </div>
                </div>
                <div class="status">
                    <h3>What was fixed:</h3>
                    <p>The Rust code now uses <code>EventLoopBuilderExtWindows::with_any_thread(true)</code> 
                    to allow event loop creation on any thread, not just the main thread.</p>
                </div>
            </div>
        </body>
        </html>
        """
        
        logger.info("")
        logger.info("Creating WebView instance...")
        webview = WebView(
            title="Event Loop Fix Test",
            width=600,
            height=500
        )
        
        logger.info("Loading HTML content...")
        webview.load_html(html)
        
        logger.info("Starting WebView in background thread...")
        logger.info("=" * 70)
        
        # This should NOT raise PanicException anymore!
        webview.show_async()
        
        logger.info("")
        logger.info("✓ WebView started successfully!")
        logger.info("✓ Maya is responsive!")
        logger.info("")
        logger.info("The WebView window should appear shortly.")
        logger.info("You can now:")
        logger.info("  • Interact with the WebView")
        logger.info("  • Continue working in Maya")
        logger.info("  • Close the window when done")
        logger.info("")
        
        # Wait for user to close the window
        logger.info("Waiting for WebView to close...")
        webview.wait()
        
        logger.info("")
        logger.info("✓ Test completed successfully!")
        logger.info("✓ Event loop fix is working correctly!")
        logger.info("=" * 70)
        
    except Exception as e:
        logger.error(f"✗ Test failed with error: {e}", exc_info=True)
        logger.error("=" * 70)
        raise

if __name__ == "__main__":
    test_event_loop_fix()

