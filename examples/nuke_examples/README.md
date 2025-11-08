# AuroraView - Nuke Integration Examples

This directory contains examples for integrating AuroraView with Foundry Nuke.

## ✨ New Simplified API

All examples now use the **simplified API** with **zero delay**!

- ✅ **No bridge waiting** - `window.aurora` is immediately available
- ✅ **No external files** - Everything built-in
- ✅ **Qt-style API** - Familiar `emit()` / `on()` syntax
- ✅ **Zero learning curve** - Just use `window.aurora.emit()` and `window.aurora.on()`

**Old way (deprecated)**:
```javascript
// ❌ Complex - needed waiting logic
function waitForBridge() {
    if (window.auroraview) {
        window.auroraview.send_event('test', {});
    } else {
        setTimeout(waitForBridge, 100);
    }
}
```

**New way (recommended)**:
```javascript
// ✅ Simple - works immediately!
window.aurora.emit('test', {});
window.aurora.on('response', (data) => { ... });
```

See [SIMPLIFIED_API.md](SIMPLIFIED_API.md) for complete documentation.

## 🚀 Quick Start

### Prerequisites
- Nuke 12.0+ (Python 3.7+)
- AuroraView installed in Nuke's Python environment

### Installing AuroraView in Nuke

**Method 1: Install to Nuke's Python (Recommended)**
```bash
# Find Nuke's Python executable path
# In Nuke Script Editor, run:
import sys
print(sys.executable)
# Example output: C:/Program Files/Nuke15.2v1/python.exe

# Install AuroraView using Nuke's Python
"C:/Program Files/Nuke15.2v1/python.exe" -m pip install auroraview
```

**Method 2: Use Virtual Environment**
```bash
# Create and activate virtual environment
python -m venv venv
venv\Scripts\activate  # Windows
source venv/bin/activate  # Linux/Mac

# Install AuroraView
pip install auroraview

# Launch Nuke from this environment
nuke
```

**Method 3: Add to PYTHONPATH**
```bash
# If you have AuroraView installed elsewhere
set PYTHONPATH=C:\path\to\auroraview\installation;%PYTHONPATH%
nuke
```

### Example 01: Basic Panel

**File**: `basic_panel.py`

Creates a basic panel that integrates with Nuke's node graph.

**Features**:
- Panel integration
- Create nodes from UI
- Query node graph
- Bidirectional Python ↔ JavaScript communication
- Non-blocking UI (Nuke remains responsive)

### Example 02: IPC Communication Test

**File**: `test_ipc_manual.py`

Manual test for verifying IPC communication between JavaScript and Python.

**Features**:
- Native WebView backend (no Qt dependency)
- window.auroraview API testing
- Create nodes via IPC signals
- Bidirectional event communication
- Visual feedback and status updates

**Usage**:
```python
import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
from nuke_examples import test_ipc_manual
test_ipc_manual.run_test()
```

## 📖 Usage

### Method 1: Run in Nuke Script Editor

1. Open Nuke
2. Open Script Editor (Alt+Shift+X)
3. Run:
```python
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')

# Method 1: Import module directly
import nuke_examples.basic_panel as example
webview = example.show()

# Method 2: Import from package
from nuke_examples import basic_panel
webview = basic_panel.show()

# To close programmatically:
# basic_panel.close()
# or
# webview.close()
```

### Method 2: Add to Menu

Add to `menu.py`:
```python
import sys
from pathlib import Path

# Add examples directory to path
examples_dir = Path(r'C:\path\to\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

# Add menu item
from nuke_examples import basic_panel
nuke.menu('Nuke').addCommand('AuroraView/Basic Panel', basic_panel.show)
```

## 🎨 Features Demonstrated

### Node Graph Integration
- Create nodes (Read, Write, Grade, Blur)
- Query selected nodes
- Get node properties
- Real-time graph updates

### UI Features
- Modern web-based interface using shadcn/ui
- Responsive design
- Dark mode support
- Interactive controls

### Communication
- Python → JavaScript events
- JavaScript → Python callbacks
- JSON data exchange
- Error handling

## 🔧 Customization

### Using CDN for UI Components

The example uses shadcn/ui components via CDN:
```html
<link href="https://cdn.jsdelivr.net/npm/@shadcn/ui@latest/dist/index.css" rel="stylesheet">
```

### Creating Custom Panels

Modify the HTML/JavaScript to create your own panels:
```python
html = """
<!DOCTYPE html>
<html>
<head>
    <title>My Nuke Panel</title>
    <link href="https://cdn.jsdelivr.net/npm/@shadcn/ui@latest/dist/index.css" rel="stylesheet">
</head>
<body>
    <!-- Your custom UI here -->
</body>
</html>
"""

webview = WebView.create("My Panel", html=html)
webview.show()
```

## 🐛 Troubleshooting

### Properly Closing the WebView

**Important**: Always close the WebView properly to prevent Nuke from hanging on exit.

**Method 1: Click the X button** (Recommended)
- Simply click the close button on the WebView window
- The window will close and clean up automatically

**Method 2: Programmatic close**
```python
# If you saved the webview reference
webview = basic_panel.show()
webview.close()

# Or use the module-level close function
import nuke_examples.basic_panel as example
example.close()
```

**Method 3: Singleton mode** (Automatic)
- The example uses singleton mode (`singleton="nuke_panel"`)
- Opening a new panel automatically closes the old one
- Only one instance can exist at a time

### Qt Warnings on Window Close

When closing the WebView window, you may see warnings like:
```
RuntimeError: Internal C++ object (PySide2.QtWidgets.QLabel) already deleted.
Traceback (most recent call last):
  File ".../hiero/ui/FnStatusBar.py", line 296, in updateChannelCountLabel
    self.channelCountLabel.setText("")
RuntimeError: Internal C++ object (PySide2.QtWidgets.QLabel) already deleted.
```

**This is harmless and can be safely ignored.**

These warnings come from Nuke/Hiero's status bar trying to update after the WebView window is closed. The warnings don't affect functionality and are a known Qt lifecycle issue in Nuke's UI framework.

The example code already includes warning suppression to minimize console noise.

### Nuke Won't Exit After Closing WebView

If Nuke doesn't exit properly after closing the WebView:

1. **Make sure you closed the WebView window** - Click the X button or call `close()`
2. **Check for multiple instances** - The singleton mode prevents this, but if you disabled it, close all instances
3. **Force close if needed**:
   ```python
   import nuke_examples.basic_panel as example
   example.close()  # Force close the active webview
   ```

### Import Errors

If you see `ModuleNotFoundError: No module named 'auroraview'`:
- Make sure AuroraView is installed in Nuke's Python environment
- See the [Installation](#installing-auroraview-in-nuke) section above

### WebView Doesn't Appear

- Check that you're running in Nuke (not standalone Python)
- Verify the HTML content is valid
- Enable debug mode: `WebView.create(..., debug=True)`

## 📚 API Reference

### WebView.create()

Factory method for Nuke integration:
```python
from auroraview import WebView

webview = WebView.create(
    title="My Panel",
    url="http://localhost:3000",  # Or use html= for inline HTML
    width=800,
    height=600,
    debug=True  # Enable dev tools
)
webview.show()
```

### Signal/Slot System (Qt-Style)

AuroraView uses a **Qt-inspired signal/slot mechanism** for bidirectional communication between Python and JavaScript. This provides:

- ✅ **Type-safe** event binding
- ✅ **Automatic queueing** if bridge isn't ready
- ✅ **Error handling** for each slot
- ✅ **Familiar API** for Qt developers

#### Python Side (Slots)

**Register event handlers (slots):**
```python
@webview.on("create_node")  # Signal name
def handle_create_node(data):  # Slot function
    """Handle create_node signal from JavaScript."""
    node_type = data.get("type", "Grade")

    # Create node in Nuke
    import nuke
    node = nuke.createNode(node_type)

    # Emit response signal
    webview.emit("node_created", {
        "name": node.name(),
        "class": node.Class()
    })
```

**Emit signals to JavaScript:**
```python
# Send data to JavaScript
webview.emit("signal_name", {"key": "value"})
```

#### JavaScript Side (Signals & Slots)

**AuroraViewBridge Class:**

The example includes a robust `AuroraViewBridge` class that handles:
- Automatic initialization waiting
- Pending call queueing
- Error handling
- Qt-style API

**Emit signals to Python:**
```javascript
// Create global bridge instance
const bridge = new AuroraViewBridge();

// Emit signal (JavaScript → Python)
bridge.emit('create_node', { type: 'Grade' });
```

**Connect slots to signals:**
```javascript
// Connect slot to signal (Python → JavaScript)
bridge.connect('node_created', (data) => {
    if (data.error) {
        console.error('Error:', data.error);
    } else {
        console.log(`Created ${data.class} node: ${data.name}`);
    }
});
```

**Complete Example:**
```javascript
// Initialize bridge
const bridge = new AuroraViewBridge();

// Connect response handlers (slots)
bridge.connect('node_created', (data) => {
    showStatus(`✅ Created ${data.class} node: ${data.name}`);
});

bridge.connect('graph_info', (data) => {
    showStatus(`Graph: ${data.total_nodes} nodes`);
});

// Emit signals to Python
function createNode(type) {
    bridge.emit('create_node', { type: type });
}

function getGraphInfo() {
    bridge.emit('get_graph_info', {});
}
```

#### Benefits Over Direct API

**❌ Old way (error-prone):**
```javascript
// May fail if bridge not ready
window.auroraview.send_event('create_node', { type: 'Grade' });

// No error handling
window.auroraview.on('node_created', (data) => {
    console.log(data.name);  // May throw if data is undefined
});
```

**✅ New way (robust):**
```javascript
// Automatically queues if not ready
bridge.emit('create_node', { type: 'Grade' });

// Built-in error handling
bridge.connect('node_created', (data) => {
    if (data.error) {
        console.error(data.error);
    } else {
        console.log(data.name);
    }
});
```

**JavaScript → Python:**
```javascript
// Send event to Python
window.auroraview.send_event('create_node', {
    type: 'Grade'
});

// Listen for response
window.auroraview.on('node_created', (data) => {
    console.log('Node created:', data.name);
});
```

## 🐛 Troubleshooting

### WebView doesn't appear
- Check Nuke's Python version: `import sys; print(sys.version)`
- Ensure AuroraView is installed in the correct Python environment
- Check console for error messages

### Nuke freezes
- Make sure you're using non-blocking mode
- Check event loop integration

### Import errors
- Verify the path in `sys.path.insert()`
- Check that `auroraview` is installed: `pip list | grep auroraview`

## 📖 See Also

- [Main Examples README](../README.md)
- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [DCC Integration Guide](../../docs/DCC_INTEGRATION_GUIDE.md)

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!

