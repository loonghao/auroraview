# AuroraView - Houdini Integration Examples

This directory contains examples for integrating AuroraView with SideFX Houdini.

## ⚠️ Important: Use Qt Backend

**The Qt backend is required for Houdini integration to prevent UI freezing!**

## 🚀 Quick Start

### Prerequisites
- Houdini 18.5+ (Python 3.7+)
- AuroraView with Qt support installed in Houdini's Python environment

### Installing AuroraView in Houdini

**Method 1: Install to Houdini's Python (Recommended)**
```bash
# Find Houdini's Python executable
# In Houdini Python Shell, run:
import sys
print(sys.executable)
# Example: C:/Program Files/Side Effects Software/Houdini 19.5.640/python39/python3.9.exe

# Install AuroraView with Qt support
"C:/Program Files/Side Effects Software/Houdini 19.5.640/python39/python3.9.exe" -m pip install auroraview[qt]
```

**Method 2: Use hython**
```bash
# Use Houdini's Python wrapper
hython -m pip install auroraview[qt]
```

**Method 3: Virtual Environment**
```bash
# Create venv and install
python -m venv venv
venv\Scripts\activate
pip install auroraview[qt]

# Launch Houdini from this environment
houdini
```

### Example 01: Basic Shelf Tool (Qt Backend)

**File**: `basic_shelf.py`

Creates a shelf tool using Qt backend for seamless Houdini integration.

**Features**:
- ✅ Qt backend integration (no UI freezing!)
- ✅ Scene node creation from UI
- ✅ Bidirectional Python ↔ JavaScript communication
- ✅ Non-blocking UI (Houdini remains fully responsive)
- ✅ Proper Qt widget hierarchy

**Why Qt Backend?**
- Native backend causes Houdini's main UI to freeze
- Qt backend integrates as a proper Qt widget
- Shares Houdini's event loop - no conflicts

## 📖 Usage

### Method 1: Run in Houdini Python Shell

1. Open Houdini
2. Open Python Shell (Windows → Python Shell)
3. Run:
```python
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')

# Method 1: Import module directly
import houdini_examples.basic_shelf as example
example.show()

# Method 2: Import from package
from houdini_examples import basic_shelf
basic_shelf.show()
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
from houdini_examples import basic_shelf
basic_shelf.show()
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

### QtWebView (Recommended for Houdini)

Qt backend integration:
```python
from auroraview import QtWebView
import hou

# Get Houdini's main window
houdini_window = hou.qt.mainWindow()

# Create WebView as Qt widget
webview = QtWebView(
    parent=houdini_window,
    title="My Tool",
    width=800,
    height=600
)

# Load HTML content
webview.load_html(html_content)
# Or load URL
# webview.load_url("http://localhost:3000")

webview.show()  # Non-blocking
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

**JavaScript → Python (Qt Backend):**
```javascript
// Send event to Python
window.emit('create_node', {
    type: 'geo'
});

// Listen for response
window.on('node_created', (data) => {
    console.log('Node created:', data.name);
});
```

## 🐛 Troubleshooting

### WebView doesn't appear
- Check Houdini's Python version: `import sys; print(sys.version)`
- Ensure AuroraView with Qt support is installed: `pip install auroraview[qt]`
- Check console for error messages

### Houdini freezes or UI blocks
- ✅ **Solution**: Use `QtWebView` instead of native backend
- The example has been updated to use Qt backend
- See `TESTING.md` for detailed testing guide

### Error: "Qt backend not available"
- Install Qt support: `pip install auroraview[qt]`
- Or reinstall: `pip install --force-reinstall auroraview[qt]`

### Error: "PySide2/PySide6 not available"
- This shouldn't happen in modern Houdini (18.5+)
- Verify Houdini has Qt: `import PySide2` or `import PySide6`

### Import errors
- Verify the path in `sys.path.insert()`
- Check that `auroraview` is installed: `pip list | grep auroraview`

## 📖 See Also

- [Houdini Testing Guide](TESTING.md) - Detailed testing instructions
- [Main Examples README](../README.md)
- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [Maya Qt Integration Example](../maya_examples/qt_integration.py) - Similar Qt approach

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!

