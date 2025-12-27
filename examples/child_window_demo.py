"""Child Window Demo - Unified child window system demonstration.

This example demonstrates the new unified child window system that allows
examples to run both standalone and as child windows of Gallery.

Key Features:
- Automatic mode detection (standalone vs child)
- Parent-child IPC communication
- Context-aware UI styling
- Seamless integration with Gallery

Usage:
    # Standalone mode
    python examples/child_window_demo.py

    # As Gallery child (Gallery sets env vars automatically)
    # Or manually:
    AURORAVIEW_PARENT_ID=gallery AURORAVIEW_CHILD_ID=test python examples/child_window_demo.py

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import sys

from auroraview import ChildContext, is_child_mode


def get_html(ctx: ChildContext) -> str:
    """Generate HTML with context-aware styling."""
    # Different colors for different modes
    if ctx.is_child:
        colors = {
            "bg1": "#1a2e1a",
            "bg2": "#0d1a0d",
            "accent": "#4ade80",
            "accent_dark": "#22c55e",
            "badge_bg": "#22c55e",
            "badge_text": "#0d1a0d",
        }
        mode_label = "CHILD MODE"
        mode_desc = "Running as child window of Gallery"
    else:
        colors = {
            "bg1": "#1a1a2e",
            "bg2": "#0d0d1a",
            "accent": "#818cf8",
            "accent_dark": "#6366f1",
            "badge_bg": "#6366f1",
            "badge_text": "#ffffff",
        }
        mode_label = "STANDALONE"
        mode_desc = "Running independently"

    return f"""
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Child Window Demo</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, {colors['bg1']} 0%, {colors['bg2']} 100%);
            color: #e4e4e4;
            min-height: 100vh;
            padding: 24px;
        }}
        .container {{ max-width: 700px; margin: 0 auto; }}
        
        .header {{
            text-align: center;
            margin-bottom: 32px;
        }}
        .header h1 {{
            font-size: 28px;
            color: {colors['accent']};
            margin-bottom: 12px;
        }}
        .mode-badge {{
            display: inline-block;
            background: {colors['badge_bg']};
            color: {colors['badge_text']};
            padding: 8px 20px;
            border-radius: 24px;
            font-size: 13px;
            font-weight: 600;
            letter-spacing: 0.5px;
        }}
        .mode-desc {{
            color: #888;
            font-size: 14px;
            margin-top: 12px;
        }}
        
        .card {{
            background: rgba(255, 255, 255, 0.03);
            border-radius: 16px;
            padding: 24px;
            margin-bottom: 20px;
            border: 1px solid rgba(255, 255, 255, 0.08);
        }}
        .card h2 {{
            color: {colors['accent']};
            font-size: 16px;
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .card h2::before {{
            content: '';
            width: 4px;
            height: 18px;
            background: {colors['accent']};
            border-radius: 2px;
        }}
        
        .info-grid {{
            display: grid;
            grid-template-columns: 140px 1fr;
            gap: 12px;
        }}
        .info-label {{
            color: #666;
            font-size: 13px;
        }}
        .info-value {{
            color: #fff;
            font-family: 'SF Mono', Monaco, monospace;
            font-size: 13px;
            background: rgba(0, 0, 0, 0.2);
            padding: 4px 10px;
            border-radius: 6px;
        }}
        
        .btn-row {{
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
            margin-bottom: 16px;
        }}
        button {{
            background: {colors['accent']};
            color: {colors['bg2']};
            border: none;
            padding: 12px 20px;
            border-radius: 10px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: all 0.2s;
        }}
        button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
        }}
        button:active {{ transform: translateY(0); }}
        button.secondary {{
            background: rgba(255, 255, 255, 0.08);
            color: #e4e4e4;
        }}
        button:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }}
        
        .log-area {{
            background: rgba(0, 0, 0, 0.3);
            border-radius: 10px;
            padding: 16px;
            max-height: 250px;
            overflow-y: auto;
            font-family: 'SF Mono', Monaco, monospace;
            font-size: 12px;
        }}
        .log-entry {{
            padding: 6px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            display: flex;
            gap: 12px;
        }}
        .log-entry:last-child {{ border-bottom: none; }}
        .log-time {{ color: #555; min-width: 70px; }}
        .log-type {{
            min-width: 60px;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 10px;
            text-transform: uppercase;
        }}
        .log-type.send {{ background: #3b82f6; color: white; }}
        .log-type.recv {{ background: #22c55e; color: white; }}
        .log-type.info {{ background: #6366f1; color: white; }}
        .log-type.error {{ background: #ef4444; color: white; }}
        .log-msg {{ color: #ccc; flex: 1; }}
        
        .input-row {{
            display: flex;
            gap: 10px;
            margin-top: 16px;
        }}
        input {{
            flex: 1;
            background: rgba(0, 0, 0, 0.3);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 10px;
            padding: 12px 16px;
            color: #fff;
            font-size: 14px;
        }}
        input:focus {{
            outline: none;
            border-color: {colors['accent']};
        }}
        
        .hidden {{ display: none !important; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Child Window Demo</h1>
            <div class="mode-badge">{mode_label}</div>
            <p class="mode-desc">{mode_desc}</p>
        </div>
        
        <div class="card">
            <h2>Context Information</h2>
            <div class="info-grid">
                <span class="info-label">Mode</span>
                <span class="info-value">{"child" if ctx.is_child else "standalone"}</span>
                
                <span class="info-label">Parent ID</span>
                <span class="info-value">{ctx.parent_id or "N/A"}</span>
                
                <span class="info-label">Child ID</span>
                <span class="info-value">{ctx.child_id or "N/A"}</span>
                
                <span class="info-label">Example Name</span>
                <span class="info-value">{ctx.example_name or "N/A"}</span>
            </div>
        </div>
        
        <div class="card {'hidden' if not ctx.is_child else ''}">
            <h2>Parent Communication</h2>
            <div class="btn-row">
                <button onclick="sendPing()">Ping Parent</button>
                <button onclick="sendHello()">Say Hello</button>
                <button onclick="requestState()" class="secondary">Request State</button>
            </div>
            
            <div class="log-area" id="log">
                <div class="log-entry">
                    <span class="log-time">--:--:--</span>
                    <span class="log-type info">INFO</span>
                    <span class="log-msg">Waiting for communication...</span>
                </div>
            </div>
            
            <div class="input-row">
                <input type="text" id="customMsg" placeholder="Type a custom message...">
                <button onclick="sendCustom()">Send</button>
            </div>
        </div>
        
        <div class="card">
            <h2>Local Actions</h2>
            <div class="btn-row">
                <button onclick="logContext()">Log Context</button>
                <button onclick="showNotification()">Show Notification</button>
                <button onclick="closeWindow()" class="secondary">Close Window</button>
            </div>
        </div>
    </div>
    
    <script>
        const logArea = document.getElementById('log');
        let logCount = 0;
        
        function getTime() {{
            return new Date().toLocaleTimeString('en-US', {{ hour12: false }});
        }}
        
        function addLog(type, msg) {{
            logCount++;
            if (logCount === 1) {{
                logArea.innerHTML = '';
            }}
            
            const entry = document.createElement('div');
            entry.className = 'log-entry';
            entry.innerHTML = `
                <span class="log-time">${{getTime()}}</span>
                <span class="log-type ${{type}}">${{type}}</span>
                <span class="log-msg">${{msg}}</span>
            `;
            logArea.appendChild(entry);
            logArea.scrollTop = logArea.scrollHeight;
        }}
        
        // Parent communication functions
        function sendPing() {{
            if (window.auroraview?.call) {{
                auroraview.call('emit_to_parent', {{ event: 'ping', data: {{ time: Date.now() }} }});
                addLog('send', 'Sent ping to parent');
            }}
        }}
        
        function sendHello() {{
            if (window.auroraview?.call) {{
                auroraview.call('emit_to_parent', {{ 
                    event: 'hello', 
                    data: {{ message: 'Hello from child!', timestamp: Date.now() }}
                }});
                addLog('send', 'Sent hello to parent');
            }}
        }}
        
        function requestState() {{
            if (window.auroraview?.call) {{
                auroraview.call('emit_to_parent', {{ 
                    event: 'request_state', 
                    data: {{ from: '{ctx.child_id or "unknown"}' }}
                }});
                addLog('send', 'Requested state from parent');
            }}
        }}
        
        function sendCustom() {{
            const input = document.getElementById('customMsg');
            const msg = input.value.trim();
            if (msg && window.auroraview?.call) {{
                auroraview.call('emit_to_parent', {{ 
                    event: 'custom_message', 
                    data: {{ message: msg }}
                }});
                addLog('send', `Sent: ${{msg}}`);
                input.value = '';
            }}
        }}
        
        // Local actions
        function logContext() {{
            console.log('Child Window Context:', {{
                isChild: {'true' if ctx.is_child else 'false'},
                parentId: '{ctx.parent_id or "null"}',
                childId: '{ctx.child_id or "null"}',
                exampleName: '{ctx.example_name or "null"}'
            }});
            addLog('info', 'Context logged to console');
        }}
        
        function showNotification() {{
            if (window.auroraview?.call) {{
                auroraview.call('show_notification', {{ 
                    title: 'Child Window Demo',
                    message: 'This is a notification from the child window!'
                }});
            }}
        }}
        
        function closeWindow() {{
            if (window.auroraview?.call) {{
                auroraview.call('close_window');
            }}
        }}
        
        // Listen for parent events
        window.addEventListener('auroraviewready', () => {{
            console.log('[ChildWindow] AuroraView ready');
            
            auroraview.on('parent:message', (data) => {{
                addLog('recv', `Parent message: ${{JSON.stringify(data)}}`);
            }});
            
            auroraview.on('parent:pong', (data) => {{
                addLog('recv', `Pong received! RTT: ${{Date.now() - data.originalTime}}ms`);
            }});
            
            auroraview.on('parent:state', (data) => {{
                addLog('recv', `State: ${{JSON.stringify(data)}}`);
            }});
        }});
        
        // Enter to send custom message
        document.getElementById('customMsg')?.addEventListener('keypress', (e) => {{
            if (e.key === 'Enter') sendCustom();
        }});
    </script>
</body>
</html>
"""


def main():
    """Run the child window demo."""

    print("[ChildWindowDemo] Starting...", file=sys.stderr)
    print(f"[ChildWindowDemo] Child mode: {is_child_mode()}", file=sys.stderr)

    # Create context
    with ChildContext() as ctx:
        if ctx.is_child:
            print(f"[ChildWindowDemo] Parent: {ctx.parent_id}", file=sys.stderr)
            print(f"[ChildWindowDemo] Child ID: {ctx.child_id}", file=sys.stderr)

        # Generate HTML
        html = get_html(ctx)

        # Create WebView
        webview = ctx.create_webview(
            title="Child Window Demo",
            width=750,
            height=700,
            html=html,
            debug=True,
        )

        # Register API handlers
        @webview.bind_call("emit_to_parent")
        def emit_to_parent(event: str = "", data: dict = None):
            """Emit event to parent window."""
            if ctx.is_child and ctx.bridge:
                ctx.emit_to_parent(event, data or {})
                print(f"[ChildWindowDemo] Emitted to parent: {event}", file=sys.stderr)
                return {"success": True}
            else:
                print("[ChildWindowDemo] Not in child mode, cannot emit", file=sys.stderr)
                return {"success": False, "reason": "not_child_mode"}

        @webview.bind_call("show_notification")
        def show_notification(title: str = "", message: str = ""):
            """Show a notification."""
            print(f"[ChildWindowDemo] Notification: {title} - {message}", file=sys.stderr)
            # In a real app, you might use native notifications here
            return {"success": True}

        @webview.bind_call("close_window")
        def close_window():
            """Close the window."""
            webview.close()

        # Listen for parent events (if in child mode)
        if ctx.bridge:
            def on_parent_pong(data):
                webview.emit("parent:pong", data)

            def on_parent_state(data):
                webview.emit("parent:state", data)

            def on_parent_message(data):
                webview.emit("parent:message", data)

            ctx.on_parent_event("pong", on_parent_pong)
            ctx.on_parent_event("state", on_parent_state)
            ctx.on_parent_event("message", on_parent_message)

        # Show the window
        webview.show()


if __name__ == "__main__":
    main()
