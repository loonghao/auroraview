"""Signals Advanced Demo - Qt-inspired Signal-Slot System.

This example demonstrates AuroraView's signal-slot system, which provides
a powerful event-driven programming pattern similar to Qt's signals and slots.

Features demonstrated:
- Creating and emitting signals
- Connecting multiple handlers to a signal
- One-time connections (connect_once)
- ConnectionGuard for automatic cleanup
- SignalRegistry for dynamic signals
- Thread-safe signal operations
- Combining signals with WebView events
"""

from __future__ import annotations

import threading
import time
from typing import List

# WebView import is done in main() to avoid circular imports
from auroraview.core.signals import ConnectionGuard, Signal, SignalRegistry

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Signals Advanced Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0;
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1100px;
            margin: 0 auto;
        }
        h1 {
            text-align: center;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #f39c12, #e74c3c);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle {
            text-align: center;
            color: #7f8c8d;
            margin-bottom: 30px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 20px;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 {
            font-size: 15px;
            color: #f39c12;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .card h2::before {
            content: '';
            width: 8px;
            height: 8px;
            background: #f39c12;
            border-radius: 50%;
        }
        .description {
            font-size: 13px;
            color: #7f8c8d;
            margin-bottom: 15px;
            line-height: 1.5;
        }
        .btn-group {
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
        }
        button {
            padding: 8px 16px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13px;
            transition: all 0.2s;
            background: #f39c12;
            color: white;
        }
        button:hover {
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(243,156,18,0.3);
        }
        button.secondary {
            background: #34495e;
        }
        button.danger {
            background: #e74c3c;
        }
        button.success {
            background: #27ae60;
        }
        .log-area {
            height: 150px;
            overflow-y: auto;
            background: rgba(0,0,0,0.3);
            border-radius: 8px;
            padding: 10px;
            margin-bottom: 15px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 12px;
        }
        .log-entry {
            padding: 4px 8px;
            margin-bottom: 4px;
            border-radius: 4px;
            background: rgba(255,255,255,0.05);
        }
        .log-entry.signal { border-left: 3px solid #f39c12; }
        .log-entry.handler { border-left: 3px solid #27ae60; }
        .log-entry.once { border-left: 3px solid #9b59b6; }
        .log-entry.guard { border-left: 3px solid #3498db; }
        .log-entry .time { color: #7f8c8d; }
        .log-entry .type { 
            display: inline-block;
            padding: 1px 6px;
            border-radius: 3px;
            font-size: 10px;
            margin-right: 5px;
        }
        .type-signal { background: #f39c12; }
        .type-handler { background: #27ae60; }
        .type-once { background: #9b59b6; }
        .type-guard { background: #3498db; }
        .counter-display {
            text-align: center;
            padding: 15px;
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
            margin-bottom: 15px;
        }
        .counter-value {
            font-size: 32px;
            font-weight: bold;
            color: #f39c12;
        }
        .counter-label {
            font-size: 11px;
            color: #7f8c8d;
        }
        .handler-list {
            list-style: none;
            margin-bottom: 15px;
        }
        .handler-list li {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 12px;
            background: rgba(0,0,0,0.2);
            border-radius: 6px;
            margin-bottom: 5px;
            font-size: 13px;
        }
        .handler-list .status {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            margin-right: 8px;
        }
        .status.active { background: #27ae60; }
        .status.inactive { background: #7f8c8d; }
        .code-example {
            background: rgba(0,0,0,0.3);
            border-radius: 8px;
            padding: 15px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 11px;
            overflow-x: auto;
            white-space: pre;
            color: #bdc3c7;
        }
        .code-example .keyword { color: #e74c3c; }
        .code-example .string { color: #27ae60; }
        .code-example .comment { color: #7f8c8d; }
        .code-example .function { color: #f39c12; }
        .full-width { grid-column: 1 / -1; }
        .two-col { grid-column: span 2; }
        .registry-signals {
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
            margin-bottom: 15px;
        }
        .signal-tag {
            display: inline-flex;
            align-items: center;
            gap: 5px;
            padding: 5px 12px;
            background: rgba(243,156,18,0.2);
            border: 1px solid #f39c12;
            border-radius: 20px;
            font-size: 12px;
        }
        .signal-tag .count {
            background: #f39c12;
            color: #1a1a2e;
            padding: 1px 6px;
            border-radius: 10px;
            font-size: 10px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Signals Advanced Demo</h1>
        <p class="subtitle">Qt-inspired Signal-Slot System for event-driven programming</p>

        <div class="grid">
            <!-- Basic Signal Demo -->
            <div class="card">
                <h2>Basic Signal</h2>
                <p class="description">
                    Create a signal and connect multiple handlers. Each handler receives the emitted value.
                </p>
                <div class="counter-display">
                    <div class="counter-value" id="basic-counter">0</div>
                    <div class="counter-label">Emission Count</div>
                </div>
                <div class="btn-group">
                    <button onclick="emitBasicSignal()">Emit Signal</button>
                    <button onclick="addHandler()" class="secondary">Add Handler</button>
                    <button onclick="removeHandler()" class="danger">Remove Handler</button>
                </div>
            </div>

            <!-- Connect Once Demo -->
            <div class="card">
                <h2>Connect Once</h2>
                <p class="description">
                    One-time handlers are automatically disconnected after the first emission.
                </p>
                <div class="log-area" id="once-log">
                    <div class="log-entry once">
                        <span class="type type-once">ONCE</span>
                        Waiting for one-time handlers...
                    </div>
                </div>
                <div class="btn-group">
                    <button onclick="connectOnce()">Connect Once</button>
                    <button onclick="emitOnceSignal()" class="secondary">Emit</button>
                </div>
            </div>

            <!-- Connection Guard Demo -->
            <div class="card">
                <h2>Connection Guard</h2>
                <p class="description">
                    Guards automatically disconnect handlers when they go out of scope (RAII pattern).
                </p>
                <div class="log-area" id="guard-log">
                    <div class="log-entry guard">
                        <span class="type type-guard">GUARD</span>
                        No active guards
                    </div>
                </div>
                <div class="btn-group">
                    <button onclick="createGuard()">Create Guard</button>
                    <button onclick="destroyGuard()" class="danger">Destroy Guard</button>
                    <button onclick="emitGuardSignal()" class="secondary">Emit</button>
                </div>
            </div>

            <!-- Signal Registry Demo -->
            <div class="card two-col">
                <h2>Signal Registry</h2>
                <p class="description">
                    Dynamic signal management - create signals by name at runtime. Perfect for plugin systems.
                </p>
                <div class="registry-signals" id="registry-signals">
                    <!-- Dynamic signal tags will appear here -->
                </div>
                <div class="btn-group">
                    <button onclick="createDynamicSignal()">Create Signal</button>
                    <button onclick="connectToRegistry()" class="secondary">Connect Handler</button>
                    <button onclick="emitRegistrySignal()" class="success">Emit All</button>
                    <button onclick="clearRegistry()" class="danger">Clear Registry</button>
                </div>
            </div>

            <!-- Multi-Handler Demo -->
            <div class="card">
                <h2>Multi-Handler</h2>
                <p class="description">
                    Multiple handlers can be connected to the same signal.
                </p>
                <ul class="handler-list" id="handler-list">
                    <!-- Handler list will be populated dynamically -->
                </ul>
                <div class="btn-group">
                    <button onclick="addMultiHandler()">Add Handler</button>
                    <button onclick="emitMultiSignal()" class="success">Emit</button>
                </div>
            </div>

            <!-- Event Log -->
            <div class="card two-col">
                <h2>Event Log</h2>
                <div class="log-area" id="event-log" style="height: 200px;">
                    <div class="log-entry signal">
                        <span class="time">[--:--:--]</span>
                        <span class="type type-signal">SIGNAL</span>
                        Demo initialized. Try the signal operations!
                    </div>
                </div>
            </div>

            <!-- Code Example -->
            <div class="card full-width">
                <h2>Python Code Example</h2>
                <div class="code-example">
<span class="keyword">from</span> auroraview.core.signals <span class="keyword">import</span> Signal, ConnectionGuard, SignalRegistry

<span class="comment"># Create a signal</span>
data_changed = <span class="function">Signal</span>(name=<span class="string">"data_changed"</span>)

<span class="comment"># Connect handlers</span>
conn1 = data_changed.<span class="function">connect</span>(<span class="keyword">lambda</span> data: <span class="function">print</span>(f<span class="string">"Handler 1: {data}"</span>))
conn2 = data_changed.<span class="function">connect</span>(<span class="keyword">lambda</span> data: <span class="function">print</span>(f<span class="string">"Handler 2: {data}"</span>))

<span class="comment"># Emit signal - calls all handlers</span>
data_changed.<span class="function">emit</span>({<span class="string">"key"</span>: <span class="string">"value"</span>})

<span class="comment"># One-time handler (auto-disconnects after first emit)</span>
data_changed.<span class="function">connect_once</span>(<span class="keyword">lambda</span> data: <span class="function">print</span>(<span class="string">"Called only once!"</span>))

<span class="comment"># ConnectionGuard for automatic cleanup</span>
<span class="keyword">def</span> <span class="function">scoped_handler</span>():
    guard = <span class="function">ConnectionGuard</span>(data_changed, data_changed.<span class="function">connect</span>(my_handler))
    <span class="comment"># Handler is automatically disconnected when guard goes out of scope</span>

<span class="comment"># SignalRegistry for dynamic signals</span>
registry = <span class="function">SignalRegistry</span>()
registry.<span class="function">connect</span>(<span class="string">"custom_event"</span>, my_handler)
registry.<span class="function">emit</span>(<span class="string">"custom_event"</span>, {<span class="string">"data"</span>: 123})
                </div>
            </div>
        </div>
    </div>

    <script>
        function log(message, type = 'signal') {
            const time = new Date().toLocaleTimeString();
            const logArea = document.getElementById('event-log');
            const typeClass = 'type-' + type;
            logArea.innerHTML = `
                <div class="log-entry ${type}">
                    <span class="time">[${time}]</span>
                    <span class="type ${typeClass}">${type.toUpperCase()}</span>
                    ${message}
                </div>
            ` + logArea.innerHTML;
        }

        function logTo(areaId, message, type = 'signal') {
            const time = new Date().toLocaleTimeString();
            const logArea = document.getElementById(areaId);
            const typeClass = 'type-' + type;
            logArea.innerHTML = `
                <div class="log-entry ${type}">
                    <span class="type ${typeClass}">${type.toUpperCase()}</span>
                    ${message}
                </div>
            ` + logArea.innerHTML;
        }

        // Basic Signal
        function emitBasicSignal() {
            window.auroraview.api.emit_basic_signal();
        }
        function addHandler() {
            window.auroraview.api.add_handler();
        }
        function removeHandler() {
            window.auroraview.api.remove_handler();
        }

        // Connect Once
        function connectOnce() {
            window.auroraview.api.connect_once();
        }
        function emitOnceSignal() {
            window.auroraview.api.emit_once_signal();
        }

        // Connection Guard
        function createGuard() {
            window.auroraview.api.create_guard();
        }
        function destroyGuard() {
            window.auroraview.api.destroy_guard();
        }
        function emitGuardSignal() {
            window.auroraview.api.emit_guard_signal();
        }

        // Signal Registry
        function createDynamicSignal() {
            window.auroraview.api.create_dynamic_signal();
        }
        function connectToRegistry() {
            window.auroraview.api.connect_to_registry();
        }
        function emitRegistrySignal() {
            window.auroraview.api.emit_registry_signal();
        }
        function clearRegistry() {
            window.auroraview.api.clear_registry();
        }

        // Multi-Handler
        function addMultiHandler() {
            window.auroraview.api.add_multi_handler();
        }
        function emitMultiSignal() {
            window.auroraview.api.emit_multi_signal();
        }

        // Listen for updates from Python
        window.addEventListener('auroraviewready', () => {
            window.auroraview.on('log', (data) => {
                log(data.message, data.type || 'signal');
            });

            window.auroraview.on('log_to', (data) => {
                logTo(data.area, data.message, data.type || 'signal');
            });

            window.auroraview.on('update_counter', (data) => {
                document.getElementById(data.id).textContent = data.value;
            });

            window.auroraview.on('update_handlers', (data) => {
                const list = document.getElementById('handler-list');
                list.innerHTML = data.handlers.map((h, i) => `
                    <li>
                        <div style="display: flex; align-items: center;">
                            <span class="status active"></span>
                            Handler ${i + 1}
                        </div>
                        <span style="color: #7f8c8d; font-size: 11px;">${h}</span>
                    </li>
                `).join('');
            });

            window.auroraview.on('update_registry', (data) => {
                const container = document.getElementById('registry-signals');
                container.innerHTML = data.signals.map(s => `
                    <span class="signal-tag">
                        ${s.name}
                        <span class="count">${s.handlers}</span>
                    </span>
                `).join('');
            });
        });
    </script>
</body>
</html>
"""


class SignalsDemo:
    """Demo class showing signal-slot system capabilities."""

    def __init__(self, view):
        self.view = view

        # Basic signal
        self.basic_signal = Signal(name="basic_signal")
        self.basic_counter = 0
        self.basic_handlers: List[str] = []

        # Once signal
        self.once_signal = Signal(name="once_signal")
        self.once_counter = 0

        # Guard signal
        self.guard_signal = Signal(name="guard_signal")
        self.active_guard = None

        # Multi-handler signal
        self.multi_signal = Signal(name="multi_signal")
        self.multi_handler_ids = []

        # Signal registry
        self.registry = SignalRegistry()
        self.registry_counter = 0

    def log(self, message: str, type: str = "signal") -> None:
        """Log to main event log."""
        self.view.emit("log", {"message": message, "type": type})

    def log_to(self, area: str, message: str, type: str = "signal") -> None:
        """Log to specific area."""
        self.view.emit("log_to", {"area": area, "message": message, "type": type})

    # Basic Signal
    def emit_basic_signal(self) -> None:
        """Emit the basic signal."""
        self.basic_counter += 1
        count = self.basic_signal.emit({"count": self.basic_counter})
        self.view.emit("update_counter", {"id": "basic-counter", "value": self.basic_counter})
        self.log(f"Emitted basic_signal (called {count} handlers)", "signal")

    def add_handler(self) -> None:
        """Add a handler to the basic signal."""
        handler_id = len(self.basic_handlers) + 1

        def handler(data):
            self.log(f"Handler {handler_id} received: {data}", "handler")

        conn = self.basic_signal.connect(handler)
        self.basic_handlers.append(str(conn))
        self.log(f"Connected Handler {handler_id} to basic_signal", "handler")

    def remove_handler(self) -> None:
        """Remove the last handler from the basic signal."""
        if self.basic_handlers:
            # Disconnect all and reconnect remaining
            self.basic_signal.disconnect_all()
            self.basic_handlers.pop()
            self.log(f"Removed last handler ({len(self.basic_handlers)} remaining)", "handler")
        else:
            self.log("No handlers to remove", "handler")

    # Connect Once
    def connect_once(self) -> None:
        """Connect a one-time handler."""
        self.once_counter += 1
        handler_num = self.once_counter

        def once_handler(data):
            self.log_to("once-log", f"One-time handler {handler_num} fired!", "once")
            self.log(f"Once-handler {handler_num} called and auto-disconnected", "once")

        self.once_signal.connect_once(once_handler)
        self.log_to("once-log", f"Connected one-time handler {handler_num}", "once")
        self.log(f"Connected once-handler {handler_num}", "once")

    def emit_once_signal(self) -> None:
        """Emit the once signal."""
        count = self.once_signal.emit({"time": time.time()})
        if count > 0:
            self.log(f"Emitted once_signal (called {count} handlers)", "signal")
        else:
            self.log_to("once-log", "No handlers connected", "once")
            self.log("No once-handlers to call", "signal")

    # Connection Guard
    def create_guard(self) -> None:
        """Create a connection guard."""
        if self.active_guard:
            self.log_to("guard-log", "Guard already exists", "guard")
            return

        def guarded_handler(data):
            self.log_to("guard-log", "Guarded handler called!", "guard")
            self.log("Guarded handler received signal", "guard")

        conn = self.guard_signal.connect(guarded_handler)
        self.active_guard = ConnectionGuard(self.guard_signal, conn)
        self.log_to("guard-log", "Created ConnectionGuard (handler connected)", "guard")
        self.log("Created ConnectionGuard for guard_signal", "guard")

    def destroy_guard(self) -> None:
        """Destroy the connection guard."""
        if self.active_guard:
            self.active_guard.disconnect()
            self.active_guard = None
            self.log_to("guard-log", "Guard destroyed (handler disconnected)", "guard")
            self.log("Destroyed ConnectionGuard - handler auto-disconnected", "guard")
        else:
            self.log_to("guard-log", "No guard to destroy", "guard")

    def emit_guard_signal(self) -> None:
        """Emit the guard signal."""
        count = self.guard_signal.emit({"time": time.time()})
        if count > 0:
            self.log(f"Emitted guard_signal (called {count} handlers)", "signal")
        else:
            self.log_to("guard-log", "No handlers connected", "guard")
            self.log("No guarded handlers to call", "signal")

    # Signal Registry
    def create_dynamic_signal(self) -> None:
        """Create a dynamic signal in the registry."""
        self.registry_counter += 1
        name = f"event_{self.registry_counter}"
        self.registry.get_or_create(name)
        self.update_registry_display()
        self.log(f"Created dynamic signal: {name}", "signal")

    def connect_to_registry(self) -> None:
        """Connect a handler to all registry signals."""
        for name in self.registry.names():
            def handler(data, n=name):
                self.log(f"Registry handler for '{n}' called", "handler")
            self.registry.connect(name, handler)
        self.update_registry_display()
        self.log(f"Connected handlers to {len(self.registry.names())} signals", "handler")

    def emit_registry_signal(self) -> None:
        """Emit all signals in the registry."""
        total = 0
        for name in self.registry.names():
            count = self.registry.emit(name, {"signal": name})
            total += count
        self.log(f"Emitted to {len(self.registry.names())} signals ({total} handlers called)", "signal")

    def clear_registry(self) -> None:
        """Clear all signals from the registry."""
        names = self.registry.names()
        for name in names:
            signal = self.registry.get(name)
            if signal:
                signal.disconnect_all()
            self.registry.remove(name)
        self.registry_counter = 0
        self.update_registry_display()
        self.log(f"Cleared {len(names)} signals from registry", "signal")

    def update_registry_display(self) -> None:
        """Update the registry display in UI."""
        signals = []
        for name in self.registry.names():
            signal = self.registry.get(name)
            if signal:
                signals.append({"name": name, "handlers": signal.handler_count})
        self.view.emit("update_registry", {"signals": signals})

    # Multi-Handler
    def add_multi_handler(self) -> None:
        """Add a handler to the multi-signal."""
        handler_num = len(self.multi_handler_ids) + 1

        def handler(data):
            self.log(f"Multi-handler {handler_num} called", "handler")

        conn = self.multi_signal.connect(handler)
        self.multi_handler_ids.append(str(conn)[:8])
        self.view.emit("update_handlers", {"handlers": self.multi_handler_ids})
        self.log(f"Added multi-handler {handler_num}", "handler")

    def emit_multi_signal(self) -> None:
        """Emit the multi-signal."""
        count = self.multi_signal.emit({"time": time.time()})
        self.log(f"Emitted multi_signal (called {count} handlers)", "signal")


def main():
    """Run the signals advanced demo."""
    from auroraview import WebView

    view = WebView(
        html=HTML,
        title="Signals Advanced Demo",
        width=1150,
        height=850,
    )

    demo = SignalsDemo(view)

    # Bind all API methods
    view.bind_call("api.emit_basic_signal", demo.emit_basic_signal)
    view.bind_call("api.add_handler", demo.add_handler)
    view.bind_call("api.remove_handler", demo.remove_handler)
    view.bind_call("api.connect_once", demo.connect_once)
    view.bind_call("api.emit_once_signal", demo.emit_once_signal)
    view.bind_call("api.create_guard", demo.create_guard)
    view.bind_call("api.destroy_guard", demo.destroy_guard)
    view.bind_call("api.emit_guard_signal", demo.emit_guard_signal)
    view.bind_call("api.create_dynamic_signal", demo.create_dynamic_signal)
    view.bind_call("api.connect_to_registry", demo.connect_to_registry)
    view.bind_call("api.emit_registry_signal", demo.emit_registry_signal)
    view.bind_call("api.clear_registry", demo.clear_registry)
    view.bind_call("api.add_multi_handler", demo.add_multi_handler)
    view.bind_call("api.emit_multi_signal", demo.emit_multi_signal)

    view.show()


if __name__ == "__main__":
    main()
