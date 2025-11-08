#!/usr/bin/env python
"""
Example 00: New API Showcase (v0.2.0)

This example demonstrates the new simplified API introduced in v0.2.0.

Key improvements:
- Simplified parameter names (parent instead of parent_hwnd)
- WebView.create() factory method
- Smart show() method with auto-detection
- DCC shortcuts (maya(), houdini(), blender())

Usage:
    python examples/00_new_api_showcase.py
"""

import logging
import sys

# Setup path to import auroraview
import _setup_path  # noqa: F401

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def main():
    """Main function showcasing the new API."""
    logger.info("=" * 60)
    logger.info("AuroraView - New API Showcase (v0.2.0)")
    logger.info("=" * 60)
    logger.info("")

    # HTML content
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>New API Showcase</title>
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
                color: white;
            }
            
            .container {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                border: 1px solid rgba(255, 255, 255, 0.2);
                border-radius: 20px;
                padding: 60px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                max-width: 800px;
                text-align: center;
            }
            
            h1 {
                font-size: 3em;
                margin-bottom: 20px;
                font-weight: 700;
            }
            
            .version {
                font-size: 1.5em;
                color: #ffd700;
                margin-bottom: 30px;
            }
            
            .features {
                text-align: left;
                margin: 40px 0;
                padding: 30px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 15px;
            }
            
            .features h2 {
                color: #ffd700;
                margin-bottom: 20px;
                font-size: 1.8em;
            }
            
            .features ul {
                list-style: none;
                padding-left: 0;
            }
            
            .features li {
                padding: 12px 0;
                padding-left: 35px;
                position: relative;
                font-size: 1.1em;
            }
            
            .features li:before {
                content: "✨";
                position: absolute;
                left: 0;
                font-size: 1.2em;
            }
            
            .code-example {
                background: rgba(0, 0, 0, 0.4);
                border-radius: 10px;
                padding: 20px;
                margin: 20px 0;
                text-align: left;
                font-family: 'Courier New', monospace;
                font-size: 0.9em;
                overflow-x: auto;
            }
            
            .code-example pre {
                margin: 0;
                color: #a8e6cf;
            }
            
            button {
                padding: 15px 40px;
                font-size: 1.2em;
                border: none;
                border-radius: 10px;
                cursor: pointer;
                background: linear-gradient(135deg, #ffd700 0%, #ffed4e 100%);
                color: #333;
                margin: 10px;
                transition: all 0.3s ease;
                box-shadow: 0 4px 15px rgba(255, 215, 0, 0.3);
                font-weight: 600;
            }
            
            button:hover {
                transform: translateY(-3px);
                box-shadow: 0 6px 20px rgba(255, 215, 0, 0.5);
            }
            
            button:active {
                transform: translateY(0);
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 AuroraView</h1>
            <div class="version">v0.2.0 - New API</div>
            
            <div class="features">
                <h2>✨ What's New</h2>
                <ul>
                    <li>Simplified parameter names</li>
                    <li>WebView.create() factory method</li>
                    <li>Smart show() with auto-detection</li>
                    <li>DCC shortcuts (maya, houdini, blender)</li>
                    <li>Auto timer management</li>
                </ul>
            </div>
            
            <div class="code-example">
                <pre># Old API (6 lines)
webview = NativeWebView(title="App", parent=hwnd)
webview.load_url("http://localhost:3000")
webview.show()
timer = EventTimer(webview)
timer.on_close(lambda: timer.stop())
timer.start()

# New API (2 lines!)
webview = WebView.create("App", parent=hwnd, url="http://localhost:3000")
webview.show()  # Auto timer!</pre>
            </div>
            
            <button onclick="alert('Hello from the new API! 🎉')">Try Me!</button>
        </div>
    </body>
    </html>
    """

    # Example 1: Using WebView.create() (recommended)
    logger.info("Example 1: Using WebView.create()")
    logger.info("-" * 60)

    webview = WebView.create(
        title="AuroraView - New API Showcase",
        width=900,
        height=700,
        html=html_content,
        debug=True,  # New parameter name (was dev_tools)
        frame=True,  # New parameter name (was decorations)
    )

    logger.info("[OK] WebView created with new API")
    logger.info("")

    # Example 2: Smart show() method
    logger.info("Example 2: Smart show() method")
    logger.info("-" * 60)
    logger.info("Calling show() - will auto-detect standalone mode and block")
    logger.info("Close the window to exit.")
    logger.info("")

    try:
        # Smart show() - auto-detects standalone mode and blocks
        webview.show()  # Equivalent to show(wait=True) for standalone
    except Exception as e:
        logger.error(f"Error showing WebView: {e}")
        return 1

    logger.info("")
    logger.info("=" * 60)
    logger.info("Window closed. Exiting.")
    logger.info("=" * 60)
    logger.info("")
    logger.info("💡 Tips:")
    logger.info("  - Use WebView.create() for cleaner code")
    logger.info("  - Use WebView.maya() for Maya integration")
    logger.info("  - Use WebView.houdini() for Houdini integration")
    logger.info("  - Use WebView.blender() for Blender integration")
    logger.info("  - show() auto-detects standalone/embedded mode")

    return 0


if __name__ == "__main__":
    sys.exit(main())
