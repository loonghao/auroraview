#!/usr/bin/env python
"""Window Manager Demo - Multi-window management with AuroraView.

This example demonstrates the WindowManager API for managing multiple WebView windows.

Features:
- Global window registry
- Active window tracking
- Window change notifications
- Event broadcasting across windows
- Ready events for lifecycle management

Usage:
    python examples/window_manager_demo.py
"""

from __future__ import annotations

import logging
import sys
import threading
import time
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root / "python"))

from auroraview import (
    WebView,
    broadcast_event,
    get_active_window,
    get_window_manager,
    get_windows,
)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


# HTML template for demo windows
WINDOW_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Window {window_id}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, {bg_start} 0%, {bg_end} 100%);
            min-height: 100vh;
            color: white;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
        }}
        h1 {{
            margin-bottom: 20px;
        }}
        .info {{
            background: rgba(255,255,255,0.1);
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 15px;
        }}
        .events {{
            background: rgba(0,0,0,0.2);
            padding: 15px;
            border-radius: 8px;
            max-height: 200px;
            overflow-y: auto;
        }}
        .event-item {{
            padding: 5px 0;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }}
        button {{
            background: rgba(255,255,255,0.2);
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            color: white;
            cursor: pointer;
            margin-right: 10px;
            margin-bottom: 10px;
        }}
        button:hover {{
            background: rgba(255,255,255,0.3);
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Window {window_id}</h1>
        <div class="info">
            <p><strong>Window ID:</strong> <span id="windowId">{window_id}</span></p>
            <p><strong>Status:</strong> <span id="status">Initializing...</span></p>
        </div>
        <div>
            <button onclick="broadcastMessage()">Broadcast Message</button>
            <button onclick="checkWindows()">Check Windows</button>
        </div>
        <h3>Received Events</h3>
        <div class="events" id="events">
            <div class="event-item">Waiting for events...</div>
        </div>
    </div>
    <script>
        const eventsDiv = document.getElementById('events');
        const statusSpan = document.getElementById('status');
        let eventCount = 0;

        function addEvent(event, data) {{
            eventCount++;
            const item = document.createElement('div');
            item.className = 'event-item';
            item.textContent = `[${{eventCount}}] ${{event}}: ${{JSON.stringify(data)}}`;
            eventsDiv.insertBefore(item, eventsDiv.firstChild);
            if (eventsDiv.children.length > 20) {{
                eventsDiv.removeChild(eventsDiv.lastChild);
            }}
        }}

        window.addEventListener('auroraviewready', () => {{
            statusSpan.textContent = 'Ready';
            addEvent('auroraviewready', {{}});

            // Subscribe to broadcast events
            auroraview.on('broadcast', (data) => {{
                addEvent('broadcast', data);
            }});

            auroraview.on('window_count', (data) => {{
                addEvent('window_count', data);
            }});
        }});

        function broadcastMessage() {{
            auroraview.call('broadcast_to_all', {{
                message: 'Hello from Window {window_id}!',
                timestamp: Date.now()
            }});
        }}

        function checkWindows() {{
            auroraview.call('get_window_count');
        }}
    </script>
</body>
</html>
"""

COLORS = [
    ("#667eea", "#764ba2"),  # Purple
    ("#f093fb", "#f5576c"),  # Pink
    ("#4facfe", "#00f2fe"),  # Blue
    ("#43e97b", "#38f9d7"),  # Green
    ("#fa709a", "#fee140"),  # Orange
]


class DemoWindow:
    """Demo window with broadcast capabilities."""

    def __init__(self, window_id: int) -> None:
        self.window_id = window_id
        colors = COLORS[window_id % len(COLORS)]
        html = WINDOW_HTML.format(
            window_id=window_id,
            bg_start=colors[0],
            bg_end=colors[1],
        )
        self.webview = WebView(
            title=f"Window {window_id}",
            html=html,
            width=500,
            height=400,
        )

        # Bind API methods
        self.webview.bind_call("broadcast_to_all", self.broadcast_to_all)
        self.webview.bind_call("get_window_count", self.get_window_count)

    def broadcast_to_all(self, message: str, timestamp: int) -> None:
        """Broadcast message to all windows."""
        broadcast_event(
            "broadcast",
            {"from": self.window_id, "message": message, "timestamp": timestamp},
        )

    def get_window_count(self) -> None:
        """Get and emit window count."""
        windows = get_windows()
        self.webview.emit("window_count", {"count": len(windows)})

    def show(self) -> None:
        """Show window non-blocking."""
        self.webview.show(wait=False)


def on_window_change(event: str, window_id: str) -> None:
    """Handle window change events."""
    logger.info(f"Window change: {event} - {window_id}")


def main() -> None:
    """Run the window manager demo."""
    logger.info("Starting Window Manager Demo")

    # Get window manager and register callback
    wm = get_window_manager()
    wm.on_change(on_window_change)

    # Create multiple windows
    windows = []
    for i in range(3):
        window = DemoWindow(i + 1)
        windows.append(window)
        logger.info(f"Created window {i + 1}")

    # Show all windows
    for window in windows:
        window.show()
        time.sleep(0.3)  # Stagger window creation

    # Log window manager state
    logger.info(f"Total windows: {len(wm.get_all())}")
    active = get_active_window()
    if active:
        logger.info(f"Active window: {active.window_id}")

    # Keep main thread alive
    logger.info("Press Ctrl+C to exit")
    try:
        while True:
            time.sleep(1)
            # Check if any windows are still open
            if len(wm.get_all()) == 0:
                logger.info("All windows closed, exiting")
                break
    except KeyboardInterrupt:
        logger.info("Interrupted, closing windows")
        for window in windows:
            try:
                window.webview.close()
            except Exception:
                pass

    logger.info("Window Manager Demo finished")


if __name__ == "__main__":
    main()
