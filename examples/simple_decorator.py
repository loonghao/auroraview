"""Simple Decorator Pattern Example - AuroraView API Demo.

This example demonstrates the simplest way to create a WebView tool
using the decorator pattern. Best for quick prototypes and simple tools.

Note: This example uses the low-level WebView API for demonstration.
For most use cases, prefer:
- QtWebView: For Qt-based DCC apps (Maya, Houdini, Nuke)
- AuroraView: For HWND-based apps (Unreal Engine)
- run_desktop: For standalone desktop applications

Usage:
    python examples/simple_decorator.py

Features demonstrated:
    - @view.bind_call() decorator for API methods
    - @view.on() decorator for event handlers
    - Python -> JavaScript communication via emit()
    - JavaScript -> Python communication via API calls

JavaScript side (index.html):
    // Call Python API
    const data = await auroraview.api.get_data();
    const result = await auroraview.api.save_item({name: "test", value: 42});

    // Send events to Python
    auroraview.send_event("item_clicked", {id: "btn1"});

    // Listen for Python events
    auroraview.on("data_updated", (data) => console.log(data));
"""

from __future__ import annotations

from auroraview import WebView


def main():
    """Run the simple decorator example."""
    # Create WebView with inline HTML for demo
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Decorator Pattern Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 600px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
            }
            .card {
                background: white;
                border-radius: 12px;
                padding: 24px;
                box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            }
            h1 { color: #333; margin-top: 0; }
            button {
                background: #667eea;
                color: white;
                border: none;
                padding: 12px 24px;
                border-radius: 6px;
                cursor: pointer;
                font-size: 14px;
                margin: 5px;
                transition: transform 0.1s;
            }
            button:hover { transform: translateY(-2px); }
            button:active { transform: translateY(0); }
            #output {
                background: #f5f5f5;
                border-radius: 8px;
                padding: 16px;
                margin-top: 20px;
                font-family: monospace;
                white-space: pre-wrap;
                max-height: 200px;
                overflow-y: auto;
            }
            .status { color: #666; font-size: 12px; margin-top: 10px; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>🎨 Decorator Pattern Demo</h1>
            <p>This demonstrates the simplest AuroraView API pattern.</p>

            <div>
                <button onclick="getData()">Get Data</button>
                <button onclick="saveItem()">Save Item</button>
                <button onclick="emitEvent()">Emit Event</button>
            </div>

            <div id="output">Click a button to see the result...</div>
            <div class="status" id="status">Ready</div>
        </div>

        <script>
            const output = document.getElementById('output');
            const status = document.getElementById('status');

            function log(msg) {
                output.textContent = JSON.stringify(msg, null, 2);
                status.textContent = `Updated: ${new Date().toLocaleTimeString()}`;
            }

            async function getData() {
                try {
                    const result = await auroraview.api.get_data();
                    log(result);
                } catch (e) {
                    log({error: e.message});
                }
            }

            async function saveItem() {
                try {
                    const result = await auroraview.api.save_item({
                        name: "test_item",
                        value: Math.floor(Math.random() * 100)
                    });
                    log(result);
                } catch (e) {
                    log({error: e.message});
                }
            }

            function emitEvent() {
                auroraview.send_event("item_clicked", {
                    id: "demo_button",
                    timestamp: Date.now()
                });
                log({message: "Event sent to Python!"});
            }

            // Listen for Python events
            auroraview.on("data_updated", (data) => {
                log({from_python: data});
            });

            auroraview.on("notification", (data) => {
                alert(data.message);
            });
        </script>
    </body>
    </html>
    """

    view = WebView(title="Decorator Pattern Demo", html=html_content, width=700, height=600)

    # ─────────────────────────────────────────────────────────────────
    # API Methods: Use @view.bind_call() to expose functions to JavaScript
    # These can be called via: await auroraview.api.method_name({...})
    # ─────────────────────────────────────────────────────────────────

    @view.bind_call("api.get_data")
    def get_data() -> dict:
        """Return sample data. JS: await auroraview.api.get_data()"""
        return {
            "items": ["apple", "banana", "cherry"],
            "count": 3,
            "timestamp": "2024-01-01T12:00:00Z",
        }

    @view.bind_call("api.save_item")
    def save_item(name: str = "", value: int = 0) -> dict:
        """Save an item. JS: await auroraview.api.save_item({name: "x", value: 1})"""
        print(f"[Python] Saving item: {name} = {value}")

        # Notify JavaScript about the update
        view.emit("data_updated", {"action": "saved", "name": name, "value": value})

        return {"ok": True, "message": f"Saved {name} with value {value}"}

    # ─────────────────────────────────────────────────────────────────
    # Event Handlers: Use @view.on() for fire-and-forget events from JS
    # These are called via: auroraview.send_event("event_name", {...})
    # ─────────────────────────────────────────────────────────────────

    @view.on("item_clicked")
    def handle_item_click(data: dict):
        """Handle click events from JavaScript."""
        item_id = data.get("id", "unknown")
        timestamp = data.get("timestamp", 0)
        print(f"[Python] Item clicked: {item_id} at {timestamp}")

        # Send a notification back to JavaScript
        view.emit("notification", {"message": f"Python received click on {item_id}!"})

    # ─────────────────────────────────────────────────────────────────
    # Show the WebView
    # ─────────────────────────────────────────────────────────────────
    print("Starting Decorator Pattern Demo...")
    print("API methods registered: get_data, save_item")
    print("Event handlers registered: item_clicked")
    view.show()


if __name__ == "__main__":
    main()

