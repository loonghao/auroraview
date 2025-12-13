#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Window Events Demo - Demonstrates window lifecycle event handling.

This example shows how to use the window event system to track window
lifecycle events like shown, hidden, focused, blurred, resized, moved, etc.

Works in standalone mode or embedded in DCC applications (Maya, Houdini, Blender).

Note: This example uses the low-level WebView API for demonstration.
For most use cases, prefer QtWebView, AuroraView, or run_desktop.
"""

from auroraview import WebView
from auroraview.core.events import WindowEventData


def create_demo_html() -> str:
    """Create demo HTML with event display."""
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Window Events Demo</title>
        <style>
            body { font-family: Arial, sans-serif; padding: 20px; background: #1a1a2e; color: #eee; }
            h1 { color: #00d4ff; }
            .event-log { background: #16213e; padding: 15px; border-radius: 8px; max-height: 400px; overflow-y: auto; }
            .event-item { padding: 8px; margin: 4px 0; border-radius: 4px; font-family: monospace; }
            .event-shown { background: #0f3460; border-left: 4px solid #00ff88; }
            .event-hidden { background: #0f3460; border-left: 4px solid #ff6b6b; }
            .event-focused { background: #0f3460; border-left: 4px solid #ffd93d; }
            .event-blurred { background: #0f3460; border-left: 4px solid #6c5ce7; }
            .event-resized { background: #0f3460; border-left: 4px solid #00d4ff; }
            .event-moved { background: #0f3460; border-left: 4px solid #ff9f43; }
            .event-closing { background: #0f3460; border-left: 4px solid #ff4757; }
            .controls { margin: 20px 0; }
            button { padding: 10px 20px; margin: 5px; border: none; border-radius: 5px; cursor: pointer; }
            .btn-primary { background: #00d4ff; color: #1a1a2e; }
            .btn-secondary { background: #6c5ce7; color: white; }
        </style>
    </head>
    <body>
        <h1>🪟 Window Events Demo</h1>
        <p>This demo shows window lifecycle events in real-time.</p>

        <div class="controls">
            <button class="btn-primary" onclick="clearLog()">Clear Log</button>
            <button class="btn-secondary" onclick="testResize()">Test Resize</button>
            <button class="btn-secondary" onclick="testMove()">Test Move</button>
        </div>

        <div class="event-log" id="eventLog">
            <div class="event-item event-shown">Waiting for events...</div>
        </div>

        <script>
            function addEvent(type, data) {
                const log = document.getElementById('eventLog');
                const item = document.createElement('div');
                item.className = 'event-item event-' + type;
                const time = new Date().toLocaleTimeString();
                item.textContent = `[${time}] ${type.toUpperCase()}: ${JSON.stringify(data)}`;
                log.insertBefore(item, log.firstChild);
            }

            function clearLog() {
                document.getElementById('eventLog').innerHTML = '';
            }

            function testResize() {
                window.auroraview.call('resize', {width: 900, height: 700});
            }

            function testMove() {
                window.auroraview.call('move', {x: 100, y: 100});
            }

            // Register event listeners
            window.auroraview.on('shown', (data) => addEvent('shown', data));
            window.auroraview.on('hidden', (data) => addEvent('hidden', data));
            window.auroraview.on('focused', (data) => addEvent('focused', data));
            window.auroraview.on('blurred', (data) => addEvent('blurred', data));
            window.auroraview.on('resized', (data) => addEvent('resized', data));
            window.auroraview.on('moved', (data) => addEvent('moved', data));
            window.auroraview.on('closing', (data) => addEvent('closing', data));
            window.auroraview.on('closed', (data) => addEvent('closed', data));
        </script>
    </body>
    </html>
    """


def main():
    """Run the window events demo."""
    # Create WebView
    webview = WebView(
        title="Window Events Demo",
        width=800,
        height=600,
        resizable=True,
    )

    # Register Python-side event handlers
    @webview.on_shown
    def on_shown(data: WindowEventData):
        print(f"[Python] Window shown: {data}")

    @webview.on_focused
    def on_focused(data: WindowEventData):
        print(f"[Python] Window focused: {data}")

    @webview.on_blurred
    def on_blurred(data: WindowEventData):
        print(f"[Python] Window blurred: {data}")

    @webview.on_resized
    def on_resized(data: WindowEventData):
        print(f"[Python] Window resized: {data.width}x{data.height}")

    @webview.on_moved
    def on_moved(data: WindowEventData):
        print(f"[Python] Window moved to: ({data.x}, {data.y})")

    @webview.on_closing
    def on_closing(data: WindowEventData):
        print("[Python] Window is closing...")
        return True  # Allow close

    # Register RPC handlers for window control
    @webview.on("resize")
    def handle_resize(data):
        webview.resize(data.get("width", 800), data.get("height", 600))
        return {"success": True}

    @webview.on("move")
    def handle_move(data):
        webview.move(data.get("x", 0), data.get("y", 0))
        return {"success": True}

    # Load HTML and show
    webview.load_html(create_demo_html())
    webview.show()


if __name__ == "__main__":
    main()

