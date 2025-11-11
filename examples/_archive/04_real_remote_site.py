#!/usr/bin/env python
"""
Example 04: Real Remote Site Integration

This example demonstrates how to integrate with a REAL remote website.
It shows how to:
- Load an actual remote URL
- Inject communication bridge into remote site
- Send/receive events with the remote site
- Handle cross-origin scenarios

IMPORTANT: The remote site doesn't need to be modified!
The Event Bridge is automatically injected by AuroraView.

Usage:
    python examples/04_real_remote_site.py
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


class RemoteSiteIntegration:
    """Integration with a remote website."""

    def __init__(self, remote_url: str):
        """Initialize the integration.

        Args:
            remote_url: URL of the remote site to integrate with
        """
        self.remote_url = remote_url
        self.webview = WebView(
            title=f"Remote Site: {remote_url}", width=1400, height=900, debug=True
        )

        # Register event handlers
        self._register_handlers()

    def _register_handlers(self):
        """Register event handlers for communication."""

        @self.webview.on("page_loaded")
        def handle_page_loaded(data):
            """Handle page load event."""
            logger.info(f"[RECV] Page loaded: {data}")

            # Inject custom JavaScript to enable communication
            self._inject_communication_layer()

        @self.webview.on("remote_event")
        def handle_remote_event(data):
            """Handle events from remote site."""
            logger.info(f"[RECV] Remote event: {data}")

            # Process the event
            event_type = data.get("type")

            if event_type == "click":
                logger.info(f"User clicked: {data.get('element')}")
            elif event_type == "input":
                logger.info(f"User input: {data.get('value')}")

            # Send response
            self.webview.emit(
                "python_response",
                {
                    "status": "received",
                    "original_event": event_type,
                    "timestamp": datetime.now().isoformat(),
                },
            )

        @self.webview.on("request_data")
        def handle_data_request(data):
            """Handle data request from remote site."""
            logger.info(f"[RECV] Data request: {data}")

            # Simulate fetching data
            response_data = {
                "users": [
                    {"id": 1, "name": "Alice", "role": "Admin"},
                    {"id": 2, "name": "Bob", "role": "User"},
                    {"id": 3, "name": "Charlie", "role": "User"},
                ],
                "timestamp": datetime.now().isoformat(),
            }

            self.webview.emit("data_response", response_data)
            logger.info("[SEND] Sent data response")

    def _inject_communication_layer(self):
        """Inject JavaScript to enable communication with remote site.

        This adds event listeners and helper functions to the remote page.
        """
        injection_script = """
        (function() {
            console.log('[AuroraView] Injecting communication layer...');
            
            // Create a helper object for easy communication
            window.AuroraView = {
                // Send event to Python
                send: function(eventName, data) {
                    console.log('[AuroraView] Sending event:', eventName, data);
                    window.dispatchEvent(new CustomEvent(eventName, {
                        detail: data || {}
                    }));
                },
                
                // Listen for events from Python
                on: function(eventName, callback) {
                    console.log('[AuroraView] Listening for:', eventName);
                    window.addEventListener(eventName, function(event) {
                        callback(event.detail);
                    });
                },
                
                // Request data from Python
                requestData: function(query) {
                    this.send('request_data', { query: query });
                }
            };
            
            // Notify Python that page is loaded
            window.AuroraView.send('page_loaded', {
                url: window.location.href,
                title: document.title,
                timestamp: Date.now()
            });
            
            // Add visual indicator
            const indicator = document.createElement('div');
            indicator.id = 'auroraview-indicator';
            indicator.innerHTML = '🔗 AuroraView Connected';
            indicator.style.cssText = `
                position: fixed;
                top: 10px;
                right: 10px;
                background: #667eea;
                color: white;
                padding: 8px 16px;
                border-radius: 20px;
                font-family: Arial, sans-serif;
                font-size: 12px;
                font-weight: bold;
                z-index: 999999;
                box-shadow: 0 2px 10px rgba(0,0,0,0.2);
                cursor: pointer;
            `;
            
            indicator.onclick = function() {
                window.AuroraView.send('remote_event', {
                    type: 'click',
                    element: 'indicator',
                    timestamp: Date.now()
                });
            };
            
            document.body.appendChild(indicator);
            
            // Add control panel
            const panel = document.createElement('div');
            panel.id = 'auroraview-panel';
            panel.innerHTML = `
                <div style="margin-bottom: 10px; font-weight: bold;">AuroraView Controls</div>
                <button id="av-request-data" style="margin: 5px; padding: 8px 16px; cursor: pointer;">
                    Request Data
                </button>
                <button id="av-send-event" style="margin: 5px; padding: 8px 16px; cursor: pointer;">
                    Send Event
                </button>
                <button id="av-close-panel" style="margin: 5px; padding: 8px 16px; cursor: pointer;">
                    Close Panel
                </button>
                <div id="av-output" style="margin-top: 10px; padding: 10px; background: #f0f0f0; border-radius: 4px; max-height: 200px; overflow-y: auto; font-size: 11px; font-family: monospace;"></div>
            `;
            panel.style.cssText = `
                position: fixed;
                bottom: 20px;
                right: 20px;
                background: white;
                padding: 15px;
                border-radius: 8px;
                box-shadow: 0 4px 20px rgba(0,0,0,0.3);
                z-index: 999999;
                min-width: 300px;
                font-family: Arial, sans-serif;
                font-size: 13px;
            `;
            
            document.body.appendChild(panel);
            
            // Panel controls
            document.getElementById('av-request-data').onclick = function() {
                window.AuroraView.requestData({ type: 'users' });
                addOutput('Requested data from Python');
            };
            
            document.getElementById('av-send-event').onclick = function() {
                window.AuroraView.send('remote_event', {
                    type: 'click',
                    element: 'send-button',
                    timestamp: Date.now()
                });
                addOutput('Sent event to Python');
            };
            
            document.getElementById('av-close-panel').onclick = function() {
                panel.style.display = 'none';
            };
            
            function addOutput(message) {
                const output = document.getElementById('av-output');
                const time = new Date().toLocaleTimeString();
                output.innerHTML += `[${time}] ${message}<br>`;
                output.scrollTop = output.scrollHeight;
            }
            
            // Listen for responses from Python
            window.AuroraView.on('python_response', function(data) {
                console.log('[AuroraView] Received response:', data);
                addOutput('Received: ' + JSON.stringify(data));
            });
            
            window.AuroraView.on('data_response', function(data) {
                console.log('[AuroraView] Received data:', data);
                addOutput('Received ' + data.users.length + ' users');
                
                // Display data
                const userList = data.users.map(u => u.name).join(', ');
                addOutput('Users: ' + userList);
            });
            
            console.log('[AuroraView] Communication layer ready!');
            addOutput('Communication layer initialized');
        })();
        """

        try:
            self.webview.eval_js(injection_script)
            logger.info("[OK] Communication layer injected")
        except Exception as e:
            logger.error(f"[ERROR] Failed to inject communication layer: {e}")

    def show(self):
        """Show the WebView window."""
        logger.info(f"Loading remote site: {self.remote_url}")
        self.webview.load_url(self.remote_url)

        logger.info("")
        logger.info("=" * 60)
        logger.info("INSTRUCTIONS:")
        logger.info("1. Wait for the page to load")
        logger.info("2. Look for the '🔗 AuroraView Connected' indicator")
        logger.info("3. Use the control panel to interact with Python")
        logger.info("4. Open DevTools (F12) to see console logs")
        logger.info("=" * 60)
        logger.info("")

        # show() now uses show_async() internally - works in all environments
        self.webview.show()
        logger.info("WebView window opened (non-blocking)")


def main():
    """Main function."""
    logger.info("=" * 60)
    logger.info("AuroraView - Example 04: Real Remote Site Integration")
    logger.info("=" * 60)
    logger.info("")

    # Choose a remote site to integrate with
    # You can change this to any website you want to test
    remote_sites = {
        "1": ("Example.com", "https://example.com"),
        "2": ("GitHub", "https://github.com"),
        "3": ("Wikipedia", "https://en.wikipedia.org"),
        "4": ("Custom URL", None),
    }

    print("Available remote sites:")
    for key, (name, url) in remote_sites.items():
        print(f"  {key} - {name}" + (f" ({url})" if url else ""))
    print()

    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        choice = input("Enter choice (1-4) [1]: ").strip() or "1"

    if choice == "4":
        remote_url = input("Enter custom URL: ").strip()
        if not remote_url:
            logger.error("No URL provided")
            return 1
    elif choice in remote_sites:
        name, remote_url = remote_sites[choice]
        logger.info(f"Selected: {name}")
    else:
        logger.error(f"Invalid choice: {choice}")
        return 1

    # Create integration
    integration = RemoteSiteIntegration(remote_url)

    try:
        integration.show()
    except Exception as e:
        logger.error(f"Error: {e}")
        return 1

    logger.info("")
    logger.info("=" * 60)
    logger.info("Window closed. Exiting.")
    logger.info("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
