# AuroraView Examples

This directory contains examples demonstrating AuroraView's capabilities across different DCC applications.

## 📁 Directory Structure

```
examples/
├── README.md                      # This file
├── 01_basic_window.py            # Standalone: Basic window
├── 02_event_communication.py     # Standalone: Event system
├── 03_remote_site_communication.py # Remote site communication
├── 04_parent_lifecycle_demo.py   # Parent window lifecycle
├── 04_real_remote_site.py        # Real website integration
├── 05_third_party_site_injection.py # JavaScript injection
├── 06_ai_chat_integration.py     # AI chat integration
├── 07_ai_chat_non_blocking.py    # AI chat (non-blocking)
├── 08_maya_integration_fixed.py  # Maya integration
├── test_baidu_maya.py            # Maya: Baidu test
├── test_maya_remote_url.py       # Maya: Remote URL testing
├── test_public_urls.py           # Public URL testing
├── blender/                       # Blender examples
│   ├── README.md
│   ├── 01_basic_window.py
│   └── 02_modal_operator.py      # ⭐ Recommended
├── maya/                          # Maya examples
│   ├── README.md
│   ├── 01_basic_integration.py
│   ├── 02_outliner_native.py
│   ├── 03_qt_integration.py
│   └── test_close_fix.py
├── houdini/                       # Houdini examples
│   ├── README.md
│   └── 01_basic_shelf.py         # ⭐ New!
├── nuke/                          # Nuke examples
│   ├── README.md
│   └── 01_basic_panel.py         # ⭐ New!
└── maya-outliner/                 # Advanced Maya project
    └── ...
```

## 🚀 Quick Start by DCC

### Blender
```python
# In Blender Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import blender.02_modal_operator as example
example.show()
```

### Maya
```python
# In Maya Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import maya.01_basic_integration as example
example.show()
```

### Houdini
```python
# In Houdini Python Shell
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import houdini.01_basic_shelf as example
example.show()
```

### Nuke
```python
# In Nuke Script Editor
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
import nuke.01_basic_panel as example
example.show()
```

## 📚 Example Categories

### 🎯 Standalone Examples

Basic examples that run without DCC software:

- **01_basic_window.py** - Simple window with HTML content
- **02_event_communication.py** - Python ↔ JavaScript events
- **03_remote_site_communication.py** - Remote website communication
- **04_parent_lifecycle_demo.py** - Parent window lifecycle management
- **05_third_party_site_injection.py** - JavaScript injection into third-party sites
- **06_ai_chat_integration.py** - AI chat integration example
- **07_ai_chat_non_blocking.py** - Non-blocking AI chat

### 🎨 DCC Integration Examples

#### Blender
- **01_basic_window.py** - Basic window (blocking mode)
- **02_modal_operator.py** ⭐ - Modal operator (non-blocking, recommended)

#### Maya
- **01_basic_integration.py** - Native backend integration
- **02_outliner_native.py** - Scene outliner with real-time updates
- **03_qt_integration.py** - Qt backend integration

#### Houdini ⭐ New!
- **01_basic_shelf.py** - Basic shelf tool with node creation

#### Nuke ⭐ New!
- **01_basic_panel.py** - Basic panel with node graph integration

## 🎨 Features Demonstrated

### Core Features
- ✅ Window creation and management
- ✅ HTML/CSS/JavaScript rendering
- ✅ Bidirectional Python ↔ JavaScript communication
- ✅ Event system with callbacks
- ✅ Remote URL loading
- ✅ JavaScript injection
- ✅ Parent window lifecycle management

### DCC-Specific Features
- ✅ Scene object creation
- ✅ Node graph integration
- ✅ Real-time scene updates
- ✅ Selection management
- ✅ Non-blocking UI (modal operators)

### UI Features
- ✅ Modern web-based interfaces
- ✅ shadcn/ui components via CDN
- ✅ Tailwind CSS styling
- ✅ Responsive design
- ✅ Interactive controls

## 📖 Learning Path

### 1. Start with Standalone Examples
Learn the basics without DCC software:
1. `01_basic_window.py` - Window creation
2. `02_event_communication.py` - Event system
3. `03_remote_site_communication.py` - Remote sites

### 2. Explore DCC Integration
Choose your DCC application:
- **Blender**: Start with `blender/02_modal_operator.py`
- **Maya**: Start with `maya/01_basic_integration.py`
- **Houdini**: Start with `houdini/01_basic_shelf.py`
- **Nuke**: Start with `nuke/01_basic_panel.py`

### 3. Advanced Topics
- `05_third_party_site_injection.py` - JavaScript injection
- `06_ai_chat_integration.py` - AI integration
- `maya-outliner/` - Full React/TypeScript project

## 🔧 Customization

All examples use inline HTML for simplicity. For production:

### Option 1: Local HTML Files
```python
webview = WebView.create(
    title="My Tool",
    url="file:///path/to/index.html"
)
```

### Option 2: Development Server
```python
webview = WebView.create(
    title="My Tool",
    url="http://localhost:3000"
)
```

### Option 3: CDN Components
```html
<link href="https://cdn.jsdelivr.net/npm/@shadcn/ui@latest/dist/index.css" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
```

## 🐛 Troubleshooting

### Import Errors
```python
# Make sure path is correct
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
```

### WebView Doesn't Appear
- Check Python version: `import sys; print(sys.version)`
- Verify AuroraView is installed: `pip list | grep auroraview`
- Check console for error messages

### DCC Freezes
- Use non-blocking examples (e.g., `blender/02_modal_operator.py`)
- Don't use blocking mode in DCC applications

## 📖 See Also

- [Main README](../README.md)
- [Architecture Documentation](../docs/ARCHITECTURE.md)
- [DCC Integration Guide](../docs/DCC_INTEGRATION_GUIDE.md)
- [API Reference](../README.md#api-reference)

## 🤝 Contributing

Found a bug or have an improvement? Please open an issue or submit a pull request!
