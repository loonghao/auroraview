#!/usr/bin/env python
"""
Example 02: Bidirectional Event Communication

This example demonstrates bidirectional communication between Python and JavaScript
using the event system.

Features:
- Python → JavaScript events
- JavaScript → Python events
- Event handlers with data payloads
- Real-time updates

Usage:
    python examples/02_event_communication.py
"""

import sys
import logging
from pathlib import Path
from datetime import datetime

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import NativeWebView

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function demonstrating event communication."""
    logger.info("=" * 60)
    logger.info("AuroraView - Example 02: Event Communication")
    logger.info("=" * 60)
    logger.info("")
    
    # Create WebView instance
    logger.info("Creating WebView instance...")
    webview = NativeWebView(
        title="AuroraView - Event Communication",
        width=900,
        height=700
    )
    logger.info("✓ WebView created")
    logger.info("")
    
    # Register event handlers
    logger.info("Registering event handlers...")
    
    @webview.on("button_clicked")
    def handle_button_click(data):
        """Handle button click from JavaScript."""
        logger.info(f"📥 Button clicked: {data}")
        
        # Send response back to JavaScript
        webview.emit("python_response", {
            "message": f"Received your click on '{data.get('button')}'!",
            "timestamp": datetime.now().isoformat()
        })
    
    @webview.on("get_data")
    def handle_get_data(data):
        """Handle data request from JavaScript."""
        logger.info(f"📥 Data requested: {data}")
        
        # Send data to JavaScript
        webview.emit("data_response", {
            "items": ["Item 1", "Item 2", "Item 3"],
            "count": 3,
            "timestamp": datetime.now().isoformat()
        })
    
    @webview.on("log_message")
    def handle_log(data):
        """Handle log message from JavaScript."""
        logger.info(f"📥 JavaScript log: {data.get('message')}")
    
    logger.info("✓ Event handlers registered")
    logger.info("")
    
    # HTML content with event communication
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Event Communication</title>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                padding: 20px;
            }
            .container {
                max-width: 800px;
                margin: 0 auto;
                background: white;
                border-radius: 12px;
                padding: 30px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            }
            h1 { color: #333; margin-bottom: 10px; }
            .subtitle { color: #666; margin-bottom: 30px; font-size: 14px; }
            .section {
                margin-bottom: 30px;
                padding: 20px;
                background: #f9f9f9;
                border-radius: 8px;
            }
            .section h2 { color: #667eea; font-size: 18px; margin-bottom: 15px; }
            .button-group { display: flex; gap: 10px; flex-wrap: wrap; }
            button {
                padding: 10px 20px;
                border: none;
                border-radius: 6px;
                font-size: 14px;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            .btn-primary { background: #667eea; color: white; }
            .btn-primary:hover {
                background: #5568d3;
                transform: translateY(-2px);
                box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
            }
            .btn-success { background: #48bb78; color: white; }
            .btn-success:hover { background: #38a169; transform: translateY(-2px); }
            .btn-info { background: #4299e1; color: white; }
            .btn-info:hover { background: #3182ce; transform: translateY(-2px); }
            .log {
                background: #1e1e1e;
                color: #e0e0e0;
                padding: 15px;
                border-radius: 6px;
                font-family: 'Courier New', monospace;
                font-size: 13px;
                max-height: 200px;
                overflow-y: auto;
                line-height: 1.6;
            }
            .log-entry { margin-bottom: 5px; }
            .log-time { color: #888; }
            .log-event { color: #4fc3f7; }
            .log-data { color: #81c784; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🔄 Event Communication</h1>
            <p class="subtitle">Bidirectional Python ↔ JavaScript Communication</p>
            
            <div class="section">
                <h2>JavaScript → Python</h2>
                <div class="button-group">
                    <button class="btn-primary" onclick="sendButtonClick('Primary')">
                        Send Primary Event
                    </button>
                    <button class="btn-success" onclick="sendButtonClick('Success')">
                        Send Success Event
                    </button>
                    <button class="btn-info" onclick="requestData()">
                        Request Data
                    </button>
                </div>
            </div>
            
            <div class="section">
                <h2>Event Log</h2>
                <div class="log" id="log"></div>
            </div>
        </div>
        
        <script>
            const logEl = document.getElementById('log');
            
            function addLog(message, type = 'info') {
                const time = new Date().toLocaleTimeString();
                const entry = document.createElement('div');
                entry.className = 'log-entry';
                entry.innerHTML = `<span class="log-time">[${time}]</span> <span class="log-event">${type}:</span> <span class="log-data">${message}</span>`;
                logEl.appendChild(entry);
                logEl.scrollTop = logEl.scrollHeight;
            }
            
            function sendButtonClick(button) {
                addLog(`Sending button_clicked event: ${button}`, 'JS→PY');
                window.dispatchEvent(new CustomEvent('button_clicked', {
                    detail: { button: button, timestamp: Date.now() }
                }));
            }
            
            function requestData() {
                addLog('Requesting data from Python', 'JS→PY');
                window.dispatchEvent(new CustomEvent('get_data', {
                    detail: { request: 'data' }
                }));
            }
            
            // Listen for responses from Python
            window.addEventListener('python_response', (event) => {
                addLog(`Received: ${event.detail.message}`, 'PY→JS');
            });
            
            window.addEventListener('data_response', (event) => {
                const data = event.detail;
                addLog(`Received ${data.count} items: ${data.items.join(', ')}`, 'PY→JS');
            });
            
            // Send ready notification
            window.addEventListener('DOMContentLoaded', () => {
                addLog('WebView ready', 'SYSTEM');
                window.dispatchEvent(new CustomEvent('log_message', {
                    detail: { message: 'JavaScript initialized and ready' }
                }));
            });
        </script>
    </body>
    </html>
    """
    
    # Load HTML
    logger.info("Loading HTML content...")
    webview.load_html(html_content)
    logger.info("✓ HTML loaded")
    logger.info("")
    
    # Show window
    logger.info("Showing WebView window...")
    logger.info("Try clicking the buttons to see event communication!")
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
