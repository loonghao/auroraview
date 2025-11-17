# Qt Integration Best Practices

This guide covers best practices for integrating AuroraView with Qt-based DCC applications (Maya, Houdini, Nuke, etc.).

## Quick Start

### ✅ Recommended: Use QtWebView

```python
from auroraview import QtWebView

# Create WebView as Qt widget
webview = QtWebView(
    parent=maya_main_window(),  # Optional: any QWidget
    title="My Tool",
    width=800,
    height=600
)

# Load content
webview.load_url("http://localhost:3000")

# Show the widget
webview.show()

# ✨ That's it! Event processing is automatic.
```

### ❌ Avoid: Manual Event Processing

```python
# DON'T DO THIS - it's unnecessary with QtWebView
from auroraview import WebView

webview = WebView.create(...)
webview.show()

# Manual event processing (NOT NEEDED with QtWebView)
def process_events():
    webview.process_events()  # ❌ Unnecessary!

cmds.scriptJob(event=["idle", process_events])
```

## Understanding Event Processing

### The Problem

AuroraView uses a message queue for JavaScript execution:

1. Python calls `eval_js(script)` → Script pushed to queue
2. Queue needs to be processed → Script executes
3. Without processing → **Script never executes, Promises hang**

### The Solution

`QtWebView` automatically processes events after every `eval_js()` call:

```python
# When you call:
webview.eval_js("console.log('Hello')")

# QtWebView automatically:
# 1. Pushes script to queue
# 2. Calls process_events()  ← Automatic!
# 3. Script executes immediately
```

## Common Patterns

### Pattern 1: Python → JavaScript Communication

```python
from auroraview import QtWebView

webview = QtWebView(title="My Tool")

# This works immediately - no manual event processing needed
webview.eval_js("console.log('Hello from Python')")
webview.emit("update_scene", {"objects": ["cube", "sphere"]})
```

### Pattern 2: JavaScript → Python Communication

```python
# Python side
@webview.on("get_scene_data")
def handle_get_scene_data(data):
    # Get scene data from DCC
    selection = cmds.ls(selection=True)
    # Send back to JavaScript - automatic event processing!
    webview.emit("scene_data_response", {"selection": selection})

# JavaScript side
window.auroraview.on("scene_data_response", (data) => {
    console.log("Selection:", data.selection);
});

window.auroraview.send_event("get_scene_data", {});
```

### Pattern 3: Using auroraview.call() API

```python
# Python side
@webview.bind_call("get_scene_hierarchy")
def get_scene_hierarchy(params):
    # Return scene hierarchy
    return {"nodes": [...]}

# JavaScript side - Promise resolves automatically!
const result = await window.auroraview.call("get_scene_hierarchy");
console.log("Hierarchy:", result);  // ✅ Works!
```

## Diagnostics

### Check Event Processing

```python
# Get diagnostic information
diag = webview.get_diagnostics()

print(f"Events processed: {diag['event_process_count']}")
print(f"Last process time: {diag['last_event_process_time']}")
print(f"Hook installed: {diag['has_post_eval_hook']}")
print(f"Hook correct: {diag['hook_is_correct']}")
```

### Troubleshooting

If `auroraview.call()` Promises are hanging:

1. **Check hook installation**:
   ```python
   diag = webview.get_diagnostics()
   assert diag['has_post_eval_hook'], "Hook not installed!"
   assert diag['hook_is_correct'], "Hook is wrong!"
   ```

2. **Check event processing**:
   ```python
   # Should increase after each eval_js/emit call
   before = webview.get_diagnostics()['event_process_count']
   webview.eval_js("console.log('test')")
   after = webview.get_diagnostics()['event_process_count']
   assert after > before, "Events not being processed!"
   ```

3. **Enable debug logging**:
   ```python
   import logging
   logging.basicConfig(level=logging.DEBUG)
   ```

## Performance Considerations

### Event Processing Overhead

- Each `eval_js()` call triggers event processing
- Overhead: ~1-2ms per call
- For high-frequency updates, batch your calls:

```python
# ❌ Inefficient: 100 event processing cycles
for i in range(100):
    webview.eval_js(f"updateNode({i})")

# ✅ Efficient: 1 event processing cycle
script = "\n".join(f"updateNode({i})" for i in range(100))
webview.eval_js(script)
```

## Summary

✅ **DO**:
- Use `QtWebView` for Qt-based DCCs
- Trust automatic event processing
- Use `get_diagnostics()` for troubleshooting
- Batch high-frequency `eval_js()` calls

❌ **DON'T**:
- Manually call `process_events()` with `QtWebView`
- Use `WebView.create()` in Qt environments
- Create scriptJobs for event processing
- Make hundreds of individual `eval_js()` calls

