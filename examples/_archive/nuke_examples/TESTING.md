# Nuke IPC Testing Guide

This guide explains how to test the IPC communication between JavaScript and Python in Nuke.

## 🎯 What We're Testing

The native WebView backend uses IPC (Inter-Process Communication) to enable bidirectional communication:

- **JavaScript → Python**: Using `window.auroraview.send_event(eventName, data)`
- **Python → JavaScript**: Using `webview.emit(eventName, data)`

## 🚀 Quick Test (Recommended)

### 1. Simple Test

The simplest way to verify IPC is working:

```python
# In Nuke Script Editor (Alt+Shift+X)
import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')

from nuke_examples import test_simple
webview = test_simple.run()
```

**What to expect:**
1. A window opens with two buttons
2. Console shows "✓ Bridge ready!"
3. Click "Test IPC" → Python receives signal and responds
4. Click "Create Node" → Nuke creates a Grade node
5. All events are logged in the window

### 2. Full Manual Test

For a more comprehensive test with visual feedback:

```python
# In Nuke Script Editor
import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')

from nuke_examples import test_ipc_manual
webview, results = test_ipc_manual.run_test()
```

**What to expect:**
1. Beautiful UI with test instructions
2. Step-by-step testing workflow
3. Visual status updates
4. Automatic result tracking

### 3. Automated Test (pytest)

For CI/CD or automated testing:

```bash
# Must be run inside Nuke's Python environment
pytest tests/test_nuke_ipc.py -v
```

## 🔍 Debugging

### Check if Bridge is Ready

Open DevTools (F12) and check console:

```javascript
// Should see:
[AuroraView] Initializing event bridge...
[AuroraView] Bridge ready!

// Test manually:
window.auroraview.send_event('test', { hello: 'world' });
```

### Verify Python Handler

In Nuke Script Editor:

```python
from auroraview import WebView

webview = WebView.create(title="Debug", debug=True)

@webview.on("test")
def handle_test(data):
    print(f"Received: {data}")
    webview.emit("response", {"status": "ok"})

webview.load_html("""
<script>
    setTimeout(() => {
        if (window.auroraview) {
            console.log('Bridge exists!');
            window.auroraview.send_event('test', {msg: 'hello'});
        } else {
            console.error('Bridge not found!');
        }
    }, 1000);
</script>
""")

webview.show()
```

### Common Issues

#### 1. "window.auroraview is undefined"

**Cause**: Bridge not initialized yet

**Solution**: Wait for initialization
```javascript
function waitForBridge() {
    if (window.auroraview && window.auroraview.send_event) {
        // Bridge ready!
        init();
    } else {
        setTimeout(waitForBridge, 50);
    }
}
waitForBridge();
```

#### 2. "Events not received in Python"

**Cause**: Handler not registered before event sent

**Solution**: Register handlers before loading HTML
```python
# ✓ Correct order
@webview.on("my_event")
def handler(data):
    pass

webview.load_html(html)  # HTML can now send events
```

#### 3. "Node not created"

**Cause**: Not running in Nuke

**Solution**: Check if Nuke is available
```python
try:
    import nuke
    node = nuke.createNode("Grade")
except ImportError:
    print("Not running in Nuke!")
```

## 📊 Test Results

After running tests, you should see:

```
============================================================
Nuke IPC Communication Test
============================================================

[1/5] Creating WebView...
      ✓ WebView created

[2/5] Registering IPC handlers...
      ✓ Handlers registered

[3/5] Loading test UI...
      ✓ HTML loaded

[4/5] Showing WebView...
      → Click 'Create Grade Node' to test IPC
      → Click 'Complete Test' when done

[5/5] Test Results:
      Nodes created: 1
      Errors: 0

      Created nodes:
        - Grade1 (Grade)

============================================================
Test completed!
============================================================
```

## 🎓 Understanding the IPC Flow

### JavaScript → Python

```javascript
// JavaScript sends event
window.auroraview.send_event('create_node', {
    type: 'Grade'
});
```

↓ IPC Message ↓

```python
# Python receives event
@webview.on("create_node")
def handle_create_node(data):
    node_type = data["type"]  # "Grade"
    node = nuke.createNode(node_type)
```

### Python → JavaScript

```python
# Python sends event
webview.emit("node_created", {
    "name": node.name(),
    "class": node.Class()
})
```

↓ IPC Message ↓

```javascript
// JavaScript receives event
window.auroraview.on('node_created', function(data) {
    console.log('Node created:', data.name);
});
```

## 🔧 Advanced Testing

### Test Multiple Events

```python
@webview.on("event1")
def handler1(data):
    print("Event 1:", data)

@webview.on("event2")
def handler2(data):
    print("Event 2:", data)
    webview.emit("response", {"from": "event2"})
```

### Test Error Handling

```python
@webview.on("risky_operation")
def handle_risky(data):
    try:
        # Risky code
        result = do_something()
        webview.emit("success", result)
    except Exception as e:
        webview.emit("error", {"message": str(e)})
```

## 📝 Next Steps

1. ✅ Run `test_simple.py` to verify basic IPC
2. ✅ Run `test_ipc_manual.py` for comprehensive testing
3. ✅ Check DevTools console for any errors
4. ✅ Verify nodes are created in Nuke
5. ✅ Review test results

If all tests pass, your IPC communication is working correctly! 🎉

