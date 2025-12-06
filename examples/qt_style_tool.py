"""Qt-Style Class Inheritance Pattern Example - AuroraView API Demo.

This example demonstrates the recommended Qt-like pattern for production tools.
Best for complex applications, team collaboration, and DCC integration.

Usage:
    python examples/qt_style_tool.py

Features demonstrated:
    - Class inheritance from WebView
    - Signal definitions (Python → JavaScript)
    - Auto-bound public methods as API
    - Event handlers with on_ prefix
    - Signal connections in setup_connections()
    - Clean separation of concerns

This pattern is inspired by Qt's signal/slot mechanism and provides:
    - Familiar syntax for Qt developers
    - Type-safe signal definitions
    - Automatic method discovery and binding
    - Clear distinction between API methods and event handlers
"""

from __future__ import annotations

from auroraview import Signal, WebView


class SceneOutliner(WebView):
    """A scene outliner tool demonstrating Qt-like patterns.

    This class shows the recommended pattern for production tools:
    - Signals for Python → JavaScript notifications
    - Public methods for JavaScript → Python API calls
    - on_ prefix methods for event handling
    """

    # ═══════════════════════════════════════════════════════════════════
    # Signal Definitions (Python → JavaScript notifications)
    # ═══════════════════════════════════════════════════════════════════
    # Signals are used to notify JavaScript about state changes.
    # They are one-way (fire-and-forget) and can have multiple listeners.

    selection_changed = Signal(list)  # Emitted when selection changes
    progress_updated = Signal(int, str)  # Emitted during long operations
    scene_loaded = Signal(str)  # Emitted when scene is loaded
    item_renamed = Signal(str, str)  # Emitted when item is renamed (old, new)

    def __init__(self):
        """Initialize the outliner tool."""
        # HTML content for demonstration
        html = self._get_demo_html()

        super().__init__(title="Scene Outliner (Qt-Style)", html=html, width=500, height=700, debug=True)

        # Internal state
        self._scene_items = ["Group1", "Mesh_Cube", "Mesh_Sphere", "Camera1", "Light_Key"]
        self._selection: list[str] = []

        # Setup signal connections
        self.setup_connections()

    # ═══════════════════════════════════════════════════════════════════
    # API Methods (JavaScript → Python, auto-bound)
    # ═══════════════════════════════════════════════════════════════════
    # Public methods are automatically exposed to JavaScript.
    # They can be called via: await auroraview.api.method_name({...})

    def get_hierarchy(self, parent: str = None) -> dict:
        """Get the scene hierarchy.

        JavaScript:
            const result = await auroraview.api.get_hierarchy();
            const result = await auroraview.api.get_hierarchy({parent: "Group1"});
        """
        return {
            "items": self._scene_items,
            "count": len(self._scene_items),
            "parent": parent,
        }

    def get_selection(self) -> dict:
        """Get current selection.

        JavaScript:
            const result = await auroraview.api.get_selection();
        """
        return {"selection": self._selection, "count": len(self._selection)}

    def set_selection(self, items: list = None) -> dict:
        """Set the current selection.

        JavaScript:
            await auroraview.api.set_selection({items: ["Mesh_Cube", "Camera1"]});
        """
        items = items or []
        old_selection = self._selection.copy()
        self._selection = [item for item in items if item in self._scene_items]

        # Emit signal to notify JavaScript
        if self._selection != old_selection:
            self.selection_changed.emit(self._selection)

        return {"ok": True, "selection": self._selection}

    def rename_item(self, old_name: str = "", new_name: str = "") -> dict:
        """Rename a scene item.

        JavaScript:
            await auroraview.api.rename_item({old_name: "Cube", new_name: "HeroCube"});
        """
        if not old_name or not new_name:
            return {"ok": False, "error": "Both old_name and new_name required"}

        if old_name not in self._scene_items:
            return {"ok": False, "error": f"Item '{old_name}' not found"}

        if new_name in self._scene_items:
            return {"ok": False, "error": f"Item '{new_name}' already exists"}

        # Perform rename
        idx = self._scene_items.index(old_name)
        self._scene_items[idx] = new_name

        # Update selection if needed
        if old_name in self._selection:
            sel_idx = self._selection.index(old_name)
            self._selection[sel_idx] = new_name

        # Emit signal
        self.item_renamed.emit(old_name, new_name)

        return {"ok": True, "old": old_name, "new": new_name}

    def delete_items(self, items: list = None) -> dict:
        """Delete scene items.

        JavaScript:
            await auroraview.api.delete_items({items: ["Mesh_Cube"]});
        """
        items = items or []
        deleted = []

        for item in items:
            if item in self._scene_items:
                self._scene_items.remove(item)
                deleted.append(item)
                if item in self._selection:
                    self._selection.remove(item)

        if deleted:
            self.selection_changed.emit(self._selection)

        return {"ok": True, "deleted": deleted, "count": len(deleted)}

    def simulate_progress(self, steps: int = 10) -> dict:
        """Simulate a long operation with progress updates.

        JavaScript:
            await auroraview.api.simulate_progress({steps: 5});
        """
        import time

        for i in range(steps):
            progress = int((i + 1) / steps * 100)
            message = f"Processing step {i + 1}/{steps}..."
            self.progress_updated.emit(progress, message)
            time.sleep(0.2)  # Simulate work

        return {"ok": True, "steps_completed": steps}

    # ═══════════════════════════════════════════════════════════════════
    # Event Handlers (on_ prefix, auto-bound)
    # ═══════════════════════════════════════════════════════════════════
    # Methods with on_ prefix are event handlers for JavaScript events.
    # They are called via: auroraview.emit("event_name", {...})

    def on_item_clicked(self, data: dict) -> None:
        """Handle item click from JavaScript.

        JavaScript:
            auroraview.emit("item_clicked", {name: "Mesh_Cube", ctrl: false});
        """
        name = data.get("name", "")
        ctrl_held = data.get("ctrl", False)

        print(f"[Python] Item clicked: {name} (ctrl={ctrl_held})")

        if ctrl_held:
            # Add to selection
            if name not in self._selection:
                self._selection.append(name)
        else:
            # Replace selection
            self._selection = [name] if name in self._scene_items else []

        self.selection_changed.emit(self._selection)

    def on_item_double_clicked(self, data: dict) -> None:
        """Handle item double-click (e.g., for rename mode).

        JavaScript:
            auroraview.emit("item_double_clicked", {name: "Mesh_Cube"});
        """
        name = data.get("name", "")
        print(f"[Python] Item double-clicked: {name} - entering rename mode")

    def on_clear_selection(self, data: dict) -> None:
        """Handle clear selection request.

        JavaScript:
            auroraview.emit("clear_selection", {});
        """
        self._selection = []
        self.selection_changed.emit(self._selection)

    # ═══════════════════════════════════════════════════════════════════
    # Signal Connections (like Qt's connect())
    # ═══════════════════════════════════════════════════════════════════

    def setup_connections(self) -> None:
        """Setup signal-slot connections.

        This is similar to Qt's pattern of connecting signals to slots
        in the constructor or a dedicated setup method.
        """
        # Connect internal signals to handlers
        self.selection_changed.connect(self._on_selection_changed)
        self.progress_updated.connect(self._on_progress_updated)
        self.item_renamed.connect(self._on_item_renamed)

    def _on_selection_changed(self, items: list) -> None:
        """Internal handler for selection changes."""
        print(f"[Python] Selection changed: {items}")

    def _on_progress_updated(self, percent: int, message: str) -> None:
        """Internal handler for progress updates."""
        print(f"[Python] Progress: {percent}% - {message}")

    def _on_item_renamed(self, old_name: str, new_name: str) -> None:
        """Internal handler for item renames."""
        print(f"[Python] Item renamed: {old_name} → {new_name}")

    # ═══════════════════════════════════════════════════════════════════
    # Private Methods (not exposed to JavaScript)
    # ═══════════════════════════════════════════════════════════════════

    def _get_demo_html(self) -> str:
        """Generate demo HTML content."""
        return """
<!DOCTYPE html>
<html>
<head>
    <title>Scene Outliner</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1e1e1e;
            color: #e0e0e0;
            padding: 16px;
        }
        h2 { color: #4fc3f7; margin-bottom: 16px; font-size: 18px; }
        .section { background: #2d2d2d; border-radius: 8px; padding: 16px; margin-bottom: 16px; }
        .item {
            padding: 8px 12px;
            margin: 4px 0;
            background: #3d3d3d;
            border-radius: 4px;
            cursor: pointer;
            transition: all 0.15s;
        }
        .item:hover { background: #4d4d4d; }
        .item.selected { background: #1976d2; color: white; }
        button {
            background: #4fc3f7;
            color: #1e1e1e;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            margin: 4px;
            font-weight: 500;
        }
        button:hover { background: #81d4fa; }
        .progress-bar {
            height: 20px;
            background: #3d3d3d;
            border-radius: 4px;
            overflow: hidden;
            margin: 8px 0;
        }
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #4fc3f7, #81d4fa);
            transition: width 0.2s;
        }
        .status { font-size: 12px; color: #888; margin-top: 8px; }
        #log {
            background: #252525;
            padding: 12px;
            border-radius: 4px;
            font-family: monospace;
            font-size: 11px;
            max-height: 120px;
            overflow-y: auto;
        }
    </style>
</head>
<body>
    <div class="section">
        <h2>📋 Scene Outliner</h2>
        <div id="items"></div>
    </div>

    <div class="section">
        <h2>🎮 Actions</h2>
        <button onclick="refresh()">Refresh</button>
        <button onclick="clearSelection()">Clear Selection</button>
        <button onclick="deleteSelected()">Delete Selected</button>
        <button onclick="runProgress()">Run Progress</button>
    </div>

    <div class="section">
        <h2>📊 Progress</h2>
        <div class="progress-bar"><div class="progress-fill" id="progress" style="width: 0%"></div></div>
        <div class="status" id="progress-text">Ready</div>
    </div>

    <div class="section">
        <h2>📜 Event Log</h2>
        <div id="log"></div>
    </div>

    <script>
        let selection = [];

        function log(msg) {
            const logEl = document.getElementById('log');
            const time = new Date().toLocaleTimeString();
            logEl.innerHTML = `[${time}] ${msg}<br>` + logEl.innerHTML;
        }

        async function refresh() {
            const result = await auroraview.api.get_hierarchy();
            renderItems(result.items);
            log(`Loaded ${result.count} items`);
        }

        function renderItems(items) {
            const container = document.getElementById('items');
            container.innerHTML = items.map(item => `
                <div class="item ${selection.includes(item) ? 'selected' : ''}"
                     onclick="selectItem('${item}', event)"
                     ondblclick="renameItem('${item}')">
                    ${item}
                </div>
            `).join('');
        }

        function selectItem(name, event) {
            auroraview.emit('item_clicked', {name, ctrl: event.ctrlKey});
        }

        function renameItem(name) {
            auroraview.emit('item_double_clicked', {name});
            const newName = prompt(`Rename ${name}:`, name);
            if (newName && newName !== name) {
                auroraview.api.rename_item({old_name: name, new_name: newName}).then(refresh);
            }
        }

        function clearSelection() {
            auroraview.emit('clear_selection', {});
        }

        async function deleteSelected() {
            if (selection.length === 0) return alert('Nothing selected');
            await auroraview.api.delete_items({items: [...selection]});
            refresh();
        }

        async function runProgress() {
            await auroraview.api.simulate_progress({steps: 10});
        }

        // Listen for Python signals
        auroraview.on('selection_changed', (items) => {
            selection = items;
            log(`Selection: [${items.join(', ')}]`);
            refresh();
        });

        auroraview.on('progress_updated', (percent, message) => {
            document.getElementById('progress').style.width = percent + '%';
            document.getElementById('progress-text').textContent = message;
        });

        auroraview.on('item_renamed', (oldName, newName) => {
            log(`Renamed: ${oldName} → ${newName}`);
        });

        // Initial load
        refresh();
    </script>
</body>
</html>
"""


def main():
    """Run the Qt-style example."""
    print("Starting Scene Outliner (Qt-Style Pattern)...")
    print()
    print("This example demonstrates:")
    print("  - Signal definitions (selection_changed, progress_updated, etc.)")
    print("  - Auto-bound API methods (get_hierarchy, set_selection, etc.)")
    print("  - Event handlers with on_ prefix (on_item_clicked, etc.)")
    print("  - Signal connections in setup_connections()")
    print()

    outliner = SceneOutliner()
    outliner.show()


if __name__ == "__main__":
    main()

