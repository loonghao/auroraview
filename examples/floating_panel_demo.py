"""Floating Panel Demo - Create floating tool windows for DCC applications.

This example demonstrates how to create a floating panel that follows
a parent application window, similar to AI assistant panels in Photoshop.

Features demonstrated:
- Frameless, transparent window
- Owner mode for window following
- Tool window style (hide from taskbar/Alt+Tab)
- Small trigger button + expandable panel
- Position relative to parent window
- Always on top support

Use cases:
- AI assistant panels in DCC apps
- Quick action toolbars
- Floating property editors
- Context-sensitive tool palettes

Supported AuroraView Parameters:
- frame=False: Frameless window (no title bar, borders)
- transparent=True: Transparent window background
- always_on_top=True: Keep window always on top
- parent_hwnd: Parent window HWND for owner relationship
- embed_mode="owner": Window follows parent minimize/restore
- tool_window=True: Hide from taskbar and Alt+Tab (WS_EX_TOOLWINDOW)

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import sys

# HTML for the floating panel UI
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
            <span class="panel-title">AI Assistant</span>
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

# HTML for the small trigger button
BUTTON_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            background: transparent;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
        }
        .trigger-btn {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background: linear-gradient(135deg, #00d4ff 0%, #0099cc 100%);
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: 0 4px 12px rgba(0, 212, 255, 0.4);
            transition: transform 0.2s, box-shadow 0.2s;
            -webkit-app-region: no-drag;
        }
        .trigger-btn:hover {
            transform: scale(1.1);
            box-shadow: 0 6px 16px rgba(0, 212, 255, 0.5);
        }
        .trigger-btn svg {
            width: 20px;
            height: 20px;
            fill: white;
        }
    </style>
</head>
<body>
    <button class="trigger-btn" onclick="togglePanel()">
        <svg viewBox="0 0 24 24">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
    </button>
    <script>
        function togglePanel() {
            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('toggle_panel');
            }
        }
    </script>
</body>
</html>
"""


def run_floating_panel_demo():
    """Run the floating panel demo.

    This demo shows two approaches:
    1. A small trigger button (24x24 or 40x40)
    2. An expandable panel that appears when clicked
    """
    from auroraview import AuroraView

    # State tracking
    panel_visible = False
    panel_webview = None

    class TriggerButton(AuroraView):
        """Small trigger button that opens the floating panel."""

        def __init__(self):
            super().__init__(
                html=BUTTON_HTML,
                width=48,
                height=48,
                frame=False,  # Frameless window
                transparent=True,  # Transparent background
                always_on_top=True,  # Keep on top of other windows
                tool_window=True,  # Hide from taskbar and Alt+Tab
            )
            self.bind_call("toggle_panel", self.toggle_panel)

        def toggle_panel(self, *args, **kwargs):
            """Toggle the floating panel visibility."""
            nonlocal panel_visible, panel_webview

            if panel_visible and panel_webview:
                panel_webview.close()
                panel_webview = None
                panel_visible = False
            else:
                # Create and show the panel
                panel_webview = FloatingPanel(parent_hwnd=self.get_hwnd())
                panel_webview.show()
                panel_visible = True

    class FloatingPanel(AuroraView):
        """The expandable floating panel."""

        def __init__(self, parent_hwnd=None):
            super().__init__(
                html=PANEL_HTML,
                width=350,
                height=180,
                frame=False,  # Frameless window
                transparent=True,  # Transparent background
                always_on_top=True,  # Keep on top of other windows
                parent_hwnd=parent_hwnd,
                embed_mode="owner",  # Follow parent window minimize/restore
                tool_window=True,  # Hide from taskbar and Alt+Tab
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
            print(f"[FloatingPanel] Received message: {message}")
            # Here you would integrate with your AI service
            self.emit("response", {"text": f"Processing: {message}"})

    # Create and show the trigger button
    print("Starting Floating Panel Demo...")
    print("Click the button to toggle the AI assistant panel.")
    print("Press Ctrl+C to exit.")

    trigger = TriggerButton()
    trigger.show()


def run_simple_panel_demo():
    """Run a simpler demo showing just the floating panel.

    This is useful for testing the panel UI without the trigger button.
    """
    from auroraview import AuroraView

    class SimpleFloatingPanel(AuroraView):
        """A simple floating panel demo."""

        def __init__(self):
            super().__init__(
                html=PANEL_HTML,
                width=350,
                height=180,
                frame=False,  # Frameless window
                transparent=True,  # Transparent background
                always_on_top=True,  # Keep on top of other windows
                tool_window=True,  # Hide from taskbar and Alt+Tab
            )
            self.bind_call("close_panel", self.close)
            self.bind_call("send_message", self.handle_message)

        def handle_message(self, message: str = ""):
            """Handle message from the input field."""
            print(f"[Panel] Message: {message}")

    print("Starting Simple Floating Panel Demo...")
    print("This shows just the panel without a trigger button.")

    panel = SimpleFloatingPanel()
    panel.show()


if __name__ == "__main__":
    # Check command line args
    if len(sys.argv) > 1 and sys.argv[1] == "--simple":
        run_simple_panel_demo()
    else:
        run_floating_panel_demo()