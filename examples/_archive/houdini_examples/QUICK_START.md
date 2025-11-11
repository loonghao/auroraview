# Houdini Qt Integration - Quick Start

## 🚀 3 Steps to Success

### Step 1: Install AuroraView with Qt Support

```bash
# In Houdini's Python environment
hython -m pip install auroraview[qt]

# Or find Houdini's Python and install directly
"C:/Program Files/Side Effects Software/Houdini 19.5.640/python39/python3.9.exe" -m pip install auroraview[qt]
```

### Step 2: Run the Example in Houdini

Open Houdini Python Shell and run:

```python
import sys
from pathlib import Path

# Update this path!
examples_dir = Path(r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
sys.path.insert(0, str(examples_dir))

import houdini_examples.basic_shelf as example
example.show()
```

### Step 3: Verify It Works

✅ **Success indicators:**
- WebView window opens
- Houdini UI stays responsive (NOT frozen!)
- Clicking buttons creates nodes
- "Get Scene Info" shows node count

❌ **If Houdini freezes:**
- You might be using the old native backend
- Make sure you have the latest `basic_shelf.py`
- Reinstall: `pip install --force-reinstall auroraview[qt]`

## 📝 Code Template

```python
from auroraview import QtWebView
import hou

def get_houdini_main_window():
    return hou.qt.mainWindow()

# HTML content
HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>My Tool</title>
</head>
<body>
    <button onclick="window.emit('my_event', {data: 'test'})">
        Click Me
    </button>
    <script>
        window.on('response', (data) => {
            console.log('Got:', data);
        });
    </script>
</body>
</html>
"""

# Create WebView
webview = QtWebView(
    parent=get_houdini_main_window(),
    title="My Houdini Tool",
    width=800,
    height=600
)

# Load HTML
webview.load_html(HTML)

# Register event handler
@webview.on('my_event')
def handle_event(data):
    print(f"Received: {data}")
    webview.emit('response', {'status': 'ok'})

# Show window
webview.show()
```

## 🔑 Key Points

1. **Always use Qt Backend for Houdini** - Native backend causes freezing
2. **Parent to Houdini's main window** - `hou.qt.mainWindow()`
3. **Use `window.emit()` in JavaScript** - Not `window.auroraview.send_event()`
4. **Keep a global reference** - Prevents garbage collection

## 📚 More Info

- **Full Documentation**: [README.md](README.md)
- **Testing Guide**: [TESTING.md](TESTING.md)
- **Migration Guide**: [MIGRATION.md](MIGRATION.md)

## 🆘 Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| "Qt backend not available" | `pip install auroraview[qt]` |
| Houdini freezes | Use `QtWebView`, not `WebView.houdini()` |
| Import error | Check `sys.path.insert()` path |
| Window doesn't show | Check console for errors |

## 💡 Pro Tips

1. **Global Instance**: Keep webview in a global variable
   ```python
   _webview = None
   def show():
       global _webview
       _webview = QtWebView(...)
   ```

2. **Close Existing**: Close old instances before creating new ones
   ```python
   if _webview is not None:
       _webview.close()
   ```

3. **Error Handling**: Wrap in try/except for better debugging
   ```python
   try:
       webview.show()
   except Exception as e:
       print(f"Error: {e}")
   ```

## ✨ What's Different from Native Backend?

| Feature | Native Backend | Qt Backend |
|---------|---------------|------------|
| Import | `WebView` | `QtWebView` |
| Creation | `WebView.houdini()` | `QtWebView(parent=...)` |
| HTML | `html=` parameter | `.load_html()` method |
| JS API | `window.auroraview.*` | `window.*` |
| Blocking | ❌ Freezes UI | ✅ Non-blocking |

---

**Ready to build amazing Houdini tools? Start coding! 🎨**

