"""System Tray Demo - Desktop application with system tray support.

This example demonstrates how to create a desktop application with:
- System tray icon
- Context menu in tray
- Hide to tray on close
- Show on tray click

Features demonstrated:
- System tray icon with tooltip
- Context menu with Show/Hide/Exit options
- Minimize to tray instead of closing
- Click tray icon to show/hide window
- Tool window style for floating panels

Use cases:
- Background applications (monitoring tools, sync services)
- Desktop assistants that stay in tray
- Notification-based tools
- Always-available utilities

Note: System tray support is currently available through run_desktop().
For advanced tray configuration, see the TrayConfig in Rust.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import sys

# HTML for the main application UI
APP_HTML = """
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
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 40px;
        }

        .container {
            text-align: center;
            max-width: 500px;
        }

        .icon {
            font-size: 64px;
            margin-bottom: 24px;
        }

        h1 {
            font-size: 28px;
            font-weight: 600;
            color: #00d4ff;
            margin-bottom: 16px;
        }

        p {
            font-size: 16px;
            color: #aaa;
            line-height: 1.6;
            margin-bottom: 24px;
        }

        .status {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 12px 24px;
            background: rgba(0, 212, 255, 0.1);
            border: 1px solid rgba(0, 212, 255, 0.3);
            border-radius: 8px;
            margin-bottom: 24px;
        }

        .status-dot {
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #00ff88;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }

        .actions {
            display: flex;
            gap: 12px;
            justify-content: center;
        }

        .btn {
            padding: 12px 24px;
            border: none;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }

        .btn-primary {
            background: linear-gradient(135deg, #00d4ff 0%, #0099cc 100%);
            color: #fff;
        }

        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 212, 255, 0.3);
        }

        .btn-secondary {
            background: rgba(255, 255, 255, 0.1);
            color: #e4e4e4;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }

        .btn-secondary:hover {
            background: rgba(255, 255, 255, 0.15);
        }

        .info {
            margin-top: 32px;
            font-size: 12px;
            color: #666;
        }

        .info code {
            background: rgba(255, 255, 255, 0.1);
            padding: 2px 6px;
            border-radius: 4px;
            font-family: 'Fira Code', monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">🎯</div>
        <h1>AuroraView Tray Demo</h1>
        <p>
            This application demonstrates system tray functionality.
            Close the window to minimize to tray, or use the tray menu
            to control the application.
        </p>
        <div class="status">
            <span class="status-dot"></span>
            <span>Running in background</span>
        </div>
        <div class="actions">
            <button class="btn btn-primary" onclick="hideToTray()">Hide to Tray</button>
            <button class="btn btn-secondary" onclick="showNotification()">Test Notification</button>
        </div>
        <div class="info">
            <p>Right-click the tray icon for options</p>
            <p>Use <code>tool_window=True</code> to hide from taskbar</p>
        </div>
    </div>

    <script>
        function hideToTray() {
            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('hide_to_tray');
            }
        }

        function showNotification() {
            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('show_notification', {
                    title: 'AuroraView',
                    message: 'This is a test notification!'
                });
            }
        }

        // Listen for tray events
        window.addEventListener('auroraviewready', () => {
            console.log('AuroraView ready - tray demo');

            // Subscribe to tray menu events
            if (window.auroraview && window.auroraview.on) {
                window.auroraview.on('tray_menu', (data) => {
                    console.log('Tray menu clicked:', data);
                });
            }
        });
    </script>
</body>
</html>
"""


def run_tray_demo():
    """Run the system tray demo.

    This demo shows how to create a desktop application with system tray support.
    Uses run_desktop() with tray parameters for full system tray functionality.
    """
    from auroraview._core import run_desktop

    print("Starting System Tray Demo...")
    print()
    print("Features:")
    print("  - System tray icon with tooltip")
    print("  - Right-click menu: Show Window / Exit")
    print("  - Click tray icon to show window")
    print("  - Close window to hide to tray")
    print()
    print("Try:")
    print("  1. Close the window (X button) - it will hide to tray")
    print("  2. Click the tray icon to show the window again")
    print("  3. Right-click tray icon for menu options")
    print()

    run_desktop(
        title="AuroraView Tray Demo",
        width=600,
        height=500,
        html=APP_HTML,
        tray_enabled=True,
        tray_tooltip="AuroraView Tray Demo",
        tray_show_on_click=True,
        tray_hide_on_close=True,
    )


def run_tool_window_demo():
    """Run a demo showing tool_window mode.

    tool_window=True creates a window that:
    - Does NOT appear in the taskbar
    - Does NOT appear in Alt+Tab
    - Has a smaller title bar (if frame=True)

    This is useful for floating tool panels, property editors, etc.
    """
    from auroraview import AuroraView

    class ToolWindow(AuroraView):
        """A tool window that hides from taskbar and Alt+Tab."""

        def __init__(self):
            super().__init__(
                title="Tool Window",
                html=APP_HTML,
                width=400,
                height=300,
                frame=True,  # Show window frame (smaller for tool windows)
                always_on_top=True,  # Keep on top
                tool_window=True,  # Hide from taskbar and Alt+Tab
            )
            self.bind_call("hide_to_tray", self.close)
            self.bind_call("show_notification", lambda **kw: print(f"Notification: {kw}"))

    print("Starting Tool Window Demo...")
    print()
    print("This window:")
    print("  - Does NOT appear in taskbar")
    print("  - Does NOT appear in Alt+Tab")
    print("  - Stays on top of other windows")
    print()

    tool = ToolWindow()
    tool.show()


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--tool":
        run_tool_window_demo()
    else:
        run_tray_demo()
