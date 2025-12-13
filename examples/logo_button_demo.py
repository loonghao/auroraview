"""Logo Button Demo - Transparent floating logo button with AI panel.

This example demonstrates how to create a floating transparent button
using the AuroraView logo image, similar to AI assistant triggers.

Features demonstrated:
- Transparent window with logo image
- Frameless, borderless window
- Tool window style (hide from taskbar/Alt+Tab)
- Click to open AI assistant panel
- Drag support

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import base64
from pathlib import Path

# Get the logo path relative to this file
ASSETS_DIR = Path(__file__).parent.parent / "assets" / "icons"
LOGO_64 = ASSETS_DIR / "auroraview-64.png"


def get_logo_data_uri():
    """Load logo as base64 data URI to avoid file:// protocol issues."""
    if not LOGO_64.exists():
        return None
    with open(LOGO_64, "rb") as f:
        data = base64.b64encode(f.read()).decode("utf-8")
    return f"data:image/png;base64,{data}"

# HTML for the logo button - transparent window showing just the logo
LOGO_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        html, body {
            width: 100%;
            height: 100%;
            background: transparent;
            overflow: hidden;
        }

        .logo-container {
            width: 100%;
            height: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .logo-btn {
            width: 64px;
            height: 64px;
            background: transparent;
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: transform 0.2s, filter 0.2s;
            -webkit-app-region: drag;
            padding: 0;
        }

        .logo-btn:hover {
            transform: scale(1.05);
        }

        .logo-btn:active {
            transform: scale(0.95);
        }

        .logo-btn img {
            width: 100%;
            height: 100%;
            object-fit: contain;
            pointer-events: none;
        }

        /* Pulse animation when idle */
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.85; }
        }

        .logo-btn.idle {
            animation: pulse 2s ease-in-out infinite;
        }
    </style>
</head>
<body>
    <div class="logo-container">
        <button class="logo-btn idle" id="logoBtn">
            <img src="LOGO_PATH_PLACEHOLDER" alt="AuroraView" draggable="false">
        </button>
    </div>

    <script>
        let clickCount = 0;

        document.getElementById('logoBtn').addEventListener('click', function(e) {
            clickCount++;
            this.classList.remove('idle');

            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('on_click', { count: clickCount });
            }

            // Resume idle animation after a delay
            setTimeout(() => {
                this.classList.add('idle');
            }, 1000);
        });
    </script>
</body>
</html>
"""

# HTML for the floating AI panel
PANEL_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: transparent;
            overflow: hidden;
        }

        .panel {
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            border-radius: 12px;
            padding: 16px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.1);
            color: #e4e4e4;
            min-width: 300px;
        }

        .panel-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 12px;
            padding-bottom: 12px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }

        .panel-title {
            font-size: 14px;
            font-weight: 600;
            color: #00d4ff;
        }

        .close-btn {
            background: none;
            border: none;
            color: #888;
            cursor: pointer;
            font-size: 18px;
            padding: 4px 8px;
            border-radius: 4px;
            transition: all 0.2s;
        }

        .close-btn:hover {
            background: rgba(255, 255, 255, 0.1);
            color: #fff;
        }

        .input-area {
            display: flex;
            gap: 8px;
            margin-bottom: 12px;
        }

        .input-field {
            flex: 1;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 8px;
            padding: 10px 14px;
            color: #fff;
            font-size: 14px;
            outline: none;
            transition: border-color 0.2s;
        }

        .input-field:focus {
            border-color: #00d4ff;
        }

        .input-field::placeholder {
            color: #666;
        }

        .send-btn {
            background: linear-gradient(135deg, #00d4ff 0%, #0099cc 100%);
            border: none;
            border-radius: 8px;
            padding: 10px 16px;
            color: #fff;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }

        .send-btn:hover {
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(0, 212, 255, 0.3);
        }

        .send-btn:active {
            transform: translateY(0);
        }

        .suggestions {
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
        }

        .suggestion-chip {
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            padding: 6px 12px;
            font-size: 12px;
            color: #aaa;
            cursor: pointer;
            transition: all 0.2s;
        }

        .suggestion-chip:hover {
            background: rgba(0, 212, 255, 0.1);
            border-color: #00d4ff;
            color: #00d4ff;
        }

        /* Drag handle for frameless window */
        .drag-handle {
            -webkit-app-region: drag;
            cursor: move;
        }

        .no-drag {
            -webkit-app-region: no-drag;
        }
    </style>
</head>
<body>
    <div class="panel">
        <div class="panel-header drag-handle">
            <span class="panel-title">AuroraView AI</span>
            <button class="close-btn no-drag" onclick="closePanel()">&times;</button>
        </div>
        <div class="input-area no-drag">
            <input type="text" class="input-field" placeholder="Ask me anything..." id="input">
            <button class="send-btn" onclick="sendMessage()">Send</button>
        </div>
        <div class="suggestions no-drag">
            <span class="suggestion-chip" onclick="selectSuggestion('Generate texture')">Generate texture</span>
            <span class="suggestion-chip" onclick="selectSuggestion('Fix UV mapping')">Fix UV mapping</span>
            <span class="suggestion-chip" onclick="selectSuggestion('Optimize mesh')">Optimize mesh</span>
        </div>
    </div>

    <script>
        function closePanel() {
            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('close_panel');
            }
        }

        function sendMessage() {
            const input = document.getElementById('input');
            const message = input.value.trim();
            if (message && window.auroraview && window.auroraview.call) {
                window.auroraview.call('send_message', { message: message });
                input.value = '';
            }
        }

        function selectSuggestion(text) {
            document.getElementById('input').value = text;
        }

        // Handle Enter key
        document.getElementById('input').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendMessage();
        });
    </script>
</body>
</html>
"""


def run_logo_button_demo():
    """Run the logo button demo.

    Creates a transparent floating window with the AuroraView logo.
    - Single click: toggle AI panel
    - Drag: move the window
    """
    from auroraview import AuroraView

    # Load logo as base64 data URI
    logo_data_uri = get_logo_data_uri()
    if not logo_data_uri:
        print(f"Logo not found: {LOGO_64}")
        return

    print(f"Loaded logo from: {LOGO_64}")

    # Replace placeholder with data URI
    html = LOGO_HTML.replace("LOGO_PATH_PLACEHOLDER", logo_data_uri)

    # State tracking
    panel_visible = False
    panel_webview = None

    class FloatingPanel(AuroraView):
        """The expandable floating AI panel."""

        def __init__(self, parent_hwnd=None):
            super().__init__(
                html=PANEL_HTML,
                width=350,
                height=180,
                frame=False,
                transparent=True,
                always_on_top=True,
                parent_hwnd=parent_hwnd,
                embed_mode="owner",
                tool_window=True,
                undecorated_shadow=False,
            )
            self.bind_call("close_panel", self.close_panel)
            self.bind_call("send_message", self.handle_message)

        def close_panel(self, *args, **kwargs):
            """Close the panel."""
            nonlocal panel_visible, panel_webview
            self.close()
            panel_webview = None
            panel_visible = False

        def handle_message(self, message: str = ""):
            """Handle message from the input field."""
            print(f"[AuroraView AI] Message: {message}")

    class LogoButton(AuroraView):
        """Floating logo button."""

        def __init__(self):
            super().__init__(
                html=html,
                width=64,
                height=64,
                frame=False,
                transparent=True,
                always_on_top=True,
                tool_window=True,
                undecorated_shadow=False,
            )
            self.bind_call("on_click", self.on_click)

        def on_click(self, count: int = 0):
            """Handle click event - toggle panel."""
            nonlocal panel_visible, panel_webview

            print(f"[LogoButton] Clicked! Count: {count}")

            if panel_visible and panel_webview:
                panel_webview.close()
                panel_webview = None
                panel_visible = False
            else:
                # Create and show the panel
                panel_webview = FloatingPanel(parent_hwnd=self.get_hwnd())
                panel_webview.show()
                panel_visible = True

    print("Starting Logo Button Demo...")
    print()
    print("Features:")
    print("  - Transparent window with logo")
    print("  - Click to toggle AI panel")
    print("  - Drag to move")
    print()

    button = LogoButton()
    button.show()


if __name__ == "__main__":
    run_logo_button_demo()
