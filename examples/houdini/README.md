# AuroraView - Houdini Integration Examples

This directory contains examples for integrating AuroraView with SideFX Houdini.

## 🚀 Quick Start

### Prerequisites
- Houdini 18.5+ (Python 3.7+)
- AuroraView installed: `pip install auroraview`

### Example 01: Basic Shelf Tool

**File**: `01_basic_shelf.py`

Creates a basic shelf tool that opens a WebView panel in Houdini.

**Features**:
- Shelf button integration
- Scene node creation from UI
- Bidirectional Python ↔ JavaScript communication
- Non-blocking UI (Houdini remains responsive)

## 📖 Usage

### Method 1: Run in Houdini Python Shell

1. Open Houdini
2. Open Python Shell (Windows → Python Shell)
3. Run:
```python
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')

# Import and run
import houdini.01_basic_shelf as example
example.show()
```

### Method 2: Add to Shelf

1. Create a new shelf tool
2. Set the script to:
```python
import sys
from pathlib import Path

# Add examples directory to path
examples_dir = Path(r'C:\path\to\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

# Import and run
import houdini.01_basic_shelf as example
example.show()
```

## 🎨 Features Demonstrated

### Scene Integration
- Create geometry nodes (Box, Sphere, Grid)
- Query scene hierarchy
- Get node information
- Real-time scene updates

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

### Creating Custom Tools

Modify the HTML/JavaScript to create your own tools:
```python
html = """
<!DOCTYPE html>
<html>
<head>
    <title>My Houdini Tool</title>
    <link href="https://cdn.jsdelivr.net/npm/@shadcn/ui@latest/dist/index.css" rel="stylesheet">
</head>
<body>
    <!-- Your custom UI here -->
</body>
</html>
"""

webview = WebView.houdini("My Tool", html=html)
webview.show()
```

## 📚 API Reference

### WebView.houdini()

Factory method for Houdini integration:
```python
from auroraview import WebView

webview = WebView.houdini(
    title="My Tool",
    url="http://localhost:3000",  # Or use html= for inline HTML
    width=800,
    height=600,
    debug=True  # Enable dev tools
)
webview.show()  # Non-blocking by default
```

### Event Communication

**Python → JavaScript:**
```python
@webview.on("create_node")
def handle_create_node(data):
    node_type = data.get("type", "geo")
    # Create node in Houdini
    import hou
    node = hou.node("/obj").createNode(node_type)
    
    # Send response back
    webview.emit("node_created", {
        "name": node.name(),
        "path": node.path()
    })
```

**JavaScript → Python:**
```javascript
// Send event to Python
window.auroraview.send_event('create_node', {
    type: 'geo'
});

// Listen for response
window.auroraview.on('node_created', (data) => {
    console.log('Node created:', data.name);
});
```

## 🐛 Troubleshooting

### WebView doesn't appear
- Check Houdini's Python version: `import sys; print(sys.version)`
- Ensure AuroraView is installed in the correct Python environment
- Check console for error messages

### Houdini freezes
- Make sure you're using `WebView.houdini()` (non-blocking)
- Don't use `WebView.create()` which is blocking

### Import errors
- Verify the path in `sys.path.insert()`
- Check that `auroraview` is installed: `pip list | grep auroraview`

## 📖 See Also

- [Main Examples README](../README.md)
- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [DCC Integration Guide](../../docs/DCC_INTEGRATION_GUIDE.md)

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!

