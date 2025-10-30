# AuroraView Examples

This directory contains examples demonstrating AuroraView's capabilities in different scenarios.

## 📁 Directory Structure

```
examples/
├── README.md                      # This file
├── 01_basic_window.py            # Standalone: Basic window
├── 02_event_communication.py     # Standalone: Event system
└── maya/                          # Maya integration examples
    ├── README.md                  # Maya-specific documentation
    ├── 01_basic_integration.py    # Native backend integration
    ├── 02_outliner_native.py      # Advanced: Scene outliner (Native)
    └── 03_qt_integration.py       # Qt backend integration
```

## 🚀 Standalone Examples

### 01: Basic Window
**File**: `01_basic_window.py`

Demonstrates creating a simple standalone WebView window with HTML content.

**Features**:
- Standalone window creation
- HTML/CSS/JavaScript rendering
- Basic UI interactions

**Usage**:
```bash
python examples/01_basic_window.py
```

### 02: Event Communication
**File**: `02_event_communication.py`

Demonstrates bidirectional communication between Python and JavaScript.

**Features**:
- Python → JavaScript events
- JavaScript → Python events
- Event handlers with data payloads
- Real-time updates

**Usage**:
```bash
python examples/02_event_communication.py
```

## 🎨 Maya Examples

See [maya/README.md](maya/README.md) for detailed Maya integration examples.

### Quick Start

**Native Backend** (recommended for most cases):
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.01_basic_integration as example
example.show()
```

**Qt Backend** (for Qt widget integration):
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.03_qt_integration as example
example.show()
```

## 📚 Learning Path

1. **Start with standalone examples** to understand basic concepts:
   - `01_basic_window.py` - Window creation and HTML rendering
   - `02_event_communication.py` - Event system

2. **Move to Maya integration**:
   - `maya/01_basic_integration.py` - Basic Maya integration
   - `maya/03_qt_integration.py` - Qt backend alternative
   - `maya/02_outliner_native.py` - Advanced real-world example

## 🔧 Backend Comparison

| Feature | Native Backend | Qt Backend |
|---------|---------------|------------|
| **Installation** | `pip install auroraview` | `pip install auroraview[qt]` |
| **Dependencies** | None | qtpy + Qt bindings |
| **Integration** | HWND parenting | Qt widget |
| **Use Case** | General purpose | Qt-based DCCs |
| **Example** | `01_basic_integration.py` | `03_qt_integration.py` |

### When to Use Native Backend
- ✅ Maximum compatibility
- ✅ Minimal dependencies
- ✅ Standalone applications
- ✅ DCCs without Qt (Blender, 3ds Max)

### When to Use Qt Backend
- ✅ Qt-based DCCs (Maya, Houdini, Nuke)
- ✅ Need Qt widget integration
- ✅ Want to use Qt layouts/signals/slots
- ✅ Seamless UI integration

## 💡 Tips

### Debugging
Enable debug logging:
```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

### Performance
- Use `show_async()` for non-blocking display in DCCs
- Keep HTML/CSS/JS optimized for best performance
- Use event system instead of polling

### Best Practices
- Always use absolute paths when loading resources
- Handle errors gracefully with try/except
- Clean up resources when closing windows
- Use `parent_mode="owner"` for cross-thread safety in Maya

## 🐛 Troubleshooting

### Import Errors
```python
# Make sure the path is correct
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview')
from auroraview import NativeWebView
```

### Qt Backend Not Available
```bash
# Install Qt support
pip install auroraview[qt]
```

### Maya Integration Issues
- Use `show_async()` instead of `show()` in Maya
- Use `parent_mode="owner"` for better stability
- Check Maya's Python version matches your installation

## 📖 Additional Resources

- [Architecture Documentation](../docs/ARCHITECTURE.md)
- [API Reference](../README.md)
- [Project Repository](https://github.com/loonghao/auroraview)

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!
