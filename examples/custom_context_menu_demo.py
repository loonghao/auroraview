"""Custom Context Menu Demo.

This example demonstrates how to disable the native browser context menu
and implement a custom right-click menu using JavaScript.

Note: This example uses the low-level WebView API for demonstration.
For most use cases, prefer QtWebView, AuroraView, or run_desktop.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from auroraview import WebView

# HTML with custom context menu
HTML_CONTENT = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Custom Context Menu Demo</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            min-height: 100vh;
        }

        .container {
            max-width: 800px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.1);
            padding: 30px;
            border-radius: 10px;
            backdrop-filter: blur(10px);
        }

        h1 {
            margin-top: 0;
        }

        .info {
            background: rgba(255, 255, 255, 0.2);
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }

        /* Custom context menu styles */
        .custom-menu {
            display: none;
            position: fixed;
            background: white;
            border: 1px solid #ccc;
            border-radius: 5px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.2);
            z-index: 1000;
            min-width: 180px;
        }

        .custom-menu ul {
            list-style: none;
            margin: 0;
            padding: 5px 0;
        }

        .custom-menu li {
            padding: 10px 20px;
            cursor: pointer;
            color: #333;
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .custom-menu li:hover {
            background: #f0f0f0;
        }

        .custom-menu li::before {
            content: '▸';
            color: #667eea;
        }

        .menu-separator {
            height: 1px;
            background: #e0e0e0;
            margin: 5px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Custom Context Menu Demo</h1>

        <div class="info">
            <p><strong>Try this:</strong> Right-click anywhere on this page to see the custom context menu!</p>
            <p>The native browser context menu has been disabled and replaced with a custom implementation.</p>
        </div>

        <div class="info">
            <h3>Features:</h3>
            <ul>
                <li>✓ Native context menu disabled</li>
                <li>✓ Custom styled menu</li>
                <li>✓ Python event integration</li>
                <li>✓ Configurable menu items</li>
            </ul>
        </div>
    </div>

    <!-- Custom context menu -->
    <div id="customMenu" class="custom-menu">
        <ul>
            <li onclick="handleMenuAction('export')">Export Scene</li>
            <li onclick="handleMenuAction('import')">Import Assets</li>
            <div class="menu-separator"></div>
            <li onclick="handleMenuAction('settings')">Settings</li>
            <li onclick="handleMenuAction('about')">About</li>
        </ul>
    </div>

    <script>
        const menu = document.getElementById('customMenu');

        // Show custom menu on right-click
        document.addEventListener('contextmenu', (e) => {
            e.preventDefault();

            // Position menu at cursor
            menu.style.display = 'block';
            menu.style.left = e.pageX + 'px';
            menu.style.top = e.pageY + 'px';

            // Adjust if menu goes off-screen
            const menuRect = menu.getBoundingClientRect();
            if (menuRect.right > window.innerWidth) {
                menu.style.left = (e.pageX - menuRect.width) + 'px';
            }
            if (menuRect.bottom > window.innerHeight) {
                menu.style.top = (e.pageY - menuRect.height) + 'px';
            }
        });

        // Hide menu on click elsewhere
        document.addEventListener('click', () => {
            menu.style.display = 'none';
        });

        // Handle menu actions
        function handleMenuAction(action) {
            console.log('Menu action:', action);

            // Send action to Python via AuroraView event system
            if (window.auroraview) {
                window.auroraview.send_event('menu_action', { action: action });
            }

            menu.style.display = 'none';
        }
    </script>
</body>
</html>
"""


def main():
    """Run the custom context menu demo."""
    # Create WebView with native context menu disabled
    webview = WebView(
        title="Custom Context Menu Demo",
        width=900,
        height=700,
        context_menu=False,  # Disable native context menu
        debug=True,  # Enable dev tools for inspection
    )

    # Register event handler for menu actions
    @webview.on("menu_action")
    def handle_menu_action(data):
        """Handle custom menu actions from JavaScript."""
        action = data.get("action")
        print(f"[Python] Menu action received: {action}")

        if action == "export":
            print("  → Exporting scene...")
        elif action == "import":
            print("  → Importing assets...")
        elif action == "settings":
            print("  → Opening settings...")
        elif action == "about":
            print("  → Showing about dialog...")

    # Load HTML content
    webview.load_html(HTML_CONTENT)

    # Show the window
    print("Custom Context Menu Demo")
    print("Right-click anywhere in the window to see the custom menu!")
    webview.show()


if __name__ == "__main__":
    main()
