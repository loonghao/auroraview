# Qt Integration Best Practices

This guide covers best practices for integrating AuroraView with Qt-based DCC applications (Maya, Houdini, Nuke, 3ds Max, etc.).

## Quick Start

### Recommended: Use `QtWebView`

```python
from auroraview import QtWebView

webview = QtWebView(
    parent=maya_main_window(),  # Optional: any QWidget
    title="My Tool",
    width=800,
    height=600,
)

webview.load_url("http://localhost:3000")
webview.show()
```

## Understanding Event Processing (Why `QtWebView` Matters)

AuroraView uses a message queue to safely marshal work to the correct UI thread.
That includes:

- `webview.eval_js(...)`
- `webview.emit(...)`
- returning results for `auroraview.call(...)`

If the queue is not processed, JS execution and RPC results can be delayed.

### The Solution

`QtWebView` installs a Qt-aware event processor (`QtEventProcessor`) so that:

- Qt events are pumped (`QCoreApplication.processEvents()`)
- AuroraView messages are flushed (`WebView.process_events()`)

This happens automatically after `emit()` / `eval_js()` (unless you disable `auto_process`).

### Avoid: Manual ScriptJobs / Idle Hooks

You generally should not build your own “idle loop” in Maya/Houdini just to call `process_events()`.
Prefer `QtWebView`, which wires the correct processing strategy for you.

## Common Patterns

### Pattern 1: Python → JavaScript (push events)

```python
from auroraview import QtWebView

webview = QtWebView(title="My Tool")
webview.emit("update_scene", {"objects": ["cube", "sphere"]})
```

### Pattern 2: JavaScript → Python (fire-and-forget)

```python
@webview.on("get_scene_data")
def handle_get_scene_data(data):
    selection = cmds.ls(selection=True)
    webview.emit("scene_data_response", {"selection": selection})
```

```javascript
window.auroraview.on("scene_data_response", (data) => {
  console.log("Selection:", data.selection);
});

window.auroraview.send_event("get_scene_data", {});
```

### Pattern 3: JavaScript → Python (RPC with return value)

```python
@webview.bind_call("api.get_scene_hierarchy")
def get_scene_hierarchy(root: str = "scene"):
    return {"root": root, "nodes": []}
```

```javascript
const result = await window.auroraview.call("api.get_scene_hierarchy", { root: "scene" });
console.log("Hierarchy:", result);
```

## Diagnostics

### Check Event Processor State

```python
diag = webview.get_diagnostics()
print(f"Processor: {diag['event_processor_type']}")
print(f"Processed: {diag['event_process_count']}")
print(f"Has processor: {diag['has_event_processor']}")
print(f"Processor OK: {diag['processor_is_correct']}")
```

### Troubleshooting: `auroraview.call()` timeouts

If `auroraview.call()` is timing out:

- Ensure you are using `@webview.bind_call(...)` / `bind_api(...)` (not `@webview.on(...)`).
- In Qt-based DCC, ensure you are using `QtWebView` (or that a Qt-aware event processor is installed).

## Embedding WebView in Existing Qt UI

A common use case is embedding AuroraView into an existing Qt application—for example, adding an AI assistant panel to a legacy tool.

::: tip Automatic Initialization
`QtWebView` follows standard Qt widget semantics. When embedded in a parent widget (like `QDockWidget`), it automatically initializes when the parent becomes visible. You don't need to call `show()` on `QtWebView` directly—just add it to a layout and show the parent.
:::

### Basic Embedding

`QtWebView` inherits from `QWidget`, so you can add it to any Qt layout:

```python
from auroraview import QtWebView
from qtpy.QtWidgets import QMainWindow, QDockWidget, QVBoxLayout, QWidget
from qtpy.QtCore import Qt

class MyExistingTool(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("My Legacy Tool")

        # Your existing UI setup...
        self.setup_main_ui()

        # Add AuroraView as a dock widget (sidebar)
        self.add_ai_panel()

    def setup_main_ui(self):
        # Your existing tool's main content
        central = QWidget()
        self.setCentralWidget(central)

    def add_ai_panel(self):
        """Add AI assistant panel to sidebar."""
        dock = QDockWidget("AI Assistant", self)
        dock.setAllowedAreas(Qt.LeftDockWidgetArea | Qt.RightDockWidgetArea)

        # Create WebView - thread safety is automatic
        self.ai_webview = QtWebView(
            parent=dock,
            url="http://localhost:3000/ai-agent",
            width=400,
            height=600,
        )

        dock.setWidget(self.ai_webview)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)
        # WebView initializes automatically when parent is shown!
```

### Two-Way Communication: Host ↔ WebView

The key to integration is setting up bidirectional communication between your existing tool and the embedded WebView.

#### 1. Expose Host Tool's API to WebView

```python
class MyExistingTool(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setup_main_ui()
        self.add_ai_panel()
        self.setup_webview_bridge()

    def setup_webview_bridge(self):
        """Set up communication between host tool and WebView."""

        # Method 1: Bind entire API object
        self.ai_webview.bind_api(HostToolAPI(self))

        # Method 2: Bind individual functions
        @self.ai_webview.bind_call("host.get_current_state")
        def get_state():
            return {
                "active_tab": self.get_active_tab(),
                "selected_items": self.get_selected_items(),
                "project_path": self.project_path,
            }

        @self.ai_webview.bind_call("host.execute_action")
        def execute_action(action: str, params: dict = None):
            return self.execute_tool_action(action, params or {})

class HostToolAPI:
    """API exposed to the AI Agent WebView."""

    def __init__(self, host: MyExistingTool):
        self.host = host

    def get_selected_items(self) -> list:
        """Get currently selected items in host tool."""
        return self.host.get_selected_items()

    def open_file(self, path: str) -> dict:
        """Open a file in the host tool."""
        try:
            self.host.open_file(path)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def run_command(self, cmd: str) -> dict:
        """Execute a command in the host tool."""
        result = self.host.execute_command(cmd)
        return {"ok": True, "result": result}
```

```javascript
// In AI Agent (WebView side)
// Call host tool's API
const state = await auroraview.call("host.get_current_state");
console.log("Current project:", state.project_path);

// Execute action in host tool
await auroraview.call("host.execute_action", {
  action: "create_node",
  params: { type: "sphere", name: "MySphere" }
});

// Or use the bound API object
const items = await auroraview.api.get_selected_items();
await auroraview.api.open_file("/path/to/file.txt");
```

#### 2. Push Updates from Host to WebView

```python
class MyExistingTool(QMainWindow):
    def on_selection_changed(self):
        """Called when user selects items in the host tool."""
        # Push selection update to AI Agent
        self.ai_webview.emit("host:selection_changed", {
            "items": self.get_selected_items(),
            "count": len(self.get_selected_items()),
        })

    def on_file_opened(self, path: str):
        """Called when a file is opened."""
        self.ai_webview.emit("host:file_opened", {
            "path": path,
            "content_type": self.detect_content_type(path),
        })

    def on_error(self, error: Exception):
        """Forward errors to AI Agent for context."""
        self.ai_webview.emit("host:error", {
            "message": str(error),
            "type": type(error).__name__,
        })
```

```javascript
// In AI Agent (WebView side)
// Listen for host events
auroraview.on("host:selection_changed", (data) => {
  console.log(`User selected ${data.count} items:`, data.items);
  // AI can now provide context-aware suggestions
});

auroraview.on("host:file_opened", (data) => {
  console.log("File opened:", data.path);
  // AI can analyze the file
});

auroraview.on("host:error", (data) => {
  console.log("Error in host tool:", data.message);
  // AI can suggest fixes
});
```

#### 3. Handle AI Agent Requests

```python
class MyExistingTool(QMainWindow):
    def setup_webview_bridge(self):
        # ... previous bindings ...

        # Listen for AI agent requests
        @self.ai_webview.on("ai:request_context")
        def handle_context_request(data):
            """AI agent requests more context."""
            context_type = data.get("type")

            if context_type == "full_state":
                self.ai_webview.emit("ai:context_response", {
                    "state": self.get_full_state(),
                    "history": self.get_action_history(),
                })
            elif context_type == "selected_content":
                self.ai_webview.emit("ai:context_response", {
                    "content": self.get_selected_content(),
                })

        @self.ai_webview.on("ai:execute_suggestion")
        def handle_suggestion(data):
            """Execute AI's suggested action."""
            action = data.get("action")
            params = data.get("params", {})

            try:
                result = self.execute_tool_action(action, params)
                self.ai_webview.emit("ai:execution_result", {
                    "ok": True,
                    "result": result,
                })
            except Exception as e:
                self.ai_webview.emit("ai:execution_result", {
                    "ok": False,
                    "error": str(e),
                })
```

### Complete Example: Asset Browser with AI Assistant

```python
from auroraview import QtWebView
from qtpy.QtWidgets import (
    QMainWindow, QDockWidget, QTreeWidget, QTreeWidgetItem,
    QVBoxLayout, QWidget, QSplitter
)
from qtpy.QtCore import Qt

class AssetBrowser(QMainWindow):
    """Legacy asset browser tool with AI assistant sidebar."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Asset Browser")
        self.resize(1200, 800)

        self._setup_ui()
        self._setup_ai_panel()
        self._connect_signals()

    def _setup_ui(self):
        """Set up the main asset browser UI."""
        splitter = QSplitter(Qt.Horizontal)

        # Asset tree (existing functionality)
        self.asset_tree = QTreeWidget()
        self.asset_tree.setHeaderLabels(["Name", "Type", "Size"])
        self._populate_assets()
        splitter.addWidget(self.asset_tree)

        # Preview panel (existing functionality)
        self.preview = QWidget()
        splitter.addWidget(self.preview)

        self.setCentralWidget(splitter)

    def _setup_ai_panel(self):
        """Add AI assistant as a dock widget."""
        dock = QDockWidget("AI Assistant", self)

        self.ai_view = QtWebView(
            parent=dock,
            url="http://localhost:3000/asset-ai",
        )

        # Expose asset browser API
        self.ai_view.bind_api(AssetBrowserAPI(self))

        dock.setWidget(self.ai_view)
        self.addDockWidget(Qt.RightDockWidgetArea, dock)
        # WebView initializes automatically when parent is shown!

    def _connect_signals(self):
        """Connect Qt signals to WebView events."""
        self.asset_tree.itemSelectionChanged.connect(self._on_selection_changed)
        self.asset_tree.itemDoubleClicked.connect(self._on_item_activated)

    def _on_selection_changed(self):
        """Forward selection to AI assistant."""
        items = self.asset_tree.selectedItems()
        self.ai_view.emit("asset:selection", {
            "assets": [self._item_to_dict(item) for item in items],
        })

    def _on_item_activated(self, item):
        """Forward activation to AI assistant."""
        self.ai_view.emit("asset:activated", self._item_to_dict(item))

    def _item_to_dict(self, item: QTreeWidgetItem) -> dict:
        return {
            "name": item.text(0),
            "type": item.text(1),
            "size": item.text(2),
        }

    def _populate_assets(self):
        # ... populate tree ...
        pass

class AssetBrowserAPI:
    """API for AI assistant to interact with asset browser."""

    def __init__(self, browser: AssetBrowser):
        self.browser = browser

    def get_selected_assets(self) -> list:
        items = self.browser.asset_tree.selectedItems()
        return [self.browser._item_to_dict(item) for item in items]

    def search_assets(self, query: str, asset_type: str = None) -> list:
        # Implement search logic
        return []

    def open_asset(self, name: str) -> dict:
        # Implement open logic
        return {"ok": True}

    def get_asset_metadata(self, name: str) -> dict:
        # Return asset metadata
        return {}
```

## Performance Considerations

### Batch High-Frequency JS Work

```python
# Inefficient: many flushes
for i in range(100):
    webview.eval_js(f"updateNode({i})")

# Efficient: one flush
script = "\n".join(f"updateNode({i})" for i in range(100))
webview.eval_js(script)
```

## Best Practices Summary

| Scenario | Recommendation |
|----------|----------------|
| Embed in Qt layout | Use `QtWebView` as regular `QWidget` |
| Expose host API | Use `bind_api()` or `@bind_call()` |
| Push updates to WebView | Use `emit()` with namespaced events |
| Handle WebView requests | Use `@on()` decorator |
| Thread safety | Automatic with `dcc_mode="auto"` |
