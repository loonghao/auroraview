"""Qt Custom Context Menu Demo.

This example demonstrates how to use custom context menus in QtWebView
for DCC applications like Maya, Houdini, etc.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import sys

try:
    from qtpy.QtWidgets import QApplication, QMainWindow, QVBoxLayout, QWidget

    from auroraview import QtWebView
except ImportError as e:
    print(f"Error: {e}")
    print("Please install Qt support: pip install auroraview[qt]")
    sys.exit(1)


class CustomMenuWindow(QMainWindow):
    """Main window with QtWebView and custom context menu."""

    def __init__(self):
        """Initialize the window."""
        super().__init__()
        self.setWindowTitle("Qt Custom Context Menu Demo")
        self.setGeometry(100, 100, 900, 700)

        # Create central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        layout = QVBoxLayout(central_widget)

        # Create QtWebView with custom context menu disabled
        self.webview = QtWebView(
            parent=self,
            title="Qt Custom Menu",
            width=900,
            height=700,
            dev_tools=True,
            context_menu=False,  # Disable native context menu
        )

        # Register event handler
        @self.webview.on("menu_action")
        def handle_menu_action(data):
            """Handle menu actions from JavaScript."""
            action = data.get("action")
            print(f"[Qt] Menu action: {action}")

            if action == "export":
                print("  → Exporting from Qt application...")
            elif action == "import":
                print("  → Importing into Qt application...")
            elif action == "settings":
                print("  → Opening Qt settings...")

        # Add webview to layout
        layout.addWidget(self.webview)

        # Load HTML content
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 20px;
                    background: #f5f5f5;
                }
                .container {
                    background: white;
                    padding: 30px;
                    border-radius: 8px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }
                .custom-menu {
                    display: none;
                    position: fixed;
                    background: white;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
                    z-index: 1000;
                    min-width: 160px;
                }
                .custom-menu ul {
                    list-style: none;
                    margin: 0;
                    padding: 4px 0;
                }
                .custom-menu li {
                    padding: 8px 16px;
                    cursor: pointer;
                    color: #333;
                }
                .custom-menu li:hover {
                    background: #e8e8e8;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Qt Custom Context Menu</h1>
                <p>Right-click anywhere to see the custom menu!</p>
                <p>This demonstrates custom menus in Qt-based DCC applications.</p>
            </div>

            <div id="customMenu" class="custom-menu">
                <ul>
                    <li onclick="handleMenuAction('export')">Export Scene</li>
                    <li onclick="handleMenuAction('import')">Import Assets</li>
                    <li onclick="handleMenuAction('settings')">Settings</li>
                </ul>
            </div>

            <script>
                const menu = document.getElementById('customMenu');

                document.addEventListener('contextmenu', (e) => {
                    e.preventDefault();
                    menu.style.display = 'block';
                    menu.style.left = e.pageX + 'px';
                    menu.style.top = e.pageY + 'px';
                });

                document.addEventListener('click', () => {
                    menu.style.display = 'none';
                });

                function handleMenuAction(action) {
                    if (window.auroraview) {
                        window.auroraview.send_event('menu_action', { action: action });
                    }
                    menu.style.display = 'none';
                }
            </script>
        </body>
        </html>
        """
        self.webview.load_html(html)


def main():
    """Run the Qt custom menu demo."""
    app = QApplication.instance() or QApplication(sys.argv)

    window = CustomMenuWindow()
    window.show()

    print("Qt Custom Context Menu Demo")
    print("Right-click in the window to see the custom menu!")

    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
