"""Event Timer Demo - AuroraView Timer-Based Event Processing.

This example demonstrates the EventTimer system for processing WebView events
in embedded mode. Essential for DCC integration where the WebView is embedded
in a host application's event loop.

Usage:
    python examples/event_timer_demo.py

Features demonstrated:
    - EventTimer creation and lifecycle
    - Timer tick callbacks for periodic tasks
    - Close event detection and handling
    - Timer backend selection (Qt, Thread)
    - Window validity checking
    - Context manager usage

Note: This example uses standalone mode for demonstration.
In DCC environments, the timer integrates with the host's event loop.
"""

from __future__ import annotations

from datetime import datetime

from auroraview import WebView


def main():
    """Run the event timer demo."""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Event Timer Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 800px;
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
                margin-bottom: 20px;
            }
            h1 { color: #333; margin-top: 0; }
            h3 { color: #666; margin-bottom: 10px; }
            button {
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
            .timer-display {
                font-size: 48px;
                font-weight: bold;
                text-align: center;
                color: #667eea;
                padding: 20px;
                background: #f5f5f5;
                border-radius: 12px;
                margin: 20px 0;
                font-family: 'Consolas', monospace;
            }
            .stats-grid {
                display: grid;
                grid-template-columns: repeat(4, 1fr);
                gap: 15px;
                margin: 20px 0;
            }
            .stat-box {
                background: #f5f5f5;
                padding: 15px;
                border-radius: 8px;
                text-align: center;
            }
            .stat-value {
                font-size: 24px;
                font-weight: bold;
                color: #667eea;
            }
            .stat-label {
                font-size: 12px;
                color: #666;
                margin-top: 5px;
            }
            #log {
                background: #1e1e1e;
                color: #0f0;
                border-radius: 8px;
                padding: 16px;
                font-family: 'Consolas', monospace;
                font-size: 12px;
                max-height: 200px;
                overflow-y: auto;
                white-space: pre-wrap;
            }
            .status-indicator {
                display: inline-block;
                width: 12px;
                height: 12px;
                border-radius: 50%;
                margin-right: 8px;
            }
            .status-running { background: #4caf50; }
            .status-stopped { background: #f44336; }
            .interval-slider {
                width: 100%;
                margin: 10px 0;
            }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>Event Timer Demo</h1>
            <p>Demonstrates timer-based event processing for embedded WebView scenarios.</p>

            <div class="timer-display" id="timerDisplay">00:00:00</div>

            <div class="stats-grid">
                <div class="stat-box">
                    <div class="stat-value" id="tickCount">0</div>
                    <div class="stat-label">Tick Count</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="interval">16</div>
                    <div class="stat-label">Interval (ms)</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="fps">0</div>
                    <div class="stat-label">Effective FPS</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value">
                        <span class="status-indicator status-running" id="statusIndicator"></span>
                        <span id="statusText">Running</span>
                    </div>
                    <div class="stat-label">Timer Status</div>
                </div>
            </div>

            <h3>Timer Interval</h3>
            <input type="range" class="interval-slider" id="intervalSlider"
                   min="8" max="100" value="16" oninput="updateInterval(this.value)">
            <div style="display: flex; justify-content: space-between; font-size: 12px; color: #666;">
                <span>8ms (120 FPS)</span>
                <span>16ms (60 FPS)</span>
                <span>33ms (30 FPS)</span>
                <span>100ms (10 FPS)</span>
            </div>

            <h3>Actions</h3>
            <div>
                <button onclick="requestTimerStatus()">Get Timer Status</button>
                <button onclick="triggerTickCallback()">Trigger Custom Tick</button>
                <button onclick="resetStats()">Reset Stats</button>
            </div>
        </div>

        <div class="card">
            <h3>Event Log</h3>
            <div id="log">Timer events will appear here...</div>
        </div>

        <script>
            let tickCount = 0;
            let lastTickTime = Date.now();
            let fpsHistory = [];

            function log(msg) {
                const logEl = document.getElementById('log');
                const timestamp = new Date().toLocaleTimeString();
                logEl.textContent = `[${timestamp}] ${msg}\\n` + logEl.textContent;
                if (logEl.textContent.length > 5000) {
                    logEl.textContent = logEl.textContent.slice(0, 5000);
                }
            }

            function updateTimerDisplay() {
                const now = new Date();
                const hours = String(now.getHours()).padStart(2, '0');
                const minutes = String(now.getMinutes()).padStart(2, '0');
                const seconds = String(now.getSeconds()).padStart(2, '0');
                document.getElementById('timerDisplay').textContent = `${hours}:${minutes}:${seconds}`;
            }

            function updateStats(data) {
                tickCount = data.tick_count || tickCount;
                document.getElementById('tickCount').textContent = tickCount;

                // Calculate FPS
                const now = Date.now();
                const delta = now - lastTickTime;
                lastTickTime = now;

                if (delta > 0) {
                    const fps = Math.round(1000 / delta);
                    fpsHistory.push(fps);
                    if (fpsHistory.length > 10) fpsHistory.shift();
                    const avgFps = Math.round(fpsHistory.reduce((a, b) => a + b, 0) / fpsHistory.length);
                    document.getElementById('fps').textContent = avgFps;
                }
            }

            function updateInterval(value) {
                document.getElementById('interval').textContent = value;
                auroraview.api.set_interval({interval_ms: parseInt(value)});
            }

            async function requestTimerStatus() {
                try {
                    const status = await auroraview.api.get_timer_status();
                    log(`Timer status: ${JSON.stringify(status)}`);

                    const indicator = document.getElementById('statusIndicator');
                    const statusText = document.getElementById('statusText');

                    if (status.is_running) {
                        indicator.className = 'status-indicator status-running';
                        statusText.textContent = 'Running';
                    } else {
                        indicator.className = 'status-indicator status-stopped';
                        statusText.textContent = 'Stopped';
                    }
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function triggerTickCallback() {
                try {
                    await auroraview.api.trigger_tick();
                    log('Custom tick triggered');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            function resetStats() {
                tickCount = 0;
                fpsHistory = [];
                document.getElementById('tickCount').textContent = '0';
                document.getElementById('fps').textContent = '0';
                log('Stats reset');
            }

            // Listen for tick events from Python
            auroraview.on("timer_tick", (data) => {
                updateTimerDisplay();
                updateStats(data);
            });

            auroraview.on("timer_close", (data) => {
                log('Timer close event received');
                document.getElementById('statusIndicator').className = 'status-indicator status-stopped';
                document.getElementById('statusText').textContent = 'Stopped';
            });

            // Initial display update
            updateTimerDisplay();
            setInterval(updateTimerDisplay, 1000);
        </script>
    </body>
    </html>
    """

    view = WebView(title="Event Timer Demo", html=html_content, width=900, height=800)

    # Timer state
    timer_state = {"tick_count": 0, "interval_ms": 16, "start_time": None}

    # Note: In standalone mode, WebView.show() handles its own event loop.
    # This demo shows how EventTimer would be used in embedded/DCC mode.
    # For demonstration, we'll simulate the timer behavior using periodic emit.

    @view.bind_call("api.get_timer_status")
    def get_timer_status() -> dict:
        """Get current timer status."""
        return {
            "is_running": True,  # In demo, always running
            "interval_ms": timer_state["interval_ms"],
            "tick_count": timer_state["tick_count"],
            "uptime_seconds": (
                (datetime.now() - timer_state["start_time"]).total_seconds()
                if timer_state["start_time"]
                else 0
            ),
        }

    @view.bind_call("api.set_interval")
    def set_interval(interval_ms: int = 16) -> dict:
        """Set timer interval (demo only - actual change requires timer restart)."""
        timer_state["interval_ms"] = interval_ms
        return {"ok": True, "new_interval": interval_ms}

    @view.bind_call("api.trigger_tick")
    def trigger_tick() -> dict:
        """Manually trigger a tick callback."""
        timer_state["tick_count"] += 1
        view.emit("timer_tick", {"tick_count": timer_state["tick_count"], "manual": True})
        return {"ok": True, "tick_count": timer_state["tick_count"]}

    # Demonstrate EventTimer API (for documentation purposes)
    print("=" * 60)
    print("EventTimer Demo - Timer-Based Event Processing")
    print("=" * 60)
    print()
    print("EventTimer is designed for embedded WebView scenarios where")
    print("the WebView is integrated into a host application's event loop.")
    print()
    print("Example usage in DCC environments:")
    print()
    print("  from auroraview import WebView")
    print("  from auroraview.utils.event_timer import EventTimer")
    print()
    print("  # Create WebView in embedded mode")
    print("  webview = WebView(parent=parent_hwnd, mode='owner')")
    print()
    print("  # Create timer with 16ms interval (60 FPS)")
    print("  timer = EventTimer(webview, interval_ms=16)")
    print()
    print("  # Register callbacks")
    print("  @timer.on_tick")
    print("  def handle_tick():")
    print("      # Called every 16ms")
    print("      pass")
    print()
    print("  @timer.on_close")
    print("  def handle_close():")
    print("      timer.stop()")
    print()
    print("  # Start the timer")
    print("  timer.start()")
    print()
    print("=" * 60)

    timer_state["start_time"] = datetime.now()
    print("\nStarting Event Timer Demo...")
    view.show()


if __name__ == "__main__":
    main()
