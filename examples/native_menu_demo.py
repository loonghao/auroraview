"""Native Menu Demo - Application menu bar with keyboard shortcuts.

This example demonstrates AuroraView's native menu bar support,
including standard menus, custom menus, submenus, and keyboard shortcuts.

Features demonstrated:
- Creating menu bars with File, Edit, View, Help menus
- Custom menu items with action handlers
- Keyboard shortcuts (accelerators)
- Checkbox menu items
- Submenus
- Menu separators
- Dynamic menu updates
"""

from __future__ import annotations

# WebView import is done in main() to avoid circular imports
from auroraview.ui.menu import Menu, MenuBar, MenuItem

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Native Menu Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .header {
            background: linear-gradient(90deg, #16213e 0%, #0f3460 100%);
            padding: 20px;
            text-align: center;
            border-bottom: 1px solid #0f3460;
        }
        h1 {
            font-size: 24px;
            margin-bottom: 5px;
        }
        .subtitle {
            color: #888;
            font-size: 14px;
        }
        .main {
            flex: 1;
            padding: 30px;
            display: flex;
            gap: 30px;
        }
        .panel {
            flex: 1;
            background: #16213e;
            border-radius: 12px;
            padding: 20px;
            border: 1px solid #0f3460;
        }
        .panel h2 {
            font-size: 16px;
            color: #e94560;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 1px solid #0f3460;
        }
        .log-container {
            height: 300px;
            overflow-y: auto;
            background: #0f0f1a;
            border-radius: 8px;
            padding: 15px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 13px;
        }
        .log-entry {
            padding: 8px 12px;
            margin-bottom: 5px;
            background: #1a1a2e;
            border-radius: 4px;
            border-left: 3px solid #e94560;
        }
        .log-entry .time {
            color: #666;
            font-size: 11px;
        }
        .log-entry .action {
            color: #4ade80;
        }
        .shortcut-list {
            list-style: none;
        }
        .shortcut-list li {
            display: flex;
            justify-content: space-between;
            padding: 10px;
            background: #0f0f1a;
            margin-bottom: 5px;
            border-radius: 4px;
        }
        .shortcut-list .key {
            background: #e94560;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-family: monospace;
            font-size: 12px;
        }
        .settings-group {
            margin-bottom: 20px;
        }
        .settings-group h3 {
            font-size: 14px;
            color: #888;
            margin-bottom: 10px;
        }
        .toggle-row {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px;
            background: #0f0f1a;
            border-radius: 4px;
            margin-bottom: 5px;
        }
        .toggle-indicator {
            width: 40px;
            height: 20px;
            background: #333;
            border-radius: 10px;
            position: relative;
            transition: background 0.3s;
        }
        .toggle-indicator.on {
            background: #4ade80;
        }
        .toggle-indicator::after {
            content: '';
            position: absolute;
            width: 16px;
            height: 16px;
            background: white;
            border-radius: 50%;
            top: 2px;
            left: 2px;
            transition: left 0.3s;
        }
        .toggle-indicator.on::after {
            left: 22px;
        }
        .zoom-display {
            text-align: center;
            padding: 20px;
            background: #0f0f1a;
            border-radius: 8px;
            margin-top: 15px;
        }
        .zoom-value {
            font-size: 48px;
            font-weight: bold;
            color: #e94560;
        }
        .zoom-label {
            color: #666;
            font-size: 12px;
            margin-top: 5px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Native Menu Demo</h1>
        <p class="subtitle">Use the menu bar above or keyboard shortcuts to interact</p>
    </div>

    <div class="main">
        <div class="panel">
            <h2>Action Log</h2>
            <div class="log-container" id="log-container">
                <div class="log-entry">
                    <span class="time">--:--:--</span>
                    <span class="action">Application started. Try the menu bar!</span>
                </div>
            </div>
        </div>

        <div class="panel">
            <h2>Keyboard Shortcuts</h2>
            <ul class="shortcut-list">
                <li><span>New</span><span class="key">Ctrl+N</span></li>
                <li><span>Open</span><span class="key">Ctrl+O</span></li>
                <li><span>Save</span><span class="key">Ctrl+S</span></li>
                <li><span>Undo</span><span class="key">Ctrl+Z</span></li>
                <li><span>Redo</span><span class="key">Ctrl+Y</span></li>
                <li><span>Cut</span><span class="key">Ctrl+X</span></li>
                <li><span>Copy</span><span class="key">Ctrl+C</span></li>
                <li><span>Paste</span><span class="key">Ctrl+V</span></li>
                <li><span>Zoom In</span><span class="key">Ctrl++</span></li>
                <li><span>Zoom Out</span><span class="key">Ctrl+-</span></li>
                <li><span>Help</span><span class="key">F1</span></li>
            </ul>
        </div>

        <div class="panel">
            <h2>View Settings</h2>
            <div class="settings-group">
                <h3>Visibility</h3>
                <div class="toggle-row">
                    <span>Toolbar</span>
                    <div class="toggle-indicator on" id="toggle-toolbar"></div>
                </div>
                <div class="toggle-row">
                    <span>Sidebar</span>
                    <div class="toggle-indicator on" id="toggle-sidebar"></div>
                </div>
                <div class="toggle-row">
                    <span>Status Bar</span>
                    <div class="toggle-indicator on" id="toggle-statusbar"></div>
                </div>
            </div>

            <div class="zoom-display">
                <div class="zoom-value" id="zoom-value">100%</div>
                <div class="zoom-label">Current Zoom Level</div>
            </div>
        </div>
    </div>
</body>
</html>
"""


class MenuDemoApp:
    """Application with native menu bar."""

    def __init__(self, view):
        self.view = view
        self.zoom_level = 100
        self.toolbar_visible = True
        self.sidebar_visible = True
        self.statusbar_visible = True

    def log_action(self, action: str) -> None:
        """Log a menu action to the UI."""
        import datetime
        time_str = datetime.datetime.now().strftime("%H:%M:%S")
        html = f'''
            <div class="log-entry">
                <span class="time">{time_str}</span>
                <span class="action">{action}</span>
            </div>
        '''
        self.view.dom("#log-container").prepend_html(html)

    def update_toggle(self, toggle_id: str, is_on: bool) -> None:
        """Update toggle indicator in UI."""
        toggle = self.view.dom(f"#{toggle_id}")
        if is_on:
            toggle.add_class("on")
        else:
            toggle.remove_class("on")

    def update_zoom_display(self) -> None:
        """Update zoom level display."""
        self.view.dom("#zoom-value").set_text(f"{self.zoom_level}%")

    # File menu actions
    def file_new(self) -> None:
        self.log_action("File > New - Creating new document...")

    def file_open(self) -> None:
        self.log_action("File > Open - Opening file dialog...")

    def file_save(self) -> None:
        self.log_action("File > Save - Saving document...")

    def file_save_as(self) -> None:
        self.log_action("File > Save As - Opening save dialog...")

    def file_export(self, format: str) -> None:
        self.log_action(f"File > Export > {format.upper()} - Exporting...")

    def file_exit(self) -> None:
        self.log_action("File > Exit - Closing application...")

    # Edit menu actions
    def edit_undo(self) -> None:
        self.log_action("Edit > Undo - Undoing last action...")

    def edit_redo(self) -> None:
        self.log_action("Edit > Redo - Redoing action...")

    def edit_cut(self) -> None:
        self.log_action("Edit > Cut - Cutting selection...")

    def edit_copy(self) -> None:
        self.log_action("Edit > Copy - Copying selection...")

    def edit_paste(self) -> None:
        self.log_action("Edit > Paste - Pasting from clipboard...")

    def edit_select_all(self) -> None:
        self.log_action("Edit > Select All - Selecting all content...")

    # View menu actions
    def view_toggle_toolbar(self) -> None:
        self.toolbar_visible = not self.toolbar_visible
        self.update_toggle("toggle-toolbar", self.toolbar_visible)
        state = "shown" if self.toolbar_visible else "hidden"
        self.log_action(f"View > Toolbar - {state}")

    def view_toggle_sidebar(self) -> None:
        self.sidebar_visible = not self.sidebar_visible
        self.update_toggle("toggle-sidebar", self.sidebar_visible)
        state = "shown" if self.sidebar_visible else "hidden"
        self.log_action(f"View > Sidebar - {state}")

    def view_toggle_statusbar(self) -> None:
        self.statusbar_visible = not self.statusbar_visible
        self.update_toggle("toggle-statusbar", self.statusbar_visible)
        state = "shown" if self.statusbar_visible else "hidden"
        self.log_action(f"View > Status Bar - {state}")

    def view_zoom_in(self) -> None:
        if self.zoom_level < 200:
            self.zoom_level += 10
            self.update_zoom_display()
            self.log_action(f"View > Zoom In - {self.zoom_level}%")

    def view_zoom_out(self) -> None:
        if self.zoom_level > 50:
            self.zoom_level -= 10
            self.update_zoom_display()
            self.log_action(f"View > Zoom Out - {self.zoom_level}%")

    def view_zoom_reset(self) -> None:
        self.zoom_level = 100
        self.update_zoom_display()
        self.log_action("View > Reset Zoom - 100%")

    # Help menu actions
    def help_docs(self) -> None:
        self.log_action("Help > Documentation - Opening docs...")

    def help_updates(self) -> None:
        self.log_action("Help > Check for Updates - Checking...")

    def help_about(self) -> None:
        self.log_action("Help > About - AuroraView Native Menu Demo v1.0")


def create_menu_bar() -> MenuBar:
    """Create the application menu bar."""
    menu_bar = MenuBar()

    # File menu
    file_menu = Menu("&File")
    file_menu.add_items([
        MenuItem.action("&New", "file.new", "Ctrl+N"),
        MenuItem.action("&Open...", "file.open", "Ctrl+O"),
        MenuItem.separator(),
        MenuItem.action("&Save", "file.save", "Ctrl+S"),
        MenuItem.action("Save &As...", "file.save_as", "Ctrl+Shift+S"),
        MenuItem.separator(),
        # Export submenu
        MenuItem.submenu("&Export", [
            MenuItem.action("As &PDF", "file.export.pdf"),
            MenuItem.action("As &HTML", "file.export.html"),
            MenuItem.action("As &JSON", "file.export.json"),
        ]),
        MenuItem.separator(),
        MenuItem.action("E&xit", "file.exit", "Alt+F4"),
    ])
    menu_bar.add_menu(file_menu)

    # Edit menu
    edit_menu = Menu("&Edit")
    edit_menu.add_items([
        MenuItem.action("&Undo", "edit.undo", "Ctrl+Z"),
        MenuItem.action("&Redo", "edit.redo", "Ctrl+Y"),
        MenuItem.separator(),
        MenuItem.action("Cu&t", "edit.cut", "Ctrl+X"),
        MenuItem.action("&Copy", "edit.copy", "Ctrl+C"),
        MenuItem.action("&Paste", "edit.paste", "Ctrl+V"),
        MenuItem.separator(),
        MenuItem.action("Select &All", "edit.select_all", "Ctrl+A"),
    ])
    menu_bar.add_menu(edit_menu)

    # View menu with checkboxes
    view_menu = Menu("&View")
    view_menu.add_items([
        MenuItem.checkbox("Show &Toolbar", "view.toolbar", checked=True),
        MenuItem.checkbox("Show &Sidebar", "view.sidebar", checked=True),
        MenuItem.checkbox("Show Status &Bar", "view.statusbar", checked=True),
        MenuItem.separator(),
        MenuItem.action("Zoom &In", "view.zoom_in", "Ctrl++"),
        MenuItem.action("Zoom &Out", "view.zoom_out", "Ctrl+-"),
        MenuItem.action("&Reset Zoom", "view.zoom_reset", "Ctrl+0"),
    ])
    menu_bar.add_menu(view_menu)

    # Help menu
    help_menu = Menu("&Help")
    help_menu.add_items([
        MenuItem.action("&Documentation", "help.docs", "F1"),
        MenuItem.action("&Check for Updates", "help.updates"),
        MenuItem.separator(),
        MenuItem.action("&About", "help.about"),
    ])
    menu_bar.add_menu(help_menu)

    return menu_bar


def main():
    """Run the native menu demo."""
    from auroraview import WebView

    view = WebView(
        html=HTML,
        title="Native Menu Demo",
        width=1100,
        height=700,
    )

    app = MenuDemoApp(view)

    # Bind menu action handler
    @view.bind_call("api.menu_action")
    def handle_menu(action_id: str):
        handlers = {
            "file.new": app.file_new,
            "file.open": app.file_open,
            "file.save": app.file_save,
            "file.save_as": app.file_save_as,
            "file.export.pdf": lambda: app.file_export("pdf"),
            "file.export.html": lambda: app.file_export("html"),
            "file.export.json": lambda: app.file_export("json"),
            "file.exit": app.file_exit,
            "edit.undo": app.edit_undo,
            "edit.redo": app.edit_redo,
            "edit.cut": app.edit_cut,
            "edit.copy": app.edit_copy,
            "edit.paste": app.edit_paste,
            "edit.select_all": app.edit_select_all,
            "view.toolbar": app.view_toggle_toolbar,
            "view.sidebar": app.view_toggle_sidebar,
            "view.statusbar": app.view_toggle_statusbar,
            "view.zoom_in": app.view_zoom_in,
            "view.zoom_out": app.view_zoom_out,
            "view.zoom_reset": app.view_zoom_reset,
            "help.docs": app.help_docs,
            "help.updates": app.help_updates,
            "help.about": app.help_about,
        }
        if action_id in handlers:
            handlers[action_id]()

    # Listen for menu actions from native menu
    @view.on("menu_action")
    def on_menu_action(data):
        action_id = data.get("action_id", "")
        handle_menu(action_id=action_id)

    view.show()


if __name__ == "__main__":
    main()
