"""Child-Aware Demo - Example that works both standalone and as Gallery child.

This example demonstrates the unified child window system:
- Runs standalone when executed directly
- Runs as child window when launched from Gallery
- Communicates with parent via IPC when in child mode

Usage:
    # Standalone mode
    python examples/child_aware_demo.py

    # Child mode (launched from Gallery)
    # Gallery sets environment variables automatically

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import sys

# Import child support utilities
from auroraview import ChildContext, is_child_mode, run_example

# HTML template with mode indicator
HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Child-Aware Demo</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, {bg_start} 0%, {bg_end} 100%);
            color: #e4e4e4;
            min-height: 100vh;
            padding: 20px;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        h1 {{
            font-size: 24px;
            color: {accent};
            margin-bottom: 10px;
        }}
        .mode-badge {{
            display: inline-block;
            background: {accent};
            color: #1a1a2e;
            padding: 6px 16px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 600;
        }}
        .card {{
            background: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 20px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }}
        .card h2 {{
            color: {accent};
            font-size: 16px;
            margin-bottom: 15px;
        }}
        .info-row {{
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }}
        .info-row:last-child {{ border-bottom: none; }}
        .info-label {{ color: #888; }}
        .info-value {{ color: #fff; font-family: monospace; }}
        .btn-group {{
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }}
        button {{
            background: {accent};
            color: #1a1a2e;
            border: none;
            padding: 10px 20px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: all 0.2s;
        }}
        button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        }}
        button.secondary {{
            background: rgba(255, 255, 255, 0.1);
            color: #e4e4e4;
        }}
        .message-area {{
            background: rgba(0, 0, 0, 0.2);
            border-radius: 8px;
            padding: 15px;
            max-height: 200px;
            overflow-y: auto;
            font-family: monospace;
            font-size: 13px;
        }}
        .message {{
            padding: 6px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }}
        .message:last-child {{ border-bottom: none; }}
        .message .time {{ color: #666; margin-right: 10px; }}
        .message .text {{ color: #e4e4e4; }}
        .input-group {{
            display: flex;
            gap: 10px;
            margin-top: 15px;
        }}
        input {{
            flex: 1;
            background: rgba(0, 0, 0, 0.2);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 8px;
            padding: 10px 15px;
            color: #fff;
            font-size: 14px;
        }}
        input:focus {{
            outline: none;
            border-color: {accent};
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Child-Aware Demo</h1>
            <span class="mode-badge">{mode_text}</span>
        </div>

        <div class="card">
            <h2>Context Information</h2>
            <div class="info-row">
                <span class="info-label">Mode</span>
                <span class="info-value">{mode}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Parent ID</span>
                <span class="info-value">{parent_id}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Child ID</span>
                <span class="info-value">{child_id}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Example Name</span>
                <span class="info-value">{example_name}</span>
            </div>
        </div>

        <div class="card" id="parent-comm" style="display: {show_parent_comm}">
            <h2>Parent Communication</h2>
            <div class="btn-group">
                <button onclick="sendToParent('hello')">Say Hello</button>
                <button onclick="sendToParent('ping')">Ping Parent</button>
                <button class="secondary" onclick="requestData()">Request Data</button>
            </div>
            <div class="message-area" id="messages">
                <div class="message">
                    <span class="time">--:--:--</span>
                    <span class="text">Waiting for messages...</span>
                </div>
            </div>
            <div class="input-group">
                <input type="text" id="customMsg" placeholder="Custom message...">
                <button onclick="sendCustom()">Send</button>
            </div>
        </div>

        <div class="card">
            <h2>Actions</h2>
            <div class="btn-group">
                <button onclick="showAlert()">Show Alert</button>
                <button onclick="logInfo()">Log Info</button>
                <button class="secondary" onclick="closeWindow()">Close</button>
            </div>
        </div>
    </div>

    <script>
        function getTime() {{
            return new Date().toLocaleTimeString();
        }}

        function addMessage(text) {{
            const area = document.getElementById('messages');
            const msg = document.createElement('div');
            msg.className = 'message';
            msg.innerHTML = `<span class="time">${{getTime()}}</span><span class="text">${{text}}</span>`;
            area.appendChild(msg);
            area.scrollTop = area.scrollHeight;
        }}

        function sendToParent(type) {{
            if (window.auroraview && window.auroraview.call) {{
                window.auroraview.call('send_to_parent', {{ type: type, timestamp: Date.now() }});
                addMessage(`Sent: ${{type}}`);
            }}
        }}

        function sendCustom() {{
            const input = document.getElementById('customMsg');
            const msg = input.value.trim();
            if (msg) {{
                sendToParent(msg);
                input.value = '';
            }}
        }}

        function requestData() {{
            if (window.auroraview && window.auroraview.call) {{
                window.auroraview.call('request_from_parent', {{ request: 'data' }});
                addMessage('Requested data from parent');
            }}
        }}

        function showAlert() {{
            alert('Hello from Child-Aware Demo!\\nMode: {mode}');
        }}

        function logInfo() {{
            console.log('Child-Aware Demo Info:', {{
                mode: '{mode}',
                parentId: '{parent_id}',
                childId: '{child_id}',
                exampleName: '{example_name}'
            }});
            addMessage('Info logged to console');
        }}

        function closeWindow() {{
            if (window.auroraview && window.auroraview.call) {{
                window.auroraview.call('close');
            }}
        }}

        // Listen for parent events
        window.addEventListener('auroraviewready', () => {{
            window.auroraview.on('parent:message', (data) => {{
                addMessage(`From parent: ${{JSON.stringify(data)}}`);
            }});

            window.auroraview.on('parent:response', (data) => {{
                addMessage(`Response: ${{JSON.stringify(data)}}`);
            }});
        }});

        // Enter key to send custom message
        document.getElementById('customMsg')?.addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') sendCustom();
        }});
    </script>
</body>
</html>
"""


def create_webview(ctx: ChildContext):
    """Create the WebView with context-aware configuration."""
    # Choose colors based on mode
    if ctx.is_child:
        bg_start = "#1a3a1a"  # Green tint for child mode
        bg_end = "#0d2a0d"
        accent = "#00ff88"
        mode_text = "CHILD WINDOW"
    else:
        bg_start = "#1a1a3a"  # Blue tint for standalone
        bg_end = "#0d0d2a"
        accent = "#00d4ff"
        mode_text = "STANDALONE"

    # Format HTML with context info
    html = HTML_TEMPLATE.format(
        bg_start=bg_start,
        bg_end=bg_end,
        accent=accent,
        mode_text=mode_text,
        mode="child" if ctx.is_child else "standalone",
        parent_id=ctx.parent_id or "N/A",
        child_id=ctx.child_id or "N/A",
        example_name=ctx.example_name or "N/A",
        show_parent_comm="block" if ctx.is_child else "none",
    )

    # Create WebView
    webview = ctx.create_webview(
        title="Child-Aware Demo",
        width=600,
        height=700,
        html=html,
        debug=True,
    )

    # Register handlers
    @webview.bind_call("send_to_parent")
    def send_to_parent(type: str = "", timestamp: int = 0):
        """Send a message to parent (if in child mode)."""
        if ctx.is_child:
            ctx.emit_to_parent("child:message", {
                "type": type,
                "timestamp": timestamp,
                "from": ctx.child_id,
            })
            print(f"[Demo] Sent to parent: {type}", file=sys.stderr)
        else:
            print(f"[Demo] Not in child mode, ignoring send: {type}", file=sys.stderr)

    @webview.bind_call("request_from_parent")
    def request_from_parent(request: str = ""):
        """Request data from parent."""
        if ctx.is_child:
            ctx.emit_to_parent("child:request", {
                "request": request,
                "from": ctx.child_id,
            })
            print(f"[Demo] Requested from parent: {request}", file=sys.stderr)

    @webview.bind_call("close")
    def close():
        """Close the window."""
        webview.close()

    # Listen for parent events (if in child mode)
    if ctx.bridge:
        ctx.on_parent_event("parent:data", lambda data: (
            webview.emit("parent:response", data)
        ))

    return webview


def main():
    """Run the demo."""
    print("[Demo] Starting Child-Aware Demo...", file=sys.stderr)
    print(f"[Demo] Child mode: {is_child_mode()}", file=sys.stderr)

    # Use run_example for automatic child mode handling
    run_example(create_webview)


if __name__ == "__main__":
    main()
