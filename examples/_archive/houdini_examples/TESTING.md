# Houdini Qt Integration Testing Guide

## Problem Fixed

**Before (Native Backend):**
- Using `WebView.houdini()` created an embedded window
- Houdini's main UI would freeze/block
- Event loop conflicts between Houdini and WebView

**After (Qt Backend):**
- Using `QtWebView` with Houdini's Qt main window as parent
- No UI blocking - seamless integration
- Proper Qt widget hierarchy

## Prerequisites

1. **Install AuroraView with Qt support:**
   ```bash
   pip install auroraview[qt]
   ```

2. **Verify Houdini has Qt support:**
   - Houdini uses PySide2 (older versions) or PySide6 (newer versions)
   - The example auto-detects which one is available

## Testing Steps

### 1. Launch Houdini

Open Houdini (any recent version that supports Python 3)

### 2. Run the Example

In Houdini's Python Shell:

```python
import sys
from pathlib import Path

# Update this path to your examples directory
examples_dir = Path(r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

import houdini_examples.basic_shelf as example
example.show()
```

### 3. Expected Behavior

✅ **Should work:**
- WebView window opens as a Qt widget
- Houdini main UI remains responsive (NOT frozen!)
- You can interact with both Houdini and the WebView simultaneously
- Buttons in WebView create nodes in Houdini
- "Get Scene Info" button shows current scene data

❌ **Should NOT happen:**
- Houdini UI freezing
- Console errors about event loops
- Window not responding

### 4. Test Interactions

1. **Create Nodes:**
   - Click "📦 Box" button → Should create a geo node in /obj
   - Click "⚪ Sphere" button → Should create a sphere node
   - Click "📐 Grid" button → Should create a grid node
   - Check Houdini's network view to see the created nodes

2. **Query Scene:**
   - Click "🔍 Get Scene Info" → Should show node count
   - Create some nodes manually in Houdini
   - Click "Get Scene Info" again → Count should update

3. **UI Responsiveness:**
   - While WebView is open, try:
     - Moving Houdini's viewport
     - Creating nodes manually
     - Opening Houdini menus
   - All should work smoothly without freezing

### 5. Close the Window

```python
# To close programmatically
example._webview_instance.close()
```

Or just click the X button on the window.

## Troubleshooting

### Error: "Qt backend not available"

**Solution:**
```bash
pip install auroraview[qt]
```

### Error: "PySide2/PySide6 not available"

**Cause:** Houdini's Python environment doesn't have Qt bindings

**Solution:** This shouldn't happen in modern Houdini versions. If it does:
1. Check Houdini version (should be 18.5+)
2. Verify Houdini's Python has Qt: `import PySide2` or `import PySide6`

### Window opens but Houdini freezes

**Cause:** You might be using the old native backend example

**Solution:** Make sure you're using the updated `basic_shelf.py` that uses `QtWebView`

## Comparison: Native vs Qt Backend

| Feature | Native Backend | Qt Backend |
|---------|---------------|------------|
| Integration | Embedded HWND | Qt Widget |
| UI Blocking | ❌ Freezes Houdini | ✅ Non-blocking |
| Event Loop | Conflicts | Shared Qt loop |
| Installation | `pip install auroraview` | `pip install auroraview[qt]` |
| Use Case | Standalone apps | DCC integration |

## Key Code Changes

**Old (Native Backend):**
```python
from auroraview import WebView

webview = WebView.houdini(
    title="Tool",
    html=HTML_CONTENT
)
webview.show()
```

**New (Qt Backend):**
```python
from auroraview import QtWebView
import hou

houdini_window = hou.qt.mainWindow()

webview = QtWebView(
    parent=houdini_window,
    title="Tool",
    width=650,
    height=500
)
webview.load_html(HTML_CONTENT)
webview.show()
```

## Success Criteria

✅ Test passes if:
1. WebView window opens successfully
2. Houdini main UI remains fully responsive
3. Buttons in WebView successfully create nodes
4. Scene info query works correctly
5. No console errors or warnings
6. Window can be closed cleanly

## Notes

- The Qt backend is the **recommended** approach for all Qt-based DCCs (Maya, Houdini, Nuke)
- Native backend is better for standalone applications
- Keep a global reference to the webview instance to prevent garbage collection

