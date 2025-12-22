"""Floating Panel Demo - Create floating tool windows for DCC applications.

This example demonstrates how to create a floating panel that follows
a parent application window, similar to AI assistant panels in Photoshop.

Features demonstrated:
- Frameless, transparent window with NO shadow (truly transparent button)
- Independent floating windows (like Qt's QDialog with no parent)
- Tool window style (hide from taskbar/Alt+Tab)
- Small trigger button + expandable panel
- Always on top support

Use cases:
- AI assistant panels in DCC apps
- Quick action toolbars
- Floating property editors
- Context-sensitive tool palettes

Key AuroraView Parameters for Transparent Floating Windows:
- frame=False: Frameless window (no title bar, borders)
- transparent=True: Transparent window background
- undecorated_shadow=False: CRITICAL - Disable shadow for truly transparent windows
- always_on_top=True: Keep window always on top
- tool_window=True: Hide from taskbar and Alt+Tab (WS_EX_TOOLWINDOW)
- embed_mode="none": Independent window (not attached to parent)

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
        <div class="panel-header drag-handle" onmousedown="startNativeDrag(event)">
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

        // Use native drag for better responsiveness
        function startNativeDrag(event) {
            // Only trigger on left mouse button and not on buttons
            if (event.button === 0 && event.target.tagName !== 'BUTTON') {
                if (window.auroraview && window.auroraview.startDrag) {
                    window.auroraview.startDrag();
                }
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

# HTML for the small trigger button - truly transparent circular button
BUTTON_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        html, body {
            background: transparent !important;
            width: 100%;
            height: 100%;
        }
        body {
            display: flex;
            align-items: center;
            justify-content: center;
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
    1. A small trigger button (truly transparent, no shadow)
    2. An expandable panel that appears when clicked (independent window)
    """
    from auroraview import AuroraView

    # State tracking
    panel_visible = False
    panel_webview = None

    class TriggerButton(AuroraView):
        """Small trigger button that opens the floating panel.

        Key configuration for truly transparent button:
        - frame=False: No window decorations
        - transparent=True: Transparent background
        - undecorated_shadow=False: CRITICAL - removes the shadow that would
          otherwise appear around the frameless window
        - tool_window=True: Hide from taskbar/Alt+Tab
        """

        def __init__(self):
            super().__init__(
                html=BUTTON_HTML,
                width=48,
                height=48,
                frame=False,  # Frameless window
                transparent=True,  # Transparent background
                undecorated_shadow=False,  # CRITICAL: No shadow for truly transparent button
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
                # Create and show the panel as an independent window
                # Note: embed_mode="none" creates an independent window (like Qt's QDialog)
                # This is different from embed_mode="owner" which would follow parent
                panel_webview = FloatingPanel()
                panel_webview.show()
                panel_visible = True

    class FloatingPanel(AuroraView):
        """The expandable floating panel.

        This is an independent window (not attached to any parent).
        Key configuration:
        - embed_mode="none": Independent window (default for AuroraView)
        - frame=False: Frameless for custom styling
        - transparent=True: Transparent background for rounded corners
        - undecorated_shadow=False: Clean look without system shadow
        """

        def __init__(self):
            super().__init__(
                html=PANEL_HTML,
                width=350,
                height=180,
                frame=False,  # Frameless window
                transparent=True,  # Transparent background
                undecorated_shadow=False,  # No shadow for clean look
                always_on_top=True,  # Keep on top of other windows
                embed_mode="none",  # Independent window (like Qt's QDialog)
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
    print("Click the circular button to toggle the AI assistant panel.")
    print("Both windows are independent and can be moved freely.")
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
                undecorated_shadow=False,  # No shadow for clean look
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
