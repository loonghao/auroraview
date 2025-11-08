# AuroraView - Blender Examples

This directory contains examples demonstrating how to use AuroraView in Blender.

## 📦 Prerequisites

### Installing AuroraView in Blender

**Method 1: Install to Blender's Python (Recommended)**
```bash
# Find Blender's Python executable
# In Blender Python Console, run:
import sys
print(sys.executable)
# Example: C:/Program Files/Blender Foundation/Blender 4.0/4.0/python/bin/python.exe

# Install AuroraView
"C:/Program Files/Blender Foundation/Blender 4.0/4.0/python/bin/python.exe" -m pip install auroraview
```

**Method 2: Install via Blender's pip**
```python
# In Blender Script Editor
import subprocess
import sys

# Install AuroraView
subprocess.check_call([sys.executable, "-m", "pip", "install", "auroraview"])
```

**Method 3: Virtual Environment**
```bash
# Create venv and install
python -m venv venv
venv\Scripts\activate
pip install auroraview

# Launch Blender from this environment
blender
```

## 🚀 Quick Start

### Example 01: Basic Window

**File**: `basic_window.py`

Basic standalone window example with HTML content.

**Usage**:
```python
# In Blender Script Editor
import sys
from pathlib import Path

examples_dir = Path(r'C:\path\to\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

# Method 1: Import module directly
import blender_examples.basic_window as example
example.show()

# Method 2: Import from package
from blender_examples import basic_window
basic_window.show()
```

### Example 02: Modal Operator ⭐ **RECOMMENDED**

**File**: `modal_operator.py`

Production-ready solution using Blender's modal operator pattern for non-blocking UI.

**Features**:
- ✅ Non-blocking UI (Blender remains responsive)
- ✅ Uses modal operator pattern (same as BQT)
- ✅ 120Hz event processing
- ✅ Automatic window handle detection

**Usage**:
```python
# In Blender Script Editor
import sys
from pathlib import Path

examples_dir = Path(r'C:\path\to\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

# Method 1: Import module directly (recommended)
import blender_examples.modal_operator as example
example.show()

# Method 2: Import from package
from blender_examples import modal_operator
modal_operator.show()
```

## 📖 How It Works

### Modal Operator Pattern

The modal operator pattern is the recommended way to integrate WebView with Blender:

1. **Modal Operator**: Uses `bpy.types.Operator` with `modal()` method
2. **High-frequency Timer**: 120Hz timer via `wm.event_timer_add()`
3. **Event Processing**: Processes WebView events without blocking Blender

This is the same pattern used by BQT (Blender Qt Tools) for Qt integration.

### Why Non-Blocking?

Blender's UI runs on the main thread. If you use a blocking WebView:
- ❌ Blender UI freezes
- ❌ Can't switch workspaces
- ❌ Can't edit objects

With modal operator:
- ✅ Blender UI stays responsive
- ✅ Can work while WebView is open
- ✅ Production-ready behavior

## 🎨 Features Demonstrated

### Scene Integration
- Create objects (Cube, Sphere, Plane)
- Query scene information
- Get selected objects
- Real-time scene updates

### UI Features
- Modern web-based interface
- Responsive design
- Interactive controls
- Status feedback

### Communication
- Python → JavaScript events
- JavaScript → Python callbacks
- JSON data exchange
- Error handling

## 🔧 Customization

### Creating Custom Tools

Modify the HTML/JavaScript to create your own tools:

```python
html = """
<!DOCTYPE html>
<html>
<head>
    <title>My Blender Tool</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body>
    <!-- Your custom UI here -->
</body>
</html>
"""

webview = WebView.blender("My Tool", html=html)
webview.show()
```

## 🐛 Troubleshooting

### WebView doesn't appear
- Check Blender's Python version: `import sys; print(sys.version)`
- Ensure AuroraView is installed: `pip list | grep auroraview`
- Check console for error messages

### Blender freezes
- Make sure you're using `02_modal_operator.py` (non-blocking)
- Don't use `01_basic_window.py` which is blocking

### Import errors
- Verify the path in `sys.path.insert()`
- Check that examples directory exists

## 📖 See Also

- [Main Examples README](../README.md)
- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [DCC Integration Guide](../../docs/DCC_INTEGRATION_GUIDE.md)

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!
