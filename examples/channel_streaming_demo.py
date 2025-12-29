"""Channel Streaming Demo - AuroraView Streaming Data Transfer.

This example demonstrates the Channel system for streaming data between
Python and JavaScript. Useful for large file transfers, real-time data,
or progress updates.

Usage:
    python examples/channel_streaming_demo.py

Features demonstrated:
    - Channel creation and management
    - Streaming data chunks to JavaScript
    - Progress reporting
    - Channel lifecycle (open, message, close)
    - ChannelManager for multiple channels

JavaScript side:
    // Receive streaming data
    auroraview.on("__channel_message__", (data) => {
        console.log("Received chunk:", data.channel_id, data.data);
    });

    auroraview.on("__channel_close__", (data) => {
        console.log("Channel closed:", data.channel_id);
    });
"""

from __future__ import annotations

import time
from typing import List

from auroraview import WebView
from auroraview.core.channel import Channel, ChannelManager


def main():
    """Run the channel streaming demo."""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Channel Streaming Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 800px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
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
                background: #11998e;
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
            button:disabled {
                background: #ccc;
                cursor: not-allowed;
                transform: none;
            }
            .progress-container {
                background: #e0e0e0;
                border-radius: 10px;
                height: 20px;
                margin: 15px 0;
                overflow: hidden;
            }
            .progress-bar {
                background: linear-gradient(90deg, #11998e, #38ef7d);
                height: 100%;
                width: 0%;
                transition: width 0.3s ease;
                border-radius: 10px;
            }
            .progress-text {
                text-align: center;
                font-size: 14px;
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
                max-height: 300px;
                overflow-y: auto;
                white-space: pre-wrap;
            }
            .log-entry { margin: 2px 0; }
            .log-open { color: #4fc3f7; }
            .log-message { color: #81c784; }
            .log-close { color: #ffb74d; }
            .stats {
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                gap: 15px;
                margin-top: 15px;
            }
            .stat-box {
                background: #f5f5f5;
                padding: 15px;
                border-radius: 8px;
                text-align: center;
            }
            .stat-value { font-size: 24px; font-weight: bold; color: #11998e; }
            .stat-label { font-size: 12px; color: #666; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>Channel Streaming Demo</h1>
            <p>Demonstrates streaming data transfer between Python and JavaScript.</p>

            <div>
                <button onclick="startFileStream()" id="btnFile">Stream File Data</button>
                <button onclick="startProgressStream()" id="btnProgress">Progress Updates</button>
                <button onclick="startMultiChannel()" id="btnMulti">Multi-Channel</button>
            </div>

            <div class="progress-container">
                <div class="progress-bar" id="progressBar"></div>
            </div>
            <div class="progress-text" id="progressText">Ready to stream...</div>

            <div class="stats">
                <div class="stat-box">
                    <div class="stat-value" id="chunksReceived">0</div>
                    <div class="stat-label">Chunks Received</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="bytesReceived">0</div>
                    <div class="stat-label">Bytes Received</div>
                </div>
                <div class="stat-box">
                    <div class="stat-value" id="activeChannels">0</div>
                    <div class="stat-label">Active Channels</div>
                </div>
            </div>
        </div>

        <div class="card">
            <h3>Channel Log</h3>
            <div id="log"></div>
        </div>

        <script>
            let chunksReceived = 0;
            let bytesReceived = 0;
            let activeChannels = new Set();

            function log(msg, type = 'message') {
                const logEl = document.getElementById('log');
                const entry = document.createElement('div');
                entry.className = `log-entry log-${type}`;
                entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
                logEl.appendChild(entry);
                logEl.scrollTop = logEl.scrollHeight;
            }

            function updateStats() {
                document.getElementById('chunksReceived').textContent = chunksReceived;
                document.getElementById('bytesReceived').textContent =
                    bytesReceived > 1024 ? `${(bytesReceived/1024).toFixed(1)}KB` : bytesReceived;
                document.getElementById('activeChannels').textContent = activeChannels.size;
            }

            function updateProgress(percent, text) {
                document.getElementById('progressBar').style.width = `${percent}%`;
                document.getElementById('progressText').textContent = text;
            }

            // Listen for channel events
            auroraview.on("__channel_open__", (data) => {
                log(`Channel opened: ${data.channel_id}`, 'open');
                activeChannels.add(data.channel_id);
                updateStats();
            });

            auroraview.on("__channel_message__", (data) => {
                chunksReceived++;
                const payload = data.data;

                if (typeof payload === 'object') {
                    if (payload.type === 'progress') {
                        updateProgress(payload.percent, payload.message);
                    } else if (payload.type === 'chunk') {
                        bytesReceived += payload.size || 0;
                        log(`Chunk ${payload.index}: ${payload.size} bytes`, 'message');
                    } else if (payload.type === 'data') {
                        log(`Data: ${JSON.stringify(payload.content).slice(0, 50)}...`, 'message');
                    }
                } else {
                    bytesReceived += String(payload).length;
                    log(`Data: ${String(payload).slice(0, 50)}...`, 'message');
                }
                updateStats();
            });

            auroraview.on("__channel_close__", (data) => {
                log(`Channel closed: ${data.channel_id}`, 'close');
                activeChannels.delete(data.channel_id);
                updateStats();
            });

            async function startFileStream() {
                chunksReceived = 0;
                bytesReceived = 0;
                updateProgress(0, 'Starting file stream...');
                document.getElementById('btnFile').disabled = true;

                try {
                    await auroraview.api.stream_file_data();
                } catch (e) {
                    log(`Error: ${e.message}`, 'close');
                }

                document.getElementById('btnFile').disabled = false;
            }

            async function startProgressStream() {
                updateProgress(0, 'Starting progress updates...');
                document.getElementById('btnProgress').disabled = true;

                try {
                    await auroraview.api.stream_progress();
                } catch (e) {
                    log(`Error: ${e.message}`, 'close');
                }

                document.getElementById('btnProgress').disabled = false;
            }

            async function startMultiChannel() {
                chunksReceived = 0;
                document.getElementById('btnMulti').disabled = true;

                try {
                    await auroraview.api.multi_channel_demo();
                } catch (e) {
                    log(`Error: ${e.message}`, 'close');
                }

                document.getElementById('btnMulti').disabled = false;
            }
        </script>
    </body>
    </html>
    """

    view = WebView(title="Channel Streaming Demo", html=html_content, width=900, height=800)

    # Create a channel manager for this webview
    channel_manager = ChannelManager()

    @view.bind_call("api.stream_file_data")
    def stream_file_data() -> dict:
        """Simulate streaming file data in chunks."""
        # Create a channel for this stream
        channel = channel_manager.create()
        channel._attach_webview(view)

        # Simulate file data (in real use, read from actual file)
        total_chunks = 10
        chunk_size = 1024

        for i in range(total_chunks):
            # Simulate chunk data
            chunk_data = {
                "type": "chunk",
                "index": i + 1,
                "total": total_chunks,
                "size": chunk_size,
                "data": f"chunk_{i}_" + "x" * 100,  # Simulated data
            }
            channel.send(chunk_data)

            # Also send progress
            progress = {
                "type": "progress",
                "percent": int((i + 1) / total_chunks * 100),
                "message": f"Streaming chunk {i + 1}/{total_chunks}...",
            }
            channel.send(progress)

            # Small delay to simulate network/disk I/O
            time.sleep(0.2)

        # Close the channel when done
        channel.close()

        return {"ok": True, "chunks_sent": total_chunks}

    @view.bind_call("api.stream_progress")
    def stream_progress() -> dict:
        """Stream progress updates for a long-running task."""
        channel = channel_manager.create("progress_channel")
        channel._attach_webview(view)

        steps = [
            "Initializing...",
            "Loading data...",
            "Processing...",
            "Validating...",
            "Finalizing...",
        ]

        for i, step in enumerate(steps):
            progress = {
                "type": "progress",
                "percent": int((i + 1) / len(steps) * 100),
                "message": step,
            }
            channel.send(progress)
            time.sleep(0.5)

        channel.close()
        return {"ok": True, "steps_completed": len(steps)}

    @view.bind_call("api.multi_channel_demo")
    def multi_channel_demo() -> dict:
        """Demonstrate multiple concurrent channels."""
        channels: List[Channel] = []

        # Create 3 channels
        for i in range(3):
            ch = channel_manager.create(f"multi_ch_{i}")
            ch._attach_webview(view)
            channels.append(ch)

        # Send data to each channel
        for round_num in range(5):
            for i, ch in enumerate(channels):
                ch.send(
                    {
                        "type": "data",
                        "channel": i,
                        "round": round_num,
                        "content": f"Data from channel {i}, round {round_num}",
                    }
                )
            time.sleep(0.3)

        # Close all channels
        for ch in channels:
            ch.close()

        return {"ok": True, "channels_used": len(channels)}

    # Show channel manager status
    @view.bind_call("api.get_channel_status")
    def get_channel_status() -> dict:
        """Get current channel manager status."""
        return {"active_channels": channel_manager.active_count, "manager": repr(channel_manager)}

    print("Starting Channel Streaming Demo...")
    print("Features: File streaming, Progress updates, Multi-channel")
    view.show()


if __name__ == "__main__":
    main()
