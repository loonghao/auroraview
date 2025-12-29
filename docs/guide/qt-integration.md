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
