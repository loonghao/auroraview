#!/usr/bin/env python
"""
Run a Simple WebView Window

This script creates and displays a simple WebView window.
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
    """Main function."""
    logger.info("=" * 70)
    logger.info("AuroraView - Simple Window")
    logger.info("=" * 70)
    logger.info("")
    
    try:
        # Create WebView
        logger.info("Creating WebView...")
        webview = WebView(
            title="AuroraView - Simple Example",
            width=800,
            height=600
        )
        logger.info(f"✓ Created: {webview}")
        logger.info("")
        
        # Create HTML content
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>AuroraView</title>
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
                    padding: 20px;
                }
                
                .container {
                    background: white;
                    border-radius: 15px;
                    padding: 50px;
                    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                    max-width: 600px;
                    text-align: center;
                    animation: slideIn 0.5s ease-out;
                }
                
                @keyframes slideIn {
                    from {
                        opacity: 0;
                        transform: translateY(20px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }
                
                h1 {
                    color: #333;
                    font-size: 2.5em;
                    margin-bottom: 10px;
                }
                
                .subtitle {
                    color: #666;
                    font-size: 1.1em;
                    margin-bottom: 30px;
                }
                
                .features {
                    text-align: left;
                    margin: 30px 0;
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 15px;
                }
                
                .feature {
                    background: #f5f5f5;
                    padding: 15px;
                    border-radius: 8px;
                    border-left: 4px solid #667eea;
                }
                
                .feature-icon {
                    font-size: 1.5em;
                    margin-bottom: 5px;
                }
                
                .feature-text {
                    font-size: 0.9em;
                    color: #666;
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
                    border-radius: 8px;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    font-weight: 600;
                }
                
                .btn-primary {
                    background: #667eea;
                    color: white;
                    flex: 1;
                }
                
                .btn-primary:hover {
                    background: #5568d3;
                    transform: translateY(-2px);
                    box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
                }
                
                .btn-secondary {
                    background: #f0f0f0;
                    color: #333;
                    flex: 1;
                }
                
                .btn-secondary:hover {
                    background: #e0e0e0;
                    transform: translateY(-2px);
                }
                
                .status {
                    margin-top: 20px;
                    padding: 15px;
                    background: #f5f5f5;
                    border-radius: 8px;
                    color: #666;
                    font-size: 0.95em;
                    min-height: 40px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                
                .status.success {
                    background: #e8f5e9;
                    color: #2e7d32;
                    border: 1px solid #c8e6c9;
                }
                
                .status.info {
                    background: #e3f2fd;
                    color: #1565c0;
                    border: 1px solid #bbdefb;
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
                        <div class="feature-text">2.5x Faster</div>
                    </div>
                    <div class="feature">
                        <div class="feature-icon">💾</div>
                        <div class="feature-text">2x Less Memory</div>
                    </div>
                    <div class="feature">
                        <div class="feature-icon">🎯</div>
                        <div class="feature-text">DCC Native</div>
                    </div>
                    <div class="feature">
                        <div class="feature-icon">🔒</div>
                        <div class="feature-text">Type Safe</div>
                    </div>
                </div>
                
                <div class="buttons">
                    <button class="btn-primary" onclick="handleClick()">Click Me!</button>
                    <button class="btn-secondary" onclick="handleInfo()">Info</button>
                </div>
                
                <div class="status" id="status">
                    Ready to interact!
                </div>
            </div>
            
            <script>
                function handleClick() {
                    const status = document.getElementById('status');
                    status.textContent = '✓ Button clicked! This is running in WebView.';
                    status.classList.add('success');
                    status.classList.remove('info');
                }
                
                function handleInfo() {
                    const status = document.getElementById('status');
                    status.textContent = 'AuroraView combines Rust performance with Python ease of use.';
                    status.classList.add('info');
                    status.classList.remove('success');
                }
                
                // Listen for events from Python
                window.addEventListener('python_event', (event) => {
                    const status = document.getElementById('status');
                    status.textContent = '📨 Event from Python: ' + JSON.stringify(event.detail);
                    status.classList.remove('success', 'info');
                });
                
                // Log that page loaded
                console.log('AuroraView page loaded successfully!');
            </script>
        </body>
        </html>
        """
        
        # Load HTML
        logger.info("Loading HTML content...")
        webview.load_html(html)
        logger.info("✓ HTML loaded")
        logger.info("")
        
        # Register event handler
        logger.info("Registering event handler...")
        @webview.on("test_event")
        def handle_test_event(data):
            logger.info(f"✓ Event received: {data}")
        logger.info("✓ Event handler registered")
        logger.info("")
        
        # Show window
        logger.info("=" * 70)
        logger.info("Showing WebView window...")
        logger.info("Close the window to exit.")
        logger.info("=" * 70)
        logger.info("")
        
        webview.show()
        
        logger.info("")
        logger.info("=" * 70)
        logger.info("Window closed. Exiting.")
        logger.info("=" * 70)
        
        return 0
        
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())

