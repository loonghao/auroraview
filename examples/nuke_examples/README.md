# AuroraView - Nuke Integration Examples

This directory contains examples for integrating AuroraView with Foundry Nuke.

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

### Event Communication

**Python → JavaScript:**
```python
@webview.on("create_node")
def handle_create_node(data):
    node_type = data.get("type", "Grade")
    # Create node in Nuke
    import nuke
    node = nuke.createNode(node_type)
    
    # Send response back
    webview.emit("node_created", {
        "name": node.name(),
        "class": node.Class()
    })
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

