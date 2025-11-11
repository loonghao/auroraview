# Migration Guide: Native Backend → Qt Backend

## Problem

When using the native backend (`WebView.houdini()`), Houdini's main UI freezes because:
- Native backend creates an embedded window using HWND
- Event loop conflicts between Houdini and the WebView
- Houdini's Qt event loop gets blocked

## Solution

Use Qt backend (`QtWebView`) which:
- Integrates as a proper Qt widget
- Shares Houdini's Qt event loop
- No UI blocking or freezing

## Code Migration

### Before (Native Backend - Causes Freezing)

```python
from auroraview import WebView

def show():
    webview = WebView.houdini(
        title="Houdini Tool",
        html=HTML_CONTENT,
        width=650,
        height=500
    )
    
    @webview.on("create_node")
    def handle_create_node(data):
        # Handler code
        pass
    
    webview.show()  # ❌ Houdini UI freezes here!
    return webview
```

**JavaScript:**
```javascript
// Old API
window.auroraview.send_event('create_node', { type: 'geo' });
window.auroraview.on('node_created', (data) => { ... });
```

### After (Qt Backend - No Freezing)

```python
from auroraview import QtWebView
import hou

def get_houdini_main_window():
    """Get Houdini's main window as a QWidget."""
    return hou.qt.mainWindow()

_webview_instance = None

def show():
    global _webview_instance
    
    # Close existing instance
    if _webview_instance is not None:
        try:
            _webview_instance.close()
        except:
            pass
        _webview_instance = None
    
    # Get Houdini main window
    houdini_window = get_houdini_main_window()
    
    # Create WebView with Qt backend
    webview = QtWebView(
        parent=houdini_window,
        title="Houdini Tool",
        width=650,
        height=500
    )
    
    # Load HTML
    webview.load_html(HTML_CONTENT)
    
    @webview.on("create_node")
    def handle_create_node(data):
        # Handler code
        pass
    
    webview.show()  # ✅ Houdini UI stays responsive!
    
    # Keep reference
    _webview_instance = webview
    return webview
```

**JavaScript:**
```javascript
// New API (Qt backend)
window.emit('create_node', { type: 'geo' });
window.on('node_created', (data) => { ... });
```

## Key Changes Summary

| Aspect | Native Backend | Qt Backend |
|--------|---------------|------------|
| **Import** | `from auroraview import WebView` | `from auroraview import QtWebView` |
| **Creation** | `WebView.houdini(...)` | `QtWebView(parent=hou.qt.mainWindow(), ...)` |
| **HTML Loading** | `html=HTML_CONTENT` parameter | `webview.load_html(HTML_CONTENT)` method |
| **JS API** | `window.auroraview.send_event()` | `window.emit()` |
| **JS Listener** | `window.auroraview.on()` | `window.on()` |
| **UI Blocking** | ❌ Freezes Houdini | ✅ Non-blocking |
| **Installation** | `pip install auroraview` | `pip install auroraview[qt]` |

## Installation

```bash
# Uninstall old version (optional)
pip uninstall auroraview

# Install with Qt support
pip install auroraview[qt]
```

## Testing

After migration, test that:
1. ✅ WebView window opens
2. ✅ Houdini main UI remains responsive
3. ✅ You can interact with both Houdini and WebView simultaneously
4. ✅ Buttons in WebView work correctly
5. ✅ No console errors

See [TESTING.md](TESTING.md) for detailed testing guide.

## Benefits of Qt Backend

1. **No UI Freezing**: Houdini remains fully responsive
2. **Better Integration**: Proper Qt widget hierarchy
3. **Shared Event Loop**: No event loop conflicts
4. **Consistent API**: Same approach as Maya, Nuke examples
5. **Production Ready**: Recommended for all Qt-based DCCs

## When to Use Each Backend

### Use Qt Backend (Recommended for DCCs)
- ✅ Maya integration
- ✅ Houdini integration
- ✅ Nuke integration
- ✅ Any Qt-based DCC application

### Use Native Backend
- ✅ Standalone applications
- ✅ Non-Qt environments
- ✅ Simple scripts without DCC integration

## Need Help?

- Check [TESTING.md](TESTING.md) for testing instructions
- See [README.md](README.md) for full documentation
- Compare with [Maya Qt example](../maya_examples/qt_integration.py)

