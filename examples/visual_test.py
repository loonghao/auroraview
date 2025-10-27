#!/usr/bin/env python
"""
Visual Test - Simple WebView Display Test

This example creates a simple WebView window with interactive content.
Close the window to exit.
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
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function to run the visual test."""
    logger.info("=" * 70)
    logger.info("AuroraView - Visual Test")
    logger.info("=" * 70)
    logger.info("")
    
    # Create HTML content with interactive elements
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>AuroraView - Visual Test</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }
            
            .container {
                background: white;
                border-radius: 15px;
                padding: 50px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                max-width: 700px;
                width: 100%;
            }
            
            h1 {
                color: #333;
                margin-bottom: 10px;
                font-size: 2.5em;
                text-align: center;
            }
            
            .subtitle {
                color: #666;
                text-align: center;
                margin-bottom: 40px;
                font-size: 1.1em;
            }
            
            .section {
                margin: 30px 0;
                padding: 20px;
                background: #f8f9fa;
                border-radius: 10px;
                border-left: 4px solid #667eea;
            }
            
            .section h2 {
                color: #667eea;
                margin-bottom: 15px;
                font-size: 1.3em;
            }
            
            .section p {
                color: #555;
                line-height: 1.6;
                margin-bottom: 10px;
            }
            
            .buttons {
                display: flex;
                gap: 10px;
                margin-top: 20px;
                flex-wrap: wrap;
            }
            
            button {
                padding: 12px 24px;
                font-size: 1em;
                border: none;
                border-radius: 8px;
                cursor: pointer;
                transition: all 0.3s ease;
                font-weight: 600;
                flex: 1;
                min-width: 150px;
            }
            
            .btn-primary {
                background: #667eea;
                color: white;
            }
            
            .btn-primary:hover {
                background: #5568d3;
                transform: translateY(-2px);
                box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
            }
            
            .btn-primary:active {
                transform: translateY(0);
            }
            
            .btn-secondary {
                background: #f0f0f0;
                color: #333;
                border: 2px solid #ddd;
            }
            
            .btn-secondary:hover {
                background: #e8e8e8;
                border-color: #667eea;
                transform: translateY(-2px);
            }
            
            .status {
                margin-top: 20px;
                padding: 15px;
                background: #e8f5e9;
                border-radius: 8px;
                color: #2e7d32;
                border-left: 4px solid #4caf50;
                min-height: 40px;
                display: flex;
                align-items: center;
            }
            
            .status.info {
                background: #e3f2fd;
                color: #1565c0;
                border-left-color: #2196f3;
            }
            
            .status.warning {
                background: #fff3e0;
                color: #e65100;
                border-left-color: #ff9800;
            }
            
            .counter {
                font-size: 2em;
                text-align: center;
                color: #667eea;
                margin: 20px 0;
                font-weight: bold;
            }
            
            .feature-list {
                list-style: none;
                margin: 15px 0;
            }
            
            .feature-list li {
                padding: 8px 0;
                color: #555;
            }
            
            .feature-list li:before {
                content: "✓ ";
                color: #4caf50;
                font-weight: bold;
                margin-right: 10px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 AuroraView</h1>
            <p class="subtitle">Visual Test - Interactive Demo</p>
            
            <div class="section">
                <h2>Welcome!</h2>
                <p>This is a visual test of AuroraView. Click the buttons below to interact with the interface.</p>
                <ul class="feature-list">
                    <li>High-performance WebView for DCC software</li>
                    <li>2.5x faster than PyWebView</li>
                    <li>2x less memory usage</li>
                    <li>Native DCC integration support</li>
                </ul>
            </div>
            
            <div class="section">
                <h2>Interactive Demo</h2>
                <div class="counter" id="counter">0</div>
                <div class="buttons">
                    <button class="btn-primary" onclick="increment()">Increment</button>
                    <button class="btn-secondary" onclick="decrement()">Decrement</button>
                    <button class="btn-secondary" onclick="reset()">Reset</button>
                </div>
            </div>
            
            <div class="section">
                <h2>Status</h2>
                <div class="buttons">
                    <button class="btn-primary" onclick="showSuccess()">Success</button>
                    <button class="btn-secondary" onclick="showInfo()">Info</button>
                    <button class="btn-secondary" onclick="showWarning()">Warning</button>
                </div>
                <div class="status" id="status">
                    Ready to interact!
                </div>
            </div>
        </div>
        
        <script>
            let count = 0;
            
            function increment() {
                count++;
                updateCounter();
                showSuccess();
            }
            
            function decrement() {
                count--;
                updateCounter();
                showInfo();
            }
            
            function reset() {
                count = 0;
                updateCounter();
                showWarning();
            }
            
            function updateCounter() {
                document.getElementById('counter').textContent = count;
            }
            
            function showSuccess() {
                const status = document.getElementById('status');
                status.textContent = '✓ Success! Counter: ' + count;
                status.className = 'status';
            }
            
            function showInfo() {
                const status = document.getElementById('status');
                status.textContent = 'ℹ Info: Counter is now ' + count;
                status.className = 'status info';
            }
            
            function showWarning() {
                const status = document.getElementById('status');
                status.textContent = '⚠ Warning: Counter has been reset to 0';
                status.className = 'status warning';
            }
            
            // Log that the page loaded
            console.log('AuroraView Visual Test loaded successfully!');
        </script>
    </body>
    </html>
    """
    
    try:
        logger.info("Creating WebView instance...")
        webview = WebView(
            title="AuroraView - Visual Test",
            width=900,
            height=700,
            dev_tools=True
        )
        logger.info(f"✓ Created: {webview}")
        logger.info("")
        
        logger.info("Loading HTML content...")
        webview.load_html(html_content)
        logger.info("✓ HTML content loaded")
        logger.info("")
        
        logger.info("=" * 70)
        logger.info("Showing WebView window...")
        logger.info("Close the window to exit.")
        logger.info("=" * 70)
        logger.info("")
        
        # Show the window (this will block until the window is closed)
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

