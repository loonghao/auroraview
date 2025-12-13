"""Multi-Window Demo - Multiple WebView windows with communication.

This example demonstrates how to create and manage multiple WebView windows
in AuroraView, including inter-window communication patterns.

Features demonstrated:
- Creating multiple independent windows
- Parent-child window relationships
- Inter-window messaging via Python
- Window lifecycle management
- Synchronized state across windows
"""

from __future__ import annotations

import json
import threading
from typing import Dict, List, Optional

from auroraview import WebView

# Shared state manager for inter-window communication
class WindowManager:
    """Manages multiple windows and their communication."""

    def __init__(self):
        self.windows: Dict[str, WebView] = {}
        self.shared_state: Dict[str, any] = {
            "theme": "dark",
            "messages": [],
            "counter": 0,
        }
        self._lock = threading.Lock()

    def register(self, window_id: str, window: WebView) -> None:
        """Register a window with the manager."""
        with self._lock:
            self.windows[window_id] = window

    def unregister(self, window_id: str) -> None:
        """Unregister a window."""
        with self._lock:
            self.windows.pop(window_id, None)

    def broadcast(self, event: str, data: dict, exclude: Optional[str] = None) -> None:
        """Broadcast an event to all windows."""
        with self._lock:
            for window_id, window in self.windows.items():
                if window_id != exclude:
                    try:
                        window.emit(event, data)
                    except Exception:
                        pass

    def send_to(self, window_id: str, event: str, data: dict) -> None:
        """Send an event to a specific window."""
        with self._lock:
            window = self.windows.get(window_id)
            if window:
                try:
                    window.emit(event, data)
                except Exception:
                    pass

    def get_window_ids(self) -> List[str]:
        """Get list of all window IDs."""
        with self._lock:
            return list(self.windows.keys())


# Global window manager
manager = WindowManager()


MAIN_WINDOW_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Main Window - Multi-Window Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1e3a5f 0%, #0d1b2a 100%);
            color: #e0e0e0;
            min-height: 100vh;
            padding: 20px;
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        h1 {
            font-size: 28px;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #4facfe, #00f2fe);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle {
            color: #7f8c8d;
            font-size: 14px;
        }
        .window-id {
            display: inline-block;
            background: #4facfe;
            color: white;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            margin-top: 10px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
            max-width: 900px;
            margin: 0 auto;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 {
            font-size: 16px;
            color: #4facfe;
            margin-bottom: 15px;
        }
        .btn-group {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }
        button {
            padding: 10px 20px;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s;
            background: #4facfe;
            color: white;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(79,172,254,0.4);
        }
        button.secondary {
            background: #34495e;
        }
        button.danger {
            background: #e74c3c;
        }
        .window-list {
            list-style: none;
        }
        .window-list li {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px;
            background: rgba(0,0,0,0.2);
            border-radius: 6px;
            margin-bottom: 8px;
        }
        .window-list .status {
            width: 8px;
            height: 8px;
            background: #2ecc71;
            border-radius: 50%;
            margin-right: 10px;
        }
        .message-area {
            height: 200px;
            overflow-y: auto;
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
            padding: 15px;
            margin-bottom: 15px;
        }
        .message {
            padding: 8px 12px;
            background: rgba(79,172,254,0.2);
            border-radius: 6px;
            margin-bottom: 8px;
            border-left: 3px solid #4facfe;
        }
        .message .from {
            font-size: 11px;
            color: #7f8c8d;
            margin-bottom: 4px;
        }
        .message-input {
            display: flex;
            gap: 10px;
        }
        .message-input input {
            flex: 1;
            padding: 10px;
            border: 1px solid rgba(255,255,255,0.2);
            border-radius: 6px;
            background: rgba(0,0,0,0.2);
            color: white;
            font-size: 14px;
        }
        .message-input input:focus {
            outline: none;
            border-color: #4facfe;
        }
        .counter-display {
            text-align: center;
            padding: 30px;
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
        }
        .counter-value {
            font-size: 48px;
            font-weight: bold;
            color: #4facfe;
        }
        .counter-label {
            color: #7f8c8d;
            font-size: 12px;
            margin-top: 5px;
        }
        .full-width {
            grid-column: 1 / -1;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Multi-Window Demo</h1>
        <p class="subtitle">Create and manage multiple WebView windows</p>
        <span class="window-id" id="window-id">Main Window</span>
    </div>

    <div class="grid">
        <!-- Window Management -->
        <div class="card">
            <h2>Window Management</h2>
            <div class="btn-group">
                <button onclick="createChildWindow()">New Child Window</button>
                <button onclick="createFloatingWindow()" class="secondary">Floating Panel</button>
            </div>
            <h3 style="margin-top: 20px; margin-bottom: 10px; font-size: 14px; color: #7f8c8d;">Active Windows</h3>
            <ul class="window-list" id="window-list">
                <li>
                    <div style="display: flex; align-items: center;">
                        <span class="status"></span>
                        <span>Main Window</span>
                    </div>
                    <span style="color: #7f8c8d; font-size: 12px;">This window</span>
                </li>
            </ul>
        </div>

        <!-- Shared Counter -->
        <div class="card">
            <h2>Shared Counter</h2>
            <div class="counter-display">
                <div class="counter-value" id="counter-value">0</div>
                <div class="counter-label">Synchronized across all windows</div>
            </div>
            <div class="btn-group" style="margin-top: 15px; justify-content: center;">
                <button onclick="incrementCounter()">+1</button>
                <button onclick="decrementCounter()" class="secondary">-1</button>
                <button onclick="resetCounter()" class="danger">Reset</button>
            </div>
        </div>

        <!-- Broadcast Messaging -->
        <div class="card full-width">
            <h2>Broadcast Messaging</h2>
            <div class="message-area" id="message-area">
                <div class="message">
                    <div class="from">System</div>
                    <div>Welcome to Multi-Window Demo! Open child windows and send messages.</div>
                </div>
            </div>
            <div class="message-input">
                <input type="text" id="message-input" placeholder="Type a message to broadcast...">
                <button onclick="broadcastMessage()">Broadcast</button>
            </div>
        </div>
    </div>

    <script>
        // Listen for events from Python
        window.addEventListener('auroraviewready', () => {
            // Counter updates
            window.auroraview.on('counter:update', (data) => {
                document.getElementById('counter-value').textContent = data.value;
            });

            // Message broadcasts
            window.auroraview.on('message:received', (data) => {
                addMessage(data.from, data.text);
            });

            // Window list updates
            window.auroraview.on('windows:update', (data) => {
                updateWindowList(data.windows);
            });
        });

        function addMessage(from, text) {
            const area = document.getElementById('message-area');
            const msg = document.createElement('div');
            msg.className = 'message';
            msg.innerHTML = `<div class="from">${from}</div><div>${text}</div>`;
            area.appendChild(msg);
            area.scrollTop = area.scrollHeight;
        }

        function updateWindowList(windows) {
            const list = document.getElementById('window-list');
            list.innerHTML = windows.map(w => `
                <li>
                    <div style="display: flex; align-items: center;">
                        <span class="status"></span>
                        <span>${w}</span>
                    </div>
                    ${w === 'main' ? '<span style="color: #7f8c8d; font-size: 12px;">This window</span>' : ''}
                </li>
            `).join('');
        }

        function createChildWindow() {
            window.auroraview.api.create_child_window();
        }

        function createFloatingWindow() {
            window.auroraview.api.create_floating_window();
        }

        function incrementCounter() {
            window.auroraview.api.update_counter({ delta: 1 });
        }

        function decrementCounter() {
            window.auroraview.api.update_counter({ delta: -1 });
        }

        function resetCounter() {
            window.auroraview.api.reset_counter();
        }

        function broadcastMessage() {
            const input = document.getElementById('message-input');
            const text = input.value.trim();
            if (text) {
                window.auroraview.api.broadcast_message({ text: text });
                input.value = '';
            }
        }

        // Enter key to send message
        document.getElementById('message-input').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') broadcastMessage();
        });
    </script>
</body>
</html>
"""


CHILD_WINDOW_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Child Window</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #2d3436 0%, #000000 100%);
            color: #e0e0e0;
            min-height: 100vh;
            padding: 20px;
        }
        .header {
            text-align: center;
            margin-bottom: 20px;
        }
        h1 {
            font-size: 20px;
            color: #00cec9;
        }
        .window-id {
            display: inline-block;
            background: #00cec9;
            color: #2d3436;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            margin-top: 10px;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 15px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 {
            font-size: 14px;
            color: #00cec9;
            margin-bottom: 15px;
        }
        .counter-display {
            text-align: center;
            padding: 20px;
        }
        .counter-value {
            font-size: 36px;
            font-weight: bold;
            color: #00cec9;
        }
        .btn-group {
            display: flex;
            gap: 10px;
            justify-content: center;
        }
        button {
            padding: 8px 16px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13px;
            background: #00cec9;
            color: #2d3436;
        }
        button:hover {
            opacity: 0.9;
        }
        button.secondary {
            background: #636e72;
            color: white;
        }
        .message-area {
            height: 150px;
            overflow-y: auto;
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
            padding: 10px;
            margin-bottom: 10px;
        }
        .message {
            padding: 6px 10px;
            background: rgba(0,206,201,0.2);
            border-radius: 4px;
            margin-bottom: 6px;
            font-size: 13px;
        }
        .message .from {
            font-size: 10px;
            color: #636e72;
        }
        .message-input {
            display: flex;
            gap: 8px;
        }
        .message-input input {
            flex: 1;
            padding: 8px;
            border: 1px solid rgba(255,255,255,0.2);
            border-radius: 4px;
            background: rgba(0,0,0,0.2);
            color: white;
            font-size: 13px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Child Window</h1>
        <span class="window-id" id="window-id">Loading...</span>
    </div>

    <div class="card">
        <h2>Shared Counter</h2>
        <div class="counter-display">
            <div class="counter-value" id="counter-value">0</div>
        </div>
        <div class="btn-group">
            <button onclick="increment()">+1</button>
            <button onclick="decrement()" class="secondary">-1</button>
        </div>
    </div>

    <div class="card">
        <h2>Messages</h2>
        <div class="message-area" id="message-area"></div>
        <div class="message-input">
            <input type="text" id="message-input" placeholder="Send message...">
            <button onclick="sendMessage()">Send</button>
        </div>
    </div>

    <script>
        let windowId = 'child';

        window.addEventListener('auroraviewready', () => {
            // Get window ID
            window.auroraview.api.get_window_id().then(data => {
                windowId = data.id;
                document.getElementById('window-id').textContent = windowId;
            });

            window.auroraview.on('counter:update', (data) => {
                document.getElementById('counter-value').textContent = data.value;
            });

            window.auroraview.on('message:received', (data) => {
                const area = document.getElementById('message-area');
                area.innerHTML += `<div class="message"><div class="from">${data.from}</div>${data.text}</div>`;
                area.scrollTop = area.scrollHeight;
            });
        });

        function increment() {
            window.auroraview.api.update_counter({ delta: 1 });
        }

        function decrement() {
            window.auroraview.api.update_counter({ delta: -1 });
        }

        function sendMessage() {
            const input = document.getElementById('message-input');
            const text = input.value.trim();
            if (text) {
                window.auroraview.api.broadcast_message({ text: text });
                input.value = '';
            }
        }

        document.getElementById('message-input').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendMessage();
        });
    </script>
</body>
</html>
"""


def create_main_window() -> WebView:
    """Create the main window."""
    view = WebView.create(
        title="Multi-Window Demo - Main",
        html=MAIN_WINDOW_HTML,
        width=950,
        height=700,
    )

    window_id = "main"
    manager.register(window_id, view)
    child_counter = [0]  # Mutable counter for child windows

    @view.bind_call("api.create_child_window")
    def create_child():
        child_counter[0] += 1
        child_id = f"child_{child_counter[0]}"
        create_child_window(child_id)
        broadcast_window_list()

    @view.bind_call("api.create_floating_window")
    def create_floating():
        child_counter[0] += 1
        child_id = f"float_{child_counter[0]}"
        create_child_window(child_id, floating=True)
        broadcast_window_list()

    @view.bind_call("api.update_counter")
    def update_counter(delta: int):
        manager.shared_state["counter"] += delta
        manager.broadcast("counter:update", {"value": manager.shared_state["counter"]})

    @view.bind_call("api.reset_counter")
    def reset_counter():
        manager.shared_state["counter"] = 0
        manager.broadcast("counter:update", {"value": 0})

    @view.bind_call("api.broadcast_message")
    def broadcast_message(text: str):
        manager.shared_state["messages"].append({"from": window_id, "text": text})
        manager.broadcast("message:received", {"from": window_id, "text": text})

    @view.on("closing")
    def on_closing(data):
        manager.unregister(window_id)

    return view


def create_child_window(window_id: str, floating: bool = False) -> WebView:
    """Create a child window."""
    view = WebView.create(
        title=f"Child Window - {window_id}",
        html=CHILD_WINDOW_HTML,
        width=400,
        height=500,
        always_on_top=floating,
    )

    manager.register(window_id, view)

    @view.bind_call("api.get_window_id")
    def get_window_id():
        return {"id": window_id}

    @view.bind_call("api.update_counter")
    def update_counter(delta: int):
        manager.shared_state["counter"] += delta
        manager.broadcast("counter:update", {"value": manager.shared_state["counter"]})

    @view.bind_call("api.broadcast_message")
    def broadcast_message(text: str):
        manager.shared_state["messages"].append({"from": window_id, "text": text})
        manager.broadcast("message:received", {"from": window_id, "text": text})

    @view.on("closing")
    def on_closing(data):
        manager.unregister(window_id)
        broadcast_window_list()

    # Sync initial state
    view.emit("counter:update", {"value": manager.shared_state["counter"]})

    return view


def broadcast_window_list():
    """Broadcast the current window list to all windows."""
    windows = manager.get_window_ids()
    manager.broadcast("windows:update", {"windows": windows})


def main():
    """Run the multi-window demo."""
    main_window = create_main_window()
    broadcast_window_list()
    main_window.run()


if __name__ == "__main__":
    main()
