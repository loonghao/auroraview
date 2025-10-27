#!/usr/bin/env python
"""
Simple WebView Window Example

This example demonstrates how to create a simple WebView window with HTML content.
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


def main():
    """Main function to run the simple window example."""
    logger.info("=" * 60)
    logger.info("AuroraView - Simple Window Example")
    logger.info("=" * 60)
    logger.info("")
    
    # Create a WebView instance
    logger.info("Creating WebView instance...")
    webview = WebView(
        title="AuroraView - Simple Example",
        width=800,
        height=600
    )
    logger.info(f"✓ Created: {webview}")
    logger.info("")
    
    # Create HTML content
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>AuroraView Example</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
                margin: 0;
                padding: 20px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            
            .container {
                background: white;
                border-radius: 10px;
                padding: 40px;
                box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
                max-width: 600px;
                text-align: center;
            }
            
            h1 {
                color: #333;
                margin: 0 0 10px 0;
                font-size: 2.5em;
            }
            
            .subtitle {
                color: #666;
                font-size: 1.1em;
                margin-bottom: 30px;
            }
            
            .features {
                text-align: left;
                margin: 30px 0;
            }
            
            .feature {
                display: flex;
                align-items: center;
                margin: 15px 0;
                font-size: 1.1em;
            }
            
            .feature-icon {
                width: 30px;
                height: 30px;
                background: #667eea;
                color: white;
                border-radius: 50%;
                display: flex;
                align-items: center;
                justify-content: center;
                margin-right: 15px;
                font-weight: bold;
            }
            
            .buttons {
                margin-top: 30px;
                display: flex;
                gap: 10px;
                justify-content: center;
            }
            
            button {
                padding: 12px 24px;
                font-size: 1em;
                border: none;
                border-radius: 5px;
                cursor: pointer;
                transition: all 0.3s ease;
                font-weight: 600;
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
            
            .btn-secondary {
                background: #f0f0f0;
                color: #333;
            }
            
            .btn-secondary:hover {
                background: #e0e0e0;
                transform: translateY(-2px);
            }
            
            .status {
                margin-top: 20px;
                padding: 15px;
                background: #f5f5f5;
                border-radius: 5px;
                color: #666;
                font-size: 0.95em;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🚀 AuroraView</h1>
            <p class="subtitle">High-Performance WebView for DCC Software</p>
            
            <div class="features">
                <div class="feature">
                    <div class="feature-icon">⚡</div>
                    <div>2.5x faster than PyWebView</div>
                </div>
                <div class="feature">
                    <div class="feature-icon">💾</div>
                    <div>2x less memory usage</div>
                </div>
                <div class="feature">
                    <div class="feature-icon">🎯</div>
                    <div>Native DCC integration</div>
                </div>
                <div class="feature">
                    <div class="feature-icon">🔒</div>
                    <div>Type-safe with Rust</div>
                </div>
            </div>
            
            <div class="buttons">
                <button class="btn-primary" onclick="handleClick()">Click Me!</button>
                <button class="btn-secondary" onclick="handleInfo()">Info</button>
            </div>
            
            <div class="status" id="status">
                Ready to interact with Python!
            </div>
        </div>
        
        <script>
            function handleClick() {
                const status = document.getElementById('status');
                status.textContent = '✓ Button clicked! This is running in WebView.';
                status.style.background = '#e8f5e9';
                status.style.color = '#2e7d32';
            }
            
            function handleInfo() {
                const status = document.getElementById('status');
                status.textContent = 'AuroraView combines Rust performance with Python ease of use.';
                status.style.background = '#e3f2fd';
                status.style.color = '#1565c0';
            }
            
            // Listen for events from Python
            window.addEventListener('python_event', (event) => {
                const status = document.getElementById('status');
                status.textContent = '📨 Event from Python: ' + JSON.stringify(event.detail);
                status.style.background = '#fff3e0';
                status.style.color = '#e65100';
            });
        </script>
    </body>
    </html>
    """
    
    # Load HTML content
    logger.info("Loading HTML content...")
    webview.load_html(html_content)
    logger.info("✓ HTML content loaded")
    logger.info("")
    
    # Register event handler
    logger.info("Registering event handler...")
    @webview.on("test_event")
    def handle_test_event(data):
        logger.info(f"✓ Received event from JavaScript: {data}")
    
    logger.info("✓ Event handler registered")
    logger.info("")
    
    # Show the window
    logger.info("Showing WebView window...")
    logger.info("Close the window to exit.")
    logger.info("")
    
    try:
        webview.show()
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

