#!/usr/bin/env python
"""
Example 03: Remote Site Communication

This example demonstrates bidirectional communication with a remote website.
It shows how to:
- Load a remote URL
- Send events from Python to the remote site
- Receive events from the remote site
- Handle real-time data exchange

Usage:
    python examples/03_remote_site_communication.py
"""

import logging
import sys
from datetime import datetime
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def main():
    """Main function demonstrating remote site communication."""
    logger.info("=" * 60)
    logger.info("AuroraView - Example 03: Remote Site Communication")
    logger.info("=" * 60)
    logger.info("")

    # Create WebView instance
    logger.info("Creating WebView instance...")
    webview = WebView(
        title="AuroraView - Remote Site Communication",
        width=1200,
        height=900,
        debug=True,  # Enable DevTools for debugging
    )
    logger.info("[OK] WebView created")
    logger.info("")

    # Register event handlers for communication with remote site
    logger.info("Registering event handlers...")

    @webview.on("page_ready")
    def handle_page_ready(data):
        """Handle page ready event from remote site."""
        logger.info(f"[RECV] Page ready: {data}")

        # Send initial data to remote site
        webview.emit(
            "init_data",
            {"app_name": "AuroraView", "version": "0.2.3", "timestamp": datetime.now().isoformat()},
        )

    @webview.on("user_action")
    def handle_user_action(data):
        """Handle user action from remote site."""
        logger.info(f"[RECV] User action: {data}")

        action_type = data.get("type")

        if action_type == "get_data":
            # Simulate fetching data from DCC or database
            response_data = {
                "items": [
                    {"id": 1, "name": "Item 1", "value": 100},
                    {"id": 2, "name": "Item 2", "value": 200},
                    {"id": 3, "name": "Item 3", "value": 300},
                ],
                "total": 3,
                "timestamp": datetime.now().isoformat(),
            }

            webview.emit("data_response", response_data)
            logger.info(f"[SEND] Sent data response with {len(response_data['items'])} items")

        elif action_type == "process":
            # Simulate processing
            logger.info(f"Processing: {data.get('payload')}")

            webview.emit(
                "process_complete",
                {
                    "status": "success",
                    "message": f"Processed {data.get('payload')}",
                    "timestamp": datetime.now().isoformat(),
                },
            )

    @webview.on("log_message")
    def handle_log(data):
        """Handle log message from remote site."""
        logger.info(f"[RECV] Remote site log: {data.get('message')}")

    logger.info("[OK] Event handlers registered")
    logger.info("")

    # Create a local HTML page that simulates a remote site
    # In production, you would use: webview.load_url("https://your-remote-site.com")
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Remote Site Simulation</title>
        <meta charset="UTF-8">
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                padding: 20px;
            }
            .container {
                max-width: 1000px;
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
            .button-group { display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 15px; }
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
            .btn-warning { background: #ed8936; color: white; }
            .btn-warning:hover { background: #dd6b20; transform: translateY(-2px); }
            .log {
                background: #1e1e1e;
                color: #e0e0e0;
                padding: 15px;
                border-radius: 6px;
                font-family: 'Courier New', monospace;
                font-size: 13px;
                max-height: 300px;
                overflow-y: auto;
                line-height: 1.6;
            }
            .log-entry { margin-bottom: 5px; }
            .log-time { color: #888; }
            .log-event { color: #4fc3f7; font-weight: bold; }
            .log-data { color: #81c784; }
            .data-display {
                background: white;
                padding: 15px;
                border-radius: 6px;
                border: 1px solid #ddd;
                margin-top: 15px;
            }
            .data-item {
                padding: 10px;
                margin: 5px 0;
                background: #f5f5f5;
                border-radius: 4px;
                display: flex;
                justify-content: space-between;
            }
            .badge {
                display: inline-block;
                padding: 4px 8px;
                border-radius: 4px;
                font-size: 12px;
                font-weight: bold;
            }
            .badge-success { background: #48bb78; color: white; }
            .badge-info { background: #4299e1; color: white; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🌐 Remote Site Communication Demo</h1>
            <p class="subtitle">Simulating a remote website communicating with Python backend</p>
            
            <div class="section">
                <h2>📤 Send Events to Python</h2>
                <div class="button-group">
                    <button class="btn-primary" onclick="requestData()">
                        Request Data
                    </button>
                    <button class="btn-success" onclick="processData()">
                        Process Data
                    </button>
                    <button class="btn-info" onclick="sendCustomEvent()">
                        Send Custom Event
                    </button>
                    <button class="btn-warning" onclick="clearLog()">
                        Clear Log
                    </button>
                </div>
                
                <div id="dataDisplay" class="data-display" style="display: none;">
                    <h3>Received Data:</h3>
                    <div id="dataContent"></div>
                </div>
            </div>
            
            <div class="section">
                <h2>📋 Event Log</h2>
                <div class="log" id="log"></div>
            </div>
        </div>
        
        <script>
            const logEl = document.getElementById('log');
            const dataDisplayEl = document.getElementById('dataDisplay');
            const dataContentEl = document.getElementById('dataContent');
            
            function addLog(message, type = 'INFO') {
                const time = new Date().toLocaleTimeString();
                const entry = document.createElement('div');
                entry.className = 'log-entry';
                entry.innerHTML = `<span class="log-time">[${time}]</span> <span class="log-event">${type}:</span> <span class="log-data">${message}</span>`;
                logEl.appendChild(entry);
                logEl.scrollTop = logEl.scrollHeight;
            }
            
            function clearLog() {
                logEl.innerHTML = '';
                addLog('Log cleared', 'SYSTEM');
            }
            
            function requestData() {
                addLog('Requesting data from Python...', 'JS→PY');
                window.dispatchEvent(new CustomEvent('user_action', {
                    detail: {
                        type: 'get_data',
                        timestamp: Date.now()
                    }
                }));
            }
            
            function processData() {
                const payload = {
                    operation: 'transform',
                    data: [1, 2, 3, 4, 5]
                };
                addLog(`Processing data: ${JSON.stringify(payload)}`, 'JS→PY');
                window.dispatchEvent(new CustomEvent('user_action', {
                    detail: {
                        type: 'process',
                        payload: payload,
                        timestamp: Date.now()
                    }
                }));
            }
            
            function sendCustomEvent() {
                addLog('Sending custom event...', 'JS→PY');
                window.dispatchEvent(new CustomEvent('log_message', {
                    detail: {
                        message: 'Custom event from remote site',
                        level: 'info'
                    }
                }));
            }
            
            // Listen for events from Python
            window.addEventListener('init_data', (event) => {
                addLog(`Received init data: ${JSON.stringify(event.detail)}`, 'PY→JS');
            });
            
            window.addEventListener('data_response', (event) => {
                const data = event.detail;
                addLog(`Received ${data.total} items from Python`, 'PY→JS');
                
                // Display data
                dataDisplayEl.style.display = 'block';
                dataContentEl.innerHTML = data.items.map(item => `
                    <div class="data-item">
                        <span><strong>${item.name}</strong> (ID: ${item.id})</span>
                        <span class="badge badge-info">Value: ${item.value}</span>
                    </div>
                `).join('');
            });
            
            window.addEventListener('process_complete', (event) => {
                addLog(`Process complete: ${event.detail.message}`, 'PY→JS');
                addLog(`Status: ${event.detail.status}`, 'PY→JS');
            });
            
            // Page initialization
            window.addEventListener('DOMContentLoaded', () => {
                addLog('Remote site loaded and ready', 'SYSTEM');
                
                // Notify Python that page is ready
                window.dispatchEvent(new CustomEvent('page_ready', {
                    detail: {
                        url: window.location.href,
                        userAgent: navigator.userAgent,
                        timestamp: Date.now()
                    }
                }));
            });
            
            // Log all custom events for debugging
            const originalDispatchEvent = EventTarget.prototype.dispatchEvent;
            EventTarget.prototype.dispatchEvent = function(event) {
                if (event instanceof CustomEvent && event.type !== 'log_message') {
                    console.log('CustomEvent dispatched:', event.type, event.detail);
                }
                return originalDispatchEvent.call(this, event);
            };
        </script>
    </body>
    </html>
    """

    # Load HTML (simulating remote site)
    logger.info("Loading HTML content (simulating remote site)...")
    webview.load_html(html_content)
    logger.info("[OK] HTML loaded")
    logger.info("")

    # In production, you would load a real remote URL:
    # webview.load_url("https://your-remote-site.com")

    # Show window
    logger.info("Showing WebView window...")
    logger.info("")
    logger.info("=" * 60)
    logger.info("INSTRUCTIONS:")
    logger.info("1. Click 'Request Data' to fetch data from Python")
    logger.info("2. Click 'Process Data' to send data for processing")
    logger.info("3. Click 'Send Custom Event' to send a custom event")
    logger.info("4. Watch the Event Log for bidirectional communication")
    logger.info("5. Open DevTools (F12) to see JavaScript console")
    logger.info("=" * 60)
    logger.info("")

    try:
        # show() now uses show_async() internally - works in all environments
        webview.show()
        logger.info("WebView window opened (non-blocking)")
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
