#!/usr/bin/env python
"""
Blender WebView Example

This example demonstrates how to use AuroraView in Blender.

Features:
- Standalone window creation in Blender
- HTML/CSS/JavaScript rendering
- Proper window lifecycle management

Usage:
    1. Open Blender
    2. Open Scripting workspace
    3. Load this script
    4. Run script (Alt+P or click Run Script button)

Note:
    This example uses show_blocking() which will block Blender's UI
    until the WebView window is closed. This is intentional for
    demonstration purposes. For production use, consider using
    show() with Blender's timer system.
"""

import logging
import sys
from pathlib import Path

# Setup path to import auroraview
# Method 1: Try to use __file__ with _setup_path (works when running from file)
try:
    _script_file = Path(__file__).resolve()
    _examples_dir = _script_file.parent.parent
    sys.path.insert(0, str(_examples_dir))

    try:
        import _setup_path  # noqa: F401

        print(f"[Method 1] Using _setup_path from: {_examples_dir}")
    except ModuleNotFoundError:
        # Fallback: manually add python path
        _project_root = _script_file.parent.parent.parent
        _python_dir = _project_root / "python"
        if str(_python_dir) not in sys.path:
            sys.path.insert(0, str(_python_dir))
            print(f"[Method 1b] Added to sys.path: {_python_dir}")
except (NameError, OSError):
    # Method 2: Use hardcoded path (fallback for Blender script editor)
    # IMPORTANT: Update this path to match your installation
    _python_dir = Path(r"C:\Users\hallo\Documents\augment-projects\dcc_webview\python")

    if _python_dir.exists():
        if str(_python_dir) not in sys.path:
            sys.path.insert(0, str(_python_dir))
            print(f"[Method 2] Added to sys.path: {_python_dir}")
    else:
        print(f"ERROR: Python directory not found: {_python_dir}")
        print("Please update the path in this script to match your installation.")
        raise

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def main():
    """Main function to run the Blender example."""
    logger.info("=" * 60)
    logger.info("AuroraView - Blender Example")
    logger.info("=" * 60)
    logger.info("")

    # Create a WebView instance using Blender shortcut
    logger.info("Creating WebView instance...")
    webview = WebView.blender(title="AuroraView - Blender", width=800, height=600)
    logger.info(f"[OK] Created: {webview}")
    logger.info("")

    # Create HTML content with Blender theme
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Blender WebView</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                color: #e0e0e0;
            }
            
            .container {
                background: rgba(45, 45, 45, 0.95);
                border: 1px solid #404040;
                border-radius: 12px;
                padding: 50px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
                max-width: 700px;
                text-align: center;
            }
            
            h1 {
                color: #4a9eff;
                font-size: 2.5em;
                margin-bottom: 15px;
                font-weight: 600;
            }
            
            .subtitle {
                color: #b0b0b0;
                font-size: 1.2em;
                margin-bottom: 40px;
            }
            
            .features {
                text-align: left;
                margin: 30px 0;
                padding: 20px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 8px;
            }
            
            .features h3 {
                color: #4a9eff;
                margin-bottom: 15px;
            }
            
            .features ul {
                list-style: none;
                padding-left: 0;
            }
            
            .features li {
                padding: 8px 0;
                padding-left: 25px;
                position: relative;
            }
            
            .features li:before {
                content: "✓";
                position: absolute;
                left: 0;
                color: #4a9eff;
                font-weight: bold;
            }
            
            button {
                padding: 15px 30px;
                font-size: 1.1em;
                border: none;
                border-radius: 6px;
                cursor: pointer;
                background: linear-gradient(135deg, #4a9eff 0%, #357abd 100%);
                color: white;
                margin: 10px;
                transition: all 0.3s ease;
                box-shadow: 0 4px 15px rgba(74, 158, 255, 0.3);
            }
            
            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 6px 20px rgba(74, 158, 255, 0.4);
            }
            
            button:active {
                transform: translateY(0);
            }
            
            .info {
                margin-top: 30px;
                padding: 15px;
                background: rgba(74, 158, 255, 0.1);
                border-left: 3px solid #4a9eff;
                border-radius: 4px;
                text-align: left;
                font-size: 0.9em;
                color: #b0b0b0;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 AuroraView in Blender</h1>
            <p class="subtitle">High-Performance WebView for DCC Software</p>
            
            <div class="features">
                <h3>Features</h3>
                <ul>
                    <li>Native WebView integration</li>
                    <li>Full HTML/CSS/JavaScript support</li>
                    <li>Hardware-accelerated rendering</li>
                    <li>Cross-platform compatibility</li>
                    <li>Easy Python API</li>
                </ul>
            </div>
            
            <button onclick="showAlert()">Click Me!</button>
            <button onclick="changeColor()">Change Color</button>
            
            <div class="info">
                <strong>Note:</strong> This window is running in blocking mode.
                Blender's UI will be frozen until you close this window.
                For production use, consider using non-blocking mode with timers.
            </div>
        </div>
        
        <script>
            function showAlert() {
                alert('Hello from Blender! 🎨');
            }
            
            function changeColor() {
                const colors = [
                    'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                    'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
                    'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
                    'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
                    'linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)'
                ];
                const randomColor = colors[Math.floor(Math.random() * colors.length)];
                document.body.style.background = randomColor;
            }
        </script>
    </body>
    </html>
    """

    # Load HTML content
    logger.info("Loading HTML content...")
    webview.load_html(html_content)
    logger.info("[OK] HTML content loaded")
    logger.info("")

    # Show the window
    logger.info("Showing WebView window...")
    logger.info("⚠️  Blender UI will be frozen until you close the window")
    logger.info("Close the window to exit.")
    logger.info("")

    try:
        # Use show_blocking() to ensure the window stays open
        # This will block Blender's UI until the window is closed
        webview.show_blocking()
    except Exception as e:
        logger.error(f"Error showing WebView: {e}")
        return 1

    logger.info("")
    logger.info("=" * 60)
    logger.info("Window closed. Exiting.")
    logger.info("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
